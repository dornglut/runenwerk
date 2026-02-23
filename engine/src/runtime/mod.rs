use crate::render::{Gfx, WorldRenderFrame};
use crate::ui::initialize_console_ui;
use anyhow::Result;
use ecs::World;
use scheduler::Scheduler;
use std::sync::Arc;
use winit::window::Window;

#[path = "../plugins/input/domain.rs"]
mod input;
mod plugin;
#[path = "../plugins/scene/domain/mod.rs"]
mod scene;
#[path = "../plugins/time/domain.rs"]
mod time;

pub use input::*;
pub use plugin::*;
pub use scene::*;
pub use time::*;

pub struct EngineData {
    pub gfx: Gfx,
    pub world_render: WorldRenderFrame,
    pub time: Time,
    pub input: InputState,
    pub scene: SceneManager,
}

pub struct Engine {
    pub scheduler: Scheduler<EngineData>,
    pub data: EngineData,
}

impl Engine {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        Self::new_with_plugins(window, &[])
    }

    pub fn new_with_plugins(
        window: Arc<Window>,
        plugins: &[Box<dyn EnginePlugin>],
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
