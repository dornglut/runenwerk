use std::sync::Arc;
use wgpu::SurfaceError;
use winit::window::Window;
use anyhow::Result;
use crate::engine::entities::Camera;
use crate::engine::gfx::*;

#[derive(Debug)]
pub struct Gfx {
	pub ctx: WgpuCtx<'static>,
	pub renderer: DefaultRenderer,
	pub headless: bool,
}

impl Gfx {
	pub fn new(window: Arc<Window>) -> Result<Self> {
		let ctx = WgpuCtx::new(window)?;
		let renderer = DefaultRenderer::new_empty();
		Ok(Self { ctx, renderer, headless: false })
	}

	pub fn new_headless() -> Result<Self> {
		let ctx = WgpuCtx::new_headless()?;
		let renderer = DefaultRenderer::new_empty();
		Ok(Self { ctx, renderer, headless: true })
	}

	pub fn init_renderer(&mut self, camera_gpu: &CameraGPU) {
		self.renderer.init_pipeline(&self.ctx.device, &camera_gpu.layout, self.ctx.surface_config.format);
	}

	pub fn render(&mut self, camera_gpu: &CameraGPU) -> Result<(), SurfaceError> {
		if self.headless {
			return Ok(());
		}

		let frame = self.ctx.get_surface_texture()?;
		let view = frame.texture.create_view(&Default::default());

		self.renderer.render(&view, &self.ctx.device, &self.ctx.queue, camera_gpu)?;
		frame.present();

		Ok(())
	}

	pub fn sync_camera(&self, camera_gpu: &CameraGPU, camera: &Camera) {
		camera_gpu.update(&self.ctx.queue, camera)
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		self.ctx.resize(width, height);
	}
}