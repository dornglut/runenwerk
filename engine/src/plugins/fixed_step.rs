use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CatchupBudget, FixedStepFrameDeltaOverride, FixedTimeConfig, FixedTimeState};
use anyhow::{Result, anyhow};
use runen_ecs::World;

#[derive(Debug, Default, Clone, Copy, runen_ecs::Component, runen_ecs::Resource)]
pub(crate) struct FixedStepRuntimeActivation;

pub(crate) fn fixed_step_is_active(world: &World) -> bool {
    world.has_resource::<FixedStepRuntimeActivation>()
}

pub trait AppFixedStepExt: Sized {
    fn run_for_fixed_steps(self, step_count: u64) -> Result<Self>;
}

impl AppFixedStepExt for App {
    fn run_for_fixed_steps(mut self, step_count: u64) -> Result<Self> {
        self.require_headless_host("run_for_fixed_steps")?;
        self.admit_composition()?;
        if !fixed_step_is_active(&self.world) {
            return Err(anyhow!(
                "run_for_fixed_steps requires FixedStepPlugin to select fixed cadence"
            ));
        }

        self.prepare_for_run()?;
        let start_total_steps = self
            .world
            .resource::<FixedTimeState>()
            .expect("FixedStepPlugin should install FixedTimeState")
            .total_completed_steps;

        while self
            .world
            .resource::<FixedTimeState>()
            .expect("FixedStepPlugin should retain FixedTimeState")
            .total_completed_steps
            .saturating_sub(start_total_steps)
            < step_count
        {
            let fixed_step_seconds = self
                .world
                .resource::<FixedTimeConfig>()
                .map(|config| config.step_seconds)
                .unwrap_or(1.0 / 60.0);
            self.world
                .insert_resource(FixedStepFrameDeltaOverride(fixed_step_seconds));
            self.run_frame()?;
        }

        Ok(self)
    }
}

pub struct FixedStepPlugin;

impl Plugin for FixedStepPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FixedStepRuntimeActivation>();
        app.init_resource::<FixedTimeConfig>();
        app.init_resource::<CatchupBudget>();
        app.init_resource::<FixedTimeState>();
    }
}
