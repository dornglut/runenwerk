use crate::{EntityHandle, World};

impl World {
    /// Remove an entity from the world.
    pub fn remove_entity(&mut self, entity: EntityHandle) {
        if let Some((key, row)) = self.entity_locations.remove(&entity) {
            if let Some(archetype) = self.archetypes.get_mut(&key) {
                archetype.remove_row(row);

                // If swap-remove moved another entity into this row, update its location.
                if row < archetype.len() {
                    if let Some(swapped_entity) = archetype.entity(row) {
                        self.entity_locations.insert(*swapped_entity, (key, row));
                    }
                }
            }
        }
    }
}
