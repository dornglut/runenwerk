use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{FixedStepBegin, ResMut};
use engine_sim::{
    SimulationProfileConfig, SimulationRng, SimulationSeed, SimulationSessionId, SimulationTick,
};

#[derive(Debug, Default, Clone, Copy, runen_ecs::Component, runen_ecs::Resource)]
struct SimulationIntegrationState;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        if app.world().has_resource::<SimulationIntegrationState>() {
            return;
        }

        app.init_resource::<SimulationIntegrationState>();
        app.init_resource::<SimulationTick>();
        app.init_resource::<SimulationProfileConfig>();
        app.init_resource::<SimulationSessionId>();

        if !app.world().has_resource::<SimulationSeed>() {
            app.insert_resource(SimulationSeed::default());
        }
        if !app.world().has_resource::<SimulationRng>() {
            let seed = *app
                .world()
                .resource::<SimulationSeed>()
                .expect("SimulationPlugin should establish a simulation seed before RNG state");
            app.insert_resource(SimulationRng::from_seed(seed));
        }

        app.add_systems(FixedStepBegin, advance_simulation_tick);
    }
}

fn advance_simulation_tick(mut tick: ResMut<SimulationTick>) {
    tick.0 = tick.0.saturating_add(1);
}
