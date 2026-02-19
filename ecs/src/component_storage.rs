use std::any::Any;

// src/component_storage.rs

/// Columnar storage for one ECS component type `T`.
/// Stores values in `Vec<T>` for fast index-based access and cache-friendly iteration.
/// Methods: `new()`, `push()`, `swap_remove()`, `get()`, `get_mut()`.
pub struct ComponentStorage<T: 'static> {
	pub data: Vec<T>,
}

impl<T: 'static> ComponentStorage<T> {
	pub fn new() -> Self {
		ComponentStorage { data: Vec::new() }
	}

	pub fn push(&mut self, data: T) {
		self.data.push(data);
	}

	pub fn swap_remove(&mut self, index: usize) -> Option<T> {
		if index >= self.data.len() { return None; }
		Some(self.data.swap_remove(index))
	}

	pub fn get(&self, index: usize) -> Option<&T> {
		self.data.get(index)
	}

	pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
		self.data.get_mut(index)
	}
}

pub trait AnyStorage {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
	fn swap_remove(&mut self, index: usize);
	fn len(&self) -> usize;
}

impl<T: 'static> AnyStorage for ComponentStorage<T> {
	fn as_any(&self) -> &dyn Any { self }
	fn as_any_mut(&mut self) -> &mut dyn Any { self }
	fn swap_remove(&mut self, index: usize) { self.swap_remove(index); }
	fn len(&self) -> usize { self.data.len() }
}