use std::{collections::BTreeMap, io::ErrorKind, time::Duration};

use anyhow::{Result, anyhow};
use editor_preview::{
    PreviewCommand, PreviewCommandEnvelope, PreviewEvent, PreviewMode, ReloadDecision,
    ReloadStatus, ReloadSubject, ReloadSubjectKind, RuntimeProductKind, RuntimeProductPayload,
    RuntimeProductRef, WorldSdfPayloadPackage, preview_session_id,
};
use runen_spatial::{ChunkCoord3, ChunkId, WorldId};
use runenwerk_editor::runtime::preview_process::{
    PreviewConnectionEvent, PreviewProcessConnection,
};
use runenwerk_runtime_preview::{RuntimePreviewConfig, RuntimePreviewHost, RuntimePreviewLoopExit};
use world_sdf::{
    SdfBrickRecord, SdfBrickSamples, SdfChunkPayload, SdfPageCoord3, SdfPageRecord,
    WorldSdfPayloadRef,
};

const EVENT_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn preview_control_channel_round_trips_over_standalone_runennet() -> Result<()> {
    let mut host = match RuntimePreviewHost::spawn(RuntimePreviewConfig::headless()) {
        Ok(host) => host,
        Err(error) if is_permission_denied(&error) => {
            eprintln!("skipping preview RunenNet loopback proof: local socket bind is denied");
            return Ok(());
        }
        Err(error) => return Err(error.context("preview server spawn failed")),
    };
    let bootstrap = host.bootstrap().clone();
    let server_task = tokio::spawn(async move {
        let exit = host.run_command_loop().await?;
        host.shutdown().await?;
        Ok::<RuntimePreviewLoopExit, anyhow::Error>(exit)
    });

    let mut connection = PreviewProcessConnection::connect(&bootstrap).await?;
    let session_id = preview_session_id(1);

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
        next_preview_event(&mut connection).await?.event,
        PreviewEvent::Ready { session_id: id } if id == session_id
    ));
    assert!(matches!(
        next_preview_event(&mut connection).await?.event,
        PreviewEvent::ModeChanged {
            session_id: id,
            mode: PreviewMode::Preview,
        } if id == session_id
    ));

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

    connection
        .send_preview_command(PreviewCommandEnvelope::new(
            5,
            PreviewCommand::Shutdown { session_id },
        ))
        .await?;
    assert!(matches!(
        next_preview_event(&mut connection).await?.event,
        PreviewEvent::ShutdownAck { session_id: id } if id == session_id
    ));

    connection.shutdown().await?;

    let server_result = tokio::time::timeout(EVENT_TIMEOUT, server_task)
        .await
        .map_err(|_| anyhow!("timed out waiting for runtime-preview server shutdown"))?
        .map_err(|error| anyhow!("runtime-preview server task join failed: {error}"))?;
    let exit =
        server_result.map_err(|error| error.context("runtime-preview server task failed"))?;
    assert_eq!(
        exit,
        RuntimePreviewLoopExit::ShutdownRequested {
            session_id: Some(session_id),
        }
    );

    Ok(())
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
