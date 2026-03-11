// Owner: Grotto Quest ECS - Query Runtime
use super::access_and_filters::{QueryAccess, QueryFilter, push_unique_type};
use crate::component::Component;
use crate::entity::Entity;
use crate::errors::QueryError;
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

    fn supports_fast_path() -> bool {
        false
    }

    fn prepare_fast_cache(_world: *mut World, _cache: &mut QueryFastCache) -> bool {
        false
    }

    fn mark_changed_fast(world: *mut World, entity: Entity, _cache: &mut QueryFastCache) {
        Self::mark_changed(world, entity);
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

#[derive(Debug, Copy, Clone, Default)]
pub struct QueryFastCache {
    pub world_ptr: *const World,
    pub store0: *mut (),
    pub store1: *mut (),
}

pub struct QueryState<Q, F = ()> {
    required_present: Vec<TypeId>,
    excluded: Vec<TypeId>,
    access: QueryAccess,
    last_run_tick: Cell<u64>,
    scratch_pool: Rc<RefCell<Vec<Vec<Entity>>>>,
    fast_path_enabled: bool,
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
        let mut entities = self.acquire_scratch_vec();
        self.matching_entities_into(world_ref, &mut entities);
        let (use_fast_path, fast_cache) = if self.fast_path_enabled {
            let mut cache = self.fast_cache.borrow_mut();
            let prepared = Q::prepare_fast_cache(world_ptr, &mut cache);
            (prepared, *cache)
        } else {
            (false, QueryFastCache::default())
        };
        self.last_run_tick.set(world_ref.current_change_tick());
        telemetry::record_query_iter(start.elapsed().as_nanos() as u64);
        QueryIter::<Q> {
            world: world_ptr,
            entities: Some(entities),
            scratch_pool: Rc::clone(&self.scratch_pool),
            use_fast_path,
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
            fast_path_enabled: Q::supports_fast_path(),
            fast_cache: RefCell::new(QueryFastCache::default()),
            _marker: PhantomData,
        }
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
    scratch_pool: Rc<RefCell<Vec<Vec<Entity>>>>,
    use_fast_path: bool,
    fast_cache: QueryFastCache,
    index: usize,
    _marker: PhantomData<Q::WorldRef<'w>>,
}

impl<'w, Q: QuerySpec> Iterator for QueryIter<'w, Q> {
    type Item = Q::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(entities) = self.entities.as_ref() else {
            return None;
        };
        let entities_ptr = entities.as_ptr();
        let entities_len = entities.len();

        if self.use_fast_path {
            while self.index < entities_len {
                // Safety: `self.index < entities_len` and `entities_ptr` points to `entities`.
                let entity = unsafe { *entities_ptr.add(self.index) };
                self.index += 1;
                Q::mark_changed_fast(self.world, entity, &mut self.fast_cache);
                // Safety: `QueryIter` holds the world borrow contract through `Q::WorldRef<'w>`.
                if let Some(item) =
                    unsafe { Q::fetch_fast(self.world, entity, &mut self.fast_cache) }
                {
                    return Some(item);
                }
            }
            return None;
        }

        while self.index < entities_len {
            // Safety: `self.index < entities_len` and `entities_ptr` points to `entities`.
            let entity = unsafe { *entities_ptr.add(self.index) };
            self.index += 1;
            Q::mark_changed(self.world, entity);
            // Safety: `QueryIter` holds the world borrow contract through `Q::WorldRef<'w>`.
            if let Some(item) = unsafe { Q::fetch(self.world, entity) } {
                return Some(item);
            }
        }
        None
    }
}

impl<'w, Q: QuerySpec> Drop for QueryIter<'w, Q> {
    fn drop(&mut self) {
        let Some(mut entities) = self.entities.take() else {
            return;
        };
        entities.clear();
        let mut pool = self.scratch_pool.borrow_mut();
        if pool.len() < 4 {
            pool.push(entities);
        }
    }
}
