// Owner: ecs World Entity - Lifecycle APIs
use crate::bundle::Bundle;
use crate::entity::Entity;
use crate::errors::EntityError;
use crate::world::change_tracking::ComponentChangeKind;
use crate::world::events::{EntityDespawnedEvent, EntitySpawnedEvent};
use crate::world::world::World;

impl World {
    pub fn contains(&self, entity: Entity) -> bool {
        self.alive_entities.contains(&entity)
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        B::register(self);
        let entity = self.allocator.allocate();
        self.alive_entities.insert(entity);
        self.place_entity_in_empty_archetype(entity);
        bundle
            .insert(self, entity)
            .expect("bundle insert should succeed for new entity");
        self.emit_event(EntitySpawnedEvent { entity });
        entity
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), EntityError> {
        self.ensure_entity_exists(entity)?;
        self.remove_entity_from_spatial_indexes(entity);

        let removed_types = self
            .entity_locations
            .get(entity)
            .and_then(|location| {
                self.archetype_registry
                    .component_types(location.archetype_id)
                    .map(|types| types.to_vec())
            })
            .unwrap_or_default();

        self.remove_entity_from_archetype_tracking(entity);
        self.alive_entities.remove(&entity);
        self.allocator.free(entity);

        for type_id in removed_types {
            let component_name = self
                .component_type_registry
                .get(&type_id)
                .map(|meta| meta.name)
                .unwrap_or("unknown_component");

            self.record_component_change(
                entity,
                type_id,
                component_name,
                ComponentChangeKind::Removed,
            );
        }

        self.emit_event(EntityDespawnedEvent { entity });
        Ok(())
    }

    pub(crate) fn place_entity_in_empty_archetype(&mut self, entity: Entity) {
        self.archetype_registry
            .set_entity_components(entity, &[], &mut self.entity_locations);
    }

    pub(crate) fn remove_entity_from_archetype_tracking(&mut self, entity: Entity) {
        let _ = self
            .archetype_registry
            .remove_entity(entity, &mut self.entity_locations);
    }

    pub(crate) fn ensure_entity_exists(&self, entity: Entity) -> Result<(), EntityError> {
        if self.contains(entity) {
            Ok(())
        } else {
            Err(EntityError::NoSuchEntity { entity })
        }
    }
}
