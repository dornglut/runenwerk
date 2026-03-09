use serde::{Deserialize, Serialize};

use crate::SimulationTick;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandSource {
    LocalPlayer,
    RemotePlayer,
    AI,
    Server,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationCommandFrame<C> {
    pub tick: SimulationTick,
    pub commands: Vec<C>,
}
