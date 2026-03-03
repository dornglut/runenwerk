use anyhow::{Context, Result};
use ecs_v2::query::{MutableQueryData, QueryFilter, ReadOnlyQueryData};
use ecs_v2::{Bundle, Entity, QueryState, Resource, World};
use scheduler::{AccessKey, SystemAccess};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub trait SystemParam: Sized {
    type State: 'static;

    fn init_state(world: &mut World) -> Result<Self::State>;
    fn access(state: &Self::State) -> SystemAccess;

    unsafe fn get_param(
        state: &mut Self::State,
        world: *mut World,
        commands: *mut ecs_v2::Commands,
    ) -> Result<Self>;
}

pub struct Query<Q, F = ()> {
    world: NonNull<World>,
    state: NonNull<QueryState<Q, F>>,
    _marker: PhantomData<(Q, F)>,
}

impl<Q, F> Query<Q, F> {
    fn new(world: *mut World, state: &mut QueryState<Q, F>) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            state: NonNull::from(state),
            _marker: PhantomData,
        }
    }
}

impl<Q, F> Query<Q, F>
where
    Q: ReadOnlyQueryData,
    F: QueryFilter,
{
    pub fn iter(&self) -> ecs_v2::query::QueryIter<'_, Q> {
        // Safety: system execution guarantees the world pointer remains valid for the system call.
        unsafe { self.state.as_ref().iter_on(self.world.as_ref()) }
    }

    pub fn get(&self, entity: Entity) -> Option<Q::Item<'_>> {
        unsafe { self.state.as_ref().get_on(self.world.as_ref(), entity) }
    }

    pub fn single(&self) -> Result<Q::Item<'_>, ecs_v2::QueryError> {
        unsafe { self.state.as_ref().single_on(self.world.as_ref()) }
    }
}

impl<Q, F> Query<Q, F>
where
    Q: MutableQueryData,
    F: QueryFilter,
{
    pub fn iter_mut(&mut self) -> ecs_v2::query::QueryIterMut<'_, Q> {
        unsafe { self.state.as_mut().iter_mut_on(self.world.as_mut()) }
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<Q::Item<'_>> {
        unsafe { self.state.as_mut().get_mut_on(self.world.as_mut(), entity) }
    }

    pub fn single_mut(&mut self) -> Result<Q::Item<'_>, ecs_v2::QueryError> {
        unsafe { self.state.as_mut().single_mut_on(self.world.as_mut()) }
    }
}

impl<Q, F> SystemParam for Query<Q, F>
where
    Q: ecs_v2::QueryData + 'static,
    F: QueryFilter + 'static,
{
    type State = QueryState<Q, F>;

    fn init_state(world: &mut World) -> Result<Self::State> {
        Ok(QueryState::new(world))
    }

    fn access(state: &Self::State) -> SystemAccess {
        query_access_to_system_access(state.access())
    }

    unsafe fn get_param(
        state: &mut Self::State,
        world: *mut World,
        _commands: *mut ecs_v2::Commands,
    ) -> Result<Self> {
        Ok(Self::new(world, state))
    }
}

pub struct Res<T> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T> Res<T> {
    fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }
}

impl<T: Resource> Deref for Res<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.world
                .as_ref()
                .resource::<T>()
                .expect("resource parameter was validated during system registration")
        }
    }
}

impl<T: Resource> SystemParam for Res<T> {
    type State = ();

    fn init_state(world: &mut World) -> Result<Self::State> {
        world
            .resource::<T>()
            .with_context(|| format!("missing resource {}", std::any::type_name::<T>()))?;
        Ok(())
    }

    fn access(_state: &Self::State) -> SystemAccess {
        SystemAccess::new().with_read(AccessKey::resource::<T>(std::any::type_name::<T>()))
    }

    unsafe fn get_param(
        _state: &mut Self::State,
        world: *mut World,
        _commands: *mut ecs_v2::Commands,
    ) -> Result<Self> {
        Ok(Self::new(world))
    }
}

pub struct ResMut<T> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T> ResMut<T> {
    fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }
}

impl<T: Resource> Deref for ResMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.world
                .as_ref()
                .resource::<T>()
                .expect("resource parameter was validated during system registration")
        }
    }
}

impl<T: Resource> DerefMut for ResMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *(self
                .world
                .as_mut()
                .resource_mut::<T>()
                .expect("resource parameter was validated during system registration")
                .deref_mut() as *mut T)
        }
    }
}

impl<T: Resource> SystemParam for ResMut<T> {
    type State = ();

    fn init_state(world: &mut World) -> Result<Self::State> {
        world
            .resource::<T>()
            .with_context(|| format!("missing resource {}", std::any::type_name::<T>()))?;
        Ok(())
    }

    fn access(_state: &Self::State) -> SystemAccess {
        SystemAccess::new().with_write(AccessKey::resource::<T>(std::any::type_name::<T>()))
    }

    unsafe fn get_param(
        _state: &mut Self::State,
        world: *mut World,
        _commands: *mut ecs_v2::Commands,
    ) -> Result<Self> {
        Ok(Self::new(world))
    }
}

pub struct Commands {
    queue: NonNull<ecs_v2::Commands>,
}

impl Commands {
    fn new(queue: *mut ecs_v2::Commands) -> Self {
        Self {
            queue: NonNull::new(queue).expect("command queue pointer must not be null"),
        }
    }

    pub fn spawn<B: Bundle + 'static>(&mut self, bundle: B) {
        unsafe { self.queue.as_mut().spawn(bundle) };
    }

    pub fn despawn(&mut self, entity: Entity) {
        unsafe { self.queue.as_mut().despawn(entity) };
    }

    pub fn insert<B: Bundle + 'static>(&mut self, entity: Entity, bundle: B) {
        unsafe { self.queue.as_mut().insert(entity, bundle) };
    }

    pub fn remove<B: Bundle + 'static>(&mut self, entity: Entity) {
        unsafe { self.queue.as_mut().remove::<B>(entity) };
    }
}

impl SystemParam for Commands {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State> {
        Ok(())
    }

    fn access(_state: &Self::State) -> SystemAccess {
        SystemAccess::new().with_write(AccessKey::structural("world_structure"))
    }

    unsafe fn get_param(
        _state: &mut Self::State,
        _world: *mut World,
        commands: *mut ecs_v2::Commands,
    ) -> Result<Self> {
        Ok(Self::new(commands))
    }
}

fn query_access_to_system_access(access: &ecs_v2::QueryAccess) -> SystemAccess {
    let mut system_access = SystemAccess::new();
    for read in access.component_reads() {
        system_access.add_read(AccessKey::component_by_id(read.type_id(), read.name()));
    }
    for write in access.component_writes() {
        system_access.add_write(AccessKey::component_by_id(write.type_id(), write.name()));
    }
    system_access
}
