// Owner: Grotto Engine Net - QUIC Runtime
async fn run_server_peer_task(
    connection: Connection,
    connection_id: engine_net::ConnectionId,
    mut message_rx: UnboundedReceiver<ServerMessage>,
    event_tx: UnboundedSender<QuicSessionEvent>,
    peer_event_tx: UnboundedSender<ServerPeerEvent>,
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
                            let _ = event_tx.send(QuicSessionEvent::Error {
                                message: error.to_string(),
                            });
                            let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                                connection_id: Some(connection_id),
                                reason: disconnect_reason.clone(),
                            });
                            let _ = peer_event_tx.send(ServerPeerEvent::Closed {
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
                            let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                                connection_id: Some(connection_id),
                                reason: disconnect_reason.clone(),
                            });
                            let _ = peer_event_tx.send(ServerPeerEvent::Closed {
                                connection_id,
                                reason: disconnect_reason,
                            });
                            return;
                        }
                    }
                    None => {
                        connection.close(0u32.into(), b"server peer dropped");
                        let _ = peer_event_tx.send(ServerPeerEvent::Closed {
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
                                let _ = event_tx.send(QuicSessionEvent::ClientMessage {
                                    connection_id: Some(connection_id),
                                    message,
                                });
                                let _ = event_tx.send(QuicSessionEvent::RttUpdated {
                                    millis: connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                                });
                            }
                            Ok(MessageEnvelope::Server(_)) => {}
                            Err(error) => {
                                let _ = event_tx.send(QuicSessionEvent::Error {
                                    message: error.to_string(),
                                });
                            }
                        }
                    }
                    Err(error) => {
                        let _ = event_tx.send(QuicSessionEvent::Error {
                            message: error.to_string(),
                        });
                        let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                            connection_id: Some(connection_id),
                            reason: None,
                        });
                        let _ = peer_event_tx.send(ServerPeerEvent::Closed {
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
