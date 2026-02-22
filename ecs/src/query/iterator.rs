use crate::query::tuple::ComponentTuple;
use crate::{Archetype, EntityHandle, World};
use std::any::TypeId;
use std::marker::PhantomData;
use tracing::debug;

/// Iterator over matching archetypes.
pub struct TypedQueryIterator<'a, T> {
    archetypes: Vec<&'a Archetype>,
    archetype_index: usize,
    row_index: usize,
    type_ids: &'a [TypeId],
    _marker: PhantomData<T>,
}

impl<'a, T> TypedQueryIterator<'a, T> {
    pub(crate) fn new(world: &'a World, type_ids: &'a [TypeId]) -> Self {
        let archetypes: Vec<_> = world
            .archetypes
            .values()
            .filter(|arc| type_ids.iter().all(|ty| arc.column_index(*ty).is_some()))
            .collect();

        debug!(count = archetypes.len(), "Query matched archetypes");

        Self {
            archetypes,
            archetype_index: 0,
            row_index: 0,
            type_ids,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for TypedQueryIterator<'a, T>
where
    T: ComponentTuple<'a>,
{
    type Item = (EntityHandle, <T as ComponentTuple<'a>>::Output);

    fn next(&mut self) -> Option<Self::Item> {
        while self.archetype_index < self.archetypes.len() {
            let archetype = self.archetypes[self.archetype_index];

            if self.row_index >= archetype.len() {
                self.row_index = 0;
                self.archetype_index += 1;
                continue;
            }

            let entity = *archetype.entity(self.row_index)?;
            let components = T::fetch(archetype, self.row_index, self.type_ids)?;

            self.row_index += 1;
            return Some((entity, components));
        }
        None
    }
}
