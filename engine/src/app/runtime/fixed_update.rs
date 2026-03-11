use crate::app::App;
use crate::prelude::Time;
use crate::runtime::fixed_time::{CatchupBudget, FixedTimeConfig, FixedTimeState, SimulationTick};
use crate::runtime::schedules::FixedUpdate;
use anyhow::Result;

impl App {
    pub(crate) fn run_fixed_update_schedule(&mut self) -> Result<()> {
        let step_seconds = self
            .world
            .resource::<FixedTimeConfig>()
            .map(|config| config.step_seconds)
            .unwrap_or(1.0 / 60.0)
            .clamp(1.0 / 240.0, 1.0 / 15.0);
        let delta_seconds = self
            .world
            .resource::<Time>()
            .map(|time| time.delta_seconds)
            .unwrap_or(step_seconds)
            .clamp(0.0, 0.25);
        let max_steps_per_frame = self
            .world
            .resource::<CatchupBudget>()
            .map(|budget| budget.max_steps_per_frame)
            .unwrap_or(4)
            .clamp(1, 16);

        {
            let mut fixed_state = self
                .world
                .resource_mut::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds = (fixed_state.accumulator_seconds + delta_seconds)
                .min(step_seconds * max_steps_per_frame as f32);
            fixed_state.steps_ran_last_frame = 0;
        }

        let mut steps = 0u32;
        loop {
            let should_step = {
                let fixed_state = self
                    .world
                    .resource::<FixedTimeState>()
                    .expect("FixedTimeState should be installed");
                fixed_state.accumulator_seconds + f32::EPSILON >= step_seconds
                    && steps < max_steps_per_frame
            };
            if !should_step {
                break;
            }

            {
                let mut tick = self
                    .world
                    .resource_mut::<SimulationTick>()
                    .expect("SimulationTick should be installed");
                tick.0 = tick.0.saturating_add(1);
            }
            self.scheduler
                .run_schedule::<FixedUpdate>(&mut self.world)?;
            steps = steps.saturating_add(1);

            let mut fixed_state = self
                .world
                .resource_mut::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds -= step_seconds;
            fixed_state.steps_ran_last_frame = steps;
        }

        let saturated = {
            let fixed_state = self
                .world
                .resource::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds + f32::EPSILON >= step_seconds
        };
        if saturated {
            let mut fixed_state = self
                .world
                .resource_mut::<FixedTimeState>()
                .expect("FixedTimeState should be installed");
            fixed_state.accumulator_seconds = 0.0;
            fixed_state.saturated_frames = fixed_state.saturated_frames.saturating_add(1);
            tracing::warn!("fixed-step loop saturated, dropping accumulated time");
        }

        Ok(())
    }
}
