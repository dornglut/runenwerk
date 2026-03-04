pub use engine_sim::SimulationTick;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FixedTimeConfig {
    pub step_seconds: f32,
}

impl Default for FixedTimeConfig {
    fn default() -> Self {
        Self {
            step_seconds: 1.0 / 60.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CatchupBudget {
    pub max_steps_per_frame: u32,
}

impl Default for CatchupBudget {
    fn default() -> Self {
        Self {
            max_steps_per_frame: 4,
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct FixedTimeState {
    pub accumulator_seconds: f32,
    pub steps_ran_last_frame: u32,
    pub saturated_frames: u64,
}
