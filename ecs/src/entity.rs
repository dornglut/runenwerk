// src/entity.rs
pub type Fk = u32; // foreign key
pub type Pk = u32; // primary key

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
	pub id: Fk,
}

pub struct EntityAllocator {
	next_id: Fk,
}

impl EntityAllocator {
	pub fn new() -> Self {
		Self { next_id: 0 }
	}

	pub fn allocate(&mut self) -> Entity {
		let e = Entity { id: self.next_id };
		self.next_id += 1;
		e
	}
}
