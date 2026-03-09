use anyhow::{Context, Result};
use engine_net::ClientSessionTarget;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::driver::net_loop::{LoopOutcome, run_live_connection_loop};
use crate::runtime::helpers::{
    parse_join_rejection_reason, send_runtime_event, wait_for_reconnect_backoff,
};
use crate::{
    QuicClientTargetProvider,
    QuicSessionCommand,
    QuicSessionEvent,
    QuicTransport,
    QuicTrustPolicy,
};

// Owner: Grotto Engine Net - QUIC Runtime
pub async fn run_client_runtime_task(
    transport: QuicTransport,
    bind_addr: SocketAddr,
    server_name: String,
    target: ClientSessionTarget,
    trust_policy: QuicTrustPolicy,
    target_provider: Option<Arc<dyn QuicClientTargetProvider>>,
    mut command_rx: Receiver<QuicSessionCommand>,
    event_tx: Sender<QuicSessionEvent>,
) -> Result<()> {
    let mut current_target = target;
    let mut current_trust_policy = trust_policy.retargeted_for(&current_target);
    let mut reconnect_attempt = 0u32;
    let mut pending_commands = Vec::new();

    loop {
        if reconnect_attempt > 0 {
            send_runtime_event(
                &event_tx,
                QuicSessionEvent::Reconnecting {
                    attempt: reconnect_attempt,
                },
            );
            if let Some(provider) = &target_provider {
                match provider.refresh_target(&current_target).await {
                    Ok(refreshed_target) => {
                        current_target = refreshed_target;
                        current_trust_policy = current_trust_policy.retargeted_for(&current_target);
                    }
                    Err(error) => {
                        send_runtime_event(
                            &event_tx,
                            QuicSessionEvent::Error {
                                message: format!("failed to refresh join grant: {error}"),
                            },
                        );
                        reconnect_attempt = reconnect_attempt.saturating_add(1);
                        if !wait_for_reconnect_backoff(&mut command_rx, &mut pending_commands)
                            .await?
                        {
                            send_runtime_event(
                                &event_tx,
                                QuicSessionEvent::ConnectionClosed {
                                    connection_id: None,
                                    reason: None,
                                },
                            );
                            return Ok(());
                        }
                        continue;
                    }
                }
            }
        }
        current_trust_policy.validate_expected_fingerprint()?;
        let trusted_certificates = current_trust_policy.trusted_certificates()?;
        let server_addr: SocketAddr = current_target.server_endpoint.parse().with_context(|| {
            format!("invalid server endpoint: {}", current_target.server_endpoint)
        })?;

        match transport
            .connect_and_handshake(
                bind_addr,
                server_addr,
                &server_name,
                &trusted_certificates,
                current_target.clone(),
            )
            .await
        {
            Ok(bootstrap) => {
                send_runtime_event(
                    &event_tx,
                    QuicSessionEvent::Connected {
                        connection_id: bootstrap.state.connection_id,
                    },
                );
                send_runtime_event(&event_tx, QuicSessionEvent::Phase(bootstrap.state.phase.clone()));
                send_runtime_event(
                    &event_tx,
                    QuicSessionEvent::JoinAccepted(bootstrap.accepted.clone()),
                );
                send_runtime_event(
                    &event_tx,
                    QuicSessionEvent::RttUpdated {
                        millis: bootstrap.connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                    },
                );
                reconnect_attempt = 0;
                let outcome = run_live_connection_loop(
                    bootstrap.connection,
                    Some(bootstrap.endpoint),
                    &mut command_rx,
                    event_tx.clone(),
                    bootstrap.state.connection_id,
                    &mut pending_commands,
                )
                .await?;
                match outcome {
                    LoopOutcome::Shutdown => return Ok(()),
                    LoopOutcome::ConnectionClosed => {
                        reconnect_attempt = reconnect_attempt.saturating_add(1);
                    }
                }
            }
            Err(error) => {
                let message = error.to_string();
                if let Some(reason) = parse_join_rejection_reason(&message) {
                    send_runtime_event(&event_tx, QuicSessionEvent::JoinRejected(reason.clone()));
                    send_runtime_event(
                        &event_tx,
                        QuicSessionEvent::ConnectionClosed {
                            connection_id: None,
                            reason: Some(reason),
                        },
                    );
                    return Ok(());
                }
                send_runtime_event(&event_tx, QuicSessionEvent::Error { message });
                reconnect_attempt = reconnect_attempt.saturating_add(1);
                if !wait_for_reconnect_backoff(&mut command_rx, &mut pending_commands).await? {
                    send_runtime_event(
                        &event_tx,
                        QuicSessionEvent::ConnectionClosed {
                            connection_id: None,
                            reason: None,
                        },
                    );
                    return Ok(());
                }
            }
        }
    }
}
