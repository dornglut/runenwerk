use crate::runtime::fixed_time::{FixedStepFrameDeltaOverride, FixedTimeConfig, FixedTimeState};
use runen_ecs::World;

// Owner: Engine Runtime - App Runner
pub trait AppRunner: Send {
    fn next_frame(&mut self, completed_frames: usize, world: &World) -> bool;

    fn before_frame(&mut self, _world: &mut World) {}
}

#[derive(Debug, Clone)]
pub struct FixedFramesRunner {
    frames_remaining: usize,
}

impl FixedFramesRunner {
    pub fn new(frame_count: usize) -> Self {
        Self {
            frames_remaining: frame_count,
        }
    }
}

impl AppRunner for FixedFramesRunner {
    fn next_frame(&mut self, _completed_frames: usize, _world: &World) -> bool {
        if self.frames_remaining == 0 {
            return false;
        }
        self.frames_remaining -= 1;
        true
    }
}

#[derive(Debug, Clone)]
pub struct FixedStepsRunner {
    requested_steps: u64,
    start_total_steps: Option<u64>,
}

impl FixedStepsRunner {
    pub fn new(step_count: u64) -> Self {
        Self {
            requested_steps: step_count,
            start_total_steps: None,
        }
    }
}

impl AppRunner for FixedStepsRunner {
    fn next_frame(&mut self, _completed_frames: usize, world: &World) -> bool {
        let current_steps = world
            .resource::<FixedTimeState>()
            .map(|state| state.total_completed_steps)
            .unwrap_or(0);
        let start_steps = *self.start_total_steps.get_or_insert(current_steps);
        current_steps.saturating_sub(start_steps) < self.requested_steps
    }

    fn before_frame(&mut self, world: &mut World) {
        let fixed_step_seconds = world
            .resource::<FixedTimeConfig>()
            .map(|config| config.step_seconds)
            .unwrap_or(1.0 / 60.0);
        world.insert_resource(FixedStepFrameDeltaOverride(fixed_step_seconds));
    }
}
