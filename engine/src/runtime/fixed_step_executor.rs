use crate::plugins::time::domain::Time;
use crate::runtime::fixed_time::{
    CatchupBudget, FixedStepFrameDeltaOverride, FixedTimeConfig, FixedTimeState,
};
use crate::runtime::schedules::{FixedStepBegin, FixedUpdate};
use anyhow::Result;
use runen_ecs::{Runtime, World};

const MIN_FIXED_STEP_SECONDS: f32 = 1.0 / 240.0;
const MAX_FIXED_STEP_SECONDS: f32 = 1.0 / 15.0;
const MAX_FRAME_DELTA_SECONDS: f32 = 0.25;
const MIN_CATCHUP_STEPS: u32 = 1;
const MAX_CATCHUP_STEPS: u32 = 16;

#[derive(Debug, Copy, Clone, PartialEq)]
struct FixedStepFrameConfig {
    step_seconds: f32,
    delta_seconds: f32,
    max_steps_per_frame: u32,
}

impl FixedStepFrameConfig {
    fn from_world(world: &mut World) -> Self {
        let step_seconds = world
            .resource::<FixedTimeConfig>()
            .map(|config| config.step_seconds)
            .unwrap_or(1.0 / 60.0)
            .clamp(MIN_FIXED_STEP_SECONDS, MAX_FIXED_STEP_SECONDS);
        let bounded_delta = world
            .remove_resource::<FixedStepFrameDeltaOverride>()
            .map(|override_delta| override_delta.0);
        let delta_seconds = bounded_delta
            .or_else(|| world.resource::<Time>().ok().map(|time| time.delta_seconds))
            .unwrap_or(step_seconds)
            .clamp(0.0, MAX_FRAME_DELTA_SECONDS);
        let max_steps_per_frame = world
            .resource::<CatchupBudget>()
            .map(|budget| budget.max_steps_per_frame)
            .unwrap_or(4)
            .clamp(MIN_CATCHUP_STEPS, MAX_CATCHUP_STEPS);

        Self {
            step_seconds,
            delta_seconds,
            max_steps_per_frame,
        }
    }
}

/// Runs the canonical fixed-step loop used by all runtime runners.
///
/// Contract:
/// - ordinary frames accumulate `Time::delta_seconds`;
/// - bounded fixed-step advancement may provide one private frame-delta override, consumed here
///   after `PreUpdate` without taking ownership of the public `Time` resource;
/// - up to `CatchupBudget::max_steps_per_frame` fixed steps run in a frame;
/// - each admitted step runs `FixedStepBegin` exactly once before `FixedUpdate`;
/// - owner integrations may observe `FixedStepBegin`; the cadence executor does not own
///   simulation identity or other domain state;
/// - `FixedTimeState::steps_ran_last_frame` is reset to `0` each frame and updated
///   after each completed fixed step;
/// - `FixedTimeState::total_completed_steps` advances after each completed `FixedUpdate`;
/// - if pending fixed-time work remains after consuming the per-frame catchup budget,
///   the remainder is dropped, `saturated_frames` is incremented, and a warning is logged.
pub(crate) fn run_fixed_update_frame(world: &mut World, scheduler: &mut Runtime) -> Result<()> {
    let config = FixedStepFrameConfig::from_world(world);

    {
        let fixed_state = world
            .resource_mut::<FixedTimeState>()
            .expect("FixedTimeState should be installed by FixedStepPlugin");
        fixed_state.accumulator_seconds += config.delta_seconds;
        fixed_state.steps_ran_last_frame = 0;
    }

    let mut steps = 0u32;
    loop {
        let should_step = {
            let fixed_state = world
                .resource::<FixedTimeState>()
                .expect("FixedTimeState should be installed by FixedStepPlugin");
            fixed_state.accumulator_seconds + f32::EPSILON >= config.step_seconds
                && steps < config.max_steps_per_frame
        };
        if !should_step {
            break;
        }

        scheduler.run_schedule::<FixedStepBegin>(world)?;
        scheduler.run_schedule::<FixedUpdate>(world)?;
        steps = steps.saturating_add(1);

        let fixed_state = world
            .resource_mut::<FixedTimeState>()
            .expect("FixedTimeState should be installed by FixedStepPlugin");
        fixed_state.accumulator_seconds -= config.step_seconds;
        fixed_state.steps_ran_last_frame = steps;
        fixed_state.total_completed_steps = fixed_state.total_completed_steps.saturating_add(1);
    }

    let saturated = {
        let fixed_state = world
            .resource::<FixedTimeState>()
            .expect("FixedTimeState should be installed by FixedStepPlugin");
        fixed_state.accumulator_seconds + f32::EPSILON >= config.step_seconds
    };
    if saturated {
        let fixed_state = world
            .resource_mut::<FixedTimeState>()
            .expect("FixedTimeState should be installed by FixedStepPlugin");
        fixed_state.accumulator_seconds = 0.0;
        fixed_state.saturated_frames = fixed_state.saturated_frames.saturating_add(1);
        tracing::warn!("fixed-step loop saturated, dropping accumulated time");
    }

    Ok(())
}
