use std::sync::Arc;
use crate::{BehaviorFn, Chunk, ComponentStorage};

pub struct EntityBuilder<'a> {
	chunk: &'a mut Chunk,
	entity_idx: usize,
	components: Vec<Box<dyn std::any::Any + Send + Sync>>,
	pipeline: Option<Arc<Vec<BehaviorFn>>>,
}

impl<'a> EntityBuilder<'a> {
	pub fn new(chunk: &'a mut Chunk) -> Self {
		let entity_idx = chunk.new_entity();
		Self { chunk, entity_idx, components: Vec::new(), pipeline: None }
	}

	pub fn with<T: 'static + Send + Sync>(mut self, comp: T) -> Self {
		self.components.push(Box::new(comp));
		self
	}

	pub fn pipeline(mut self, pipeline: Arc<Vec<BehaviorFn>>) -> Self {
		self.pipeline = Some(pipeline);
		self
	}

	pub fn build(self) -> usize {
		for comp in self.components {
			self.chunk.component_storage.add_boxed(comp);
		}
		if let Some(pipe) = self.pipeline {
			self.chunk.set_pipeline(self.entity_idx, pipe);
		}
		self.entity_idx
	}
}

impl ComponentStorage {
	fn add_boxed(&mut self, comp: Box<dyn std::any::Any + Send + Sync>) {
		let id = (*comp).type_id();
		let vec = self.components.entry(id).or_default();
		vec.push(comp);
	}
}
