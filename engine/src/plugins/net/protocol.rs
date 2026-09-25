use engine_sim::SimulationTick;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct NetEntityId(pub u64);

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct SnapshotCursor(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputFrame {
    pub tick: SimulationTick,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ack {
    pub cursor: SnapshotCursor,
    pub last_received_tick: SimulationTick,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: SimulationTick,
    pub cursor: SnapshotCursor,
    pub last_applied: SnapshotCursor,
    pub entity_ids: Vec<NetEntityId>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaSnapshot {
    pub tick: SimulationTick,
    pub base: SnapshotCursor,
    pub cursor: SnapshotCursor,
    pub entity_ids: Vec<NetEntityId>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientMessage {
    InputFrame(InputFrame),
    Ack(Ack),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMessage {
    Snapshot(Snapshot),
    DeltaSnapshot(DeltaSnapshot),
}
