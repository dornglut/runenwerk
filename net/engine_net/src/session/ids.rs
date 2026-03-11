use crate::protocol::{
    ClientMessage, DisconnectReason, JoinAccepted, JoinRequest, ProtocolVersion, ServerMessage,
};
use crate::transport::ConnectionId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::{AuthoritativeJoinState, ClientSessionTarget};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Component)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ecs::Component)]
pub struct ClientSessionState {
    pub phase: SessionPhase,
    pub target: Option<ClientSessionTarget>,
    pub connection_id: Option<ConnectionId>,
    pub last_disconnect: Option<DisconnectReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Component)]
pub struct ServerSessionState {
    pub phase: SessionPhase,
    pub config: ServerSessionConfig,
    pub next_connection_id: u64,
    pub active_connection: Option<ConnectionId>,
    pub active_connections: BTreeSet<ConnectionId>,
    pub last_join_request: Option<JoinRequest>,
    pub last_join_state: Option<AuthoritativeJoinState>,
    pub last_disconnect: Option<DisconnectReason>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionRuntimeCommand {
    Client(ClientMessage),
    ServerToConnection {
        connection_id: ConnectionId,
        message: ServerMessage,
    },
    ServerBroadcast(ServerMessage),
    SetDrainMode {
        enabled: bool,
    },
    DisconnectConnection {
        connection_id: ConnectionId,
        reason: DisconnectReason,
    },
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionRuntimeEvent {
    Connected {
        connection_id: Option<ConnectionId>,
    },
    ClientMessage {
        connection_id: Option<ConnectionId>,
        message: ClientMessage,
    },
    ServerMessage(ServerMessage),
    Phase(SessionPhase),
    Reconnecting {
        attempt: u32,
    },
    JoinAccepted(JoinAccepted),
    JoinRejected(DisconnectReason),
    RttUpdated {
        millis: u32,
    },
    ConnectionClosed {
        connection_id: Option<ConnectionId>,
        reason: Option<DisconnectReason>,
    },
    Error {
        message: String,
    },
}

impl Default for ServerSessionState {
    fn default() -> Self {
        Self {
            phase: SessionPhase::Idle,
            config: ServerSessionConfig::default(),
            next_connection_id: 1,
            active_connection: None,
            active_connections: BTreeSet::new(),
            last_join_request: None,
            last_join_state: None,
            last_disconnect: None,
        }
    }
}

pub fn configure_server_session(state: &mut ServerSessionState, config: ServerSessionConfig) {
    state.config = config;
    state.phase = SessionPhase::Idle;
    state.active_connection = None;
    state.active_connections.clear();
    state.last_join_request = None;
    state.last_join_state = None;
    state.last_disconnect = None;
}

pub fn remove_server_connection(
    state: &mut ServerSessionState,
    connection_id: ConnectionId,
    reason: Option<DisconnectReason>,
) {
    state.active_connections.remove(&connection_id);
    if state.active_connection == Some(connection_id) {
        state.active_connection = state.active_connections.iter().next_back().copied();
    }
    state.last_disconnect = reason;
    state.phase = if state.active_connections.is_empty() {
        SessionPhase::Closed
    } else {
        SessionPhase::Active
    };
}
