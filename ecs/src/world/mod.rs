use crate::component_registry::ComponentRegistry;
use crate::{AnyStorage, Archetype, ArchetypeKey, ComponentKey, EntityAllocator, EntityHandle};
use std::any::TypeId;
use std::collections::HashMap;
use tracing::debug;

mod archetypes;
mod components;
mod spawn;

/// The ECS world, containing archetypes and entity-component mapping.
pub struct World {
    pub entity_allocator: EntityAllocator,
    pub archetypes: HashMap<ArchetypeKey, Archetype>,
    pub entity_locations: HashMap<EntityHandle, (ArchetypeKey, usize)>,
    pub component_registry: ComponentRegistry,
}

impl World {
    /// Create a new empty world.
    pub fn new() -> Self {
        Self {
            entity_allocator: EntityAllocator::new(),
            archetypes: HashMap::new(),
            entity_locations: HashMap::new(),
            component_registry: ComponentRegistry::new(),
        }
    }

    /// Register a component type `T` with the world.
    pub fn register_component<T: 'static>(&mut self) -> ComponentKey {
        self.register_component_named::<T>(std::any::type_name::<T>())
    }

    /// Register `T` only if it has not already been registered.
    pub fn ensure_component_registered<T: 'static>(&mut self) -> ComponentKey {
        let type_id = TypeId::of::<T>();
        if let Some(key) = self.component_registry.get_key_by_type(type_id) {
            return key.clone();
        }

        self.register_component::<T>()
    }

    /// Register a component type `T` with an explicit display name.
    pub fn register_component_named<T: 'static>(
        &mut self,
        name: impl Into<String>,
    ) -> ComponentKey {
        let key = self.component_registry.register::<T>(name);
        debug!("registered component '{}'", key.name);
        key
    }

    /// Allocate a new entity handle.
    pub fn allocate_entity(&mut self) -> EntityHandle {
        let entity = self.entity_allocator.allocate();
        debug!(?entity, "Allocated entity");
        entity
    }

    /// Get or create an archetype for a set of component keys.
    pub fn get_or_create_archetype(&mut self, keys: &[ComponentKey]) -> &mut Archetype {
        let key = ArchetypeKey::new(keys.to_vec());

        self.archetypes.entry(key.clone()).or_insert_with(|| {
            let mut constructors: HashMap<TypeId, fn() -> Box<dyn AnyStorage>> = HashMap::new();
            for component_key in keys {
                if let Some(constructor) = self.component_registry.get_constructor(component_key) {
                    constructors.insert(component_key.type_id, *constructor);
                } else {
                    panic!("Component {} not registered", component_key.name);
                }
            }

            Archetype::new(keys.to_vec(), &constructors)
        })
    }
}
