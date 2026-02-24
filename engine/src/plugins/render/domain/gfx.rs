use super::{
    RenderGraphRegistryResource, RenderPassExecutorRegistryResource, Renderer,
    RendererFrameTimings, WgpuCtx, WorldRenderFrame,
};
use crate::plugins::ui::domain::UiDrawList;
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use wgpu::SurfaceError;
use winit::window::Window;

#[derive(Debug)]
pub struct Gfx {
    pub ctx: WgpuCtx<'static>,
    pub renderer: Renderer,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GfxFrameTimings {
    pub acquire_ms: f32,
    pub renderer: RendererFrameTimings,
    pub present_ms: f32,
}

impl Gfx {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let ctx = WgpuCtx::new(window)?;
        Ok(Self {
            ctx,
            renderer: Renderer::new(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }

    pub fn poll_shader_hot_reload(&mut self) -> Vec<String> {
        self.renderer.poll_shader_hot_reload()
    }

    pub fn force_shader_reload(&mut self) -> Vec<String> {
        self.renderer.force_shader_reload()
    }

    pub fn set_shader_watch_enabled(&mut self, enabled: bool) {
        self.renderer.set_shader_watch_enabled(enabled);
    }

    pub fn shader_watch_enabled(&self) -> bool {
        self.renderer.shader_watch_enabled()
    }

    pub fn shader_status_lines(&self) -> Vec<String> {
        self.renderer.shader_status_lines()
    }

    pub fn poll_model_hot_reload(&mut self) -> Vec<String> {
        self.renderer.poll_model_hot_reload()
    }

    pub fn force_model_reload(&mut self) -> Vec<String> {
        self.renderer.force_model_reload()
    }

    pub fn set_model_watch_enabled(&mut self, enabled: bool) {
        self.renderer.set_model_watch_enabled(enabled);
    }

    pub fn model_watch_enabled(&self) -> bool {
        self.renderer.model_watch_enabled()
    }

    pub fn model_status_lines(&self) -> Vec<String> {
        self.renderer.model_status_lines()
    }

    pub fn render(
        &mut self,
        world_frame: &WorldRenderFrame,
        draw_list: &UiDrawList,
        render_graph_registry: &RenderGraphRegistryResource,
        render_executor_registry: &RenderPassExecutorRegistryResource,
    ) -> Result<GfxFrameTimings, SurfaceError> {
        let mut timings = GfxFrameTimings::default();
        let acquire_start = Instant::now();
        let frame = self.ctx.get_current_texture()?;
        timings.acquire_ms = acquire_start.elapsed().as_secs_f32() * 1000.0;
        let view = frame.texture.create_view(&Default::default());
        let packet = self.renderer.prepare_packet(
            &self.ctx.device,
            &self.ctx.queue,
            world_frame,
            draw_list,
            self.ctx.surface_config.format,
            self.ctx.surface_config.width as f32,
            self.ctx.surface_config.height as f32,
        );
        timings.renderer = self.renderer.render_packet(
            &self.ctx.device,
            &self.ctx.queue,
            &view,
            packet,
            render_graph_registry,
            render_executor_registry,
        );

        let present_start = Instant::now();
        frame.present();
        timings.present_ms = present_start.elapsed().as_secs_f32() * 1000.0;
        Ok(timings)
    }
}
