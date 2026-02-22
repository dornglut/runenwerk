use super::{PassSlot, PipelineKey, PipelineSelection, Renderer, WgpuCtx, WorldRenderFrame};
use crate::ui::UiDrawList;
use anyhow::Result;
use std::sync::Arc;
use wgpu::SurfaceError;
use winit::window::Window;

#[derive(Debug)]
pub struct Gfx {
    pub ctx: WgpuCtx<'static>,
    pub renderer: Renderer,
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

    pub fn pipeline_selection(&self) -> PipelineSelection {
        self.renderer.pipeline_selection()
    }

    pub fn set_pipeline_for_slot(&mut self, slot: PassSlot, key: PipelineKey) -> Result<()> {
        self.renderer.set_pipeline_for_slot(slot, key)
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
    ) -> Result<(), SurfaceError> {
        let frame = self.ctx.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        self.renderer.render(
            &self.ctx.device,
            &self.ctx.queue,
            &view,
            world_frame,
            draw_list,
            self.ctx.surface_config.format,
            self.ctx.surface_config.width as f32,
            self.ctx.surface_config.height as f32,
        );

        frame.present();
        Ok(())
    }
}
