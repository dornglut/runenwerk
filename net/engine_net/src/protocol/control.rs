use crate::session::AuthoritativeJoinState;
use crate::transport::TransportKind;
use serde::{Deserialize, Serialize};

use super::version::ProtocolVersion;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hello {
    pub protocol: ProtocolVersion,
    pub transport: TransportKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinRequest {
    pub protocol: ProtocolVersion,
    pub server_id: String,
    pub ticket: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinAccepted {
    pub connection_id: u64,
    pub tick_rate_hz: u16,
    pub join_state: AuthoritativeJoinState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinRejected {
    pub reason: DisconnectReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectReason {
    VersionMismatch,
    InvalidTicket,
    TicketExpired,
    WrongServer,
    ServerShuttingDown,
    TimedOut,
}
