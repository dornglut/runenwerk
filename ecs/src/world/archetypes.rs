use crate::{EntityDespawnedEvent, EntityHandle, World};

impl World {
    /// Remove an entity from the world.
    pub fn remove_entity(&mut self, entity: EntityHandle) {
        if let Some((key, row)) = self.entity_locations.remove(&entity) {
            let removed_components: Vec<_> = key
                .components
                .iter()
                .map(|component| (component.type_id, component.name.clone()))
                .collect();
            if let Some(archetype) = self.archetypes.get_mut(&key) {
                archetype.remove_row(row);

                // If swap-remove moved another entity into this row, update its location.
                if row < archetype.len() {
                    if let Some(swapped_entity) = archetype.entity(row) {
                        self.entity_locations.insert(*swapped_entity, (key, row));
                    }
                }
            }
            self.entity_allocator.free(entity);
            for (component_type, component_name) in removed_components {
                self.mark_component_removed_for_entity(entity, component_type, component_name);
            }
            self.emit_event(EntityDespawnedEvent { entity });
        }
    }
}
