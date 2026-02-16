use std::{
	any::{Any, TypeId},
	collections::HashMap,
	sync::Arc,
};
use crate::Chunk;

/// Behavior function type
pub type BehaviorFn = fn(&mut Chunk, usize, f32);

/// Component storage for generic components
pub struct ComponentStorage {
	pub(crate) components: std::collections::HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
}

impl ComponentStorage {
	pub fn new() -> Self {
		Self { components: std::collections::HashMap::new() }
	}

	pub fn add<T: 'static + Send + Sync>(&mut self, component: T) -> usize {
		let id = TypeId::of::<T>();
		let vec = self.components.entry(id).or_insert_with(Vec::new);
		vec.push(Box::new(component));
		vec.len() - 1
	}

	pub fn get<T: 'static + Send + Sync>(&self, index: usize) -> Option<&T> {
		let id = TypeId::of::<T>();
		self.components.get(&id)?.get(index)?.downcast_ref::<T>()
	}

	pub fn get_mut<T: 'static + Send + Sync>(&mut self, index: usize) -> Option<&mut T> {
		let id = TypeId::of::<T>();
		self.components.get_mut(&id)?.get_mut(index)?.downcast_mut::<T>()
	}
}