// Owner: Grotto Quest ECS - Query Runtime
use super::access_and_filters::{QueryAccess, QueryFilter, push_unique_type};
use crate::component::Component;
use crate::entity::Entity;
use crate::errors::QueryError;
use crate::world::World;
use std::any::TypeId;
use std::marker::PhantomData;

pub trait QueryData {
    type Item<'w>;

    fn query_types() -> Vec<TypeId>;
    fn append_access(access: &mut QueryAccess);
}

pub trait ReadOnlyQueryData: QueryData {
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>>;
}

pub trait MutableQueryData: QueryData {
    fn mark_changed(world: &mut World, entity: Entity);

    /// Safety: the caller must uphold the access guarantees described by `Self::append_access`.
    unsafe fn fetch_mut<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>>;
}

pub struct QueryState<Q, F = ()> {
    required: Vec<TypeId>,
    excluded: Vec<TypeId>,
    access: QueryAccess,
    _marker: PhantomData<(Q, F)>,
}

impl<Q: QueryData, F: QueryFilter> QueryState<Q, F> {
    pub fn new(_world: &mut World) -> Self {
        Self::detached()
    }

    pub fn access(&self) -> &QueryAccess {
        &self.access
    }

    pub fn with<T: Component>(mut self) -> Self {
        push_unique_type(&mut self.required, TypeId::of::<T>());
        self
    }

    pub fn without<T: Component>(mut self) -> Self {
        push_unique_type(&mut self.excluded, TypeId::of::<T>());
        self
    }

    pub fn iter_on<'w>(&self, world: &'w World) -> QueryIter<'w, Q>
    where
        Q: ReadOnlyQueryData,
    {
        QueryIter {
            world,
            entities: self.matching_entities(world),
            index: 0,
            _marker: PhantomData,
        }
    }

    pub fn get_on<'w>(&self, world: &'w World, entity: Entity) -> Option<Q::Item<'w>>
    where
        Q: ReadOnlyQueryData,
    {
        if !self.matches_entity(world, entity) {
            return None;
        }
        Q::fetch(world, entity)
    }

    pub fn single_on<'w>(&self, world: &'w World) -> Result<Q::Item<'w>, QueryError>
    where
        Q: ReadOnlyQueryData,
    {
        let entities = self.matching_entities(world);
        if entities.is_empty() {
            return Err(QueryError::NoResults);
        }
        if entities.len() > 1 {
            return Err(QueryError::MultipleResults {
                count: entities.len(),
            });
        }
        Q::fetch(world, entities[0]).ok_or(QueryError::NoResults)
    }

    pub fn iter_mut_on<'w>(&mut self, world: &'w mut World) -> QueryIterMut<'w, Q>
    where
        Q: MutableQueryData,
    {
        let entities = self.matching_entities(world);
        QueryIterMut {
            world,
            entities,
            index: 0,
            _marker: PhantomData,
        }
    }

    pub fn get_mut_on<'w>(&mut self, world: &'w mut World, entity: Entity) -> Option<Q::Item<'w>>
    where
        Q: MutableQueryData,
    {
        if !self.matches_entity(world, entity) {
            return None;
        }
        Q::mark_changed(world, entity);
        // Safety: access conflicts are guarded by the query type and runtime.
        unsafe { Q::fetch_mut(world as *mut World, entity) }
    }

    pub fn single_mut_on<'w>(&mut self, world: &'w mut World) -> Result<Q::Item<'w>, QueryError>
    where
        Q: MutableQueryData,
    {
        let entities = self.matching_entities(world);
        if entities.is_empty() {
            return Err(QueryError::NoResults);
        }
        if entities.len() > 1 {
            return Err(QueryError::MultipleResults {
                count: entities.len(),
            });
        }
        Q::mark_changed(world, entities[0]);
        // Safety: exactly one matching entity exists and mutable access is serialized.
        unsafe { Q::fetch_mut(world as *mut World, entities[0]) }.ok_or(QueryError::NoResults)
    }

    pub(crate) fn detached() -> Self {
        let mut required = Vec::new();
        let mut excluded = Vec::new();
        F::configure(&mut required, &mut excluded);

        let mut access = QueryAccess::default();
        Q::append_access(&mut access);

        Self {
            required,
            excluded,
            access,
            _marker: PhantomData,
        }
    }

    fn matching_entities(&self, world: &World) -> Vec<Entity> {
        world.matching_entities(&Q::query_types(), &self.required, &self.excluded)
    }

    fn matches_entity(&self, world: &World, entity: Entity) -> bool {
        world.contains(entity)
            && self
                .matching_entities(world)
                .iter()
                .any(|candidate| *candidate == entity)
    }
}

pub struct QueryBorrow<'w, Q, F = ()> {
    world: &'w World,
    state: QueryState<Q, F>,
}

impl<'w, Q: QueryData, F: QueryFilter> QueryBorrow<'w, Q, F> {
    pub(crate) fn new(world: &'w World) -> Self {
        Self {
            world,
            state: QueryState::detached(),
        }
    }

    pub fn access(&self) -> &QueryAccess {
        self.state.access()
    }

    pub fn with<T: Component>(mut self) -> Self {
        self.state = self.state.with::<T>();
        self
    }

    pub fn without<T: Component>(mut self) -> Self {
        self.state = self.state.without::<T>();
        self
    }

    pub fn iter(&'w self) -> QueryIter<'w, Q>
    where
        Q: ReadOnlyQueryData,
    {
        self.state.iter_on(self.world)
    }

    pub fn get(&'w self, entity: Entity) -> Option<Q::Item<'w>>
    where
        Q: ReadOnlyQueryData,
    {
        self.state.get_on(self.world, entity)
    }

    pub fn single(&'w self) -> Result<Q::Item<'w>, QueryError>
    where
        Q: ReadOnlyQueryData,
    {
        self.state.single_on(self.world)
    }
}

pub struct QueryBorrowMut<'w, Q, F = ()> {
    world: &'w mut World,
    state: QueryState<Q, F>,
}

impl<'w, Q: QueryData, F: QueryFilter> QueryBorrowMut<'w, Q, F> {
    pub(crate) fn new(world: &'w mut World) -> Self {
        Self {
            world,
            state: QueryState::detached(),
        }
    }

    pub fn access(&self) -> &QueryAccess {
        self.state.access()
    }

    pub fn with<T: Component>(mut self) -> Self {
        self.state = self.state.with::<T>();
        self
    }

    pub fn without<T: Component>(mut self) -> Self {
        self.state = self.state.without::<T>();
        self
    }

    pub fn iter_mut(&mut self) -> QueryIterMut<'_, Q>
    where
        Q: MutableQueryData,
    {
        self.state.iter_mut_on(self.world)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<Q::Item<'_>>
    where
        Q: MutableQueryData,
    {
        self.state.get_mut_on(self.world, entity)
    }

    pub fn single_mut(&mut self) -> Result<Q::Item<'_>, QueryError>
    where
        Q: MutableQueryData,
    {
        self.state.single_mut_on(self.world)
    }
}

pub struct QueryIter<'w, Q: ReadOnlyQueryData> {
    world: &'w World,
    entities: Vec<Entity>,
    index: usize,
    _marker: PhantomData<Q>,
}

impl<'w, Q: ReadOnlyQueryData> Iterator for QueryIter<'w, Q> {
    type Item = Q::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.get(self.index).copied() {
            self.index += 1;
            if let Some(item) = Q::fetch(self.world, entity) {
                return Some(item);
            }
        }
        None
    }
}

pub struct QueryIterMut<'w, Q: MutableQueryData> {
    world: &'w mut World,
    entities: Vec<Entity>,
    index: usize,
    _marker: PhantomData<Q>,
}

impl<'w, Q: MutableQueryData> Iterator for QueryIterMut<'w, Q> {
    type Item = Q::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.get(self.index).copied() {
            self.index += 1;
            Q::mark_changed(self.world, entity);
            // Safety: `QueryIterMut` owns the only mutable access to the world for the duration
            // of iteration, and `Q` defines the aliasing contract for its returned item type.
            if let Some(item) = unsafe { Q::fetch_mut(self.world as *mut World, entity) } {
                return Some(item);
            }
        }
        None
    }
}
