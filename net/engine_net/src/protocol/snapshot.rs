use crate::replication::SnapshotCursor;
use crate::simulation::NetworkEntityId;
use engine_sim::SimulationTick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: SimulationTick,
    pub cursor: SnapshotCursor,
    pub last_applied: SnapshotCursor,
    pub entity_ids: Vec<NetworkEntityId>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaSnapshot {
    pub tick: SimulationTick,
    pub base: SnapshotCursor,
    pub cursor: SnapshotCursor,
    pub entity_ids: Vec<NetworkEntityId>,
    pub payload: Vec<u8>,
}
