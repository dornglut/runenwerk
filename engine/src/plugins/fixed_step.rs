use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CatchupBudget, FixedTimeConfig, FixedTimeState};
use runen_ecs::World;

#[derive(Debug, Default, Clone, Copy, runen_ecs::Component, runen_ecs::Resource)]
pub(crate) struct FixedStepRuntimeActivation;

pub(crate) fn fixed_step_is_active(world: &World) -> bool {
    world.has_resource::<FixedStepRuntimeActivation>()
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
