use crate::{Archetype, ComponentStorage, Entity, World};
use std::any::TypeId;
use std::marker::PhantomData;
use tracing::{error, debug};

/// Query builder for components `(C1, C2, ...)`
pub struct QueryBuilder<'a, T = ()> {
	world: &'a World,
	type_ids: Vec<TypeId>,
	_marker: PhantomData<T>,
}

impl<'a, T> QueryBuilder<'a, T> {
	/// Add a component to the query tuple
	pub fn with<C: 'static>(mut self) -> QueryBuilder<'a, (C, T)> {
		let type_id = TypeId::of::<C>();
		self.type_ids.push(type_id);
		QueryBuilder {
			world: self.world,
			type_ids: self.type_ids,
			_marker: PhantomData,
		}
	}

	/// Execute an immutable query
	pub fn iter(&'a self) -> TypedQueryIterator<'a, T> {
		TypedQueryIterator::new(self.world, &self.type_ids)
	}
}

/// Iterator over matching archetypes
pub struct TypedQueryIterator<'a, T> {
	archetypes: Vec<&'a Archetype>,
	archetype_index: usize,
	row_index: usize,
	type_ids: &'a [TypeId],
	_marker: PhantomData<T>,
}

impl<'a, T> TypedQueryIterator<'a, T> {
	fn new(world: &'a World, type_ids: &'a [TypeId]) -> Self {
		let archetypes: Vec<_> = world.archetypes.values()
			.filter(|arc| type_ids.iter().all(|ty| arc.component_indices.contains_key(ty)))
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

/// Trait to fetch components from a row in an archetype
pub trait ComponentTuple<'a> {
	type Output;
	fn fetch(archetype: &'a Archetype, row: usize, type_ids: &[TypeId]) -> Option<Self::Output>;
}

/// Macro to implement tuples for immutable queries
macro_rules! impl_tuple {
    ($($name:ident),*) => {
        impl<'a, $($name: 'static),*> ComponentTuple<'a> for ($($name,)*) {
            type Output = ($(&'a $name,)*);

            fn fetch(archetype: &'a Archetype, row: usize, type_ids: &[TypeId]) -> Option<Self::Output> {
                let mut idx = 0;
                Some((
                    $({
                        let storage = archetype.storage(type_ids[idx])?;
                        let typed_storage = storage.as_any()
                            .downcast_ref::<ComponentStorage<$name>>()?;
                        idx += 1;
                        typed_storage.get(row)?
                    },)*
                ))
            }
        }
    };
}

// Implement tuples up to 4 components (expandable)
impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);

impl<'a, T> Iterator for TypedQueryIterator<'a, T>
where
	T: ComponentTuple<'a>,
{
	type Item = (Entity, <T as ComponentTuple<'a>>::Output);

	fn next(&mut self) -> Option<Self::Item> {
		while self.archetype_index < self.archetypes.len() {
			let archetype = self.archetypes[self.archetype_index];

			if self.row_index >= archetype.len() {
				self.row_index = 0;
				self.archetype_index += 1;
				continue;
			}

			let entity = archetype.entities[self.row_index];
			let components = T::fetch(archetype, self.row_index, self.type_ids)?;

			self.row_index += 1;
			return Some((entity, components));
		}
		None
	}
}

/// Extension trait for World to start queries
pub trait WorldQueryExt {
	fn query(&self) -> QueryBuilder<'_>;
}

impl WorldQueryExt for World {
	fn query(&self) -> QueryBuilder<'_> {
		QueryBuilder {
			world: self,
			type_ids: Vec::new(),
			_marker: PhantomData,
		}
	}
}
