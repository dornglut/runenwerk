use crate::render::{Gfx, WorldRenderFrame};
use crate::systems::{
    clear_input_system, game_command_apply_system, game_command_execute_system,
    scene_overlay_apply_messages_system, scene_overlay_format_messages_system,
    scene_transition_system, time_system, ui_build_batches_system, ui_editor_system,
    ui_hot_reload_system, ui_input_system, ui_layout_system, ui_render_extract_system,
    ui_render_submit_system, world_render_extract_system, world_scene_update_system,
};
use crate::ui::{initialize_console_ui, load_console_template};
use anyhow::Result;
use ecs::World;
use scheduler::{Node, Scheduler, SchedulerBuilder};
use std::sync::Arc;
use winit::window::Window;

mod input;
mod scene;
mod time;

pub use input::*;
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
        let gfx = Gfx::new(window.clone())?;
        let mut overlay_world = World::new();
        let overlay_ui = initialize_console_ui(&mut overlay_world);
        let overlay_runtime = OverlaySceneRuntime {
            world: overlay_world,
            ui: overlay_ui,
        };
        let mut scene = SceneManager::new(overlay_runtime)?;
        if let Some(path) = template_path_for_scene(scene.active_overlay()) {
            let _ = load_console_template(
                &mut scene.overlay_runtime.world,
                &mut scene.overlay_runtime.ui,
                std::path::Path::new(path),
            );
        }
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

        let scheduler = SchedulerBuilder::<EngineData>::new()
            .add_node("time", Node::new("time", time_system))
            .add_node_with_edges(
                "overlay_ui_hot_reload",
                Node::new("overlay_ui_hot_reload", ui_hot_reload_system),
                &["time"],
            )
            .add_node_with_edges(
                "overlay_ui_input",
                Node::new("overlay_ui_input", ui_input_system),
                &["overlay_ui_hot_reload"],
            )
            .add_node_with_edges(
                "overlay_ui_editor",
                Node::new("overlay_ui_editor", ui_editor_system),
                &["overlay_ui_input"],
            )
            .add_node_with_edges(
                "scene_transition",
                Node::new("scene_transition", scene_transition_system),
                &["overlay_ui_editor"],
            )
            .add_node_with_edges(
                "world_scene_update",
                Node::new("world_scene_update", world_scene_update_system),
                &["scene_transition"],
            )
            .add_node_with_edges(
                "world_render_extract",
                Node::new("world_render_extract", world_render_extract_system),
                &["world_scene_update"],
            )
            .add_node_with_edges(
                "scene_overlay_format_messages",
                Node::new(
                    "scene_overlay_format_messages",
                    scene_overlay_format_messages_system,
                ),
                &["world_render_extract"],
            )
            .add_node_with_edges(
                "scene_overlay_apply_messages",
                Node::new(
                    "scene_overlay_apply_messages",
                    scene_overlay_apply_messages_system,
                ),
                &["scene_overlay_format_messages"],
            )
            .add_node_with_edges(
                "game_command_apply",
                Node::new("game_command_apply", game_command_apply_system),
                &["scene_overlay_apply_messages"],
            )
            .add_node_with_edges(
                "game_command_execute",
                Node::new("game_command_execute", game_command_execute_system),
                &["game_command_apply"],
            )
            .add_node_with_edges(
                "overlay_ui_layout",
                Node::new("overlay_ui_layout", ui_layout_system),
                &["game_command_execute"],
            )
            .add_node_with_edges(
                "overlay_ui_build_batches",
                Node::new("overlay_ui_build_batches", ui_build_batches_system),
                &["overlay_ui_layout"],
            )
            .add_node_with_edges(
                "overlay_ui_render_extract",
                Node::new("overlay_ui_render_extract", ui_render_extract_system),
                &["overlay_ui_build_batches"],
            )
            .add_node_with_edges(
                "frame_render_submit",
                Node::new("frame_render_submit", ui_render_submit_system),
                &["overlay_ui_render_extract"],
            )
            .add_node_with_edges(
                "clear_input",
                Node::new("clear_input", clear_input_system),
                &["frame_render_submit"],
            )
            .build()?;

        Ok(Self { scheduler, data })
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
