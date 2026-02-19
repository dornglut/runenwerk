use crate::{ArchetypeKey, ComponentStorage, Entity, World};
use std::any::TypeId;
use tracing::{debug, info};

/// Builder for creating an entity with multiple components
pub struct EntityBuilder<'a> {
	pub world: &'a mut World,
	components: Vec<(TypeId, Box<dyn FnOnce(&mut World, Entity, &ArchetypeKey) + 'a>)>,
}

impl<'a> EntityBuilder<'a> {
	/// Start building a new entity
	pub fn new(world: &'a mut World) -> Self {
		Self {
			world,
			components: Vec::new(),
		}
	}

	/// Add a component to the builder
	pub fn with<T: 'static>(mut self, component: T) -> Self {
		let type_id = TypeId::of::<T>();

		self.components.push((
			type_id,
			Box::new(move |world: &mut World, entity: Entity, _key: &ArchetypeKey| {
				// Insert component into the appropriate archetype storage
				if let Some(comp) = world.get_component_mut::<T>(entity) {
					*comp = component;
					debug!(?entity, "Updated existing component {:?}", type_id);
				} else {
					world.add_entity(entity, component);
					debug!(?entity, "Inserted new component {:?}", type_id);
				}
			}),
		));

		self
	}

	/// Finalize entity creation and insert all components
	/// Finalize entity creation and insert all components
	pub fn build(mut self) -> Entity {
		// 1️⃣ Allocate entity ID first
		let entity = self.world.allocate_entity();

		// 2️⃣ Sort type IDs to match archetype layout
		let mut sorted_types: Vec<_> = self.components.iter().map(|(ty, _)| *ty).collect();
		sorted_types.sort();
		let key = ArchetypeKey::new(sorted_types.clone());

		// 3️⃣ Ensure the archetype exists
		self.world.get_or_create_archetype(&sorted_types); // mutable borrow ends immediately

		// 4️⃣ Apply all component closures
		// Each closure borrows &mut World separately, no overlapping borrow
		for (_type_id, add_fn) in self.components {
			add_fn(self.world, entity, &key);
		}

		// 5️⃣ Now insert entity into the archetype
		let archetype = self.world.get_or_create_archetype(&sorted_types);
		archetype.add_entity(entity);
		let row_index = archetype.len() - 1;
		self.world.entity_locations.insert(entity, (key, row_index));

		debug!(?entity, ?sorted_types, "Entity created successfully");
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
			let builder = builder_fn(builder); // builder returned
			builder.build(); // build happens here
		}
	}
}
