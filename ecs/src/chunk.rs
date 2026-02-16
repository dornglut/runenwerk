use std::collections::HashMap;
use std::sync::Arc;
use crate::{BehaviorFn, ComponentStorage};

pub struct Chunk {
	pub entity_count: usize,
	pub component_storage: ComponentStorage,

	/// Each entity can have its own pipeline of behaviors.
	/// Use `Option` so missing pipelines are skipped efficiently.
	pub pipelines: Vec<Option<Arc<Vec<BehaviorFn>>>>,
}

impl Chunk {
	pub fn new() -> Self {
		Self {
			entity_count: 0,
			component_storage: ComponentStorage::new(),
			pipelines: Vec::new(),
		}
	}

	/// Add a new entity and return its index
	pub fn new_entity(&mut self) -> usize {
		let idx = self.entity_count;
		self.entity_count += 1;

		// Ensure pipelines Vec is large enough
		if self.pipelines.len() < self.entity_count {
			self.pipelines.resize_with(self.entity_count, || None);
		}

		idx
	}

	/// Attach a component to an entity
	pub fn add_component<T: 'static + Send + Sync>(&mut self, _entity_idx: usize, comp: T) {
		self.component_storage.add(comp);
	}

	/// Assign a pipeline to an entity
	pub fn set_pipeline(&mut self, entity_idx: usize, pipeline: Arc<Vec<BehaviorFn>>) {
		if entity_idx >= self.pipelines.len() {
			self.pipelines.resize_with(entity_idx + 1, || None);
		}
		self.pipelines[entity_idx] = Some(pipeline);
	}

	/// Update all entities
	pub fn update(&mut self, dt: f32) {
		for i in 0..self.entity_count {
			if let Some(pipeline) = &self.pipelines[i] {
				for behavior in pipeline.iter() {
					behavior(self, i, dt); // safe mutable borrow
				}
			}
		}
	}

	/// Query entities that have a component of type T
	pub fn entities_with<T: 'static + Send + Sync>(&self) -> Vec<usize> {
		let id = std::any::TypeId::of::<T>();
		self.component_storage.components.get(&id)
			.map(|vec| (0..vec.len()).collect())
			.unwrap_or_default()
	}
}
