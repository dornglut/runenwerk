use crate::replication::SnapshotCursor;
use engine_sim::SimulationTick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ack {
    pub cursor: SnapshotCursor,
    pub last_received_tick: SimulationTick,
}
