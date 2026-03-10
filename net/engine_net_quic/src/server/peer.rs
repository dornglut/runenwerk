use anyhow::{Context, Result};
use engine_net::{MessageEnvelope, ServerMessage, decode_message, encode_message};
use quinn::Connection;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::runtime::event_dispatch::{send_peer_event, send_runtime_event};
use crate::server::runtime::ServerPeerEvent;
use crate::QuicSessionEvent;

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
                        let send_result: Result<()> = (|| {
                            let bytes = encode_message(&MessageEnvelope::Server(message))?;
                            connection
                                .send_datagram(bytes.into())
                                .context("failed to send server datagram")?;
                            Ok(())
                        })();
                        if let Err(error) = send_result {
                            send_runtime_event(&event_tx, QuicSessionEvent::Error {
                                message: error.to_string(),
                            });
                            send_runtime_event(&event_tx, QuicSessionEvent::ConnectionClosed {
                                connection_id: Some(connection_id),
                                reason: disconnect_reason.clone(),
                            });
                            send_peer_event(&peer_event_tx, ServerPeerEvent::Closed {
                                connection_id,
                                reason: disconnect_reason,
                            });
                            return;
                        }
                        if should_close {
                            let close_reason = disconnect_reason
                                .as_ref()
                                .map(|reason| format!("{reason:?}"))
                                .unwrap_or_else(|| "server disconnect".to_string());
                            connection.close(0u32.into(), close_reason.as_bytes());
                            send_runtime_event(&event_tx, QuicSessionEvent::ConnectionClosed {
                                connection_id: Some(connection_id),
                                reason: disconnect_reason.clone(),
                            });
                            send_peer_event(&peer_event_tx, ServerPeerEvent::Closed {
                                connection_id,
                                reason: disconnect_reason,
                            });
                            return;
                        }
                    }
                    None => {
                        connection.close(0u32.into(), b"server peer dropped");
                        send_peer_event(&peer_event_tx, ServerPeerEvent::Closed {
                            connection_id,
                            reason: None,
                        });
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
                                send_runtime_event(&event_tx, QuicSessionEvent::RttUpdated {
                                    millis: connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                                });
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
                        send_runtime_event(&event_tx, QuicSessionEvent::Error {
                            message: error.to_string(),
                        });
                        send_runtime_event(&event_tx, QuicSessionEvent::ConnectionClosed {
                            connection_id: Some(connection_id),
                            reason: None,
                        });
                        send_peer_event(&peer_event_tx, ServerPeerEvent::Closed {
                            connection_id,
                            reason: None,
                        });
                        return;
                    }
                }
            }
        }
    }
}
