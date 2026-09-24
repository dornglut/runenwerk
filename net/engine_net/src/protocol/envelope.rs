use serde::{Deserialize, Serialize};

use super::{Ack, DeltaSnapshot, InputFrame, Snapshot};

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
pub struct TypedPayloadMessage {
    pub channel: String,
    pub type_name: String,
    pub schema_version: u16,
    pub payload: Vec<u8>,
}

impl TypedPayloadMessage {
    pub fn new(
        channel: impl Into<String>,
        type_name: impl Into<String>,
        schema_version: u16,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            channel: channel.into(),
            type_name: type_name.into(),
            schema_version,
            payload,
        }
    }

    pub fn encode<T: Serialize>(
        channel: impl Into<String>,
        type_name: impl Into<String>,
        schema_version: u16,
        value: &T,
    ) -> Result<Self, postcard::Error> {
        Ok(Self::new(
            channel,
            type_name,
            schema_version,
            encode_payload(value)?,
        ))
    }

    pub fn decode<'de, T: Deserialize<'de>>(&'de self) -> Result<T, postcard::Error> {
        decode_payload(&self.payload)
    }
}

/// Retained replication/input migration envelope.
///
/// Connection/session negotiation and admission are owned by standalone RunenNet Core and are not
/// represented here. This enum remains only until the later replication cut disposes the old
/// payload family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    InputFrame(InputFrame),
    Ack(Ack),
    TypedPayload(TypedPayloadMessage),
}

/// Retained replication/application migration envelope.
///
/// Handshake, join, rejection, and disconnect lifecycle variants were removed in RN8 N2; those
/// semantics are now exclusively RunenNet Core concerns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMessage {
    Snapshot(Snapshot),
    DeltaSnapshot(DeltaSnapshot),
    RunEvent(RunEvent),
    RunResult(RunResult),
    TypedPayload(TypedPayloadMessage),
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

pub fn encode_payload<T: Serialize>(payload: &T) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(payload)
}

pub fn decode_payload<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}
