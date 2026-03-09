use super::*;
use crate::protocol::{ClientMessage, JoinAccepted, JoinRequest, ProtocolVersion, ServerMessage};
use crate::transport::{ConnectionId, TransportKind};

#[test]
fn client_bootstrap_emits_hello_and_join_request() {
    let mut state = ClientSessionState::default();
    let messages = begin_client_session(
        &mut state,
        ClientSessionTarget {
            server_id: "srv-1".to_string(),
            server_endpoint: "127.0.0.1:7000".to_string(),
            transport: TransportKind::Quic,
            protocol: ProtocolVersion::new(1, 1, 1),
            server_cert_fingerprint_sha256: "a".repeat(64),
            ticket: "ticket-1".to_string(),
        },
    );
    assert_eq!(state.phase, SessionPhase::Handshaking);
    assert_eq!(messages.len(), 2);
    assert!(matches!(messages[0], ClientMessage::Hello(_)));
    assert!(matches!(messages[1], ClientMessage::JoinRequest(_)));
}

#[test]
fn server_accepts_valid_join_request() {
    let mut state = ServerSessionState::default();
    configure_server_session(
        &mut state,
        ServerSessionConfig {
            server_id: "srv-1".to_string(),
            protocol: ProtocolVersion::new(1, 1, 1),
            tick_rate_hz: 60,
        },
    );
    let responses = handle_client_message(
        &mut state,
        &ClientMessage::JoinRequest(JoinRequest {
            protocol: ProtocolVersion::new(1, 1, 1),
            server_id: "srv-1".to_string(),
            ticket: "ticket-1".to_string(),
        }),
    );
    assert_eq!(state.phase, SessionPhase::Active);
    assert_eq!(state.active_connection, Some(ConnectionId(1)));
    assert!(state.active_connections.contains(&ConnectionId(1)));
    assert_eq!(responses.len(), 1);
    assert!(matches!(
        responses[0],
        ServerMessage::JoinAccepted(JoinAccepted {
            connection_id: 1,
            tick_rate_hz: 60,
            ..
        })
    ));
}
