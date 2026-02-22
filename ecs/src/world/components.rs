use crate::{EntityHandle, World};
use std::any::TypeId;

impl World {
    /// Get an immutable reference to a component for a given entity.
    pub fn get_component<T: 'static>(&self, entity: EntityHandle) -> Option<&T> {
        let (key, row) = self.entity_locations.get(&entity)?;
        let archetype = self.archetypes.get(key)?;
        archetype.column::<T>(TypeId::of::<T>())?.get(*row)
    }

    /// Get a mutable reference to a component for a given entity.
    pub fn get_component_mut<T: 'static>(&mut self, entity: EntityHandle) -> Option<&mut T> {
        let (key, row) = self.entity_locations.get(&entity)?;
        let archetype = self.archetypes.get_mut(key)?;
        archetype.column_mut::<T>(TypeId::of::<T>())?.get_mut(*row)
    }

    /// Iterate over all entities with a given component type.
    pub fn entities_with<T: 'static>(&self) -> impl Iterator<Item = EntityHandle> + '_ {
        let type_id = TypeId::of::<T>();
        self.archetypes
            .values()
            .filter(move |arc| arc.column_index(type_id).is_some())
            .flat_map(|arc| arc.iter_entities().cloned())
    }

    /// Iterate entities that contain `A` and `B`, mutating `A` while reading `B`.
    pub fn query_mut<A: 'static, B: 'static, F>(&mut self, mut f: F)
    where
        F: FnMut(EntityHandle, &mut A, &B),
    {
        let a_ty = TypeId::of::<A>();
        let b_ty = TypeId::of::<B>();

        if a_ty == b_ty {
            panic!("query_mut requires two distinct component types");
        }

        for archetype in self.archetypes.values_mut() {
            if !archetype.has_all(&[a_ty, b_ty]) {
                continue;
            }

            let entities: Vec<EntityHandle> = archetype.iter_entities().copied().collect();
            let len = archetype.len();
            let Some((a_col, b_col)) = archetype.columns2_mut_immut::<A, B>(a_ty, b_ty) else {
                continue;
            };

            for row in 0..len {
                let Some(entity) = entities.get(row).copied() else {
                    continue;
                };
                let Some(a) = a_col.get_mut(row) else {
                    continue;
                };
                let Some(b) = b_col.get(row) else {
                    continue;
                };
                f(entity, a, b);
            }
        }
    }
}
