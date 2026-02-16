// src/resource_registry.rs
use std::{
	any::{Any, TypeId},
	collections::HashMap,
	sync::Arc,
};

pub struct ResourceRegistry {
	resources: HashMap<TypeId, Vec<Arc<dyn Any + Send + Sync>>>,
}

impl ResourceRegistry {
	pub fn new() -> Self {
		Self {
			resources: HashMap::new(),
		}
	}

	/// Register a new resource of type T, return its index
	pub fn register<T: 'static + Send + Sync>(&mut self, resource: T) -> usize {
		let id = TypeId::of::<T>();
		let vec = self.resources.entry(id).or_insert_with(Vec::new);
		vec.push(Arc::new(resource));
		vec.len() - 1
	}

	/// Get a resource of type T by index
	pub fn get<T: 'static + Send + Sync>(&self, index: usize) -> Option<Arc<T>> {
		let id = TypeId::of::<T>();
		self.resources.get(&id).and_then(|vec| {
			vec.get(index)
				.map(|arc_any| arc_any.clone().downcast::<T>().ok())
				.flatten()
		})
	}
}
