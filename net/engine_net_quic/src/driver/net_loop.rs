use anyhow::{Context, Result};
use engine_net::{JoinRejected, MessageEnvelope, ServerMessage, decode_message, encode_message};
use quinn::{Connection, Endpoint};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::runtime::helpers::{parse_join_rejection_reason, send_runtime_event};
use crate::{QuicSessionCommand, QuicSessionEvent};

// Owner: Grotto Engine Net - QUIC Runtime
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum LoopOutcome {
    Shutdown,
    ConnectionClosed,
}

pub(crate) async fn run_live_connection_loop(
    connection: Connection,
    _endpoint: Option<Endpoint>,
    command_rx: &mut Receiver<QuicSessionCommand>,
    event_tx: Sender<QuicSessionEvent>,
    source_connection_id: Option<engine_net::ConnectionId>,
    pending_commands: &mut Vec<QuicSessionCommand>,
) -> Result<LoopOutcome> {
    for command in pending_commands.drain(..) {
        if dispatch_runtime_command(&connection, &event_tx, command).await? {
            return Ok(LoopOutcome::ConnectionClosed);
        }
    }
    loop {
        tokio::select! {
            command = command_rx.recv() => {
                match command {
                    Some(QuicSessionCommand::Shutdown) | None => {
                        connection.close(0u32.into(), b"shutdown");
                        return Ok(LoopOutcome::Shutdown);
                    }
                    Some(command) => {
                        if dispatch_runtime_command(&connection, &event_tx, command).await? {
                            return Ok(LoopOutcome::ConnectionClosed);
                        }
                    }
                }
            }
            incoming = connection.read_datagram() => {
                match incoming {
                    Ok(bytes) => {
                        let envelope: MessageEnvelope = decode_message(&bytes)?;
                        match envelope {
                            MessageEnvelope::Client(message) => {
                                send_runtime_event(&event_tx, QuicSessionEvent::ClientMessage {
                                    connection_id: source_connection_id,
                                    message,
                                });
                            }
                            MessageEnvelope::Server(message) => {
                                if let ServerMessage::JoinRejected(JoinRejected { reason }) = &message {
                                    send_runtime_event(&event_tx, QuicSessionEvent::JoinRejected(reason.clone()));
                                }
                                if let ServerMessage::JoinAccepted(join) = &message {
                                    send_runtime_event(&event_tx, QuicSessionEvent::JoinAccepted(join.clone()));
                                }
                                if let ServerMessage::Disconnect(reason) = &message {
                                    send_runtime_event(&event_tx, QuicSessionEvent::ConnectionClosed {
                                        connection_id: source_connection_id,
                                        reason: Some(reason.clone()),
                                    });
                                    return Ok(LoopOutcome::ConnectionClosed);
                                }
                                send_runtime_event(&event_tx, QuicSessionEvent::ServerMessage(message));
                            }
                        }
                        send_runtime_event(&event_tx, QuicSessionEvent::RttUpdated {
                            millis: connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                        });
                    }
                    Err(error) => {
                        let message = error.to_string();
                        let reason = parse_join_rejection_reason(&message);
                        send_runtime_event(&event_tx, QuicSessionEvent::Error {
                            message,
                        });
                        send_runtime_event(&event_tx, QuicSessionEvent::ConnectionClosed {
                            connection_id: source_connection_id,
                            reason,
                        });
                        return Ok(LoopOutcome::ConnectionClosed);
                    }
                }
            }
        }
    }
}

async fn dispatch_runtime_command(
    connection: &Connection,
    _event_tx: &Sender<QuicSessionEvent>,
    command: QuicSessionCommand,
) -> Result<bool> {
    match command {
        QuicSessionCommand::Client(message) => {
            let bytes = encode_message(&MessageEnvelope::Client(message))?;
            connection
                .send_datagram(bytes.into())
                .context("failed to send client datagram")?;
            Ok(false)
        }
        _ => Ok(false),
    }
}
