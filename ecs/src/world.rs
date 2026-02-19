use std::any::{TypeId};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::{AnyStorage, Archetype, ArchetypeKey, ComponentStorage, Entity, EntityAllocator};

/// The ECS World holds entities, archetypes, and component type info
pub struct World {
	pub entity_allocator: EntityAllocator,
	pub archetypes: HashMap<ArchetypeKey, Archetype>,
	pub entity_locations: HashMap<Entity, (ArchetypeKey, usize)>,
	pub component_constructors: HashMap<TypeId, fn() -> Box<dyn AnyStorage>>,
}

impl World {
	pub fn new() -> Self {
		Self {
			entity_allocator: EntityAllocator::new(),
			archetypes: HashMap::new(),
			entity_locations: HashMap::new(),
			component_constructors: HashMap::new(),
		}
	}

	/// Register a component type for the world
	pub fn register_component<T: 'static>(&mut self) {
		let type_id = TypeId::of::<T>();
		self.component_constructors
			.entry(type_id)
			.or_insert(|| Box::new(ComponentStorage::<T>::new()));
		debug!("Registered component {:?}", type_id);
	}

	/// Allocate a new entity ID
	pub fn allocate_entity(&mut self) -> Entity {
		let entity = self.entity_allocator.allocate();
		debug!(?entity, "Allocated new entity");
		entity
	}

	/// Get or create an archetype for a set of component types
	pub fn get_or_create_archetype(&mut self, types: &[TypeId]) -> &mut Archetype {
		let key = ArchetypeKey::new(types.to_vec());
		let constructors = &mut self.component_constructors;

		self.archetypes.entry(key.clone())
			.or_insert_with(|| {
				debug!(?key, "Creating new archetype");
				Archetype::new(types.to_vec(), constructors)
			})
	}

	/// Get a component reference for an entity
	pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
		let (key, row) = self.entity_locations.get(&entity)?;
		let archetype = self.archetypes.get(key)?;
		let storage = archetype.storage(TypeId::of::<T>())?;
		let typed_storage = storage.as_any().downcast_ref::<ComponentStorage<T>>()?;
		typed_storage.get(*row)
	}

	/// Get a mutable component reference for an entity
	pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
		let (key, row) = self.entity_locations.get(&entity)?;
		let archetype = self.archetypes.get_mut(key)?;
		let storage = archetype.storage_mut(TypeId::of::<T>())?;
		let typed_storage = storage.as_any_mut().downcast_mut::<ComponentStorage<T>>()?;
		typed_storage.get_mut(*row)
	}

	/// Check if an entity has a specific component
	pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
		if let Some((key, _row)) = self.entity_locations.get(&entity) {
			if let Some(archetype) = self.archetypes.get(key) {
				return archetype.component_indices.contains_key(&TypeId::of::<T>());
			}
		}
		false
	}

	/// Add a new entity with a single component
	pub fn add_entity<T: 'static>(&mut self, entity: Entity, component: T) {
		let type_id = TypeId::of::<T>();
		let key = ArchetypeKey::new(vec![type_id]);

		let row_index = {
			let archetype = self.get_or_create_archetype(&[type_id]);
			archetype.add_entity(entity);

			let storage = archetype.storage_mut(type_id).unwrap();
			let typed_storage = storage.as_any_mut()
				.downcast_mut::<ComponentStorage<T>>()
				.unwrap();
			typed_storage.push(component);

			archetype.len() - 1
		};

		self.entity_locations.insert(entity, (key, row_index));
		debug!(?entity, "Added entity with single component {:?}", type_id);
	}

	/// Remove a component from an entity
	pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Option<T> {
		let type_id = TypeId::of::<T>();
		let (key, row) = self.entity_locations.get(&entity)?.clone();

		let archetype = self.archetypes.get_mut(&key)?;
		let storage = archetype.storage_mut(type_id)?;
		let typed_storage = storage.as_any_mut().downcast_mut::<ComponentStorage<T>>()?;

		let removed = typed_storage.data.remove(row);
		debug!(?entity, "Removed component {:?}", type_id);

		// Move entity to a new archetype without the removed component
		let mut new_types: Vec<_> = archetype.component_types.iter().cloned().filter(|&t| t != type_id).collect();
		new_types.sort();
		self.move_entity(entity, &new_types);

		Some(removed)
	}

	/// Move an entity to a new archetype
	pub fn move_entity(&mut self, entity: Entity, new_types: &[TypeId]) {
		let new_key = ArchetypeKey::new(new_types.to_vec());

		let row_index = {
			if let Some((old_key, old_index)) = self.entity_locations.remove(&entity) {
				let old_archetype = self.archetypes.get_mut(&old_key).unwrap();
				old_archetype.remove_entity(old_index);
			}

			let new_archetype = self.get_or_create_archetype(new_types);
			new_archetype.add_entity(entity);
			new_archetype.len() - 1
		};

		self.entity_locations.insert(entity, (new_key, row_index));
		debug!(?entity, ?new_types, "Moved entity to new archetype");
	}

	/// Iterate all entities with a single component
	pub fn entities_with<T: 'static>(&self) -> impl Iterator<Item = Entity> + '_ {
		let type_id = TypeId::of::<T>();
		self.archetypes.values()
			.filter(move |arc| arc.component_indices.contains_key(&type_id))
			.flat_map(|arc| arc.entities.iter().cloned())
	}
}
