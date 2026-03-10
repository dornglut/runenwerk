use anyhow::Result;
use engine_net::{
    DisconnectReason,
    JoinAccepted,
    ServerMessage,
    ServerSessionConfig,
    ServerSessionState,
    SessionPhase,
};
use quinn::Endpoint;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::runtime::event_dispatch::send_runtime_event;
use crate::server::accept::accept_incoming_connection;
use crate::server::peer::run_server_peer_task;
use crate::{QuicServerJoinVerifier, QuicSessionCommand, QuicSessionEvent, QuicTransport};

// Owner: Grotto Engine Net - QUIC Runtime
const ADMISSION_RATE_LIMIT_WINDOW_SECONDS: u64 = 1;
const ADMISSION_RATE_LIMIT_MAX_ATTEMPTS: usize = 32;
const SERVER_PEER_EVENT_CHANNEL_CAPACITY: usize = 512;
const SERVER_PEER_OUTBOX_CHANNEL_CAPACITY: usize = 256;

pub async fn run_server_runtime_task(
    _transport: QuicTransport,
    endpoint: Endpoint,
    session_config: ServerSessionConfig,
    verifier: Option<Arc<dyn QuicServerJoinVerifier>>,
    mut command_rx: Receiver<QuicSessionCommand>,
    event_tx: Sender<QuicSessionEvent>,
) -> Result<()> {
    let mut session_state = ServerSessionState::default();
    engine_net::configure_server_session(&mut session_state, session_config);
    let (peer_event_tx, mut peer_event_rx) = channel::<ServerPeerEvent>(SERVER_PEER_EVENT_CHANNEL_CAPACITY);
    let mut server_peers = BTreeMap::<engine_net::ConnectionId, Sender<ServerMessage>>::new();
    let mut drain_mode = false;
    let mut admission_limiter = AdmissionRateLimiter::new(
        std::time::Duration::from_secs(ADMISSION_RATE_LIMIT_WINDOW_SECONDS),
        ADMISSION_RATE_LIMIT_MAX_ATTEMPTS,
    );
    loop {
        tokio::select! {
            biased;
            command = command_rx.recv() => {
                match command {
                    Some(QuicSessionCommand::Shutdown) | None => {
                        endpoint.close(0u32.into(), b"shutdown");
                        send_runtime_event(&event_tx, QuicSessionEvent::ConnectionClosed {
                            connection_id: None,
                            reason: None,
                        });
                        return Ok(());
                    }
                    Some(QuicSessionCommand::Server(message)) => {
                        let stale_connections = server_peers
                            .iter()
                            .filter_map(|(connection_id, sender)| {
                                sender.try_send(message.clone()).err().map(|_| *connection_id)
                            })
                            .collect::<Vec<_>>();
                        for connection_id in stale_connections {
                            server_peers.remove(&connection_id);
                            engine_net::remove_server_connection(&mut session_state, connection_id, None);
                        }
                    }
                    Some(QuicSessionCommand::Client(_)) => {}
                    Some(QuicSessionCommand::SetDrainMode { enabled }) => {
                        drain_mode = enabled;
                    }
                    Some(QuicSessionCommand::DisconnectConnection {
                        connection_id,
                        reason,
                    }) => {
                        if let Some(sender) = server_peers.get(&connection_id) {
                            let _ = sender.try_send(ServerMessage::Disconnect(reason));
                        }
                    }
                }
            }
            peer_event = peer_event_rx.recv() => {
                if let Some(ServerPeerEvent::Closed { connection_id, reason }) = peer_event {
                    server_peers.remove(&connection_id);
                    engine_net::remove_server_connection(&mut session_state, connection_id, reason);
                }
            }
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else {
                    send_runtime_event(&event_tx, QuicSessionEvent::ConnectionClosed {
                        connection_id: None,
                        reason: None,
                    });
                    return Ok(());
                };
                let rate_limited = admission_limiter.should_reject(std::time::Instant::now());
                let forced_rejection_reason = if drain_mode {
                    Some(DisconnectReason::ServerShuttingDown)
                } else if rate_limited {
                    Some(DisconnectReason::TimedOut)
                } else {
                    None
                };
                let bootstrap =
                    accept_incoming_connection(
                        incoming,
                        &mut session_state,
                        verifier.as_deref(),
                        forced_rejection_reason,
                    )
                    .await?;
                send_runtime_event(&event_tx, QuicSessionEvent::Phase(bootstrap.state.phase.clone()));
                if let SessionPhase::Rejected(reason) = bootstrap.state.phase.clone() {
                    send_runtime_event(&event_tx, QuicSessionEvent::JoinRejected(reason.clone()));
                    let close_reason = format!("{reason:?}");
                    let rejected_connection = bootstrap.connection;
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                        rejected_connection.close(0u32.into(), close_reason.as_bytes());
                    });
                    continue;
                }
                if let Some(connection_id) = bootstrap.state.active_connection {
                    send_runtime_event(&event_tx, QuicSessionEvent::Connected {
                        connection_id: Some(connection_id),
                    });
                    send_runtime_event(&event_tx, QuicSessionEvent::JoinAccepted(JoinAccepted {
                        connection_id: connection_id.0,
                        tick_rate_hz: bootstrap.state.config.tick_rate_hz,
                        join_state: bootstrap.state.last_join_state.clone().unwrap_or_default(),
                    }));
                    send_runtime_event(&event_tx, QuicSessionEvent::RttUpdated {
                        millis: bootstrap.connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                    });
                    let (peer_tx, peer_rx) = channel(SERVER_PEER_OUTBOX_CHANNEL_CAPACITY);
                    server_peers.insert(connection_id, peer_tx);
                    tokio::spawn(run_server_peer_task(
                        bootstrap.connection,
                        connection_id,
                        peer_rx,
                        event_tx.clone(),
                        peer_event_tx.clone(),
                    ));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdmissionRateLimiter {
    window: std::time::Duration,
    max_attempts: usize,
    attempts: std::collections::VecDeque<std::time::Instant>,
}

impl AdmissionRateLimiter {
    fn new(window: std::time::Duration, max_attempts: usize) -> Self {
        Self {
            window,
            max_attempts: max_attempts.max(1),
            attempts: std::collections::VecDeque::new(),
        }
    }

    fn should_reject(&mut self, now: std::time::Instant) -> bool {
        while let Some(oldest) = self.attempts.front().copied() {
            if now.duration_since(oldest) >= self.window {
                self.attempts.pop_front();
            } else {
                break;
            }
        }
        if self.attempts.len() >= self.max_attempts {
            return true;
        }
        self.attempts.push_back(now);
        false
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ServerPeerEvent {
    Closed {
        connection_id: engine_net::ConnectionId,
        reason: Option<DisconnectReason>,
    },
}

#[cfg(test)]
mod server_runtime_tests {
    use super::AdmissionRateLimiter;

    #[test]
    fn admission_rate_limiter_rejects_after_limit_and_recovers_after_window() {
        let now = std::time::Instant::now();
        let mut limiter = AdmissionRateLimiter::new(std::time::Duration::from_millis(50), 2);
        assert!(!limiter.should_reject(now));
        assert!(!limiter.should_reject(now + std::time::Duration::from_millis(1)));
        assert!(limiter.should_reject(now + std::time::Duration::from_millis(2)));
        assert!(!limiter.should_reject(now + std::time::Duration::from_millis(60)));
    }
}
