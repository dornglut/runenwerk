use crate::app::App;
use crate::app::domain::mode::AppMode;
use crate::plugins::InputState;
use crate::prelude::Time;
use crate::*;

impl App {
    pub(crate) fn install_builtin_resources(&mut self) {
        if self.world.resource::<Time>().is_err() {
            self.world.insert_resource(Time::new());
        }
        if self.world.resource::<InputState>().is_err() {
            self.world.insert_resource(InputState::new());
        }
        if self.world.resource::<WindowState>().is_err() {
            let state = match self.mode {
                AppMode::Windowed => WindowState::windowed(self.title.clone()),
                AppMode::Headless => WindowState::headless(self.title.clone()),
            };
            self.world.insert_resource(state);
        }
        if !self.world.has_resource::<SceneCatalog>() {
            self.world.insert_resource(SceneCatalog::default());
        }
        if !self.world.has_resource::<StartupState>() {
            self.world.insert_resource(StartupState::default());
        }
        if !self.world.has_resource::<SceneRuntimeState>() {
            self.world.insert_resource(SceneRuntimeState::default());
        }
        if !self.world.has_resource::<UiOverlayState>() {
            self.world.insert_resource(UiOverlayState::default());
        }
        if !self.world.has_resource::<GameplayRuntimeConfig>() {
            self.world.insert_resource(GameplayRuntimeConfig::default());
        }
        if !self.world.has_resource::<SessionRuntimeState>() {
            self.world.insert_resource(SessionRuntimeState::default());
        }
        if !self.world.has_resource::<FixedTimeConfig>() {
            self.world.insert_resource(FixedTimeConfig::default());
        }
        if !self.world.has_resource::<CatchupBudget>() {
            self.world.insert_resource(CatchupBudget::default());
        }
        if !self.world.has_resource::<FixedTimeState>() {
            self.world.insert_resource(FixedTimeState::default());
        }
        if !self.world.has_resource::<SimulationTick>() {
            self.world.insert_resource(SimulationTick::default());
        }
        if !self.world.has_resource::<SimulationProfileConfig>() {
            self.world.insert_resource(SimulationProfileConfig::default());
        }
        if !self.world.has_resource::<SimulationSessionId>() {
            self.world.insert_resource(SimulationSessionId::default());
        }
        if !self.world.has_resource::<SimulationSeed>() {
            let seed = SimulationSeed::default();
            self.world.insert_resource(seed);
            self.world.insert_resource(SimulationRng::from_seed(seed));
        } else if !self.world.has_resource::<SimulationRng>() {
            let seed = self
                .world
                .resource::<SimulationSeed>()
                .copied()
                .unwrap_or_default();
            self.world.insert_resource(SimulationRng::from_seed(seed));
        }
    }
}
