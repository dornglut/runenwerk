// Owner: Grotto Engine Net - QUIC Runtime
#[derive(Copy, Clone, PartialEq, Eq)]
enum LoopOutcome {
    Shutdown,
    ConnectionClosed,
}

async fn run_live_connection_loop(
    connection: Connection,
    _endpoint: Option<Endpoint>,
    command_rx: &mut UnboundedReceiver<QuicSessionCommand>,
    event_tx: UnboundedSender<QuicSessionEvent>,
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
                                let _ = event_tx.send(QuicSessionEvent::ClientMessage {
                                    connection_id: source_connection_id,
                                    message,
                                });
                            }
                            MessageEnvelope::Server(message) => {
                                if let ServerMessage::JoinRejected(JoinRejected { reason }) = &message {
                                    let _ = event_tx.send(QuicSessionEvent::JoinRejected(reason.clone()));
                                }
                                if let ServerMessage::JoinAccepted(join) = &message {
                                    let _ = event_tx.send(QuicSessionEvent::JoinAccepted(join.clone()));
                                }
                                if let ServerMessage::Disconnect(reason) = &message {
                                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                                        connection_id: source_connection_id,
                                        reason: Some(reason.clone()),
                                    });
                                    return Ok(LoopOutcome::ConnectionClosed);
                                }
                                let _ = event_tx.send(QuicSessionEvent::ServerMessage(message));
                            }
                        }
                        let _ = event_tx.send(QuicSessionEvent::RttUpdated {
                            millis: connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                        });
                    }
                    Err(error) => {
                        let message = error.to_string();
                        let reason = parse_join_rejection_reason(&message);
                        let _ = event_tx.send(QuicSessionEvent::Error {
                            message,
                        });
                        let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
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
    _event_tx: &UnboundedSender<QuicSessionEvent>,
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
