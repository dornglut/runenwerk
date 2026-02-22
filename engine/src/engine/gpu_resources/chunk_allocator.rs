use crate::engine::chunk::Chunk;
use glam::IVec3;
use std::collections::{HashMap, HashSet, VecDeque};
use wgpu::Device;

pub struct ChunkAllocator {
	chunks: Vec<Chunk>,
	world_to_slot: HashMap<IVec3, u32>,
	free_slots: VecDeque<u32>,
}

impl ChunkAllocator {
	pub fn new(device: &Device, max_chunks: usize) -> Self {
		let mut chunks = Vec::with_capacity(max_chunks);
		let mut free_slots = VecDeque::with_capacity(max_chunks);

		for slot_index in 0..max_chunks as u32 {
			chunks.push(Chunk::new(device, IVec3::ZERO, slot_index));
			free_slots.push_back(slot_index);
		}

		Self {
			chunks,
			world_to_slot: HashMap::with_capacity(max_chunks),
			free_slots,
		}
	}

	pub fn allocate(&mut self, world_coord: IVec3, active_coords: &HashSet<IVec3>) -> &mut Chunk {
		if let Some(&slot) = self.world_to_slot.get(&world_coord) {
			return &mut self.chunks[slot as usize];
		}

		let slot = if let Some(free) = self.free_slots.pop_front() {
			free
		} else {
			// Collect potential eviction candidates
			let candidates: Vec<_> = self.world_to_slot
				.keys()
				.filter(|coord| !active_coords.contains(coord))
				.cloned()
				.collect();

			if candidates.is_empty() {
				panic!(
					"No chunks to evict! \n\
                Total allocated chunks: {} \n\
                Active coordinates (player vicinity): {:?} \n\
                All allocated chunk coordinates: {:?} \n\
                Free slots: {:?}",
					self.world_to_slot.len(),
					active_coords,
					self.world_to_slot.keys().collect::<Vec<_>>(),
					self.free_slots,
				);
			}

			// Evict the first candidate
			let evict_coord = candidates[0];
			let evict_slot = self.world_to_slot.remove(&evict_coord).unwrap();
			evict_slot
		};

		let chunk = &mut self.chunks[slot as usize];
		chunk.world_coord = world_coord;
		chunk.mark_dirty();

		self.world_to_slot.insert(world_coord, slot);
		chunk
	}

	pub fn release(&mut self, world_coord: IVec3) {
		if let Some(slot) = self.world_to_slot.remove(&world_coord) {
			self.free_slots.push_back(slot);
		}
	}

	pub fn get(&self, world_coord: IVec3) -> Option<&Chunk> {
		self.world_to_slot.get(&world_coord).map(|&slot| &self.chunks[slot as usize])
	}

	pub fn get_mut(&mut self, world_coord: IVec3) -> Option<&mut Chunk> {
		let slot = *self.world_to_slot.get(&world_coord)?;
		Some(&mut self.chunks[slot as usize])
	}

	pub fn chunks(&self) -> &[Chunk] {
		&self.chunks
	}

	pub fn chunks_mut(&mut self) -> &mut Vec<Chunk> {
		&mut self.chunks
	}


}