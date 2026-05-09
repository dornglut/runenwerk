use crate::protocol::{
    ClientMessage, Hello, JoinAccepted, JoinRejected, JoinRequest, ProtocolVersion, ServerMessage,
};
use crate::transport::{ConnectionId, TransportKind};
use serde::{Deserialize, Serialize};

use super::{ClientSessionState, SessionPhase};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientSessionTarget {
    pub server_id: String,
    pub server_endpoint: String,
    pub transport: TransportKind,
    pub protocol: ProtocolVersion,
    pub server_cert_fingerprint_sha256: String,
    pub ticket: String,
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
        | ServerMessage::RunResult(_)
        | ServerMessage::TypedPayload(_) => {}
    }
}
