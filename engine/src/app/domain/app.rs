use crate::app::domain::lifecycle::AppLifecycle;
use crate::app::domain::mode::AppMode;
use crate::app::domain::runner::{AppRunner, FixedFramesRunner};
use crate::app::domain::state::WindowedAppState;
use crate::plugins::input::{ActionState, InputState, PhysicalKeyIdentity};
use crate::plugins::render::inspect::{RenderDebugConfigResource, RenderDebugControlResource};
use crate::plugins::render::{RenderFlow, RenderFlowRegistryResource};
use crate::plugins::{
    SceneReplayArchive, load_replay, seek_loaded_replay, start_recording, stop_recording,
};
use crate::prelude::IntoPlugins;
use crate::runtime::publication::{
    ProductPublicationOccurrence, PublicationHandlers, QuerySnapshotPublicationOccurrence,
};
use crate::runtime::system::IntoSystemConfigs;
use crate::*;
use anyhow::Result;
use engine_sim::*;
use runen_ecs::{Resource, Runtime, RuntimeError, ScheduleLabel, World};
use std::error::Error;
use std::fmt;

const DEFAULT_WINDOW_TITLE: &str = "Runenwerk - Engine";

pub struct App {
    pub(crate) world: World,
    pub(crate) scheduler: Runtime,
    pub(crate) runner: Box<dyn AppRunner>,
    pub(crate) lifecycle: AppLifecycle,
    pub(crate) mode: AppMode,
    pub(crate) title: String,
    composition_errors: Vec<AppCompositionError>,
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
            scheduler: Runtime::new(),
            runner: Box::new(FixedFramesRunner::new(1)),
            lifecycle: AppLifecycle::default(),
            mode,
            title: title.clone(),
            composition_errors: Vec::new(),
        };
        app.install_builtin_resources();
        app
    }

    pub fn add_plugin<P>(&mut self, plugin: P) -> &mut Self
    where
        P: Plugin + 'static,
    {
        if !self.allow_topology_mutation("add_plugin", None) {
            return self;
        }
        plugin.build(self);
        self
    }

    pub fn add_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) -> &mut Self {
        if !self.allow_topology_mutation("add_boxed_plugin", None) {
            return self;
        }
        plugin.build(self);
        self
    }

    pub fn add_plugins<P>(&mut self, plugins: P) -> &mut Self
    where
        P: IntoPlugins,
    {
        if !self.allow_topology_mutation("add_plugins", None) {
            return self;
        }
        plugins.add_to_app(self);
        self
    }

    pub fn add_systems<L, S, Marker>(&mut self, _schedule: L, systems: S) -> &mut Self
    where
        L: ScheduleLabel,
        S: IntoSystemConfigs<Marker>,
    {
        if !self.allow_topology_mutation("add_systems", Some(L::name())) {
            return self;
        }
        if let Err(source) = self
            .scheduler
            .add_systems::<L, S, Marker>(_schedule, systems)
        {
            self.composition_errors
                .push(AppCompositionError::SystemRegistration {
                    schedule: L::name(),
                    source,
                });
        }
        self
    }

    pub fn add_product_publication_handler<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(&ProductPublicationOccurrence, &mut World) -> Result<()> + 'static,
    {
        if !self.allow_topology_mutation("add_product_publication_handler", None) {
            return self;
        }
        if !self.world.has_resource::<PublicationHandlers>() {
            self.world.insert_resource(PublicationHandlers::default());
        }
        self.world
            .resource_mut::<PublicationHandlers>()
            .expect("publication handler resource should be installed")
            .add_product(handler);
        self
    }

    pub fn add_query_snapshot_publication_handler<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(&QuerySnapshotPublicationOccurrence, &mut World) -> Result<()> + 'static,
    {
        if !self.allow_topology_mutation("add_query_snapshot_publication_handler", None) {
            return self;
        }
        if !self.world.has_resource::<PublicationHandlers>() {
            self.world.insert_resource(PublicationHandlers::default());
        }
        self.world
            .resource_mut::<PublicationHandlers>()
            .expect("publication handler resource should be installed")
            .add_query_snapshot(handler);
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
        if let Ok(window) = self.world.resource_mut::<WindowState>() {
            window.set_title(self.title.clone());
        }
        self
    }

    pub fn add_input_bindings<I>(&mut self, bindings: I) -> &mut Self
    where
        I: IntoIterator<Item = (&'static str, PhysicalKeyIdentity)>,
    {
        self.init_resource::<InputState>();
        self.init_resource::<ActionState>();
        let mut actions = self
            .world
            .remove_resource::<ActionState>()
            .unwrap_or_default();
        {
            let input = self
                .world
                .resource::<InputState>()
                .expect("input state should be installed");
            for (action, key) in bindings {
                actions.map_key(input, action.to_string(), key);
            }
        }
        self.world.insert_resource(actions);
        self
    }

    pub fn add_render_flow(&mut self, flow: RenderFlow) -> &mut Self {
        if self.world.resource::<RenderFlowRegistryResource>().is_err() {
            self.world
                .insert_resource(RenderFlowRegistryResource::default());
        }
        if let Ok(registry) = self.world.resource_mut::<RenderFlowRegistryResource>() {
            registry.upsert_flow(flow);
        }
        self
    }

    pub fn update_render_debug_control<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugControlResource),
    {
        self.init_resource::<RenderDebugControlResource>();
        if let Ok(control) = self.world.resource_mut::<RenderDebugControlResource>() {
            update(control);
        }
        self
    }

    pub fn update_render_debug_config<F>(&mut self, update: F) -> &mut Self
    where
        F: FnOnce(&mut RenderDebugConfigResource),
    {
        self.init_resource::<RenderDebugConfigResource>();
        if let Ok(config) = self.world.resource_mut::<RenderDebugConfigResource>() {
            update(config);
        }
        self
    }

    pub fn set_simulation_profile(&mut self, profile: SimulationProfile) -> &mut Self {
        self.init_resource::<SimulationProfileConfig>();
        if let Ok(config) = self.world.resource_mut::<SimulationProfileConfig>() {
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
        self.init_resource::<SimulationProfileConfig>();
        if let Ok(config) = self.world.resource_mut::<SimulationProfileConfig>() {
            config.authority = authority;
        }
        if let Ok(world_runtime_config) =
            self.world
                .resource_mut::<crate::plugins::world::plugin::WorldRuntimeConfig>()
        {
            world_runtime_config.mode =
                crate::plugins::world::plugin::world_runtime_mode_for_authority(authority);
        }
        self
    }

    pub fn set_simulation_seed(&mut self, seed: SimulationSeed) -> &mut Self {
        self.world.insert_resource(seed);
        self.world.insert_resource(SimulationRng::from_seed(seed));
        self
    }

    pub fn start_recording(&mut self) -> Result<&mut Self> {
        start_recording(&mut self.world)?;
        Ok(self)
    }

    pub fn stop_recording(&mut self) -> Result<SceneReplayArchive> {
        stop_recording(&mut self.world)
    }

    pub fn load_replay(&mut self, archive: SceneReplayArchive) -> Result<&mut Self> {
        load_replay(&mut self.world, archive)?;
        Ok(self)
    }

    pub fn seek_tick(&mut self, tick: u64) -> Result<ReplayValidationReport> {
        seek_loaded_replay(&mut self.world, SimulationTick(tick))
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
        if let Ok(catalog) = self.world.resource_mut::<SceneCatalog>() {
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
        if let Ok(catalog) = self.world.resource_mut::<SceneCatalog>() {
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

    pub fn with_frame_pacing(&mut self, policy: FramePacingPolicyResource) -> &mut Self {
        self.world.insert_resource(policy);
        if let Ok(runtime_state) = self.world.resource_mut::<FramePacingRuntimeStateResource>() {
            runtime_state.observe_policy(policy);
        }
        self
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub(crate) fn into_windowed_state(mut self) -> WindowedAppState {
        if self.lifecycle.is_configuring() {
            self.admit_composition()
                .expect("windowed Host-state transfer requires admitted App composition");
            self.lifecycle
                .prepare_for_execution()
                .expect("Configuring App lifecycle should prepare for windowed Host transfer");
        }
        self.prepare_windowed_frame_pacing();
        WindowedAppState {
            world: self.world,
            scheduler: self.scheduler,
            startup_ran: self.lifecycle,
            title: self.title,
        }
    }

    pub(crate) fn admit_composition(&mut self) -> Result<()> {
        if !self.composition_errors.is_empty() {
            let errors = std::mem::take(&mut self.composition_errors);
            return Err(anyhow::Error::new(
                AppCompositionAdmissionError::Registration(errors),
            ));
        }

        self.scheduler
            .validate()
            .map_err(|source| anyhow::Error::new(AppCompositionAdmissionError::Topology(source)))
    }

    pub(crate) fn prepare_lifecycle_for_execution(&mut self) -> Result<()> {
        self.lifecycle.prepare_for_execution()
    }

    fn allow_topology_mutation(
        &mut self,
        operation: &'static str,
        target: Option<&'static str>,
    ) -> bool {
        if self.lifecycle.is_configuring() {
            return true;
        }

        self.composition_errors
            .push(AppCompositionError::LateTopologyMutation {
                operation,
                target,
                lifecycle: self.lifecycle,
            });
        false
    }
}

#[derive(Debug)]
enum AppCompositionError {
    SystemRegistration {
        schedule: &'static str,
        source: RuntimeError,
    },
    LateTopologyMutation {
        operation: &'static str,
        target: Option<&'static str>,
        lifecycle: AppLifecycle,
    },
}

impl fmt::Display for AppCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SystemRegistration { schedule, source } => write!(
                formatter,
                "failed to register systems in schedule '{}': {}",
                schedule, source
            ),
            Self::LateTopologyMutation {
                operation,
                target,
                lifecycle,
            } => {
                write!(
                    formatter,
                    "{operation} rejected after App composition was sealed (lifecycle: {})",
                    lifecycle.name()
                )?;
                if let Some(target) = target {
                    write!(formatter, "; target: {target}")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for AppCompositionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SystemRegistration { source, .. } => Some(source),
            Self::LateTopologyMutation { .. } => None,
        }
    }
}

#[derive(Debug)]
enum AppCompositionAdmissionError {
    Registration(Vec<AppCompositionError>),
    Topology(RuntimeError),
}

impl fmt::Display for AppCompositionAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registration(errors) => {
                write!(formatter, "App composition admission rejected")?;
                for (index, error) in errors.iter().enumerate() {
                    write!(formatter, "; registration error {}: {error}", index + 1)?;
                }
                Ok(())
            }
            Self::Topology(source) => {
                write!(formatter, "App topology admission rejected: {source}")
            }
        }
    }
}

impl Error for AppCompositionAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registration(_) => None,
            Self::Topology(source) => Some(source),
        }
    }
}
