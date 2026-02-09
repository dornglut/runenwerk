// archetype.rs
use std::any::TypeId;
use std::collections::HashMap;
use crate::component::{Component, ComponentVec};
use crate::world::Entity;

/// Archetype stores entities + their component vectors
pub struct Archetype {
	pub entities: Vec<Entity>,
	pub component_vectors: HashMap<TypeId, Box<dyn ComponentVec>>,
}

impl Archetype {
	pub fn new() -> Self {
		Self {
			entities: Vec::new(),
			component_vectors: HashMap::new(),
		}
	}

	pub fn has_component<T: Component>(&self) -> bool {
		self.component_vectors.contains_key(&TypeId::of::<T>())
	}

	pub fn add_component_vec<T: Component + 'static>(&mut self, vec: Vec<T>) {
		self.component_vectors.insert(TypeId::of::<T>(), Box::new(vec));
	}

	pub fn get_component_vec<T: Component + 'static>(&self) -> Option<&Vec<T>> {
		self.component_vectors.get(&TypeId::of::<T>())
			.and_then(|b| b.as_any().downcast_ref::<Vec<T>>())
	}

	pub fn get_component_vec_mut<T: Component + 'static>(&mut self) -> Option<&mut Vec<T>> {
		self.component_vectors.get_mut(&TypeId::of::<T>())
			.and_then(|b| b.as_any_mut().downcast_mut::<Vec<T>>())
	}
}
