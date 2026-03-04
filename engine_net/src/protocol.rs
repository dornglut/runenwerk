use crate::replication::SnapshotCursor;
use crate::simulation::{ClientCommandEnvelope, NetworkEntityId, NetworkTick};
use crate::transport::TransportKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub protocol_version: u32,
    pub game_content_version: u32,
    pub schema_version: u32,
}

impl ProtocolVersion {
    pub const fn new(
        protocol_version: u32,
        game_content_version: u32,
        schema_version: u32,
    ) -> Self {
        Self {
            protocol_version,
            game_content_version,
            schema_version,
        }
    }

    pub const fn is_compatible_with(self, other: Self) -> bool {
        self.protocol_version == other.protocol_version
            && self.game_content_version == other.game_content_version
            && self.schema_version == other.schema_version
    }
}

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinRejected {
    pub reason: DisconnectReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: NetworkTick,
    pub last_applied: SnapshotCursor,
    pub entity_ids: Vec<NetworkEntityId>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaSnapshot {
    pub tick: NetworkTick,
    pub base: SnapshotCursor,
    pub cursor: SnapshotCursor,
    pub entity_ids: Vec<NetworkEntityId>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputFrame {
    pub tick: NetworkTick,
    pub commands: Vec<ClientCommandEnvelope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ack {
    pub cursor: SnapshotCursor,
    pub last_received_tick: NetworkTick,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunEvent {
    pub code: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunResult {
    pub outcome: String,
    pub payload: Vec<u8>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    Hello(Hello),
    JoinRequest(JoinRequest),
    InputFrame(InputFrame),
    Ack(Ack),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMessage {
    Hello(Hello),
    JoinAccepted(JoinAccepted),
    JoinRejected(JoinRejected),
    Snapshot(Snapshot),
    DeltaSnapshot(DeltaSnapshot),
    RunEvent(RunEvent),
    RunResult(RunResult),
    Disconnect(DisconnectReason),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageEnvelope {
    Client(ClientMessage),
    Server(ServerMessage),
}

pub fn encode_message<T: Serialize>(value: &T) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(value)
}

pub fn decode_message<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_requires_exact_match() {
        let baseline = ProtocolVersion::new(1, 2, 3);
        assert!(baseline.is_compatible_with(ProtocolVersion::new(1, 2, 3)));
        assert!(!baseline.is_compatible_with(ProtocolVersion::new(2, 2, 3)));
        assert!(!baseline.is_compatible_with(ProtocolVersion::new(1, 9, 3)));
        assert!(!baseline.is_compatible_with(ProtocolVersion::new(1, 2, 4)));
    }

    #[test]
    fn postcard_round_trips_message_envelopes() {
        let envelope = MessageEnvelope::Client(ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));

        let bytes = encode_message(&envelope).expect("message should encode");
        let decoded: MessageEnvelope = decode_message(&bytes).expect("message should decode");
        assert_eq!(decoded, envelope);
    }
}
