use crate::plugins::input::domain::InputState;
use crate::plugins::render::domain::{
    Gfx, GfxFrameTimings, RenderGraphRegistryResource, RenderPassExecutorRegistryResource,
    ShaderRegistryResource, WorldRenderFrame,
};
use crate::plugins::scene::domain::{OverlaySceneRuntime, SceneManager};
use crate::plugins::time::domain::Time;
use crate::plugins::ui::domain::initialize_console_ui;
use anyhow::Result;
use ecs::World;
use scheduler::{Scheduler, set_slow_node_logging_enabled};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use winit::window::Window;

mod plugin;
pub use plugin::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneRegistration {
    pub id: String,
    pub template_path: String,
}

impl SceneRegistration {
    pub fn new(id: impl Into<String>, template_path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            template_path: template_path.into(),
        }
    }

    pub fn from_template_path(template_path: impl Into<String>) -> Self {
        let template_path = template_path.into();
        let id = Self::derive_id_from_template_path(&template_path);
        Self { id, template_path }
    }

    pub fn derive_id_from_template_path(template_path: &str) -> String {
        let raw = Path::new(template_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("scene");
        let mut out = String::new();
        let mut previous_was_sep = false;
        for ch in raw.chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch.to_ascii_lowercase());
                previous_was_sep = false;
            } else if !previous_was_sep {
                out.push('_');
                previous_was_sep = true;
            }
        }
        let normalized = out.trim_matches('_');
        if normalized.is_empty() {
            "scene".to_string()
        } else {
            normalized.to_string()
        }
    }
}

impl From<String> for SceneRegistration {
    fn from(template_path: String) -> Self {
        Self::from_template_path(template_path)
    }
}

impl From<&str> for SceneRegistration {
    fn from(template_path: &str) -> Self {
        Self::from_template_path(template_path)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SceneHandle(u32);

impl SceneHandle {
    pub fn index(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredScene {
    pub handle: SceneHandle,
    pub id: String,
    pub template_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct SceneCatalog {
    scenes: Vec<RegisteredScene>,
    by_id: HashMap<String, SceneHandle>,
}

impl SceneCatalog {
    pub fn from_registrations(registrations: &[SceneRegistration]) -> Self {
        let mut catalog = Self::default();
        for registration in registrations {
            catalog.register(registration.id.clone(), registration.template_path.clone());
        }
        catalog
    }

    pub fn register(
        &mut self,
        id: impl Into<String>,
        template_path: impl Into<String>,
    ) -> SceneHandle {
        let id = id.into();
        if let Some(existing) = self.by_id.get(&id).copied() {
            tracing::warn!(scene_id = %id, "duplicate scene registration id ignored");
            return existing;
        }

        let handle = SceneHandle(self.scenes.len() as u32);
        let scene = RegisteredScene {
            handle,
            id: id.clone(),
            template_path: template_path.into(),
        };
        self.by_id.insert(id, handle);
        self.scenes.push(scene);
        handle
    }

    pub fn len(&self) -> usize {
        self.scenes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }

    pub fn handle(&self, id: &str) -> Option<SceneHandle> {
        self.by_id.get(id).copied()
    }

    pub fn get(&self, handle: SceneHandle) -> Option<&RegisteredScene> {
        self.scenes.get(handle.index() as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RegisteredScene> {
        self.scenes.iter()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StartupPhase {
    Loading,
    Ready,
}

#[derive(Debug, Copy, Clone)]
pub struct StartupState {
    pub phase: StartupPhase,
    pub stable_frames: u32,
    pub required_stable_frames: u32,
    pub elapsed_loading_seconds: f32,
    pub max_loading_seconds: f32,
}

impl StartupState {
    pub const DEFAULT_REQUIRED_STABLE_FRAMES: u32 = 8;
    pub const DEFAULT_MAX_LOADING_SECONDS: f32 = 5.0;

    pub fn loading() -> Self {
        Self {
            phase: StartupPhase::Loading,
            stable_frames: 0,
            required_stable_frames: Self::DEFAULT_REQUIRED_STABLE_FRAMES,
            elapsed_loading_seconds: 0.0,
            max_loading_seconds: Self::DEFAULT_MAX_LOADING_SECONDS,
        }
    }

    pub fn ready() -> Self {
        let mut state = Self::loading();
        state.phase = StartupPhase::Ready;
        state.stable_frames = state.required_stable_frames;
        state
    }

    pub fn from_loading_enabled(enabled: bool) -> Self {
        if enabled {
            Self::loading()
        } else {
            Self::ready()
        }
    }

    pub fn is_loading(&self) -> bool {
        self.phase == StartupPhase::Loading
    }

    pub fn is_ready(&self) -> bool {
        self.phase == StartupPhase::Ready
    }

    /// Returns true when warmup transitions from loading -> ready.
    pub fn observe_render_warm_frame(&mut self, warm_frame: bool, delta_seconds: f32) -> bool {
        if self.is_ready() {
            return false;
        }

        self.elapsed_loading_seconds += delta_seconds.max(0.0);
        if warm_frame {
            self.stable_frames = self.stable_frames.saturating_add(1);
        } else {
            self.stable_frames = 0;
        }

        let stable_target = self.required_stable_frames.max(1);
        let timed_out = self.elapsed_loading_seconds >= self.max_loading_seconds.max(0.0);
        if self.stable_frames >= stable_target || timed_out {
            self.phase = StartupPhase::Ready;
            return true;
        }

        false
    }
}

pub struct EngineData {
    pub gfx: Gfx,
    pub render_resources: World,
    pub shader_registry: ShaderRegistryResource,
    pub render_graph_registry: RenderGraphRegistryResource,
    pub render_executor_registry: RenderPassExecutorRegistryResource,
    pub time: Time,
    pub input: InputState,
    pub scene: SceneManager,
    pub scene_catalog: SceneCatalog,
    pub startup: StartupState,
    pub debug_metrics: DebugMetricsState,
}

impl EngineData {
    pub fn world_render(&self) -> &WorldRenderFrame {
        self.render_resources
            .get_resource::<WorldRenderFrame>()
            .expect("world render frame resource should be present")
    }

    pub fn world_render_mut(&mut self) -> &mut WorldRenderFrame {
        self.render_resources
            .get_resource_mut::<WorldRenderFrame>()
            .expect("world render frame resource should be present")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DebugMetricsState {
    pub visible: bool,
    pub fps_ema: f32,
    pub frame_ms_ema: f32,
    pub last_timings: Option<GfxFrameTimings>,
}

impl Default for DebugMetricsState {
    fn default() -> Self {
        Self {
            visible: false,
            fps_ema: 0.0,
            frame_ms_ema: 0.0,
            last_timings: None,
        }
    }
}

impl DebugMetricsState {
    pub fn observe_frame_delta(&mut self, delta_seconds: f32) {
        let safe_dt = delta_seconds.max(1.0 / 1000.0);
        let fps = (1.0 / safe_dt).clamp(0.0, 2000.0);
        let frame_ms = (safe_dt * 1000.0).clamp(0.0, 1000.0);
        let alpha = 0.12;
        if self.fps_ema <= f32::EPSILON {
            self.fps_ema = fps;
        } else {
            self.fps_ema = self.fps_ema + (fps - self.fps_ema) * alpha;
        }
        if self.frame_ms_ema <= f32::EPSILON {
            self.frame_ms_ema = frame_ms;
        } else {
            self.frame_ms_ema = self.frame_ms_ema + (frame_ms - self.frame_ms_ema) * alpha;
        }
    }
}

pub struct Engine {
    pub scheduler: Scheduler<EngineData>,
    pub data: EngineData,
}

impl Engine {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        Self::new_with_plugins_and_scenes(window, &[], Vec::new())
    }

    pub fn new_with_plugins(
        window: Arc<Window>,
        plugins: &[Box<dyn EnginePlugin>],
    ) -> Result<Self> {
        Self::new_with_plugins_and_scenes(window, plugins, Vec::new())
    }

    pub fn new_with_plugins_and_scenes(
        window: Arc<Window>,
        plugins: &[Box<dyn EnginePlugin>],
        scene_registrations: Vec<SceneRegistration>,
    ) -> Result<Self> {
        let gfx = Gfx::new(window.clone())?;
        let mut overlay_world = World::new();
        let overlay_ui = initialize_console_ui(&mut overlay_world);
        let overlay_runtime = OverlaySceneRuntime {
            world: overlay_world,
            ui: overlay_ui,
        };
        let mut scene = SceneManager::new(overlay_runtime)?;
        scene.overlay_runtime.ui.screen_size = (
            gfx.ctx.surface_config.width as f32,
            gfx.ctx.surface_config.height as f32,
        );
        scene.overlay_runtime.ui.scale = ui_scale_from_window_factor(window.scale_factor());
        let scene_catalog = SceneCatalog::from_registrations(&scene_registrations);
        // Startup warmup is globally enabled by default so heavy first-frame work
        // is absorbed before diagnostics and transition-dependent logic treat the
        // runtime as fully ready.
        let startup = StartupState::loading();
        set_slow_node_logging_enabled(startup.is_ready());

        let mut render_resources = World::new();
        render_resources.insert_resource(WorldRenderFrame::default());

        let data = EngineData {
            gfx,
            render_resources,
            shader_registry: ShaderRegistryResource::new(),
            render_graph_registry: RenderGraphRegistryResource::default(),
            render_executor_registry: RenderPassExecutorRegistryResource::default(),
            time: Time::new(),
            input: InputState::new(),
            scene,
            scene_catalog,
            startup,
            debug_metrics: DebugMetricsState::default(),
        };

        let mut schedule_builder = EngineScheduleBuilder::new();
        for plugin in plugins {
            tracing::info!(plugin = plugin.name(), "configuring engine plugin");
            plugin.configure(&mut schedule_builder)?;
        }
        let scheduler = schedule_builder.build_scheduler()?;
        let mut engine = Self { scheduler, data };
        for plugin in plugins {
            tracing::info!(plugin = plugin.name(), "setting up engine plugin");
            plugin.setup(&mut engine.data)?;
        }

        Ok(engine)
    }

    pub fn update(&mut self) {
        if let Err(err) = self.scheduler.run(&mut self.data) {
            tracing::error!(?err, "scheduler run failed");
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.data.gfx.resize(width, height);
        let scale = self.data.scene.overlay_runtime.ui.scale;
        self.data
            .scene
            .set_overlay_viewport((width as f32, height as f32), scale);
    }

    pub fn set_ui_scale_from_window_factor(&mut self, window_scale_factor: f64) {
        let next_scale = ui_scale_from_window_factor(window_scale_factor);
        if (self.data.scene.overlay_runtime.ui.scale - next_scale).abs() > f32::EPSILON {
            let size = self.data.scene.overlay_runtime.ui.screen_size;
            self.data.scene.set_overlay_viewport(size, next_scale);
        }
    }
}

fn ui_scale_from_window_factor(window_scale_factor: f64) -> f32 {
    window_scale_factor as f32
}

#[cfg(test)]
mod tests {
    use super::{SceneCatalog, SceneRegistration, StartupPhase, StartupState};

    #[test]
    fn scene_catalog_assigns_stable_sequential_handles() {
        let registrations = vec![
            SceneRegistration::new("main_menu", "a/main.ron"),
            SceneRegistration::new("settings_menu", "a/settings.ron"),
            SceneRegistration::new("game_scene", "a/game.ron"),
        ];
        let catalog = SceneCatalog::from_registrations(&registrations);
        assert_eq!(catalog.len(), 3);

        let main = catalog
            .handle("main_menu")
            .expect("main menu handle should exist");
        let settings = catalog
            .handle("settings_menu")
            .expect("settings handle should exist");
        let game = catalog
            .handle("game_scene")
            .expect("game handle should exist");
        assert_eq!(main.index(), 0);
        assert_eq!(settings.index(), 1);
        assert_eq!(game.index(), 2);
    }

    #[test]
    fn scene_catalog_ignores_duplicate_ids() {
        let registrations = vec![
            SceneRegistration::new("main_menu", "a/main.ron"),
            SceneRegistration::new("main_menu", "a/duplicate.ron"),
        ];
        let catalog = SceneCatalog::from_registrations(&registrations);
        assert_eq!(catalog.len(), 1);
        let main = catalog
            .handle("main_menu")
            .expect("main menu handle should exist");
        let descriptor = catalog.get(main).expect("descriptor should exist");
        assert_eq!(descriptor.template_path, "a/main.ron");
    }

    #[test]
    fn scene_registration_derives_id_from_template_path() {
        let registration = SceneRegistration::from_template_path("assets/scenes/Main-Menu.ron");
        assert_eq!(registration.id, "main_menu");
        assert_eq!(registration.template_path, "assets/scenes/Main-Menu.ron");
    }

    #[test]
    fn startup_state_can_start_ready_without_loading_scene() {
        let startup = StartupState::from_loading_enabled(false);
        assert!(startup.is_ready());
        assert_eq!(startup.phase, StartupPhase::Ready);
    }

    #[test]
    fn startup_state_transitions_to_ready_after_stable_warm_frames() {
        let mut startup = StartupState::loading();
        startup.required_stable_frames = 3;
        startup.max_loading_seconds = 100.0;

        assert!(!startup.observe_render_warm_frame(false, 0.016));
        assert!(!startup.observe_render_warm_frame(true, 0.016));
        assert!(!startup.observe_render_warm_frame(true, 0.016));
        assert!(startup.observe_render_warm_frame(true, 0.016));
        assert!(startup.is_ready());
    }

    #[test]
    fn startup_state_transitions_to_ready_on_timeout() {
        let mut startup = StartupState::loading();
        startup.required_stable_frames = 100;
        startup.max_loading_seconds = 0.05;

        assert!(!startup.observe_render_warm_frame(false, 0.01));
        assert!(!startup.observe_render_warm_frame(false, 0.02));
        assert!(startup.observe_render_warm_frame(false, 0.02));
        assert!(startup.is_ready());
    }
}
