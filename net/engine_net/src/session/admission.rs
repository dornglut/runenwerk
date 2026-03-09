use crate::protocol::{
    ClientMessage, DisconnectReason, Hello, JoinAccepted, JoinRejected, ServerMessage,
};
use crate::transport::ConnectionId;
use serde::{Deserialize, Serialize};

use super::{ServerSessionState, SessionPhase};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AuthoritativeJoinState {
    pub lobby_id: Option<String>,
    pub roster_player_codes: Vec<String>,
    pub max_players: u8,
    pub ai_fill_target: u8,
    pub settings_json: Option<String>,
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
            state.active_connections.insert(connection);
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
