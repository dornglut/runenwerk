// Owner: Grotto Online - Join Handoff Tests
    use super::*;

    fn valid_grant() -> JoinGrant {
        JoinGrant {
            server_id: "srv-1".to_string(),
            server_endpoint: "127.0.0.1:7000".to_string(),
            transport_kind: TransportKind::Quic,
            protocol_version: ProtocolVersion::new(1, 1, 1),
            server_cert_fingerprint_sha256: "a".repeat(64),
            ticket: "ticket-123".to_string(),
        }
    }

    #[test]
    fn join_grant_validation_accepts_supported_quic_handoff() {
        let grant = valid_grant();
        assert!(
            grant
                .validate_for("srv-1", ProtocolVersion::new(1, 1, 1))
                .is_ok()
        );
    }

    #[test]
    fn join_grant_validation_rejects_wrong_server_and_bad_fingerprint() {
        let grant = valid_grant();
        assert_eq!(
            grant
                .validate_for("srv-2", ProtocolVersion::new(1, 1, 1))
                .unwrap_err(),
            JoinGrantError::WrongServer {
                expected: "srv-2".to_string(),
                actual: "srv-1".to_string(),
            }
        );

        let mut bad_grant = valid_grant();
        bad_grant.server_cert_fingerprint_sha256 = "not-hex".to_string();
        assert_eq!(
            bad_grant
                .validate_for("srv-1", ProtocolVersion::new(1, 1, 1))
                .unwrap_err(),
            JoinGrantError::InvalidFingerprint
        );
    }

    #[test]
    fn join_grant_converts_to_client_session_target() {
        let target = valid_grant()
            .into_client_session_target("srv-1", ProtocolVersion::new(1, 1, 1))
            .expect("valid grant should become a client target");
        assert_eq!(target.server_id, "srv-1");
        assert_eq!(target.server_endpoint, "127.0.0.1:7000");
        assert_eq!(target.transport, TransportKind::Quic);
    }
