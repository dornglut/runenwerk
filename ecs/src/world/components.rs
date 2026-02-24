use crate::{EntityHandle, World};
use std::any::{TypeId, type_name};
use std::marker::PhantomData;

pub struct MutQueryBuilder<'a, A: 'static> {
    world: &'a mut World,
    required: Vec<TypeId>,
    excluded: Vec<TypeId>,
    _marker: PhantomData<A>,
}

impl<'a, A: 'static> MutQueryBuilder<'a, A> {
    pub(crate) fn new(world: &'a mut World) -> Self {
        Self {
            world,
            required: Vec::new(),
            excluded: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Require component `T` to exist on matched entities.
    pub fn with<T: 'static>(mut self) -> Self {
        let type_id = TypeId::of::<T>();
        if !self.required.contains(&type_id) {
            self.required.push(type_id);
        }
        self
    }

    /// Exclude entities that contain component `T`.
    pub fn without<T: 'static>(mut self) -> Self {
        let type_id = TypeId::of::<T>();
        if !self.excluded.contains(&type_id) {
            self.excluded.push(type_id);
        }
        self
    }

    /// Iterate entities with mutable component `A`.
    pub fn for_each<F>(self, mut f: F)
    where
        F: FnMut(EntityHandle, &mut A),
    {
        let a_ty = TypeId::of::<A>();
        let mut modified_entities = Vec::<EntityHandle>::new();

        for archetype in self.world.archetypes.values_mut() {
            if archetype.column_index(a_ty).is_none() {
                continue;
            }
            if !self
                .required
                .iter()
                .all(|required| archetype.column_index(*required).is_some())
            {
                continue;
            }
            if self
                .excluded
                .iter()
                .any(|excluded| archetype.column_index(*excluded).is_some())
            {
                continue;
            }

            let entities: Vec<EntityHandle> = archetype.iter_entities().copied().collect();
            let len = archetype.len();
            let Some(a_col) = archetype.column_mut::<A>(a_ty) else {
                continue;
            };

            for row in 0..len {
                let Some(entity) = entities.get(row).copied() else {
                    continue;
                };
                let Some(a) = a_col.get_mut(row) else {
                    continue;
                };
                modified_entities.push(entity);
                f(entity, a);
            }
        }

        modified_entities.sort_by_key(|entity| (entity.id, entity.generation));
        modified_entities.dedup();
        for entity in modified_entities {
            self.world
                .mark_component_modified_for_entity(entity, a_ty, type_name::<A>());
        }
    }

    /// Iterate entities with mutable `A` and readable `B`.
    pub fn for_each_with<B: 'static, F>(self, mut f: F)
    where
        F: FnMut(EntityHandle, &mut A, &B),
    {
        let a_ty = TypeId::of::<A>();
        let b_ty = TypeId::of::<B>();
        if a_ty == b_ty {
            panic!("mutable/read query requires distinct component types");
        }

        let mut modified_entities = Vec::<EntityHandle>::new();
        for archetype in self.world.archetypes.values_mut() {
            if !archetype.has_all(&[a_ty, b_ty]) {
                continue;
            }
            if !self
                .required
                .iter()
                .all(|required| archetype.column_index(*required).is_some())
            {
                continue;
            }
            if self
                .excluded
                .iter()
                .any(|excluded| archetype.column_index(*excluded).is_some())
            {
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
                modified_entities.push(entity);
                f(entity, a, b);
            }
        }

        modified_entities.sort_by_key(|entity| (entity.id, entity.generation));
        modified_entities.dedup();
        for entity in modified_entities {
            self.world
                .mark_component_modified_for_entity(entity, a_ty, type_name::<A>());
        }
    }
}

impl World {
    /// Get an immutable reference to a component for a given entity.
    pub fn get_component<T: 'static>(&self, entity: EntityHandle) -> Option<&T> {
        let (key, row) = self.entity_locations.get(&entity)?;
        let archetype = self.archetypes.get(key)?;
        archetype.column::<T>(TypeId::of::<T>())?.get(*row)
    }

    /// Get a mutable reference to a component for a given entity.
    pub fn get_component_mut<T: 'static>(&mut self, entity: EntityHandle) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let (key, row) = self.entity_locations.get(&entity)?.clone();
        let _ = self.archetypes.get(&key)?.column::<T>(type_id)?.get(row)?;
        self.mark_component_modified_for_entity(entity, type_id, type_name::<T>());
        let archetype = self.archetypes.get_mut(&key)?;
        archetype.column_mut::<T>(type_id)?.get_mut(row)
    }

    /// Iterate over all entities with a given component type.
    pub fn entities_with<T: 'static>(&self) -> impl Iterator<Item = EntityHandle> + '_ {
        let type_id = TypeId::of::<T>();
        self.archetypes
            .values()
            .filter(move |arc| arc.column_index(type_id).is_some())
            .flat_map(|arc| arc.iter_entities().cloned())
    }

    /// Start a mutable query for component `A` with optional `with/without` filters.
    pub fn query_mut_components<A: 'static>(&mut self) -> MutQueryBuilder<'_, A> {
        MutQueryBuilder::new(self)
    }

    /// Backward-compatible mutable query helper for `(A mut, B read)`.
    pub fn query_mut<A: 'static, B: 'static, F>(&mut self, f: F)
    where
        F: FnMut(EntityHandle, &mut A, &B),
    {
        self.query_mut_components::<A>().for_each_with::<B, _>(f);
    }
}
