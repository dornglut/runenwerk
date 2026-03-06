// Owner: Grotto Engine Net - QUIC Runtime
async fn run_client_runtime_task(
    transport: QuicTransport,
    bind_addr: SocketAddr,
    server_name: String,
    target: ClientSessionTarget,
    trust_policy: QuicTrustPolicy,
    target_provider: Option<Arc<dyn QuicClientTargetProvider>>,
    mut command_rx: UnboundedReceiver<QuicSessionCommand>,
    event_tx: UnboundedSender<QuicSessionEvent>,
) -> Result<()> {
    let mut current_target = target;
    let mut current_trust_policy = trust_policy.retargeted_for(&current_target);
    let mut reconnect_attempt = 0u32;
    let mut pending_commands = Vec::new();

    loop {
        if reconnect_attempt > 0 {
            let _ = event_tx.send(QuicSessionEvent::Reconnecting {
                attempt: reconnect_attempt,
            });
            if let Some(provider) = &target_provider {
                match provider.refresh_target(&current_target).await {
                    Ok(refreshed_target) => {
                        current_target = refreshed_target;
                        current_trust_policy = current_trust_policy.retargeted_for(&current_target);
                    }
                    Err(error) => {
                        let _ = event_tx.send(QuicSessionEvent::Error {
                            message: format!("failed to refresh join grant: {error}"),
                        });
                        reconnect_attempt = reconnect_attempt.saturating_add(1);
                        if !wait_for_reconnect_backoff(&mut command_rx, &mut pending_commands)
                            .await?
                        {
                            let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                                connection_id: None,
                                reason: None,
                            });
                            return Ok(());
                        }
                        continue;
                    }
                }
            }
        }
        current_trust_policy.validate_expected_fingerprint()?;
        let trusted_certificates = current_trust_policy.trusted_certificates()?;
        let server_addr: SocketAddr =
            current_target.server_endpoint.parse().with_context(|| {
                format!(
                    "invalid server endpoint: {}",
                    current_target.server_endpoint
                )
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
                let _ = event_tx.send(QuicSessionEvent::Connected {
                    connection_id: bootstrap.state.connection_id,
                });
                let _ = event_tx.send(QuicSessionEvent::Phase(bootstrap.state.phase.clone()));
                let _ = event_tx.send(QuicSessionEvent::JoinAccepted(bootstrap.accepted.clone()));
                let _ = event_tx.send(QuicSessionEvent::RttUpdated {
                    millis: bootstrap.connection.rtt().as_millis().min(u32::MAX as u128) as u32,
                });
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
                    let _ = event_tx.send(QuicSessionEvent::JoinRejected(reason.clone()));
                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                        connection_id: None,
                        reason: Some(reason),
                    });
                    return Ok(());
                }
                let _ = event_tx.send(QuicSessionEvent::Error { message });
                reconnect_attempt = reconnect_attempt.saturating_add(1);
                if !wait_for_reconnect_backoff(&mut command_rx, &mut pending_commands).await? {
                    let _ = event_tx.send(QuicSessionEvent::ConnectionClosed {
                        connection_id: None,
                        reason: None,
                    });
                    return Ok(());
                }
            }
        }
    }
}

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

async fn wait_for_reconnect_backoff(
    command_rx: &mut UnboundedReceiver<QuicSessionCommand>,
    pending_commands: &mut Vec<QuicSessionCommand>,
) -> Result<bool> {
    let sleep = tokio::time::sleep(std::time::Duration::from_millis(250));
    tokio::pin!(sleep);
    loop {
        tokio::select! {
            _ = &mut sleep => return Ok(true),
            command = command_rx.recv() => {
                match command {
                    Some(QuicSessionCommand::Shutdown) | None => return Ok(false),
                    Some(command) => pending_commands.push(command),
                }
            }
        }
    }
}

fn parse_join_rejection_reason(message: &str) -> Option<engine_net::DisconnectReason> {
    if message.contains("WrongServer") {
        return Some(engine_net::DisconnectReason::WrongServer);
    }
    if message.contains("VersionMismatch") {
        return Some(engine_net::DisconnectReason::VersionMismatch);
    }
    if message.contains("InvalidTicket") {
        return Some(engine_net::DisconnectReason::InvalidTicket);
    }
    if message.contains("TicketExpired") {
        return Some(engine_net::DisconnectReason::TicketExpired);
    }
    if message.contains("ServerShuttingDown") {
        return Some(engine_net::DisconnectReason::ServerShuttingDown);
    }
    if message.contains("TimedOut") {
        return Some(engine_net::DisconnectReason::TimedOut);
    }
    None
}

async fn accept_incoming_connection(
    incoming: quinn::Incoming,
    state: &mut ServerSessionState,
    verifier: Option<&dyn QuicServerJoinVerifier>,
    drain_mode: bool,
) -> Result<QuicServerBootstrap> {
    state.phase = SessionPhase::Idle;
    state.active_connection = None;
    state.last_join_request = None;
    state.last_join_state = None;
    state.last_disconnect = None;
    let connection = incoming.await.context("server accept failed")?;
    let (mut send, mut recv) = connection.accept_bi().await?;
    while let Some(message) = read_message(&mut recv).await? {
        let MessageEnvelope::Client(client_message) = message else {
            continue;
        };
        if drain_mode && let ClientMessage::JoinRequest(request) = &client_message {
            let reason = DisconnectReason::ServerShuttingDown;
            state.last_join_request = Some(request.clone());
            state.last_join_state = None;
            state.phase = SessionPhase::Rejected(reason.clone());
            state.last_disconnect = Some(reason.clone());
            write_message(
                &mut send,
                &MessageEnvelope::Server(ServerMessage::JoinRejected(JoinRejected { reason })),
            )
            .await?;
            break;
        }
        if let ClientMessage::JoinRequest(request) = &client_message
            && let Some(verifier) = verifier
        {
            match verifier.verify_join_request(request, &state.config).await {
                Ok(join_state) => {
                    state.last_join_state = Some(join_state);
                }
                Err(QuicJoinVerificationError::Rejected(reason)) => {
                    state.last_join_request = Some(request.clone());
                    state.last_join_state = None;
                    state.phase = SessionPhase::Rejected(reason.clone());
                    state.last_disconnect = Some(reason.clone());
                    write_message(
                        &mut send,
                        &MessageEnvelope::Server(ServerMessage::JoinRejected(JoinRejected {
                            reason,
                        })),
                    )
                    .await?;
                    break;
                }
                Err(QuicJoinVerificationError::Other(error)) => return Err(error),
            }
        }
        for response in handle_client_message(state, &client_message) {
            write_message(&mut send, &MessageEnvelope::Server(response)).await?;
        }
        if matches!(state.phase, SessionPhase::Active)
            || matches!(state.phase, SessionPhase::Rejected(_))
        {
            break;
        }
    }
    send.finish()?;
    Ok(QuicServerBootstrap {
        connection,
        state: state.clone(),
    })
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => '0',
    }
}
