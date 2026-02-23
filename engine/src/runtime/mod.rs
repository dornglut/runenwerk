use crate::plugins::input::domain::InputState;
use crate::plugins::render::domain::{Gfx, WorldRenderFrame};
use crate::plugins::scene::domain::{OverlaySceneRuntime, SceneManager};
use crate::plugins::time::domain::Time;
use crate::plugins::ui::domain::initialize_console_ui;
use anyhow::Result;
use ecs::World;
use scheduler::Scheduler;
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

pub struct EngineData {
    pub gfx: Gfx,
    pub world_render: WorldRenderFrame,
    pub time: Time,
    pub input: InputState,
    pub scene: SceneManager,
    pub scene_catalog: SceneCatalog,
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

        let data = EngineData {
            gfx,
            world_render: WorldRenderFrame::default(),
            time: Time::new(),
            input: InputState::new(),
            scene,
            scene_catalog: SceneCatalog::from_registrations(&scene_registrations),
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
    use super::{SceneCatalog, SceneRegistration};

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
}
