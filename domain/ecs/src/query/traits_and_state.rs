// Owner: Grotto Quest ecs - Query Runtime
use super::access_and_filters::{QueryAccess, QueryFilter, push_unique_type};
use crate::component::Component;
use crate::entity::Entity;
use crate::errors::QueryError;
use crate::storage::ArchetypeExecutionBinding;
use crate::telemetry;
use crate::world::World;
use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;
use std::time::Instant;

pub trait QueryData {
    type Item<'w>;
    type WorldRef<'w>: QueryWorldRef<'w>;

    fn query_types() -> Vec<TypeId>;
    fn append_access(access: &mut QueryAccess);

    fn mark_changed(_world: *mut World, _entity: Entity) {}

    /// Enables cached mark/fetch hooks that avoid per-entity setup work inside the iterator loop.
    fn supports_fast_path() -> bool {
        false
    }

    fn prepare_fast_cache(_world: *mut World, _cache: &mut QueryFastCache) -> bool {
        false
    }

    fn mark_changed_fast(world: *mut World, entity: Entity, _cache: &mut QueryFastCache) {
        Self::mark_changed(world, entity);
    }

    /// Enables archetype-row execution instead of the entity-list fallback path.
    fn supports_archetype_execution() -> bool {
        false
    }

    fn collect_archetype_rows(
        _world: *mut World,
        _required_present: &[TypeId],
        _excluded: &[TypeId],
        _rows: &mut Vec<QueryArchetypeRow>,
        _cache: &mut QueryFastCache,
    ) -> bool {
        false
    }

    /// Safety: the caller must uphold the access guarantees described by `Self::append_access`.
    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>>;

    /// Safety: the caller must uphold the access guarantees described by `Self::append_access`.
    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        _cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        unsafe { Self::fetch(world, entity) }
    }

    /// Safety: `world` must point to a valid world for the returned borrow lifetime.
    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w>;
}

#[doc(hidden)]
pub trait QuerySpec {
    type Item<'w>;
    type WorldRef<'w>: QueryWorldRef<'w>;

    #[doc(hidden)]
    fn query_types() -> Vec<TypeId>;

    #[doc(hidden)]
    fn append_access(access: &mut QueryAccess);

    #[doc(hidden)]
    fn mark_changed(world: *mut World, entity: Entity);

    #[doc(hidden)]
    fn supports_fast_path() -> bool;

    #[doc(hidden)]
    fn prepare_fast_cache(world: *mut World, cache: &mut QueryFastCache) -> bool;

    #[doc(hidden)]
    fn mark_changed_fast(world: *mut World, entity: Entity, cache: &mut QueryFastCache);

    #[doc(hidden)]
    fn supports_archetype_execution() -> bool;

    #[doc(hidden)]
    fn collect_archetype_rows(
        world: *mut World,
        required_present: &[TypeId],
        excluded: &[TypeId],
        rows: &mut Vec<QueryArchetypeRow>,
        cache: &mut QueryFastCache,
    ) -> bool;

    /// # Safety
    /// The caller must uphold the access guarantees described by `Self::append_access`.
    #[doc(hidden)]
    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>>;

    /// # Safety
    /// The caller must uphold the access guarantees described by `Self::append_access`.
    #[doc(hidden)]
    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>>;

    /// # Safety
    /// `world` must point to a valid world for the returned borrow lifetime.
    #[doc(hidden)]
    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w>;
}

impl<T> QuerySpec for T
where
    T: QueryData,
{
    type Item<'w> = T::Item<'w>;
    type WorldRef<'w> = T::WorldRef<'w>;

    fn query_types() -> Vec<TypeId> {
        T::query_types()
    }

    fn append_access(access: &mut QueryAccess) {
        T::append_access(access);
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        T::mark_changed(world, entity);
    }

    fn supports_fast_path() -> bool {
        T::supports_fast_path()
    }

    fn prepare_fast_cache(world: *mut World, cache: &mut QueryFastCache) -> bool {
        T::prepare_fast_cache(world, cache)
    }

    fn mark_changed_fast(world: *mut World, entity: Entity, cache: &mut QueryFastCache) {
        T::mark_changed_fast(world, entity, cache);
    }

    fn supports_archetype_execution() -> bool {
        T::supports_archetype_execution()
    }

    fn collect_archetype_rows(
        world: *mut World,
        required_present: &[TypeId],
        excluded: &[TypeId],
        rows: &mut Vec<QueryArchetypeRow>,
        cache: &mut QueryFastCache,
    ) -> bool {
        T::collect_archetype_rows(world, required_present, excluded, rows, cache)
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        unsafe { T::fetch(world, entity) }
    }

    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        unsafe { T::fetch_fast(world, entity, cache) }
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { T::world_ref(world) }
    }
}

#[doc(hidden)]
pub trait QueryWorldRef<'w> {
    fn into_world_ptr(self) -> *mut World;
}

impl<'w> QueryWorldRef<'w> for &'w World {
    fn into_world_ptr(self) -> *mut World {
        self as *const World as *mut World
    }
}

impl<'w> QueryWorldRef<'w> for &'w mut World {
    fn into_world_ptr(self) -> *mut World {
        self as *mut World
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryArchetypeRow {
    pub entity: Entity,
    pub archetype_index: usize,
    pub row: usize,
}

#[derive(Debug, Clone, Default)]
pub struct QueryFastCache {
    // Tracks cache ownership across reused `QueryState` runs and cross-world rebinding.
    pub(crate) world_ptr: *const World,
    // Reused archetype bindings for archetype-row execution forms.
    pub(crate) archetype_bindings: Vec<ArchetypeExecutionBinding>,
}

pub struct QueryState<Q, F = ()> {
    required_present: Vec<TypeId>,
    excluded: Vec<TypeId>,
    access: QueryAccess,
    last_run_tick: Cell<u64>,
    scratch_pool: Rc<RefCell<Vec<Vec<Entity>>>>,
    archetype_row_scratch_pool: Rc<RefCell<Vec<Vec<QueryArchetypeRow>>>>,
    fast_fetch_enabled: bool,
    archetype_execution_enabled: bool,
    fast_cache: RefCell<QueryFastCache>,
    _marker: PhantomData<(Q, F)>,
}

impl<Q: QuerySpec, F: QueryFilter> QueryState<Q, F> {
    pub fn new(_world: &World) -> Self {
        Self::detached()
    }

    pub fn access(&self) -> &QueryAccess {
        &self.access
    }

    pub fn with<T: Component>(mut self) -> Self {
        push_unique_type(&mut self.required_present, TypeId::of::<T>());
        self
    }

    pub fn without<T: Component>(mut self) -> Self {
        push_unique_type(&mut self.excluded, TypeId::of::<T>());
        self
    }

    pub fn iter<'w>(&self, world: Q::WorldRef<'w>) -> impl Iterator<Item = Q::Item<'w>> + 'w
    where
        Q: 'w,
    {
        let start = Instant::now();
        let world_ptr = world.into_world_ptr();
        let world_ref = unsafe { &*world_ptr };
        let since_tick = self.last_run_tick.get();
        let (use_fast_fetch, mut fast_cache) = self.prepare_fast_fetch(world_ptr);

        if self.archetype_execution_enabled {
            let mut rows = self.acquire_archetype_row_vec();
            if Q::collect_archetype_rows(
                world_ptr,
                &self.required_present,
                &self.excluded,
                &mut rows,
                &mut fast_cache,
            ) {
                if F::needs_tick_filter() {
                    rows.retain(|row| F::matches_entity(world_ref, row.entity, since_tick));
                }
                self.last_run_tick.set(world_ref.current_change_tick());
                telemetry::record_query_iter(start.elapsed().as_nanos() as u64);
                return QueryIter::<Q> {
                    world: world_ptr,
                    entities: None,
                    archetype_rows: Some(rows),
                    scratch_pool: Rc::clone(&self.scratch_pool),
                    archetype_row_scratch_pool: Rc::clone(&self.archetype_row_scratch_pool),
                    use_fast_fetch,
                    fast_cache,
                    index: 0,
                    _marker: PhantomData,
                };
            }
            self.release_archetype_row_vec(rows);
        }

        let mut entities = self.acquire_scratch_vec();
        // Fallback path for query forms that do not support archetype-row execution.
        self.matching_entities_into(world_ref, &mut entities);
        self.last_run_tick.set(world_ref.current_change_tick());
        telemetry::record_query_iter(start.elapsed().as_nanos() as u64);
        QueryIter::<Q> {
            world: world_ptr,
            entities: Some(entities),
            archetype_rows: None,
            scratch_pool: Rc::clone(&self.scratch_pool),
            archetype_row_scratch_pool: Rc::clone(&self.archetype_row_scratch_pool),
            use_fast_fetch,
            fast_cache,
            index: 0,
            _marker: PhantomData,
        }
    }

    pub fn get<'w>(&self, world: Q::WorldRef<'w>, entity: Entity) -> Option<Q::Item<'w>> {
        let start = Instant::now();
        let world_ptr = world.into_world_ptr();
        let world_ref = unsafe { &*world_ptr };
        let matches = self.matches_entity(world_ref, entity);
        self.last_run_tick.set(world_ref.current_change_tick());
        if !matches {
            telemetry::record_query_get(start.elapsed().as_nanos() as u64);
            return None;
        }
        Q::mark_changed(world_ptr, entity);
        // Safety: access conflicts are guarded by the query type and runtime.
        let item = unsafe { Q::fetch(world_ptr, entity) };
        telemetry::record_query_get(start.elapsed().as_nanos() as u64);
        item
    }

    pub fn single<'w>(&self, world: Q::WorldRef<'w>) -> Result<Q::Item<'w>, QueryError> {
        let start = Instant::now();
        let world_ptr = world.into_world_ptr();
        let world_ref = unsafe { &*world_ptr };
        let mut entities = self.acquire_scratch_vec();
        self.matching_entities_into(world_ref, &mut entities);
        self.last_run_tick.set(world_ref.current_change_tick());
        if entities.is_empty() {
            self.release_scratch_vec(entities);
            telemetry::record_query_single(start.elapsed().as_nanos() as u64);
            return Err(QueryError::NoResults);
        }
        if entities.len() > 1 {
            let count = entities.len();
            self.release_scratch_vec(entities);
            telemetry::record_query_single(start.elapsed().as_nanos() as u64);
            return Err(QueryError::MultipleResults { count });
        }
        Q::mark_changed(world_ptr, entities[0]);
        // Safety: exactly one matching entity exists and query access rules are upheld.
        let result = unsafe { Q::fetch(world_ptr, entities[0]) }.ok_or(QueryError::NoResults);
        self.release_scratch_vec(entities);
        telemetry::record_query_single(start.elapsed().as_nanos() as u64);
        result
    }

    pub(crate) fn detached() -> Self {
        let query_types = Q::query_types();
        let mut required = Vec::new();
        let mut excluded = Vec::new();
        F::configure(&mut required, &mut excluded);
        let mut required_present = query_types.clone();
        for type_id in &required {
            push_unique_type(&mut required_present, *type_id);
        }

        let mut access = QueryAccess::default();
        Q::append_access(&mut access);
        F::append_access(&mut access);

        Self {
            required_present,
            excluded,
            access,
            last_run_tick: Cell::new(0),
            scratch_pool: Rc::new(RefCell::new(Vec::new())),
            archetype_row_scratch_pool: Rc::new(RefCell::new(Vec::new())),
            fast_fetch_enabled: Q::supports_fast_path(),
            archetype_execution_enabled: Q::supports_archetype_execution(),
            fast_cache: RefCell::new(QueryFastCache::default()),
            _marker: PhantomData,
        }
    }

    fn prepare_fast_fetch(&self, world_ptr: *mut World) -> (bool, QueryFastCache) {
        if !self.fast_fetch_enabled {
            return (false, QueryFastCache::default());
        }

        let mut cache = self.fast_cache.borrow_mut();
        let prepared = Q::prepare_fast_cache(world_ptr, &mut cache);
        (prepared, cache.clone())
    }

    fn matching_entities_into(&self, world: &World, out: &mut Vec<Entity>) {
        let since_tick = self.last_run_tick.get();
        world.matching_entities_into(&self.required_present, &self.excluded, out);
        if F::needs_tick_filter() {
            out.retain(|entity| F::matches_entity(world, *entity, since_tick));
        }
    }

    fn matches_entity(&self, world: &World, entity: Entity) -> bool {
        let since_tick = self.last_run_tick.get();
        world.entity_matches_component_constraints(entity, &self.required_present, &self.excluded)
            && (!F::needs_tick_filter() || F::matches_entity(world, entity, since_tick))
    }

    fn acquire_scratch_vec(&self) -> Vec<Entity> {
        self.scratch_pool.borrow_mut().pop().unwrap_or_default()
    }

    fn release_scratch_vec(&self, mut entities: Vec<Entity>) {
        entities.clear();
        let mut pool = self.scratch_pool.borrow_mut();
        if pool.len() < 4 {
            pool.push(entities);
        }
    }

    fn acquire_archetype_row_vec(&self) -> Vec<QueryArchetypeRow> {
        self.archetype_row_scratch_pool
            .borrow_mut()
            .pop()
            .unwrap_or_default()
    }

    fn release_archetype_row_vec(&self, mut rows: Vec<QueryArchetypeRow>) {
        rows.clear();
        let mut pool = self.archetype_row_scratch_pool.borrow_mut();
        if pool.len() < 4 {
            pool.push(rows);
        }
    }
}

pub struct Query<Q, F = ()> {
    world: NonNull<World>,
    state: NonNull<QueryState<Q, F>>,
    _marker: PhantomData<(Q, F)>,
}

impl<Q, F> Query<Q, F> {
    pub(crate) fn new(world: *mut World, state: &mut QueryState<Q, F>) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            state: NonNull::from(state),
            _marker: PhantomData,
        }
    }
}

impl<Q: QuerySpec, F: QueryFilter> Query<Q, F> {
    pub fn access(&self) -> &QueryAccess {
        unsafe { self.state.as_ref().access() }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = Q::Item<'_>> + '_ {
        // Safety: system execution guarantees the world pointer remains valid for this call.
        unsafe { self.state.as_ref().iter(Q::world_ref(self.world.as_ptr())) }
    }

    pub fn get(&mut self, entity: Entity) -> Option<Q::Item<'_>> {
        // Safety: system execution guarantees the world pointer remains valid for this call.
        unsafe {
            self.state
                .as_ref()
                .get(Q::world_ref(self.world.as_ptr()), entity)
        }
    }

    pub fn single(&mut self) -> Result<Q::Item<'_>, QueryError> {
        // Safety: system execution guarantees the world pointer remains valid for this call.
        unsafe {
            self.state
                .as_ref()
                .single(Q::world_ref(self.world.as_ptr()))
        }
    }
}

struct QueryIter<'w, Q: QuerySpec> {
    world: *mut World,
    entities: Option<Vec<Entity>>,
    archetype_rows: Option<Vec<QueryArchetypeRow>>,
    scratch_pool: Rc<RefCell<Vec<Vec<Entity>>>>,
    archetype_row_scratch_pool: Rc<RefCell<Vec<Vec<QueryArchetypeRow>>>>,
    use_fast_fetch: bool,
    fast_cache: QueryFastCache,
    index: usize,
    _marker: PhantomData<Q::WorldRef<'w>>,
}

impl<'w, Q: QuerySpec> QueryIter<'w, Q> {
    fn mark_and_fetch(&mut self, entity: Entity) -> Option<Q::Item<'w>> {
        if self.use_fast_fetch {
            Q::mark_changed_fast(self.world, entity, &mut self.fast_cache);
            // Safety: `QueryIter` holds the world borrow contract through `Q::WorldRef<'w>`.
            return unsafe { Q::fetch_fast(self.world, entity, &mut self.fast_cache) };
        }

        Q::mark_changed(self.world, entity);
        // Safety: `QueryIter` holds the world borrow contract through `Q::WorldRef<'w>`.
        unsafe { Q::fetch(self.world, entity) }
    }
}

impl<'w, Q: QuerySpec> Iterator for QueryIter<'w, Q> {
    type Item = Q::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(rows) = self.archetype_rows.as_ref() {
            let rows_ptr = rows.as_ptr();
            let rows_len = rows.len();

            while self.index < rows_len {
                // Safety: `self.index < rows_len` and `rows_ptr` points to `rows`.
                let row = unsafe { *rows_ptr.add(self.index) };
                self.index += 1;
                if let Some(item) = self.mark_and_fetch(row.entity) {
                    return Some(item);
                }
            }
            return None;
        }

        let entities = self.entities.as_ref()?;
        let entities_ptr = entities.as_ptr();
        let entities_len = entities.len();

        while self.index < entities_len {
            // Safety: `self.index < entities_len` and `entities_ptr` points to `entities`.
            let entity = unsafe { *entities_ptr.add(self.index) };
            self.index += 1;
            if let Some(item) = self.mark_and_fetch(entity) {
                return Some(item);
            }
        }
        None
    }
}

impl<'w, Q: QuerySpec> Drop for QueryIter<'w, Q> {
    fn drop(&mut self) {
        if let Some(mut entities) = self.entities.take() {
            entities.clear();
            let mut pool = self.scratch_pool.borrow_mut();
            if pool.len() < 4 {
                pool.push(entities);
            }
        }

        if let Some(mut rows) = self.archetype_rows.take() {
            rows.clear();
            let mut pool = self.archetype_row_scratch_pool.borrow_mut();
            if pool.len() < 4 {
                pool.push(rows);
            }
        }
    }
}
