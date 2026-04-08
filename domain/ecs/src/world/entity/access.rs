// Owner: ecs World Entity - Access APIs
use crate::bundle::Bundle;
use crate::component::Component;
use crate::entity::Entity;
use crate::errors::EntityError;
use crate::world::entity_handles::{EntityMut, EntityRef};
use crate::world::world::World;

impl World {
    pub fn insert<B: Bundle>(&mut self, entity: Entity, bundle: B) -> Result<(), EntityError> {
        self.ensure_entity_exists(entity)?;
        B::register(self);
        bundle.insert(self, entity)
    }

    pub fn remove<B: Bundle>(&mut self, entity: Entity) -> Result<B, EntityError> {
        self.ensure_entity_exists(entity)?;
        B::remove(self, entity)
    }

    pub fn entity(&self, entity: Entity) -> Result<EntityRef<'_>, EntityError> {
        self.ensure_entity_exists(entity)?;
        Ok(EntityRef {
            world: self,
            entity,
        })
    }

    pub fn entity_mut(&mut self, entity: Entity) -> Result<EntityMut<'_>, EntityError> {
        self.ensure_entity_exists(entity)?;
        Ok(EntityMut {
            world: self,
            entity,
        })
    }

    #[doc(hidden)]
    pub fn __entity_archetype_location(&self, entity: Entity) -> Option<(usize, usize)> {
        self.entity_locations
            .get(entity)
            .map(|location| (location.archetype_id.index(), location.row))
    }

    #[doc(hidden)]
    pub fn __entity_archetype_component_count(&self, entity: Entity) -> Option<usize> {
        let location = self.entity_locations.get(entity)?;
        self.archetype_registry
            .component_count(location.archetype_id)
    }

    #[doc(hidden)]
    pub fn __entity_component_ticks<T: Component>(&self, entity: Entity) -> Option<(u64, u64)> {
        self.archetype_component_metadata::<T>(entity)
    }
}
