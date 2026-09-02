use std::{collections::BTreeMap, io::ErrorKind, time::Duration};

use anyhow::{Result, anyhow};
use editor_preview::{
    PreviewCommand, PreviewCommandEnvelope, PreviewEvent, PreviewMode, PreviewSessionId,
    ReloadDecision, ReloadStatus, ReloadSubject, ReloadSubjectKind, RuntimeProductKind,
    RuntimeProductPayload, RuntimeProductRef, WorldSdfPayloadPackage, preview_session_id,
};
use runen_spatial::{ChunkCoord3, ChunkId, WorldId};
use runenwerk_editor::runtime::preview_process::{
    PreviewConnectionEvent, PreviewProcessConnection,
};
use runenwerk_runtime_preview::{RuntimePreviewConfig, RuntimePreviewHost, RuntimePreviewLoopExit};
use tokio::task::JoinHandle;
use world_sdf::{
    SdfBrickRecord, SdfBrickSamples, SdfChunkPayload, SdfPageCoord3, SdfPageRecord,
    WorldSdfPayloadRef,
};

const EVENT_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_STRESS_ITERATIONS: u64 = 8;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preview_control_channel_round_trips_over_standalone_runennet() -> Result<()> {
    let host = match RuntimePreviewHost::spawn(RuntimePreviewConfig::headless()) {
        Ok(host) => host,
        Err(error) if is_permission_denied(&error) => {
            eprintln!("skipping preview RunenNet loopback proof: local socket bind is denied");
            return Ok(());
        }
        Err(error) => return Err(error.context("preview server spawn failed")),
    };
    let bootstrap = host.bootstrap().clone();
    let server_task = spawn_server_task(host);

    let mut connection = PreviewProcessConnection::connect(&bootstrap).await?;
    let session_id = preview_session_id(1);
    start_preview_session(&mut connection, session_id).await?;

    connection
        .send_preview_command(PreviewCommandEnvelope::new(
            2,
            PreviewCommand::Heartbeat { session_id },
        ))
        .await?;
    assert!(matches!(
        next_preview_event(&mut connection).await?.event,
        PreviewEvent::Heartbeat { session_id: id } if id == session_id
    ));

    connection
        .send_preview_command(PreviewCommandEnvelope::new(
            3,
            PreviewCommand::ApplyReload {
                session_id,
                status: Box::new(ReloadStatus::new(
                    ReloadSubject::new(ReloadSubjectKind::Shader, "shader"),
                    ReloadDecision::LiveReload,
                    "shader reloaded",
                )),
            },
        ))
        .await?;
    assert!(matches!(
        next_preview_event(&mut connection).await?.event,
        PreviewEvent::ReloadStatus { session_id: id, .. } if id == session_id
    ));

    let product = representative_world_sdf_product();
    let expected_product = match &product {
        RuntimeProductPayload::WorldSdf(package) => package.product_ref.clone(),
        RuntimeProductPayload::Descriptor(_) => unreachable!("fixture must exercise WorldSDF"),
    };
    connection
        .send_preview_command(PreviewCommandEnvelope::new(
            4,
            PreviewCommand::PublishProduct {
                session_id,
                payload: Box::new(product),
            },
        ))
        .await?;
    assert!(matches!(
        next_preview_event(&mut connection).await?.event,
        PreviewEvent::ProductLoaded {
            session_id: id,
            product,
        } if id == session_id && *product == expected_product
    ));

    request_application_shutdown(&mut connection, session_id, 5).await?;
    connection.shutdown().await?;
    assert_eq!(
        await_server_exit(server_task).await?,
        RuntimePreviewLoopExit::ShutdownRequested {
            session_id: Some(session_id),
        }
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preview_shutdown_lifecycle_repeats_without_transport_eof_dependency() -> Result<()> {
    for iteration in 1..=SHUTDOWN_STRESS_ITERATIONS {
        let host = match RuntimePreviewHost::spawn(RuntimePreviewConfig::headless()) {
            Ok(host) => host,
            Err(error) if is_permission_denied(&error) => {
                eprintln!("skipping preview RunenNet shutdown stress proof: local socket bind is denied");
                return Ok(());
            }
            Err(error) => return Err(error.context("preview server spawn failed")),
        };
        let bootstrap = host.bootstrap().clone();
        let server_task = spawn_server_task(host);
        let mut connection = PreviewProcessConnection::connect(&bootstrap).await?;
        let session_id = preview_session_id(iteration);

        start_preview_session(&mut connection, session_id).await?;
        request_application_shutdown(&mut connection, session_id, 2).await?;
        connection.shutdown().await?;
        assert_eq!(
            await_server_exit(server_task).await?,
            RuntimePreviewLoopExit::ShutdownRequested {
                session_id: Some(session_id),
            }
        );
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preview_transport_close_without_shutdown_is_not_application_shutdown() -> Result<()> {
    let host = match RuntimePreviewHost::spawn(RuntimePreviewConfig::headless()) {
        Ok(host) => host,
        Err(error) if is_permission_denied(&error) => {
            eprintln!("skipping preview RunenNet close proof: local socket bind is denied");
            return Ok(());
        }
        Err(error) => return Err(error.context("preview server spawn failed")),
    };
    let bootstrap = host.bootstrap().clone();
    let server_task = spawn_server_task(host);
    let mut connection = PreviewProcessConnection::connect(&bootstrap).await?;
    let session_id = preview_session_id(1);

    start_preview_session(&mut connection, session_id).await?;
    connection.shutdown().await?;
    assert_eq!(
        await_server_exit(server_task).await?,
        RuntimePreviewLoopExit::TransportClosed
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preview_control_channel_rejects_mismatched_trusted_certificate() -> Result<()> {
    let server = match RuntimePreviewHost::spawn(RuntimePreviewConfig::headless()) {
        Ok(host) => host,
        Err(error) if is_permission_denied(&error) => {
            eprintln!("skipping preview RunenNet trust proof: local socket bind is denied");
            return Ok(());
        }
        Err(error) => return Err(error.context("preview server spawn failed")),
    };
    let other_server = match RuntimePreviewHost::spawn(RuntimePreviewConfig::headless()) {
        Ok(host) => host,
        Err(error) if is_permission_denied(&error) => {
            server.shutdown().await?;
            eprintln!("skipping preview RunenNet trust proof: second local socket bind is denied");
            return Ok(());
        }
        Err(error) => {
            server.shutdown().await?;
            return Err(error.context("second preview server spawn failed"));
        }
    };

    let mut bootstrap = server.bootstrap().clone();
    bootstrap.trusted_certificate_der_hex =
        other_server.bootstrap().trusted_certificate_der_hex.clone();

    let connect_result = PreviewProcessConnection::connect(&bootstrap).await;
    let server_cleanup = server.shutdown().await;
    let other_server_cleanup = other_server.shutdown().await;

    if let Ok(connection) = connect_result {
        let client_cleanup = connection.shutdown().await;
        return Err(anyhow!(
            "runtime-preview connection accepted a different valid trust anchor; server cleanup: {server_cleanup:?}; second server cleanup: {other_server_cleanup:?}; client cleanup: {client_cleanup:?}"
        ));
    }

    // The contacted server may report the peer's expected TLS handshake rejection as its task
    // result. Awaiting shutdown proves ownership cleanup; that handshake result is not a clean-session
    // requirement for this negative trust test.
    let _ = server_cleanup;
    other_server_cleanup?;
    Ok(())
}

async fn start_preview_session(
    connection: &mut PreviewProcessConnection,
    session_id: PreviewSessionId,
) -> Result<()> {
    connection
        .send_preview_command(PreviewCommandEnvelope::new(
            1,
            PreviewCommand::StartSession {
                session_id,
                mode: PreviewMode::Preview,
            },
        ))
        .await?;
    assert!(matches!(
        next_preview_event(connection).await?.event,
        PreviewEvent::Ready { session_id: id } if id == session_id
    ));
    assert!(matches!(
        next_preview_event(connection).await?.event,
        PreviewEvent::ModeChanged {
            session_id: id,
            mode: PreviewMode::Preview,
        } if id == session_id
    ));
    Ok(())
}

async fn request_application_shutdown(
    connection: &mut PreviewProcessConnection,
    session_id: PreviewSessionId,
    sequence: u64,
) -> Result<()> {
    connection
        .send_preview_command(PreviewCommandEnvelope::new(
            sequence,
            PreviewCommand::Shutdown { session_id },
        ))
        .await?;
    assert!(matches!(
        next_preview_event(connection).await?.event,
        PreviewEvent::ShutdownAck { session_id: id } if id == session_id
    ));
    Ok(())
}

fn spawn_server_task(mut host: RuntimePreviewHost) -> JoinHandle<Result<RuntimePreviewLoopExit>> {
    tokio::spawn(async move {
        let run_result = host.run_command_loop().await;
        let shutdown_result = host.shutdown().await;
        match (run_result, shutdown_result) {
            (Ok(exit), Ok(())) => Ok(exit),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Err(run_error), Err(shutdown_error)) => Err(anyhow!(
                "runtime-preview server failed: {run_error:#}; shutdown also failed: {shutdown_error:#}"
            )),
        }
    })
}

async fn await_server_exit(
    server_task: JoinHandle<Result<RuntimePreviewLoopExit>>,
) -> Result<RuntimePreviewLoopExit> {
    let server_result = tokio::time::timeout(EVENT_TIMEOUT, server_task)
        .await
        .map_err(|_| anyhow!("timed out waiting for runtime-preview server shutdown"))?
        .map_err(|error| anyhow!("runtime-preview server task join failed: {error}"))?;
    server_result.map_err(|error| error.context("runtime-preview server task failed"))
}

fn representative_world_sdf_product() -> RuntimeProductPayload {
    let chunk_id = ChunkId::new(WorldId::new(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
    let chunk = SdfChunkPayload {
        chunk_id,
        chunk_revision: Default::default(),
        chunk_generation: Default::default(),
        page_table: BTreeMap::from([(
            SdfPageCoord3 { x: 0, y: 0, z: 0 },
            SdfPageRecord {
                page_generation: 1,
                bricks: BTreeMap::from([(
                    [0, 0, 0],
                    SdfBrickRecord {
                        metadata: Default::default(),
                        samples: SdfBrickSamples {
                            distances: vec![0; 8 * 8 * 8],
                        },
                    },
                )]),
            },
        )]),
        hierarchy_revision: 1,
        checksum: 7,
    };
    let product_ref =
        RuntimeProductRef::new(RuntimeProductKind::WorldSdfPayload, "loopback-world-sdf")
            .with_payload_refs(vec![WorldSdfPayloadRef::from(&chunk)]);
    RuntimeProductPayload::WorldSdf(WorldSdfPayloadPackage::new(product_ref, vec![chunk]))
}

async fn next_preview_event(
    connection: &mut PreviewProcessConnection,
) -> Result<editor_preview::PreviewEventEnvelope> {
    let event = tokio::time::timeout(EVENT_TIMEOUT, connection.next_event())
        .await
        .map_err(|_| anyhow!("timed out waiting for preview connection event"))?
        .ok_or_else(|| anyhow!("preview connection event stream closed"))?;
    match event {
        PreviewConnectionEvent::Preview(event) => Ok(event),
        PreviewConnectionEvent::Closed => {
            Err(anyhow!("preview connection closed before expected event"))
        }
        PreviewConnectionEvent::Error(message) => {
            Err(anyhow!("preview connection failed: {message}"))
        }
    }
}

fn is_permission_denied(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_error| io_error.kind() == ErrorKind::PermissionDenied)
    })
}
