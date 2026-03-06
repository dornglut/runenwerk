    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_runtime_disconnect_connection_targets_only_one_peer() {
        let transport = QuicTransport::default();
        let server_handle = transport
            .spawn_server_runtime(
                default_client_bind_addr(),
                "localhost",
                ServerSessionConfig {
                    server_id: "srv-local".to_string(),
                    protocol: ProtocolVersion::new(1, 1, 1),
                    tick_rate_hz: 60,
                },
            )
            .expect("server runtime should spawn");
        let local_addr = server_handle.local_addr;
        let server_cert = server_handle.certificate.clone();
        let fingerprint = server_handle.certificate_fingerprint_sha256.clone();
        let server_name = server_handle.server_name.clone();
        let (server_tx, mut server_events) = server_handle.into_channels();

        let spawn_client = |ticket: &str| {
            transport
                .spawn_client_runtime(
                    default_client_bind_addr(),
                    &server_name,
                    ClientSessionTarget {
                        server_id: "srv-local".to_string(),
                        server_endpoint: local_addr.to_string(),
                        transport: TransportKind::Quic,
                        protocol: ProtocolVersion::new(1, 1, 1),
                        server_cert_fingerprint_sha256: fingerprint.clone(),
                        ticket: ticket.to_string(),
                    },
                    QuicTrustPolicy::PinnedServer {
                        expected_fingerprint_sha256: fingerprint.clone(),
                        trusted_certificates: vec![server_cert.clone()],
                    },
                )
                .expect("client runtime should spawn")
        };

        let first_client = spawn_client("ticket-a");
        let second_client = spawn_client("ticket-b");
        let (first_client_tx, mut first_client_events) = first_client.into_channels();
        let (second_client_tx, mut second_client_events) = second_client.into_channels();

        let mut first_join = None;
        let mut second_join = None;
        for _ in 0..40 {
            if first_join.is_none()
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    first_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                first_join = Some(join.connection_id);
            }
            if second_join.is_none()
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    second_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                second_join = Some(join.connection_id);
            }
            if first_join.is_some() && second_join.is_some() {
                break;
            }
        }
        assert!(first_join.is_some(), "first client should join");
        assert!(second_join.is_some(), "second client should join");
        assert_ne!(first_join, second_join);

        let disconnect_connection_id = first_join.expect("first join connection id");
        server_tx
            .send(QuicSessionCommand::DisconnectConnection {
                connection_id: ConnectionId(disconnect_connection_id),
                reason: DisconnectReason::TimedOut,
            })
            .expect("targeted disconnect should send");

        let mut first_disconnected = false;
        for _ in 0..30 {
            if let Ok(event) = tokio::time::timeout(
                std::time::Duration::from_millis(250),
                first_client_events.recv(),
            )
            .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::ConnectionClosed {
                        reason: Some(DisconnectReason::TimedOut),
                        ..
                    }
                    | QuicSessionEvent::ServerMessage(ServerMessage::Disconnect(
                        DisconnectReason::TimedOut,
                    )) => {
                        first_disconnected = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
        assert!(
            first_disconnected,
            "disconnected client should observe timed-out disconnect"
        );

        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::Snapshot(
                Snapshot {
                    tick: engine_net::SimulationTick(99),
                    cursor: SnapshotCursor(9),
                    last_applied: SnapshotCursor(0),
                    entity_ids: Vec::new(),
                    payload: vec![7, 7, 7],
                },
            )))
            .expect("server snapshot should send");
        second_client_tx
            .send(QuicSessionCommand::Client(ClientMessage::InputFrame(
                InputFrame {
                    tick: engine_net::SimulationTick(99),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 0.5, y: 0.0 })],
                },
            )))
            .expect("second client input should send");

        let mut second_saw_snapshot = false;
        let mut server_saw_second_input = false;
        for _ in 0..40 {
            if !second_saw_snapshot
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    second_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::ServerMessage(ServerMessage::Snapshot(_))) = event
            {
                second_saw_snapshot = true;
            }
            if !server_saw_second_input
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    server_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::ClientMessage {
                    connection_id: Some(ConnectionId(id)),
                    message: ClientMessage::InputFrame(_),
                }) = event
                && Some(id) == second_join
            {
                server_saw_second_input = true;
            }
            if second_saw_snapshot && server_saw_second_input {
                break;
            }
        }
        assert!(
            second_saw_snapshot,
            "remaining client should still receive snapshots"
        );
        assert!(
            server_saw_second_input,
            "server should still receive input from remaining client"
        );

        first_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("first client shutdown should send");
        second_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("second client shutdown should send");
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn client_runtime_reconnects_after_server_disconnect() {
        let transport = QuicTransport::default();
        let server_handle = transport
            .spawn_server_runtime(
                default_client_bind_addr(),
                "localhost",
                ServerSessionConfig {
                    server_id: "srv-local".to_string(),
                    protocol: ProtocolVersion::new(1, 1, 1),
                    tick_rate_hz: 60,
                },
            )
            .expect("server runtime should spawn");
        let local_addr = server_handle.local_addr;
        let server_cert = server_handle.certificate.clone();
        let fingerprint = server_handle.certificate_fingerprint_sha256.clone();
        let server_name = server_handle.server_name.clone();
        let (server_tx, _server_events) = server_handle.into_channels();

        let client_handle = transport
            .spawn_client_runtime(
                default_client_bind_addr(),
                &server_name,
                ClientSessionTarget {
                    server_id: "srv-local".to_string(),
                    server_endpoint: local_addr.to_string(),
                    transport: TransportKind::Quic,
                    protocol: ProtocolVersion::new(1, 1, 1),
                    server_cert_fingerprint_sha256: fingerprint.clone(),
                    ticket: "ticket-1".to_string(),
                },
                QuicTrustPolicy::PinnedServer {
                    expected_fingerprint_sha256: fingerprint,
                    trusted_certificates: vec![server_cert],
                },
            )
            .expect("client runtime should spawn");
        let (client_tx, mut client_events) = client_handle.into_channels();

        let mut first_join = None;
        for _ in 0..20 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                first_join = Some(join.connection_id);
                break;
            }
        }
        assert_eq!(first_join, Some(1));

        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::Disconnect(
                engine_net::DisconnectReason::TimedOut,
            )))
            .expect("server disconnect should send");

        let mut saw_reconnecting = false;
        let mut second_join = None;
        for _ in 0..80 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::Reconnecting { attempt } => {
                        saw_reconnecting = saw_reconnecting || attempt >= 1;
                    }
                    QuicSessionEvent::JoinAccepted(join) => {
                        if join.connection_id > 1 {
                            second_join = Some(join.connection_id);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        assert!(saw_reconnecting, "client should emit reconnect attempts");
        assert_eq!(second_join, Some(2));

        client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("client shutdown should send");
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }
