use crate::plugin::Plugin;
use crate::plugins::input::domain::InputState;
use crate::plugins::replay as replay_runtime;
use crate::plugins::scene::SceneReplayArchive;
use crate::plugins::time::domain::Time;
use crate::runtime::fixed_time::{CatchupBudget, FixedTimeConfig, FixedTimeState, SimulationTick};
use crate::runtime::schedules::{
    FixedUpdate, FrameEnd, PreUpdate, RenderPrepare, RenderSubmit, Startup, Update,
};
use crate::runtime::system::IntoSystemConfigs;
use crate::runtime::window::WindowState;
use crate::runtime::winit_runner;
use crate::state::{
    GameplayRuntimeConfig, SceneCatalog, SceneRegistration, SceneRuntimeState, SessionRuntimeState,
    StartupState, UiOverlayState,
};
use anyhow::{Result, anyhow};
use ecs::{Resource, World};
use engine_replay::ReplayValidationReport;
use engine_sim::{
    AuthorityRole, SimulationProfile, SimulationProfileConfig, SimulationRng, SimulationSeed,
    SimulationSessionId,
};
use scheduler::label::ScheduleLabel;
use scheduler::plan::ExecutionScheduler;
use winit::event_loop::ControlFlow;

const DEFAULT_WINDOW_TITLE: &str = "Grotto Quest - Engine";

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
pub struct FixedTicksRunner {
    target_ticks: u64,
}

impl FixedTicksRunner {
    pub fn new(target_ticks: u64) -> Self {
        Self { target_ticks }
    }
}

impl AppRunner for FixedTicksRunner {
    fn next_frame(&mut self, _completed_frames: usize, world: &World) -> bool {
        world
            .resource::<SimulationTick>()
            .map(|tick| tick.0 < self.target_ticks)
            .unwrap_or(false)
    }

    fn before_frame(&mut self, world: &mut World) {
        let fixed_step_seconds = world
            .resource::<FixedTimeConfig>()
            .map(|config| config.step_seconds)
            .unwrap_or(1.0 / 60.0);
        if let Ok(mut time) = world.resource_mut::<Time>() {
            time.delta_seconds = fixed_step_seconds;
        }
    }
}

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

    pub fn run(mut self) -> Result<()> {
        match self.mode {
            AppMode::Windowed => winit_runner::run(self.into_windowed_state()),
            AppMode::Headless => {
                self.run_headless()?;
                Ok(())
            }
        }
    }

    pub fn run_for_frames(mut self, frame_count: usize) -> Result<Self> {
        self.set_runner(FixedFramesRunner::new(frame_count));
        self.run_headless()?;
        Ok(self)
    }

    pub fn run_for_ticks(mut self, tick_count: u64) -> Result<Self> {
        self.set_runner(FixedTicksRunner::new(tick_count));
        self.run_headless()?;
        Ok(self)
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

    fn install_builtin_resources(&mut self) {
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
            self.world
                .insert_resource(SimulationProfileConfig::default());
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

    fn run_headless(&mut self) -> Result<()> {
        self.prepare_for_run(true)?;

        let mut completed_frames = 0usize;
        while self.runner.next_frame(completed_frames, &self.world) {
            self.runner.before_frame(&mut self.world);
            self.run_frame()?;
            completed_frames = completed_frames.saturating_add(1);
        }

        Ok(())
    }

    pub(crate) fn prepare_for_run(&mut self, headless: bool) -> Result<()> {
        self.ensure_build_ready()?;
        if let Ok(mut window) = self.world.resource_mut::<WindowState>() {
            window.set_headless(headless);
            window.redraw_requested = false;
            window.close_requested = false;
            window.title = self.title.clone();
        }
        if !self.startup_ran {
            self.scheduler.run_schedule::<Startup>(&mut self.world)?;
            self.startup_ran = true;
        }
        Ok(())
    }

    pub(crate) fn run_frame(&mut self) -> Result<()> {
        self.scheduler.run_schedule::<PreUpdate>(&mut self.world)?;
        self.run_fixed_update_schedule()?;
        self.scheduler.run_schedule::<Update>(&mut self.world)?;
        self.scheduler
            .run_schedule::<RenderPrepare>(&mut self.world)?;
        self.scheduler
            .run_schedule::<RenderSubmit>(&mut self.world)?;
        self.scheduler.run_schedule::<FrameEnd>(&mut self.world)?;
        Ok(())
    }

    fn ensure_build_ready(&self) -> Result<()> {
        if self.build_errors.is_empty() {
            return Ok(());
        }
        let messages: Vec<_> = self.build_errors.iter().map(ToString::to_string).collect();
        Err(anyhow!("app setup failed:\n{}", messages.join("\n")))
    }

    fn run_fixed_update_schedule(&mut self) -> Result<()> {
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

        if steps > 0 {
            let mut tick = self
                .world
                .resource_mut::<SimulationTick>()
                .expect("SimulationTick should be installed");
            tick.0 = tick.0.saturating_add(steps as u64);
        }

        Ok(())
    }
}

pub trait IntoPlugins {
    fn add_to_app(self, app: &mut App);
}

impl<P> IntoPlugins for P
where
    P: Plugin + 'static,
{
    fn add_to_app(self, app: &mut App) {
        app.add_plugin(self);
    }
}

impl IntoPlugins for Box<dyn Plugin> {
    fn add_to_app(self, app: &mut App) {
        app.add_boxed_plugin(self);
    }
}

impl IntoPlugins for Vec<Box<dyn Plugin>> {
    fn add_to_app(self, app: &mut App) {
        for plugin in self {
            app.add_boxed_plugin(plugin);
        }
    }
}

macro_rules! impl_into_plugins_tuple {
    ($(($name:ident, $index:tt)),+ $(,)?) => {
        impl<$($name),+> IntoPlugins for ($($name,)+)
        where
            $($name: IntoPlugins,)+
        {
            fn add_to_app(self, app: &mut App) {
                $(
                    self.$index.add_to_app(app);
                )+
            }
        }
    };
}

impl_into_plugins_tuple!((A, 0), (B, 1));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
impl_into_plugins_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
impl_into_plugins_tuple!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);
