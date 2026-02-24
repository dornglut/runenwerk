use crate::{Component, ComponentBundle, ComponentKey, EntityHandle, EntitySpawnedEvent, World};
use std::any::{Any, TypeId};
use std::collections::HashMap;

impl World {
    pub fn spawn_entity(
        &mut self,
        components: impl IntoIterator<Item = Box<dyn Any>>,
    ) -> EntityHandle {
        let components: Vec<(TypeId, Box<dyn Any>)> = components
            .into_iter()
            .map(|component| {
                let type_id = component.as_ref().type_id();
                (type_id, component)
            })
            .collect();

        let keys: Vec<ComponentKey> = components
            .iter()
            .map(|(type_id, _)| {
                self.component_registry
                    .get_key_by_type(*type_id)
                    .cloned()
                    .unwrap_or_else(|| {
                        panic!("Component with TypeId {:?} is not registered", type_id)
                    })
            })
            .collect();

        let entity = self.entity_allocator.allocate();
        let archetype = self.get_or_create_archetype(&keys);
        let key = archetype.key().clone();

        archetype.add_row_multiple(entity, components);

        let row = archetype.len() - 1;
        self.entity_locations.insert(entity, (key, row));
        for component in &keys {
            self.mark_component_added_for_entity(entity, component.type_id, component.name.clone());
        }
        self.emit_event(EntitySpawnedEvent { entity });

        entity
    }

    pub fn spawn_entity_typed<T: Component>(&mut self, component: T) -> EntityHandle {
        self.ensure_component_registered::<T>();
        self.spawn_entity([Box::new(component) as Box<dyn Any>])
    }

    pub fn spawn_bundle<B: ComponentBundle>(&mut self, bundle: B) -> EntityHandle {
        B::register_components(self);
        self.spawn_entity(bundle.into_components())
    }

    /// Adds an entity with a set of prebuilt components.
    pub fn add_entity_with_components(
        &mut self,
        components: HashMap<TypeId, Box<dyn Any>>,
    ) -> EntityHandle {
        let entity = self.entity_allocator.allocate();

        let keys: Vec<ComponentKey> = components
            .keys()
            .copied()
            .map(|type_id| {
                self.component_registry
                    .get_key_by_type(type_id)
                    .cloned()
                    .unwrap_or_else(|| {
                        panic!("Component with TypeId {:?} is not registered", type_id)
                    })
            })
            .collect();

        let archetype = self.get_or_create_archetype(&keys);
        let key = archetype.key().clone();

        archetype.add_row(entity, components);

        let row = archetype.len() - 1;
        self.entity_locations.insert(entity, (key, row));
        for component in &keys {
            self.mark_component_added_for_entity(entity, component.type_id, component.name.clone());
        }
        self.emit_event(EntitySpawnedEvent { entity });

        entity
    }
}
