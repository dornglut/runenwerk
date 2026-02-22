use crate::{EntityHandle, World};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use tracing::debug;

/// Builder for creating an entity with multiple components
pub struct EntityBuilder<'a> {
	world: &'a mut World,
	components: HashMap<TypeId, Box<dyn Any>>,
}

impl<'a> EntityBuilder<'a> {
	/// Start building a new entity
	pub fn new(world: &'a mut World) -> Self {
		Self {
			world,
			components: HashMap::new(),
		}
	}

	/// Add a component to the builder
	pub fn with<T: 'static>(mut self, component: T) -> Self {
		self.world.ensure_component_registered::<T>();
		self.components.insert(TypeId::of::<T>(), Box::new(component));
		self
	}

	/// Finalize entity creation and insert all components atomically
	pub fn build(self) -> EntityHandle {
		let entity = self.world.add_entity_with_components(self.components);
		debug!(?entity, "Entity built successfully");
		entity
	}
}

/// Extension trait to add `.entity()` to World
pub trait WorldBuilderExt {
	fn entity(&'_ mut self) -> EntityBuilder<'_>;
}

impl WorldBuilderExt for World {
	fn entity(&'_ mut self) -> EntityBuilder<'_> {
		EntityBuilder::new(self)
	}
}

impl World {
	/// Spawn multiple entities using a builder closure for each
	pub fn spawn_many<F>(&mut self, count: usize, mut builder_fn: F)
	where
		F: FnMut(EntityBuilder) -> EntityBuilder,
	{
		for _ in 0..count {
			let builder = self.entity();
			let builder = builder_fn(builder); // let user add components
			builder.build(); // atomic insertion via add_entity_with_components
		}
	}
}
