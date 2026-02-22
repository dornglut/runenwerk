use crate::render::Gfx;
use crate::systems::{
    clear_input_system, game_command_apply_system, game_command_execute_system, time_system,
    ui_build_batches_system, ui_input_system, ui_layout_system, ui_render_extract_system,
    ui_render_submit_system,
};
use crate::ui::{initialize_console_ui, ConsoleUiState};
use anyhow::Result;
use ecs::World;
use scheduler::{Node, Scheduler, SchedulerBuilder};
use std::sync::Arc;
use winit::window::Window;

mod input;
mod time;

pub use input::*;
pub use time::*;

pub struct EngineData {
    pub gfx: Gfx,
    pub world: World,
    pub time: Time,
    pub input: InputState,
    pub ui: ConsoleUiState,
}

pub struct Engine {
    pub scheduler: Scheduler<EngineData>,
    pub data: EngineData,
}

impl Engine {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let gfx = Gfx::new(window.clone())?;
        let mut world = World::new();
        let mut ui = initialize_console_ui(&mut world);
        ui.screen_size = (
            gfx.ctx.surface_config.width as f32,
            gfx.ctx.surface_config.height as f32,
        );
        ui.scale = ui_scale_from_window_factor(window.scale_factor());

        let data = EngineData {
            gfx,
            world,
            time: Time::new(),
            input: InputState::new(),
            ui,
        };

        let scheduler = SchedulerBuilder::<EngineData>::new()
            .add_node("time", Node::new("time", time_system))
            .add_node_with_edges("ui_input", Node::new("ui_input", ui_input_system), &["time"])
            .add_node_with_edges(
                "game_command_apply",
                Node::new("game_command_apply", game_command_apply_system),
                &["ui_input"],
            )
            .add_node_with_edges(
                "game_command_execute",
                Node::new("game_command_execute", game_command_execute_system),
                &["game_command_apply"],
            )
            .add_node_with_edges(
                "ui_layout",
                Node::new("ui_layout", ui_layout_system),
                &["game_command_execute"],
            )
            .add_node_with_edges(
                "ui_build_batches",
                Node::new("ui_build_batches", ui_build_batches_system),
                &["ui_layout"],
            )
            .add_node_with_edges(
                "ui_render_extract",
                Node::new("ui_render_extract", ui_render_extract_system),
                &["ui_build_batches"],
            )
            .add_node_with_edges(
                "ui_render_submit",
                Node::new("ui_render_submit", ui_render_submit_system),
                &["ui_render_extract"],
            )
            .add_node_with_edges(
                "clear_input",
                Node::new("clear_input", clear_input_system),
                &["ui_render_submit"],
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
        self.data.ui.screen_size = (width as f32, height as f32);
        self.data.ui.layout_dirty = true;
    }

    pub fn set_ui_scale_from_window_factor(&mut self, window_scale_factor: f64) {
        let next_scale = ui_scale_from_window_factor(window_scale_factor);
        if (self.data.ui.scale - next_scale).abs() > f32::EPSILON {
            self.data.ui.scale = next_scale;
            self.data.ui.layout_dirty = true;
        }
    }
}

fn ui_scale_from_window_factor(window_scale_factor: f64) -> f32 {
    window_scale_factor as f32
}
