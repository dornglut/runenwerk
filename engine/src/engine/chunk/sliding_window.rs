use crate::engine::chunk::Chunk;
use crate::engine::gpu_resources::chunk_allocator::ChunkAllocator;
use glam::IVec3;
use std::collections::HashSet;
use wgpu::Device;

pub struct SlidingWindow {
	allocator: ChunkAllocator,
	active_coords: HashSet<IVec3>,
	radius: i32,
}

impl SlidingWindow {
	pub fn new(device: &Device, max_chunks: usize, radius: i32) -> Self {
		let allocator = ChunkAllocator::new(device, max_chunks);
		Self {
			allocator,
			active_coords: HashSet::new(),
			radius,
		}
	}

	pub fn update_window(&mut self, _device: &Device, player_pos: IVec3) {
		let mut needed: HashSet<IVec3> = HashSet::new();

		for x in -self.radius..=self.radius {
			for y in -self.radius..=self.radius {
				for z in -self.radius..=self.radius {
					let coord = player_pos + IVec3::new(x, y, z);
					needed.insert(coord);
				}
			}
		}

		for &coord in needed.difference(&self.active_coords) {
			self.allocator.allocate(coord, &self.active_coords);
		}

		for &coord in self.active_coords.difference(&needed) {
			self.allocator.release(coord)
		}

		self.active_coords = needed;
	}

	pub fn get_chunk(&self, coord: IVec3) -> Option<&Chunk> {
		self.allocator.get(coord)
	}

	pub fn get_chunk_mut(&mut self, coord: IVec3) -> Option<&mut Chunk> {
		self.allocator.get_mut(coord)
	}
}