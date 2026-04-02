use super::extract::{SystemParam, SystemParamError};
use crate::Commands;
use crate::World;
use crate::component::{Component, Resource};
use crate::query::{
    Query, QueryAccess, QueryFilter, QueryOrphaned, QueryOrphanedState, QuerySpec, QueryState,
};
use crate::telemetry;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::time::Instant;

pub struct Res<T: Resource> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

/// Semantic alias for read-only resource system params.
pub type ResView<T> = Res<T>;

impl<T: Resource> Res<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }
}

impl<T: Resource> Deref for Res<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe {
            self.world
                .as_ref()
                .resource::<T>()
                .expect("resource parameter was validated during system registration")
        }
    }
}

pub struct ResMut<T: Resource> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: Resource> ResMut<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }
}

impl<T: Resource> Deref for ResMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: extraction guarantees a live world pointer during system execution.
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
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe {
            self.world
                .as_mut()
                .resource_mut::<T>()
                .expect("resource parameter was validated during system registration")
        }
    }
}

pub struct EventReader<T: 'static> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: 'static> EventReader<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer during system execution.
        let events = unsafe { self.world.as_ref().read_events::<T>() };
        telemetry::record_event_reader(start.elapsed().as_nanos() as u64, events.len() as u64);
        events.iter()
    }
}

#[derive(Debug, Default)]
pub struct EventChannelState {
    next_sequence: u64,
}

pub struct EventChannel<T: 'static> {
    world: NonNull<World>,
    state: NonNull<EventChannelState>,
    _marker: PhantomData<T>,
}

impl<T: 'static> EventChannel<T> {
    pub(crate) fn new(world: *mut World, state: *mut EventChannelState) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            state: NonNull::new(state).expect("event channel state pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn iter_all(&self) -> std::slice::Iter<'_, T> {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer during system execution.
        let events = unsafe { self.world.as_ref().read_events::<T>() };
        telemetry::record_event_reader(start.elapsed().as_nanos() as u64, events.len() as u64);
        events.iter()
    }

    pub fn iter_new(&mut self) -> std::slice::Iter<'_, T> {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer and param state pointer.
        let state = unsafe { self.state.as_mut() };
        // Safety: extraction guarantees a live world pointer during system execution.
        let (events, next_sequence) = unsafe {
            self.world
                .as_ref()
                .read_events_since::<T>(state.next_sequence)
        };
        state.next_sequence = next_sequence;
        telemetry::record_event_reader(start.elapsed().as_nanos() as u64, events.len() as u64);
        events.iter()
    }

    pub fn send(&mut self, event: T) {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().emit_event(event) };
        telemetry::record_event_writer(start.elapsed().as_nanos() as u64, 1);
    }
}

pub struct EventWriter<T: 'static> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: 'static> EventWriter<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn send(&mut self, event: T) {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().emit_event(event) };
        telemetry::record_event_writer(start.elapsed().as_nanos() as u64, 1);
    }
}

impl<'w, Q, F> SystemParam<'w> for Query<Q, F>
where
    Q: QuerySpec + 'static,
    F: QueryFilter + 'static,
{
    type State = QueryState<Q, F>;

    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(QueryState::new(world))
    }

    fn access(state: &Self::State) -> QueryAccess {
        state.access().clone()
    }

    unsafe fn extract(
        state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(Query::new(world, state))
    }
}

impl<'w, T> SystemParam<'w> for QueryOrphaned<T>
where
    T: Component + 'static,
{
    type State = QueryOrphanedState<T>;

    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(QueryOrphanedState::new(world))
    }

    fn access(state: &Self::State) -> QueryAccess {
        state.access().clone()
    }

    unsafe fn extract(
        state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(QueryOrphaned::new(world, state))
    }
}

impl<'w, T: Resource> SystemParam<'w> for Res<T> {
    type State = ();

    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError> {
        world.resource::<T>()?;
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_resource_read::<T>();
        access
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(Res::new(world))
    }
}

impl<'w, T: Resource> SystemParam<'w> for ResMut<T> {
    type State = ();

    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError> {
        world.resource::<T>()?;
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_resource_write::<T>();
        access
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(ResMut::new(world))
    }
}

impl<'w> SystemParam<'w> for Commands {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.set_deferred_structural_mutation();
        access
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        _world: *mut World,
        commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(Commands::from_external(commands))
    }
}

impl<'w, T: 'static> SystemParam<'w> for EventReader<T> {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_resource_read_named::<T>(std::any::type_name::<T>());
        access
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(EventReader::new(world))
    }
}

impl<'w, T: 'static> SystemParam<'w> for EventWriter<T> {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_resource_write_named::<T>(std::any::type_name::<T>());
        access
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(EventWriter::new(world))
    }
}

impl<'w, T: 'static> SystemParam<'w> for EventChannel<T> {
    type State = EventChannelState;

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(EventChannelState::default())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_resource_write_named::<T>(std::any::type_name::<T>());
        access
    }

    unsafe fn extract(
        state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(EventChannel::new(world, state as *mut EventChannelState))
    }
}
