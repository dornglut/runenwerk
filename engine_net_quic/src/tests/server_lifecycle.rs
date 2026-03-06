    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_runtime_accepts_a_second_client_after_disconnect() {
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

        let first_client = transport
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
                    expected_fingerprint_sha256: fingerprint.clone(),
                    trusted_certificates: vec![server_cert.clone()],
                },
            )
            .expect("first client runtime should spawn");
        let (first_client_tx, mut first_client_events) = first_client.into_channels();

        let mut first_join = None;
        for _ in 0..20 {
            if let Ok(event) = tokio::time::timeout(
                std::time::Duration::from_millis(250),
                first_client_events.recv(),
            )
            .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                first_join = Some(join.connection_id);
                break;
            }
        }
        assert_eq!(first_join, Some(1));

        first_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("first client shutdown should send");

        let mut saw_close = false;
        for _ in 0..40 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), server_events.recv())
                    .await
                && let Some(QuicSessionEvent::ConnectionClosed { .. }) = event
            {
                saw_close = true;
                break;
            }
        }
        assert!(saw_close, "server should observe the first disconnect");

        let second_client = transport
            .spawn_client_runtime(
                default_client_bind_addr(),
                &server_name,
                ClientSessionTarget {
                    server_id: "srv-local".to_string(),
                    server_endpoint: local_addr.to_string(),
                    transport: TransportKind::Quic,
                    protocol: ProtocolVersion::new(1, 1, 1),
                    server_cert_fingerprint_sha256: fingerprint,
                    ticket: "ticket-2".to_string(),
                },
                QuicTrustPolicy::PinnedServer {
                    expected_fingerprint_sha256: certificate_fingerprint_sha256(&server_cert),
                    trusted_certificates: vec![server_cert],
                },
            )
            .expect("second client runtime should spawn");
        let (second_client_tx, mut second_client_events) = second_client.into_channels();

        let mut second_join = None;
        for _ in 0..20 {
            if let Ok(event) = tokio::time::timeout(
                std::time::Duration::from_millis(250),
                second_client_events.recv(),
            )
            .await
                && let Some(QuicSessionEvent::JoinAccepted(join)) = event
            {
                second_join = Some(join.connection_id);
                break;
            }
        }
        assert_eq!(second_join, Some(2));

        second_client_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("second client shutdown should send");
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn server_runtime_broadcasts_to_multiple_connected_clients() {
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

        let first_client = spawn_client("ticket-1");
        let (first_client_tx, mut first_client_events) = first_client.into_channels();
        let second_client = spawn_client("ticket-2");
        let (second_client_tx, mut second_client_events) = second_client.into_channels();

        let mut first_join = None;
        let mut second_join = None;
        for _ in 0..30 {
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
        assert!(matches!(first_join, Some(1 | 2)));
        assert!(matches!(second_join, Some(1 | 2)));
        assert_ne!(first_join, second_join);

        first_client_tx
            .send(QuicSessionCommand::Client(ClientMessage::InputFrame(
                InputFrame {
                    tick: engine_net::SimulationTick(10),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 1.0, y: 0.0 })],
                },
            )))
            .expect("first client input should send");
        second_client_tx
            .send(QuicSessionCommand::Client(ClientMessage::InputFrame(
                InputFrame {
                    tick: engine_net::SimulationTick(10),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 0.0, y: 1.0 })],
                },
            )))
            .expect("second client input should send");
        server_tx
            .send(QuicSessionCommand::Server(ServerMessage::Snapshot(
                Snapshot {
                    tick: engine_net::SimulationTick(11),
                    cursor: SnapshotCursor(3),
                    last_applied: SnapshotCursor(0),
                    entity_ids: Vec::new(),
                    payload: vec![9, 9, 9],
                },
            )))
            .expect("server snapshot should send");

        let mut saw_input_1 = false;
        let mut saw_input_2 = false;
        for _ in 0..60 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), server_events.recv())
                    .await
                && let Some(event) = event
            {
                match event {
                    QuicSessionEvent::ClientMessage {
                        connection_id: Some(ConnectionId(connection_id)),
                        message: ClientMessage::InputFrame(_),
                    } if Some(connection_id) == first_join => {
                        saw_input_1 = true;
                    }
                    QuicSessionEvent::ClientMessage {
                        connection_id: Some(ConnectionId(connection_id)),
                        message: ClientMessage::InputFrame(_),
                    } if Some(connection_id) == second_join => {
                        saw_input_2 = true;
                    }
                    _ => {}
                }
            }
            if saw_input_1 && saw_input_2 {
                break;
            }
        }
        assert!(saw_input_1, "server should receive input from connection 1");
        assert!(saw_input_2, "server should receive input from connection 2");

        let mut first_saw_snapshot = false;
        let mut second_saw_snapshot = false;
        for _ in 0..40 {
            if !first_saw_snapshot
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    first_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::ServerMessage(ServerMessage::Snapshot(snapshot))) =
                    event
            {
                first_saw_snapshot = snapshot.cursor == SnapshotCursor(3);
            }
            if !second_saw_snapshot
                && let Ok(event) = tokio::time::timeout(
                    std::time::Duration::from_millis(250),
                    second_client_events.recv(),
                )
                .await
                && let Some(QuicSessionEvent::ServerMessage(ServerMessage::Snapshot(snapshot))) =
                    event
            {
                second_saw_snapshot = snapshot.cursor == SnapshotCursor(3);
            }
            if first_saw_snapshot && second_saw_snapshot {
                break;
            }
        }

        assert!(
            first_saw_snapshot,
            "first client should receive broadcast snapshot"
        );
        assert!(
            second_saw_snapshot,
            "second client should receive broadcast snapshot"
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
    async fn server_runtime_drain_mode_rejects_new_join_requests() {
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

        server_tx
            .send(QuicSessionCommand::SetDrainMode { enabled: true })
            .expect("drain mode command should send");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client_handle = transport
            .spawn_client_runtime(
                default_client_bind_addr(),
                &server_name,
                ClientSessionTarget {
                    server_id: "srv-local".to_string(),
                    server_endpoint: local_addr.to_string(),
                    transport: TransportKind::Quic,
                    protocol: ProtocolVersion::new(1, 1, 1),
                    server_cert_fingerprint_sha256: fingerprint,
                    ticket: "ticket-drain".to_string(),
                },
                QuicTrustPolicy::PinnedServer {
                    expected_fingerprint_sha256: certificate_fingerprint_sha256(&server_cert),
                    trusted_certificates: vec![server_cert],
                },
            )
            .expect("client runtime should spawn");
        let (client_tx, mut client_events) = client_handle.into_channels();

        let mut join_rejected = None;
        for _ in 0..30 {
            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_millis(250), client_events.recv())
                    .await
                && let Some(QuicSessionEvent::JoinRejected(reason)) = event
            {
                join_rejected = Some(reason);
                break;
            }
        }
        assert_eq!(join_rejected, Some(DisconnectReason::ServerShuttingDown));

        let _ = client_tx.send(QuicSessionCommand::Shutdown);
        server_tx
            .send(QuicSessionCommand::Shutdown)
            .expect("server shutdown should send");
    }
