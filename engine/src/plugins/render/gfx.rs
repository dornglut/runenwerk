use super::render_executor_registry::{
    RenderFrameDataRegistry, RenderPassExecutorRegistryResource,
};
use super::render_graph_registry::RenderGraphRegistryResource;
use super::renderer::{Renderer, RendererFrameTimings};
use super::shader_manager::{ShaderHandle, ShaderRegistryResource};
use super::wgpu_ctx::WgpuCtx;
use crate::plugins::ui::domain::UiDrawList;
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use wgpu::SurfaceError;
use winit::window::Window;

#[derive(Debug, ecs::Component)]
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

    pub fn render(
        &mut self,
        frame_data: &RenderFrameDataRegistry<'_>,
        draw_list: &UiDrawList,
        shader_registry: &mut ShaderRegistryResource,
        render_graph_registry: &RenderGraphRegistryResource,
        render_executor_registry: &RenderPassExecutorRegistryResource,
        ui_rect_shader: Option<ShaderHandle>,
    ) -> Result<GfxFrameTimings, SurfaceError> {
        let mut timings = GfxFrameTimings::default();
        let acquire_start = Instant::now();
        let frame = self.ctx.get_current_texture()?;
        timings.acquire_ms = acquire_start.elapsed().as_secs_f32() * 1000.0;
        let view = frame.texture.create_view(&Default::default());
        let packet = self.renderer.prepare_packet(
            &self.ctx.device,
            &self.ctx.queue,
            frame_data,
            draw_list,
            shader_registry,
            ui_rect_shader,
            self.ctx.surface_config.format,
            self.ctx.surface_config.width as f32,
            self.ctx.surface_config.height as f32,
        );
        timings.renderer = self.renderer.render_packet(
            &self.ctx.device,
            &self.ctx.queue,
            &view,
            frame_data,
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
