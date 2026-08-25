// Owner: ecs World Entity - Lifecycle APIs
use crate::bundle::Bundle;
use crate::entity::Entity;
use crate::errors::{EntityAllocationError, EntityError};
use crate::world::World;
use crate::world::change_tracking::ComponentChangeKind;
use crate::world::messaging::{EntityDespawnedEvent, EntitySpawnedEvent};

impl World {
    pub fn contains(&self, entity: Entity) -> bool {
        self.allocator.contains(entity) && self.alive_entities.contains(&entity)
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Result<Entity, EntityAllocationError> {
        let entity = self.allocator.allocate()?;
        B::register(self);
        self.alive_entities.insert(entity);
        self.place_entity_in_empty_archetype(entity);
        bundle
            .insert(self, entity)
            .expect("bundle insert should succeed for new entity");
        self.publish_broadcast(EntitySpawnedEvent { entity });
        Ok(entity)
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
        self.allocator.free(entity)?;

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

        self.publish_broadcast(EntityDespawnedEvent { entity });
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
        self.allocator.validate(entity)?;
        if self.alive_entities.contains(&entity) {
            Ok(())
        } else {
            Err(EntityError::UnknownEntity { entity })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(crate::Component)]
    struct Marker;

    #[test]
    fn fresh_worlds_never_alias_equal_local_entity_positions() {
        let mut first_world = World::new();
        let mut second_world = World::new();
        let first = first_world
            .spawn(Marker)
            .expect("first spawn should succeed");
        let second = second_world
            .spawn(Marker)
            .expect("second spawn should succeed");

        assert_eq!(first.index(), second.index());
        assert_eq!(first.generation(), second.generation());
        assert_ne!(first, second);
        assert!(!first_world.contains(second));
        assert!(matches!(
            first_world.entity(second),
            Err(EntityError::ForeignWorld { .. })
        ));
        assert!(matches!(
            first_world.require::<Marker>(second),
            Err(EntityError::ForeignWorld { .. })
        ));
        assert!(first_world.contains(first));
    }

    #[test]
    fn stale_and_double_free_entities_are_rejected_before_world_mutation() {
        let mut world = World::new();
        let first = world.spawn(Marker).expect("spawn should succeed");
        world.despawn(first).expect("despawn should succeed");
        assert!(matches!(
            world.despawn(first),
            Err(EntityError::AlreadyFreed { .. })
        ));

        let second = world.spawn(Marker).expect("reuse should succeed");
        assert_eq!(first.index(), second.index());
        assert_ne!(first.generation(), second.generation());
        assert!(matches!(
            world.entity(first),
            Err(EntityError::StaleGeneration { .. })
        ));
        assert!(world.contains(second));
    }
}
