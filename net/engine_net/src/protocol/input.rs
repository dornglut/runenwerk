use engine_sim::SimulationTick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputFrame {
    pub tick: SimulationTick,
    pub payload: Vec<u8>,
}
