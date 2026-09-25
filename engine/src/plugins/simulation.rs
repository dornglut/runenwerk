use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{FixedStepBegin, ResMut};
use engine_sim::{
    AuthorityRole, DeterminismLevel, SimulationProfile, SimulationProfileConfig, SimulationRng,
    SimulationSeed, SimulationSessionId, SimulationTick,
};

#[derive(Debug, Default, Clone, Copy, runen_ecs::Component, runen_ecs::Resource)]
struct SimulationIntegrationState;

const INITIAL_SIMULATION_SESSION_ID: SimulationSessionId = SimulationSessionId(1);

pub struct SimulationPlugin;

pub trait AppSimulationExt {
    fn set_simulation_profile(&mut self, profile: SimulationProfile) -> &mut Self;
    fn set_authority_role(&mut self, authority: AuthorityRole) -> &mut Self;
    fn set_simulation_seed(&mut self, seed: SimulationSeed) -> &mut Self;
    fn current_tick(&self) -> u64;
}

impl AppSimulationExt for App {
    fn set_simulation_profile(&mut self, profile: SimulationProfile) -> &mut Self {
        self.init_resource::<SimulationProfileConfig>();
        if let Ok(config) = self.world_mut().resource_mut::<SimulationProfileConfig>() {
            config.profile = profile;
            config.determinism =
                match profile {
                    SimulationProfile::DeterministicLockstep
                    | SimulationProfile::RollbackSession => DeterminismLevel::Strict,
                    SimulationProfile::HighThroughputAuthority => DeterminismLevel::BestEffort,
                    SimulationProfile::LocalSinglePlayer
                    | SimulationProfile::DedicatedAuthority => DeterminismLevel::Validated,
                };
        }
        self
    }

    fn set_authority_role(&mut self, authority: AuthorityRole) -> &mut Self {
        self.init_resource::<SimulationProfileConfig>();
        if let Ok(config) = self.world_mut().resource_mut::<SimulationProfileConfig>() {
            config.authority = authority;
        }
        if let Ok(world_runtime_config) =
            self.world_mut()
                .resource_mut::<crate::plugins::world::plugin::WorldRuntimeConfig>()
        {
            world_runtime_config.mode =
                crate::plugins::world::plugin::world_runtime_mode_for_authority(authority);
        }
        self
    }

    fn set_simulation_seed(&mut self, seed: SimulationSeed) -> &mut Self {
        self.insert_resource(seed);
        self.insert_resource(SimulationRng::from_seed(seed));
        self
    }

    fn current_tick(&self) -> u64 {
        self.world()
            .resource::<SimulationTick>()
            .map(|tick| tick.0)
            .unwrap_or(0)
    }
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        if app.world().has_resource::<SimulationIntegrationState>() {
            return;
        }

        app.init_resource::<SimulationIntegrationState>();
        app.init_resource::<SimulationTick>();
        app.init_resource::<SimulationProfileConfig>();
        if !app.world().has_resource::<SimulationSessionId>() {
            app.insert_resource(INITIAL_SIMULATION_SESSION_ID);
        }

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
