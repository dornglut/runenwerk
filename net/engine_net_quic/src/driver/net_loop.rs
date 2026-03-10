use anyhow::{Context, Result};
use engine_net::{ConnectionId, JoinRejected, MessageEnvelope, ServerMessage, decode_message};
use quinn::{Connection, Endpoint};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::runtime::event_dispatch::send_runtime_event;
use crate::runtime::join_rejection::parse_join_rejection_reason;
use crate::runtime::message_transport::{receive_stream_envelope, send_envelope};
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
    source_connection_id: Option<ConnectionId>,
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
                        if handle_incoming_envelope(&event_tx, source_connection_id, envelope) {
                            return Ok(LoopOutcome::ConnectionClosed);
                        }
                        emit_rtt_update(&event_tx, &connection);
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
            incoming = receive_stream_envelope(&connection) => {
                match incoming {
                    Ok(Some(envelope)) => {
                        if handle_incoming_envelope(&event_tx, source_connection_id, envelope) {
                            return Ok(LoopOutcome::ConnectionClosed);
                        }
                        emit_rtt_update(&event_tx, &connection);
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let message = error.to_string();
                        let reason = parse_join_rejection_reason(&message);
                        send_runtime_event(&event_tx, QuicSessionEvent::Error { message });
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

fn handle_incoming_envelope(
    event_tx: &Sender<QuicSessionEvent>,
    source_connection_id: Option<ConnectionId>,
    envelope: MessageEnvelope,
) -> bool {
    match envelope {
        MessageEnvelope::Client(message) => {
            send_runtime_event(
                event_tx,
                QuicSessionEvent::ClientMessage {
                    connection_id: source_connection_id,
                    message,
                },
            );
            false
        }
        MessageEnvelope::Server(message) => {
            if let ServerMessage::JoinRejected(JoinRejected { reason }) = &message {
                send_runtime_event(event_tx, QuicSessionEvent::JoinRejected(reason.clone()));
            }
            if let ServerMessage::JoinAccepted(join) = &message {
                send_runtime_event(event_tx, QuicSessionEvent::JoinAccepted(join.clone()));
            }
            if let ServerMessage::Disconnect(reason) = &message {
                send_runtime_event(
                    event_tx,
                    QuicSessionEvent::ConnectionClosed {
                        connection_id: source_connection_id,
                        reason: Some(reason.clone()),
                    },
                );
                return true;
            }
            send_runtime_event(event_tx, QuicSessionEvent::ServerMessage(message));
            false
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

async fn dispatch_runtime_command(
    connection: &Connection,
    _event_tx: &Sender<QuicSessionEvent>,
    command: QuicSessionCommand,
) -> Result<bool> {
    match command {
        QuicSessionCommand::Client(message) => {
            send_envelope(connection, &MessageEnvelope::Client(message))
                .await
                .context("failed to send client message")?;
            Ok(false)
        }
        _ => Ok(false),
    }
}
