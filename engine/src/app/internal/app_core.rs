// Owner: Engine Runtime - App Core
const DEFAULT_WINDOW_TITLE: &str = "Grotto Quest - Engine";

enum AppMode {
    Windowed,
    Headless,
}

pub(crate) struct WindowedAppState {
    pub(crate) world: World,
    pub(crate) scheduler: ExecutionScheduler<World>,
    pub(crate) build_errors: Vec<anyhow::Error>,
    pub(crate) startup_ran: bool,
    pub(crate) title: String,
    pub(crate) control_flow: ControlFlow,
}

pub struct App {
    world: World,
    scheduler: ExecutionScheduler<World>,
    runner: Box<dyn AppRunner>,
    build_errors: Vec<anyhow::Error>,
    startup_ran: bool,
    mode: AppMode,
    title: String,
    control_flow: ControlFlow,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self::with_mode(AppMode::Windowed)
    }

    pub fn headless() -> Self {
        Self::with_mode(AppMode::Headless)
    }

    fn with_mode(mode: AppMode) -> Self {
        let title = DEFAULT_WINDOW_TITLE.to_string();
        let mut app = Self {
            world: World::new(),
            scheduler: ExecutionScheduler::new(),
            runner: Box::new(FixedFramesRunner::new(1)),
            build_errors: Vec::new(),
            startup_ran: false,
            mode,
            title: title.clone(),
            control_flow: ControlFlow::Poll,
        };
        app.install_builtin_resources();
        app
    }

    pub fn add_plugin<P>(&mut self, plugin: P) -> &mut Self
    where
        P: Plugin + 'static,
    {
        plugin.build(self);
        self
    }

    pub fn add_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn add_plugins<P>(&mut self, plugins: P) -> &mut Self
    where
        P: IntoPlugins,
    {
        plugins.add_to_app(self);
        self
    }

    pub fn add_systems<L, S, Marker>(&mut self, _schedule: L, systems: S) -> &mut Self
    where
        L: ScheduleLabel,
        S: IntoSystemConfigs<Marker>,
    {
        systems.register::<L>(&mut self.world, &mut self.scheduler, &mut self.build_errors);
        self
    }

    pub fn init_resource<R>(&mut self) -> &mut Self
    where
        R: Resource + Default,
    {
        if self.world.resource::<R>().is_err() {
            self.world.insert_resource(R::default());
        }
        self
    }

    pub fn insert_resource<R>(&mut self, value: R) -> &mut Self
    where
        R: Resource,
    {
        self.world.insert_resource(value);
        self
    }

    pub fn set_runner<R>(&mut self, runner: R) -> &mut Self
    where
        R: AppRunner + 'static,
    {
        self.runner = Box::new(runner);
        self
    }

    pub fn set_title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = title.into();
        if let Ok(mut window) = self.world.resource_mut::<WindowState>() {
            window.set_title(self.title.clone());
        }
        self
    }

    pub fn set_simulation_profile(&mut self, profile: SimulationProfile) -> &mut Self {
        if let Ok(mut config) = self.world.resource_mut::<SimulationProfileConfig>() {
            config.profile = profile;
            config.determinism = match profile {
                SimulationProfile::DeterministicLockstep | SimulationProfile::RollbackSession => {
                    engine_sim::DeterminismLevel::Strict
                }
                SimulationProfile::HighThroughputAuthority => {
                    engine_sim::DeterminismLevel::BestEffort
                }
                SimulationProfile::LocalSinglePlayer | SimulationProfile::DedicatedAuthority => {
                    engine_sim::DeterminismLevel::Validated
                }
            };
        }
        self
    }

    pub fn set_authority_role(&mut self, authority: AuthorityRole) -> &mut Self {
        if let Ok(mut config) = self.world.resource_mut::<SimulationProfileConfig>() {
            config.authority = authority;
        }
        self
    }

    pub fn set_simulation_seed(&mut self, seed: SimulationSeed) -> &mut Self {
        self.world.insert_resource(seed);
        if let Ok(mut rng) = self.world.resource_mut::<SimulationRng>() {
            rng.reseed(seed);
        }
        self
    }

    pub fn start_recording(&mut self) -> Result<&mut Self> {
        replay_runtime::start_recording(&mut self.world)?;
        Ok(self)
    }

    pub fn stop_recording(&mut self) -> Result<SceneReplayArchive> {
        replay_runtime::stop_recording(&mut self.world)
    }

    pub fn load_replay(&mut self, archive: SceneReplayArchive) -> Result<&mut Self> {
        replay_runtime::load_replay(&mut self.world, archive)?;
        Ok(self)
    }

    pub fn seek_tick(&mut self, tick: u64) -> Result<ReplayValidationReport> {
        replay_runtime::seek_loaded_replay(&mut self.world, SimulationTick(tick))
    }

    pub fn current_tick(&self) -> u64 {
        self.world
            .resource::<SimulationTick>()
            .map(|tick| tick.0)
            .unwrap_or(0)
    }

    pub fn add_scene<S>(&mut self, scene: S) -> &mut Self
    where
        S: Into<SceneRegistration>,
    {
        let scene = scene.into();
        if self.world.resource::<SceneCatalog>().is_err() {
            self.world.insert_resource(SceneCatalog::default());
        }
        if let Ok(mut catalog) = self.world.resource_mut::<SceneCatalog>() {
            catalog.register(scene.id, scene.template_path);
        }
        self
    }

    pub fn add_scene_template(&mut self, template_path: impl Into<String>) -> &mut Self {
        let template_path = template_path.into();
        let mut id = SceneRegistration::derive_id_from_template_path(&template_path);
        if self.world.resource::<SceneCatalog>().is_err() {
            self.world.insert_resource(SceneCatalog::default());
        }
        if let Ok(mut catalog) = self.world.resource_mut::<SceneCatalog>() {
            if catalog.handle(&id).is_some() {
                let mut suffix = 2usize;
                let base = id.clone();
                while catalog.handle(&format!("{base}_{suffix}")).is_some() {
                    suffix = suffix.saturating_add(1);
                }
                id = format!("{base}_{suffix}");
            }
            catalog.register(id, template_path);
        }
        self
    }

    pub fn registered_scene_count(&self) -> usize {
        self.world
            .resource::<SceneCatalog>()
            .map(|catalog| catalog.len())
            .unwrap_or(0)
    }

    pub fn with_control_flow(&mut self, control_flow: ControlFlow) -> &mut Self {
        self.control_flow = control_flow;
        self
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub(crate) fn into_windowed_state(self) -> WindowedAppState {
        WindowedAppState {
            world: self.world,
            scheduler: self.scheduler,
            build_errors: self.build_errors,
            startup_ran: self.startup_ran,
            title: self.title,
            control_flow: self.control_flow,
        }
    }
}
