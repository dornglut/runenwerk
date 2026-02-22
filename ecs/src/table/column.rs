use std::any::Any;

// src/table/column

/// Columnar storage for one ECS component type `T`.
/// Stores values in `Vec<T>` for fast index-based access and cache-friendly iteration.
pub struct Column<T: 'static> {
	data: Vec<T>,
}

impl<T: 'static> Column<T> {
	pub fn new() -> Self {
		Column { data: Vec::new() }
	}

	pub fn push(&mut self, value: T) {
		self.data.push(value);
	}

	pub fn swap_remove(&mut self, index: usize) -> Option<T> {
		if index < self.data.len() {
			Some(self.data.swap_remove(index))
		} else {
			None
		}
	}

	pub fn get(&self, index: usize) -> Option<&T> {
		self.data.get(index)
	}

	pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
		self.data.get_mut(index)
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> std::slice::Iter<'_, T> {
		self.data.iter()
	}

	pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
		self.data.iter_mut()
	}
}

pub trait AnyStorage {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;

	/// Push a boxed value, returns Err if type mismatch
	fn push_box(&mut self, value: Box<dyn Any>) -> Result<(), ()>;

	/// Swap-remove a value at index, returns None if out-of-bounds
	fn swap_remove(&mut self, index: usize) -> Option<Box<dyn Any>>;

	fn len(&self) -> usize;
	fn default_box(&self) -> Option<Box<dyn Any>> { None }
}

impl<T: 'static> AnyStorage for Column<T> {
	fn as_any(&self) -> &dyn Any { self }
	fn as_any_mut(&mut self) -> &mut dyn Any { self }

	fn push_box(&mut self, value: Box<dyn Any>) -> Result<(), ()> {
		match value.downcast::<T>() {
			Ok(v) => { self.data.push(*v); Ok(()) }
			Err(_) => Err(()),
		}
	}

	fn swap_remove(&mut self, index: usize) -> Option<Box<dyn Any>> {
		if index < self.data.len() {
			Some(Box::new(self.data.swap_remove(index)) as Box<dyn Any>)
		} else {
			None
		}
	}

	fn len(&self) -> usize { self.data.len() }
}
