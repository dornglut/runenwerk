use crate::WorldHash;
use engine_sim::SimulationTick;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayMismatch {
    MissingCheckpoint {
        target_tick: SimulationTick,
    },
    TickHashMismatch {
        tick: SimulationTick,
        expected: WorldHash,
        actual: WorldHash,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayValidationReport {
    pub mismatches: Vec<ReplayMismatch>,
}

impl ReplayValidationReport {
    pub fn is_clean(&self) -> bool {
        self.mismatches.is_empty()
    }
}
