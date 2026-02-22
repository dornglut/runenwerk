#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub struct EntityHandle {
	pub id: u32,
	pub generation: u32,
}

pub struct EntityAllocator {
	next_id: u32,
	free_list: Vec<u32>,
	generations: Vec<u32>,
}

impl EntityAllocator {
	pub fn new() -> Self {
		Self {
			next_id: 0,
			free_list: Vec::new(),
			generations: Vec::new(),
		}
	}

	pub fn allocate(&mut self) -> EntityHandle {
		if let Some(id) = self.free_list.pop() {
			let generation = self.generations[id as usize];
			EntityHandle { id, generation }
		} else {
			let id = self.next_id;
			self.next_id += 1;
			self.generations.push(0);
			EntityHandle { id, generation: 0 }
		}
	}

	pub fn free(&mut self, entity: EntityHandle) {
		self.generations[entity.id as usize] += 1;
		self.free_list.push(entity.id);
	}

	pub fn generation(&self, entity: EntityHandle) -> u32 {
		self.generations[entity.id as usize]
	}
}