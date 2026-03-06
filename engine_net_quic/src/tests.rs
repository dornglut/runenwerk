// Owner: Grotto Engine Net - QUIC Tests
mod tests {
    use super::*;
    use engine_net::{
        Ack, ClientCommandEnvelope, ClientMessage, ConnectionId, DeltaSnapshot, InputFrame,
        MoveCommand, ProtocolVersion, Snapshot, SnapshotCursor,
    };

    #[test]
    fn quic_transport_defaults_to_quic_and_grotto_alpn() {
        let transport = QuicTransport::default();
        assert_eq!(transport.kind(), TransportKind::Quic);
        assert_eq!(
            transport.config().alpn_protocols,
            vec![b"grottoq/1".to_vec()]
        );
        let _config = transport.build_transport_config();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn certificate_fingerprint_is_sha256_hex() {
        let transport = QuicTransport::default();
        let server = transport
            .bind_server_endpoint(default_client_bind_addr())
            .expect("server endpoint should build");
        assert_eq!(server.certificate_fingerprint_sha256.len(), 64);
        assert!(
            server
                .certificate_fingerprint_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn loopback_quic_handshake_reaches_join_accepted() {
        let transport = QuicTransport::default();
        let server = transport
            .bind_server_endpoint(default_client_bind_addr())
            .expect("server endpoint should bind");
        let server_addr = server
            .endpoint
            .local_addr()
            .expect("server addr should exist");
        let server_cert = server.certificate.clone();
        let server_name = server.server_name.clone();

        let server_task = tokio::spawn({
            let endpoint = server.endpoint.clone();
            let transport = transport.clone();
            async move {
                transport
                    .accept_and_handshake(
                        &endpoint,
                        ServerSessionConfig {
                            server_id: "srv-local".to_string(),
                            protocol: ProtocolVersion::new(1, 1, 1),
                            tick_rate_hz: 60,
                        },
                    )
                    .await
            }
        });

        let client_endpoint = transport
            .bind_client_endpoint(default_client_bind_addr(), &[server_cert])
            .expect("client endpoint should bind");
        let connection = client_endpoint
            .connect(server_addr, &server_name)
            .expect("client connect should start")
            .await
            .expect("client connect should complete");
        let target = ClientSessionTarget {
            server_id: "srv-local".to_string(),
            server_endpoint: server_addr.to_string(),
            transport: TransportKind::Quic,
            protocol: ProtocolVersion::new(1, 1, 1),
            server_cert_fingerprint_sha256: "unused-for-direct-root-store".to_string(),
            ticket: "ticket-1".to_string(),
        };
        let mut state = ClientSessionState::default();
        let outbound = begin_client_session(&mut state, target);
        let (mut send, mut recv) = connection
            .open_bi()
            .await
            .expect("client control stream should open");
        for message in outbound {
            write_message(&mut send, &MessageEnvelope::Client(message))
                .await
                .expect("client message should write");
        }
        send.finish().expect("handshake stream should finish");

        let mut accepted = None;
        while let Some(message) = read_message(&mut recv)
            .await
            .expect("client should read server response")
        {
            let MessageEnvelope::Server(server_message) = message else {
                continue;
            };
            observe_server_message(&mut state, &server_message);
            if let ServerMessage::JoinAccepted(join) = server_message {
                accepted = Some(join);
                break;
            }
        }
        let server_result = server_task
            .await
            .expect("server task should join")
            .expect("server handshake should succeed");
        assert_eq!(state.phase, SessionPhase::Active);
        assert_eq!(accepted.expect("join accepted").connection_id, 1);
        assert_eq!(server_result.state.phase, SessionPhase::Active);
        assert_eq!(server_result.state.active_connection, Some(ConnectionId(1)));
    }

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
}
