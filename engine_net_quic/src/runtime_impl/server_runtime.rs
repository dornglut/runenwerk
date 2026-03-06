// Owner: Grotto Engine Net - QUIC Runtime
async fn run_server_runtime_task(
    _transport: QuicTransport,
    endpoint: Endpoint,
    session_config: ServerSessionConfig,
    verifier: Option<Arc<dyn QuicServerJoinVerifier>>,
    mut command_rx: UnboundedReceiver<QuicSessionCommand>,
    event_tx: UnboundedSender<QuicSessionEvent>,
) -> Result<()> {
    let mut session_state = ServerSessionState::default();
    engine_net::configure_server_session(&mut session_state, session_config);
    let (peer_event_tx, mut peer_event_rx) = unbounded_channel::<ServerPeerEvent>();
    let mut server_peers =
        BTreeMap::<engine_net::ConnectionId, UnboundedSender<ServerMessage>>::new();
    let mut drain_mode = false;
    loop {
        tokio::select! {
            biased;
            command = command_rx.recv() => {
                match command {
                    Some(QuicSessionCommand::Shutdown) | None => {
                        endpoint.close(0u32.into(), b"shutdown");
                        let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                            connection_id: None,
                            reason: None,
                        });
                        return Ok(());
                    }
                    Some(QuicSessionCommand::Server(message)) => {
                        let stale_connections = server_peers
                            .iter()
                            .filter_map(|(connection_id, sender)| {
                                sender.send(message.clone()).err().map(|_| *connection_id)
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
                            let _ = sender.send(ServerMessage::Disconnect(reason));
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
                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                        connection_id: None,
                        reason: None,
                    });
                    return Ok(());
                };
                let bootstrap =
                    accept_incoming_connection(
                        incoming,
                        &mut session_state,
                        verifier.as_deref(),
                        drain_mode,
                    )
                    .await?;
                let _ = event_tx.send(QuicSessionEvent::Connected {
                    connection_id: bootstrap.state.active_connection,
                });
                let _ = event_tx.send(QuicSessionEvent::Phase(bootstrap.state.phase.clone()));
                if let SessionPhase::Rejected(reason) = bootstrap.state.phase.clone() {
                    let _ = event_tx.send(QuicSessionEvent::JoinRejected(reason.clone()));
                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                        connection_id: None,
                        reason: Some(reason.clone()),
                    });
                    let close_reason = format!("{reason:?}");
                    let rejected_connection = bootstrap.connection;
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                        rejected_connection.close(0u32.into(), close_reason.as_bytes());
                    });
                    continue;
                }
                if let Some(connection_id) = bootstrap.state.active_connection {
                    let _ = event_tx.send(QuicSessionEvent::JoinAccepted(JoinAccepted {
                        connection_id: connection_id.0,
                        tick_rate_hz: bootstrap.state.config.tick_rate_hz,
                        join_state: bootstrap.state.last_join_state.clone().unwrap_or_default(),
                    }));
                    let _ = event_tx.send(QuicSessionEvent::RttUpdated {
                        millis: bootstrap.connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                    });
                    let (peer_tx, peer_rx) = unbounded_channel();
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
enum ServerPeerEvent {
    Closed {
        connection_id: engine_net::ConnectionId,
        reason: Option<DisconnectReason>,
    },
}
