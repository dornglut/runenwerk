use crate::World;
use std::any::TypeId;
use std::marker::PhantomData;

mod iterator;
mod tuple;

pub use iterator::TypedQueryIterator;
pub use tuple::ComponentTuple;

/// Query builder for components `(C1, C2, ...)`.
pub struct QueryBuilder<'a, T = ()> {
    world: &'a World,
    type_ids: Vec<TypeId>,
    _marker: PhantomData<T>,
}

impl<'a, T> QueryBuilder<'a, T> {
    /// Add a component to the query tuple.
    pub fn with<C: 'static>(mut self) -> QueryBuilder<'a, (C, T)> {
        self.type_ids.push(TypeId::of::<C>());
        QueryBuilder {
            world: self.world,
            type_ids: self.type_ids,
            _marker: PhantomData,
        }
    }

    /// Execute an immutable query.
    pub fn iter(&'a self) -> TypedQueryIterator<'a, T> {
        TypedQueryIterator::new(self.world, &self.type_ids)
    }
}

/// Extension trait for `World` to start queries.
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
