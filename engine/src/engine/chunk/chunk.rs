use glam::IVec3;
use wgpu::*;
use crate::engine::gpu_resources::{SdfTexture, VoxelBuffer};

pub const CHUNK_SIZE: usize = 32;
pub const HALO: usize = 2;
pub const STRIDE: usize = CHUNK_SIZE + 2 * HALO;

pub struct Chunk {
	pub world_coord: IVec3,   // integer chunk-space coordinate
	pub slot_index: u32,      // stable index into bind arrays

	pub voxels: VoxelBuffer,
	pub sdf: SdfTexture,
}

impl Chunk {
	pub fn new(device: &Device, world_coord: IVec3, slot_index: u32) -> Self {
		let voxels = VoxelBuffer::new(device, STRIDE, slot_index);
		let sdf = SdfTexture::new(device, STRIDE, slot_index);

		Self {
			world_coord,
			slot_index,
			voxels,
			sdf,
		}
	}
	pub fn mark_dirty(&mut self) {
		self.voxels.mark_dirty();
		self.sdf.mark_dirty();
	}

	pub fn mark_clean(&mut self) {
		self.voxels.mark_clean();
		self.sdf.mark_clean();
	}

	pub fn update_voxels(&mut self, queue: &Queue, data: &[u32]) {
		assert_eq!(data.len(), self.voxels.stride.pow(3));
		queue.write_buffer(&self.voxels.buffer, 0, bytemuck::cast_slice(data));
		self.voxels.mark_clean();
	}

	pub fn update_sdf(&mut self, queue: &Queue, data: &[f32]) {
		self.sdf.update_if_dirty(queue, data);
	}

	pub fn voxel_bind_group(&self) -> &BindGroup {
		self.voxels.bind_group()
	}
	pub fn sdf_bind_group(&self) -> &BindGroup {
		self.sdf.bind_group()
	}

	pub fn update_all(&mut self, queue: &Queue, voxel_data: &[u32], sdf_data: &[f32]) {
		self.update_voxels(queue, voxel_data);
		self.update_sdf(queue, sdf_data);
	}
}
