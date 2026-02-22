use std::sync::Arc;
use winit::window::Window;
use anyhow::Result;
use tracing::*;
use scheduler::{Node, Scheduler, SchedulerBuilder};
use crate::engine::{chunk_system, clear_input_system, gpu_sync_system, render_system, time_system};
use crate::engine::gfx::gfx::Gfx;
use crate::engine::gpu_resources::GpuRegistry;
use crate::engine::input::InputState;
use crate::engine::systems::world_system::world_update_system;
use crate::engine::time::Time;
use crate::engine::world::World;
use crate::utils::Metrics;

/// Holds the actual engine state that systems will mutate
pub struct EngineData {
    pub gfx: Gfx,
    pub running: bool,
    pub gpu_resources: GpuRegistry,
    pub world: World,
    pub time: Time,
    pub metrics: Metrics,
    pub input: InputState,
}

/// Engine struct that owns the scheduler and engine data
pub struct Engine {
    pub scheduler: Scheduler<EngineData>,
    pub data: EngineData,
}

impl Engine {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        info!("Initializing Engine with window...");

        // Initialize graphics
        let mut gfx = Gfx::new(window)?;
        info!("Graphics context initialized {:?}", gfx.ctx.adapter.get_info());

        // GPU resources
        let gpu_resources = GpuRegistry::new(gfx.ctx.device.clone());
        info!("GPU registry initialized");

        gfx.init_renderer(&gpu_resources.camera);
        info!("GPU renderer initialized");

        // World, input, time, metrics
        let world = World::new();
        let time = Time::new();
        let input = InputState::new();
        let metrics = Metrics::new(0.1);

        let data = EngineData {
            gfx,
            running: true,
            gpu_resources,
            world,
            time,
            metrics,
            input,
        };

        // Scheduler nodes
        let mut scheduler = SchedulerBuilder::<EngineData>::new()
          .add_node("TimeSystem", Node::new("TimeSystem", time_system))
          .add_node_with_edges("WorldUpdate", Node::new("WorldUpdate", world_update_system), &["TimeSystem"])
          //.add_node_with_edges("ChunkSystem", Node::new("ChunkSystem", chunk_system), &["WorldUpdate"])
          .add_node_with_edges("GPUSyncSystem", Node::new("GPUSyncSystem", gpu_sync_system), &["WorldUpdate", "ChunkSystem"])
          .add_node_with_edges("RenderSystem", Node::new("RenderSystem", render_system), &["GPUSyncSystem"])
          .add_node_with_edges("ClearInputSystem", Node::new("ClearInputSystem", clear_input_system), &["RenderSystem"])
          .build();
        scheduler.rebuild_execution_order()?;


        info!("Engine initialization complete");

        Ok(Self { scheduler, data })
    }

    pub fn new_headless() -> Result<Self> {
        info!("Initializing headless engine...");

        let mut gfx = Gfx::new_headless()?;
        info!("Headless graphics context initialized: {:?}", gfx.ctx.adapter.get_info());

        let gpu_resources = GpuRegistry::new(gfx.ctx.device.clone());
        info!("GPU registry initialized");

        gfx.init_renderer(&gpu_resources.camera);
        info!("GPU renderer initialized");

        let world = World::new();
        let time = Time::new();
        let input = InputState::new();
        let metrics = Metrics::new(0.1);

        let data = EngineData {
            gfx,
            running: true,
            gpu_resources,
            world,
            time,
            metrics,
            input,
        };

        let mut scheduler = SchedulerBuilder::<EngineData>::new()
          .add_node("ClearInputSystem", Node::new("ClearInputSystem", clear_input_system))
          .add_node("TimeSystem", Node::new("TimeSystem", time_system))
          .add_node_with_edges("WorldUpdate", Node::new("WorldUpdate", world_update_system), &["ClearInputSystem"])
          .add_node_with_edges("ChunkSystem", Node::new("ChunkSystem", chunk_system), &["WorldUpdate"])
          .add_node_with_edges("GPUSyncSystem", Node::new("GPUSyncSystem", gpu_sync_system), &["WorldUpdate", "ChunkSystem"])
          .build();
        scheduler.rebuild_execution_order()?;

        info!("Headless Engine initialization complete");

        Ok(Self { scheduler, data })
    }

    /// Frame update using scheduler
    pub fn update(&mut self) {
        // Run scheduler on EngineData
        if let Err(e) = self.scheduler.run(&mut self.data) {
            error!("Scheduler run failed: {:?}", e);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.data.gfx.resize(width, height);
        self.data.world.camera.aspect_ratio = width as f32 / height as f32;
    }
}
