use crate::protocol::{
    ClientMessage, DisconnectReason, Hello, JoinAccepted, JoinRejected, JoinRequest,
    ProtocolVersion, ServerMessage,
};
use crate::transport::{ConnectionId, TransportKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AuthoritativeJoinState {
    pub lobby_id: Option<String>,
    pub roster_player_codes: Vec<String>,
    pub max_players: u8,
    pub ai_fill_target: u8,
    pub settings_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientSessionTarget {
    pub server_id: String,
    pub server_endpoint: String,
    pub transport: TransportKind,
    pub protocol: ProtocolVersion,
    pub server_cert_fingerprint_sha256: String,
    pub ticket: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSessionConfig {
    pub server_id: String,
    pub protocol: ProtocolVersion,
    pub tick_rate_hz: u16,
}

impl Default for ServerSessionConfig {
    fn default() -> Self {
        Self {
            server_id: "srv-local".to_string(),
            protocol: ProtocolVersion::new(1, 1, 1),
            tick_rate_hz: 60,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SessionPhase {
    #[default]
    Idle,
    Handshaking,
    AwaitingJoin,
    Active,
    Rejected(DisconnectReason),
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ClientSessionState {
    pub phase: SessionPhase,
    pub target: Option<ClientSessionTarget>,
    pub connection_id: Option<ConnectionId>,
    pub last_disconnect: Option<DisconnectReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSessionState {
    pub phase: SessionPhase,
    pub config: ServerSessionConfig,
    pub next_connection_id: u64,
    pub active_connection: Option<ConnectionId>,
    pub last_join_request: Option<JoinRequest>,
    pub last_join_state: Option<AuthoritativeJoinState>,
    pub last_disconnect: Option<DisconnectReason>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionRuntimeCommand {
    Client(ClientMessage),
    Server(ServerMessage),
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionRuntimeEvent {
    Connected { connection_id: Option<ConnectionId> },
    ClientMessage(ClientMessage),
    ServerMessage(ServerMessage),
    Phase(SessionPhase),
    Reconnecting { attempt: u32 },
    JoinAccepted(JoinAccepted),
    JoinRejected(DisconnectReason),
    RttUpdated { millis: u32 },
    ConnectionClosed { reason: Option<DisconnectReason> },
    Error { message: String },
}

impl Default for ServerSessionState {
    fn default() -> Self {
        Self {
            phase: SessionPhase::Idle,
            config: ServerSessionConfig::default(),
            next_connection_id: 1,
            active_connection: None,
            last_join_request: None,
            last_join_state: None,
            last_disconnect: None,
        }
    }
}

pub fn begin_client_session(
    state: &mut ClientSessionState,
    target: ClientSessionTarget,
) -> Vec<ClientMessage> {
    state.phase = SessionPhase::Handshaking;
    state.connection_id = None;
    state.last_disconnect = None;
    state.target = Some(target.clone());
    vec![
        ClientMessage::Hello(Hello {
            protocol: target.protocol,
            transport: target.transport,
        }),
        ClientMessage::JoinRequest(JoinRequest {
            protocol: target.protocol,
            server_id: target.server_id,
            ticket: target.ticket,
        }),
    ]
}

pub fn observe_server_message(state: &mut ClientSessionState, message: &ServerMessage) {
    match message {
        ServerMessage::Hello(_) => {
            if matches!(state.phase, SessionPhase::Handshaking | SessionPhase::Idle) {
                state.phase = SessionPhase::AwaitingJoin;
            }
        }
        ServerMessage::JoinAccepted(JoinAccepted { connection_id, .. }) => {
            state.phase = SessionPhase::Active;
            state.connection_id = Some(ConnectionId(*connection_id));
            state.last_disconnect = None;
        }
        ServerMessage::JoinRejected(JoinRejected { reason })
        | ServerMessage::Disconnect(reason) => {
            state.phase = SessionPhase::Rejected(reason.clone());
            state.connection_id = None;
            state.last_disconnect = Some(reason.clone());
        }
        ServerMessage::Snapshot(_)
        | ServerMessage::DeltaSnapshot(_)
        | ServerMessage::RunEvent(_)
        | ServerMessage::RunResult(_) => {}
    }
}

pub fn configure_server_session(state: &mut ServerSessionState, config: ServerSessionConfig) {
    state.config = config;
    state.phase = SessionPhase::Idle;
    state.active_connection = None;
    state.last_join_request = None;
    state.last_join_state = None;
    state.last_disconnect = None;
}

pub fn handle_client_message(
    state: &mut ServerSessionState,
    message: &ClientMessage,
) -> Vec<ServerMessage> {
    match message {
        ClientMessage::Hello(hello) => {
            state.phase = SessionPhase::Handshaking;
            vec![ServerMessage::Hello(Hello {
                protocol: state.config.protocol,
                transport: hello.transport,
            })]
        }
        ClientMessage::JoinRequest(request) => {
            state.last_join_request = Some(request.clone());
            if request.server_id != state.config.server_id {
                let reason = DisconnectReason::WrongServer;
                state.phase = SessionPhase::Rejected(reason.clone());
                state.last_disconnect = Some(reason.clone());
                return vec![ServerMessage::JoinRejected(JoinRejected { reason })];
            }
            if !request.protocol.is_compatible_with(state.config.protocol) {
                let reason = DisconnectReason::VersionMismatch;
                state.phase = SessionPhase::Rejected(reason.clone());
                state.last_disconnect = Some(reason.clone());
                return vec![ServerMessage::JoinRejected(JoinRejected { reason })];
            }
            if request.ticket.trim().is_empty() {
                let reason = DisconnectReason::InvalidTicket;
                state.phase = SessionPhase::Rejected(reason.clone());
                state.last_disconnect = Some(reason.clone());
                return vec![ServerMessage::JoinRejected(JoinRejected { reason })];
            }

            let connection = ConnectionId(state.next_connection_id);
            state.next_connection_id = state.next_connection_id.saturating_add(1);
            state.phase = SessionPhase::Active;
            state.active_connection = Some(connection);
            state.last_disconnect = None;
            let join_state = state.last_join_state.clone().unwrap_or_default();
            vec![ServerMessage::JoinAccepted(JoinAccepted {
                connection_id: connection.0,
                tick_rate_hz: state.config.tick_rate_hz,
                join_state,
            })]
        }
        ClientMessage::InputFrame(_) | ClientMessage::Ack(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
