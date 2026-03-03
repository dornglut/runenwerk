use crate::component::Component;
use crate::entity::Entity;
use crate::errors::QueryError;
use crate::world::{TypedStore, World};
use std::any::TypeId;
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct QueryTypeAccess {
    type_id: TypeId,
    name: &'static str,
}

impl QueryTypeAccess {
    pub fn of<T: 'static>(name: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            name,
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[derive(Debug, Clone, Default)]
pub struct QueryAccess {
    component_reads: Vec<QueryTypeAccess>,
    component_writes: Vec<QueryTypeAccess>,
    resource_reads: Vec<QueryTypeAccess>,
    resource_writes: Vec<QueryTypeAccess>,
    deferred_structural_mutation: bool,
}

impl QueryAccess {
    pub fn component_reads(&self) -> &[QueryTypeAccess] {
        &self.component_reads
    }

    pub fn component_writes(&self) -> &[QueryTypeAccess] {
        &self.component_writes
    }

    pub fn resource_reads(&self) -> &[QueryTypeAccess] {
        &self.resource_reads
    }

    pub fn resource_writes(&self) -> &[QueryTypeAccess] {
        &self.resource_writes
    }

    pub fn deferred_structural_mutation(&self) -> bool {
        self.deferred_structural_mutation
    }

    fn add_component_read<T: Component>(&mut self) {
        push_unique_access(
            &mut self.component_reads,
            QueryTypeAccess::of::<T>(T::component_name()),
        );
    }

    fn add_component_write<T: Component>(&mut self) {
        push_unique_access(
            &mut self.component_writes,
            QueryTypeAccess::of::<T>(T::component_name()),
        );
    }
}

pub trait QueryFilter {
    fn configure(required: &mut Vec<TypeId>, excluded: &mut Vec<TypeId>);
}

impl QueryFilter for () {
    fn configure(_required: &mut Vec<TypeId>, _excluded: &mut Vec<TypeId>) {}
}

pub struct With<T: Component>(PhantomData<T>);
pub struct Without<T: Component>(PhantomData<T>);

impl<T: Component> QueryFilter for With<T> {
    fn configure(required: &mut Vec<TypeId>, _excluded: &mut Vec<TypeId>) {
        push_unique_type(required, TypeId::of::<T>());
    }
}

impl<T: Component> QueryFilter for Without<T> {
    fn configure(_required: &mut Vec<TypeId>, excluded: &mut Vec<TypeId>) {
        push_unique_type(excluded, TypeId::of::<T>());
    }
}

impl<A: QueryFilter, B: QueryFilter> QueryFilter for (A, B) {
    fn configure(required: &mut Vec<TypeId>, excluded: &mut Vec<TypeId>) {
        A::configure(required, excluded);
        B::configure(required, excluded);
    }
}

impl<T: Component> With<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component> Without<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub trait QueryData {
    type Item<'w>;

    fn query_types() -> Vec<TypeId>;
    fn append_access(access: &mut QueryAccess);
}

pub trait ReadOnlyQueryData: QueryData {
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>>;
}

pub trait MutableQueryData: QueryData {
    fn mark_changed(world: &mut World);

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
        if !entities.is_empty() {
            Q::mark_changed(world);
        }
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
        Q::mark_changed(world);
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
        Q::mark_changed(world);
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
            // Safety: `QueryIterMut` owns the only mutable access to the world for the duration
            // of iteration, and `Q` defines the aliasing contract for its returned item type.
            if let Some(item) = unsafe { Q::fetch_mut(self.world as *mut World, entity) } {
                return Some(item);
            }
        }
        None
    }
}

impl<T: Component> QueryData for &T {
    type Item<'w> = &'w T;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }
}

impl<T: Component> ReadOnlyQueryData for &T {
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        world.store::<T>().and_then(|store| store.get(entity))
    }
}

impl<T: Component> QueryData for &mut T {
    type Item<'w> = &'w mut T;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<T>();
    }
}

impl<T: Component> MutableQueryData for &mut T {
    fn mark_changed(world: &mut World) {
        world.mark_component_type_changed::<T>();
    }

    unsafe fn fetch_mut<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        unsafe { (&mut *world).store_mut::<T>() }.and_then(|store| store.get_mut(entity))
    }
}

impl<T: Component> QueryData for (Entity, &T) {
    type Item<'w> = (Entity, &'w T);

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }
}

impl<T: Component> ReadOnlyQueryData for (Entity, &T) {
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        world
            .store::<T>()
            .and_then(|store| store.get(entity).map(|value| (entity, value)))
    }
}

impl<A: Component, B: Component> QueryData for (&A, &B) {
    type Item<'w> = (&'w A, &'w B);

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<A>();
        access.add_component_read::<B>();
    }
}

impl<A: Component, B: Component> ReadOnlyQueryData for (&A, &B) {
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        let a = world.store::<A>().and_then(|store| store.get(entity))?;
        let b = world.store::<B>().and_then(|store| store.get(entity))?;
        Some((a, b))
    }
}

impl<A: Component, B: Component> QueryData for (&mut A, &B) {
    type Item<'w> = (&'w mut A, &'w B);

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<A>();
        access.add_component_read::<B>();
    }
}

impl<A: Component, B: Component> MutableQueryData for (&mut A, &B) {
    fn mark_changed(world: &mut World) {
        world.mark_component_type_changed::<A>();
    }

    unsafe fn fetch_mut<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "mutable/read query requires distinct component types",
        );

        let world_mut = unsafe { &mut *world };
        let b = world_mut.store::<B>().and_then(|store| store.get(entity))? as *const B;
        let a = world_mut
            .store_mut::<A>()
            .and_then(|store| store.get_mut(entity))? as *mut A;

        // Safety: the mutable query contract requires `A` and `B` to be distinct component
        // types, and `QueryIterMut` serializes access to the world for the duration of iteration.
        Some(unsafe { (&mut *a, &*b) })
    }
}

fn push_unique_type(target: &mut Vec<TypeId>, type_id: TypeId) {
    if !target.contains(&type_id) {
        target.push(type_id);
    }
}

fn push_unique_access(target: &mut Vec<QueryTypeAccess>, access: QueryTypeAccess) {
    if !target.iter().any(|entry| entry.type_id == access.type_id) {
        target.push(access);
    }
}

pub(crate) trait StoreAccess<T: Component> {
    fn get(&self, entity: Entity) -> Option<&T>;
    fn get_mut(&mut self, entity: Entity) -> Option<&mut T>;
}

impl<T: Component> StoreAccess<T> for TypedStore<T> {
    fn get(&self, entity: Entity) -> Option<&T> {
        self.values.get(&entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.values.get_mut(&entity)
    }
}
