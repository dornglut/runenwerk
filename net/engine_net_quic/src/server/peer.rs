use anyhow::Result;
use engine_net::{DisconnectReason, MessageEnvelope, ServerMessage, decode_message};
use quinn::Connection;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::QuicSessionEvent;
use crate::runtime::event_dispatch::{send_peer_event, send_runtime_event};
use crate::runtime::message_transport::{receive_stream_envelope, send_envelope};
use crate::server::runtime::ServerPeerEvent;

// Owner: Grotto Engine Net - QUIC Runtime
pub(crate) async fn run_server_peer_task(
    connection: Connection,
    connection_id: engine_net::ConnectionId,
    mut message_rx: Receiver<ServerMessage>,
    event_tx: Sender<QuicSessionEvent>,
    peer_event_tx: Sender<ServerPeerEvent>,
) {
    loop {
        tokio::select! {
            message = message_rx.recv() => {
                match message {
                    Some(message) => {
                        let should_close = matches!(message, ServerMessage::Disconnect(_));
                        let disconnect_reason = if let ServerMessage::Disconnect(reason) = &message {
                            Some(reason.clone())
                        } else {
                            None
                        };
                        let send_result: Result<()> = send_envelope(
                            &connection,
                            &MessageEnvelope::Server(message),
                        )
                        .await;
                        if let Err(error) = send_result {
                            send_runtime_event(&event_tx, QuicSessionEvent::Error {
                                message: error.to_string(),
                            });
                            emit_peer_closed(
                                &event_tx,
                                &peer_event_tx,
                                connection_id,
                                disconnect_reason,
                            );
                            return;
                        }
                        if should_close {
                            let close_reason = disconnect_reason
                                .as_ref()
                                .map(|reason| format!("{reason:?}"))
                                .unwrap_or_else(|| "server disconnect".to_string());
                            connection.close(0u32.into(), close_reason.as_bytes());
                            emit_peer_closed(
                                &event_tx,
                                &peer_event_tx,
                                connection_id,
                                disconnect_reason,
                            );
                            return;
                        }
                    }
                    None => {
                        connection.close(0u32.into(), b"server peer dropped");
                        emit_peer_closed(&event_tx, &peer_event_tx, connection_id, None);
                        return;
                    }
                }
            }
            incoming = connection.read_datagram() => {
                match incoming {
                    Ok(bytes) => {
                        match decode_message::<MessageEnvelope>(&bytes) {
                            Ok(MessageEnvelope::Client(message)) => {
                                send_runtime_event(&event_tx, QuicSessionEvent::ClientMessage {
                                    connection_id: Some(connection_id),
                                    message,
                                });
                                emit_rtt_update(&event_tx, &connection);
                            }
                            Ok(MessageEnvelope::Server(_)) => {}
                            Err(error) => {
                                send_runtime_event(&event_tx, QuicSessionEvent::Error {
                                    message: error.to_string(),
                                });
                            }
                        }
                    }
                    Err(error) => {
                        let reason = map_runtime_disconnect_reason(&error.to_string());
                        send_runtime_event(&event_tx, QuicSessionEvent::Error {
                            message: error.to_string(),
                        });
                        emit_peer_closed(&event_tx, &peer_event_tx, connection_id, reason);
                        return;
                    }
                }
            }
            incoming = receive_stream_envelope(&connection) => {
                match incoming {
                    Ok(Some(MessageEnvelope::Client(message))) => {
                        send_runtime_event(&event_tx, QuicSessionEvent::ClientMessage {
                            connection_id: Some(connection_id),
                            message,
                        });
                        emit_rtt_update(&event_tx, &connection);
                    }
                    Ok(Some(MessageEnvelope::Server(_))) => {}
                    Ok(None) => {}
                    Err(error) => {
                        let reason = map_runtime_disconnect_reason(&error.to_string());
                        send_runtime_event(&event_tx, QuicSessionEvent::Error {
                            message: error.to_string(),
                        });
                        emit_peer_closed(&event_tx, &peer_event_tx, connection_id, reason);
                        return;
                    }
                }
            }
        }
    }
}

fn emit_rtt_update(event_tx: &Sender<QuicSessionEvent>, connection: &Connection) {
    send_runtime_event(
        event_tx,
        QuicSessionEvent::RttUpdated {
            millis: connection.rtt().as_millis().min(u32::MAX as u128) as u32,
        },
    );
}

fn emit_peer_closed(
    event_tx: &Sender<QuicSessionEvent>,
    peer_event_tx: &Sender<ServerPeerEvent>,
    connection_id: engine_net::ConnectionId,
    reason: Option<DisconnectReason>,
) {
    send_runtime_event(
        event_tx,
        QuicSessionEvent::ConnectionClosed {
            connection_id: Some(connection_id),
            reason: reason.clone(),
        },
    );
    send_peer_event(
        peer_event_tx,
        ServerPeerEvent::Closed {
            connection_id,
            reason,
        },
    );
}

fn map_runtime_disconnect_reason(message: &str) -> Option<DisconnectReason> {
    let message = message.to_ascii_lowercase();
    if message.contains("timed out") || message.contains("timeout") {
        Some(DisconnectReason::TimedOut)
    } else if message.contains("server shutting down") {
        Some(DisconnectReason::ServerShuttingDown)
    } else {
        None
    }
}
