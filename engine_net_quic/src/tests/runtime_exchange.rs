    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn runtime_handles_exchange_datagrams_after_handshake() {
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

        let mut client_joined = false;
        let mut server_connected = false;
        for _ in 0..20 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
            {
                if matches!(event, Some(QuicSessionEvent::JoinAccepted(_))) {
                    client_joined = true;
                    break;
                }
            }
        }
        for _ in 0..20 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), server_events.recv())
                    .await
            {
                if matches!(event, Some(QuicSessionEvent::Connected { .. })) {
                    server_connected = true;
                    break;
                }
            }
        }
        assert!(client_joined, "client should reach JoinAccepted");
        assert!(
            server_connected,
            "server should accept the client connection"
        );

        client_tx
            .send(QuicSessionCommand::Client(ClientMessage::InputFrame(
                InputFrame {
                    tick: engine_net::SimulationTick(3),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 1.0, y: -1.0 })],
                },
            )))
            .expect("client command should send");
        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::Snapshot(
                Snapshot {
                    tick: engine_net::SimulationTick(3),
                    cursor: SnapshotCursor(1),
                    last_applied: SnapshotCursor(0),
                    entity_ids: Vec::new(),
                    payload: vec![1, 2, 3],
                },
            )))
            .expect("server snapshot should send");
        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::DeltaSnapshot(
                DeltaSnapshot {
                    tick: engine_net::SimulationTick(4),
                    base: SnapshotCursor(0),
                    cursor: SnapshotCursor(1),
                    entity_ids: Vec::new(),
                    payload: vec![4, 5, 6],
                },
            )))
            .expect("server delta should send");
        client_tx
            .send(QuicSessionCommand::Client(ClientMessage::Ack(Ack {
                cursor: SnapshotCursor(1),
                last_received_tick: engine_net::SimulationTick(4),
            })))
            .expect("client ack should send");

        let mut saw_input = false;
        let mut saw_ack = false;
        let mut saw_snapshot = false;
        let mut saw_delta = false;
        for _ in 0..40 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), server_events.recv())
                    .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::ClientMessage {
                        message: ClientMessage::InputFrame(_),
                        ..
                    } => {
                        saw_input = true;
                    }
                    QuicSessionEvent::ClientMessage {
                        message: ClientMessage::Ack(_),
                        ..
                    } => {
                        saw_ack = true;
                    }
                    _ => {}
                }
            }
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::ServerMessage(ServerMessage::Snapshot(_)) => {
                        saw_snapshot = true;
                    }
                    QuicSessionEvent::ServerMessage(ServerMessage::DeltaSnapshot(_)) => {
                        saw_delta = true;
                    }
                    _ => {}
                }
            }
            if saw_input && saw_ack && saw_snapshot && saw_delta {
                break;
            }
        }

        assert!(saw_input, "server should receive input datagrams");
        assert!(saw_ack, "server should receive ack datagrams");
        assert!(
            saw_snapshot,
            "client should receive server snapshot datagrams"
        );
        assert!(saw_delta, "client should receive delta snapshot datagrams");

        client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("client shutdown should send");
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }
