use crate::render::{Renderer, WgpuCtx};
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

    pub fn render(&mut self, draw_list: &UiDrawList) -> Result<(), SurfaceError> {
        let frame = self.ctx.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        self.renderer.render(
            &self.ctx.device,
            &self.ctx.queue,
            &view,
            draw_list,
            self.ctx.surface_config.format,
            self.ctx.surface_config.width as f32,
            self.ctx.surface_config.height as f32,
        );

        frame.present();
        Ok(())
    }
}
