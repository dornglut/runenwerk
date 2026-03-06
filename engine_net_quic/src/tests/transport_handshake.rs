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

