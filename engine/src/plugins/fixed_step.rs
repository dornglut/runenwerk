use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CatchupBudget, FixedTimeConfig, FixedTimeState, SimulationTick};

pub struct FixedStepPlugin;

impl Plugin for FixedStepPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FixedTimeConfig>();
        app.init_resource::<CatchupBudget>();
        app.init_resource::<FixedTimeState>();
        app.init_resource::<SimulationTick>();
    }
}
