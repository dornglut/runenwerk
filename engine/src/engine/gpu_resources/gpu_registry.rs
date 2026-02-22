use std::sync::Arc;
use glam::IVec3;
use wgpu::*;
use crate::engine::chunk::SlidingWindow;
use crate::engine::entities::Camera;
use crate::engine::gfx::CameraGPU;

pub struct GpuRegistry {
	pub device: Arc<Device>,
	pub camera: CameraGPU,
	pub chunk_window: SlidingWindow,
}

impl GpuRegistry {
	pub fn new(device: Arc<Device>) -> Self {
		let chunk_window = SlidingWindow::new(&device, 100, 1);
		Self {
			device: device.clone(),
			camera: CameraGPU::new(&device),
			chunk_window,
		}
	}

	pub fn update_camera(&self, queue: &Queue, cpu_camera: &Camera, ) {
		self.camera.update(queue, cpu_camera);
	}

	pub fn update_chunks(&mut self, player_pos: IVec3) {
		self.chunk_window.update_window(&self.device, player_pos);
	}
}