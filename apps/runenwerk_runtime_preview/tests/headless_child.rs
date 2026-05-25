use anyhow::{Context, Result, anyhow};
use editor_preview::{
    PreviewBootstrap, PreviewCommand, PreviewCommandEnvelope, PreviewEvent, PreviewEventEnvelope,
    decode_lower_hex, preview_session_id,
};
use engine_net::{ClientMessage, ServerMessage, SessionRuntimeCommand, SessionRuntimeEvent};
use engine_net_quic::{QuicTransport, QuicTrustPolicy, default_client_bind_addr};
use runenwerk_runtime_preview::{
    RuntimePreviewConfig, RuntimePreviewHost, client_target_from_bootstrap, encode_preview_command,
    wait_for_join_acceptance,
};
use rustls::pki_types::CertificateDer;
use std::io::ErrorKind;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn headless_child_process_round_trips_heartbeat_and_shutdown() -> Result<()> {
    if runtime_preview_transport_permission_denied().await? {
        eprintln!(
            "skipping runtime preview headless-child transport test: local socket bind is denied"
        );
        return Ok(());
    }

    let executable = env!("CARGO_BIN_EXE_runenwerk_runtime_preview");
    let mut child = Command::new(executable)
        .arg("--headless")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("runtime preview child should spawn")?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("runtime preview child stdout was not captured"))?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .context("runtime preview child should print bootstrap line")?;
    let bootstrap = PreviewBootstrap::parse_stdout_line(&line)
        .context("runtime preview child bootstrap should parse")?;

    let transport = QuicTransport::default();
    let client = transport.spawn_client_runtime(
        default_client_bind_addr(),
        &bootstrap.server_name,
        client_target_from_bootstrap(&bootstrap),
        QuicTrustPolicy::PinnedServer {
            expected_fingerprint_sha256: bootstrap.certificate_fingerprint_sha256.clone(),
            trusted_certificates: vec![CertificateDer::from(decode_lower_hex(
                &bootstrap.trusted_certificate_der_hex,
            )?)],
        },
    )?;
    let (client_tx, mut client_events) = client.into_channels();
    wait_for_join_acceptance(&mut client_events, std::time::Duration::from_secs(5)).await?;

    send_preview_command(
        &client_tx,
        PreviewCommandEnvelope::new(
            1,
            PreviewCommand::Heartbeat {
                session_id: preview_session_id(1),
            },
        ),
    )
    .await?;
    assert!(matches!(
        next_preview_event(&mut client_events).await?.event,
        PreviewEvent::Heartbeat { .. }
    ));

    send_preview_command(
        &client_tx,
        PreviewCommandEnvelope::new(
            2,
            PreviewCommand::Shutdown {
                session_id: preview_session_id(1),
            },
        ),
    )
    .await?;
    assert!(matches!(
        next_preview_event(&mut client_events).await?.event,
        PreviewEvent::ShutdownAck { .. }
    ));

    wait_for_child_exit(&mut child, std::time::Duration::from_secs(5)).await?;
    Ok(())
}

async fn runtime_preview_transport_permission_denied() -> Result<bool> {
    match RuntimePreviewHost::spawn(RuntimePreviewConfig::headless()) {
        Ok(host) => {
            let _ = host.shutdown().await;
            Ok(false)
        }
        Err(error) if is_permission_denied(&error) => Ok(true),
        Err(error) => Err(error).context("runtime preview transport preflight should spawn"),
    }
}

fn is_permission_denied(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_error| io_error.kind() == ErrorKind::PermissionDenied)
    })
}

async fn send_preview_command(
    client_tx: &tokio::sync::mpsc::Sender<SessionRuntimeCommand>,
    command: PreviewCommandEnvelope,
) -> Result<()> {
    let payload = encode_preview_command(&command)?;
    client_tx
        .send(SessionRuntimeCommand::Client(ClientMessage::TypedPayload(
            payload,
        )))
        .await
        .context("client command channel should stay open")
}

async fn next_preview_event(
    client_events: &mut tokio::sync::mpsc::Receiver<SessionRuntimeEvent>,
) -> Result<PreviewEventEnvelope> {
    let started = std::time::Instant::now();
    loop {
        let remaining = std::time::Duration::from_secs(5)
            .checked_sub(started.elapsed())
            .ok_or_else(|| anyhow!("timed out waiting for preview event"))?;
        let event = tokio::time::timeout(remaining, client_events.recv())
            .await
            .context("timed out waiting for preview event")?
            .ok_or_else(|| anyhow!("client event channel closed"))?;
        if let SessionRuntimeEvent::ServerMessage(ServerMessage::TypedPayload(payload)) = event {
            return Ok(runenwerk_runtime_preview::decode_preview_event(&payload)?);
        }
    }
}

async fn wait_for_child_exit(
    child: &mut std::process::Child,
    timeout: std::time::Duration,
) -> Result<()> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            child.kill()?;
            child.wait()?;
            return Err(anyhow!("runtime preview child did not exit after shutdown"));
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
