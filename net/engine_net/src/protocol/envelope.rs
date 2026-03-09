use serde::{Deserialize, Serialize};

use super::{
    Ack, DeltaSnapshot, DisconnectReason, Hello, InputFrame, JoinAccepted, JoinRejected,
    JoinRequest, Snapshot,
};

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

pub fn encode_payload<T: Serialize>(payload: &T) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(payload)
}

pub fn decode_payload<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}
