use super::extract::{SystemParam, SystemParamError};
use crate::Commands;
use crate::World;
use crate::component::{Component, Resource};
use crate::query::{
    Query, QueryAccess, QueryFilter, QueryOrphaned, QueryOrphanedState, QuerySpec, QueryState,
};
use crate::telemetry;
use crate::world::messaging::{InputStreamPushError, QueueEnqueueError};
use scheduler::system::ParamSlotDescriptor;
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

#[derive(Debug, Default)]
pub struct BroadcastReaderState {
    next_sequence: u64,
}

pub struct BroadcastReader<T: 'static> {
    world: NonNull<World>,
    state: NonNull<BroadcastReaderState>,
    _marker: PhantomData<T>,
}

impl<T: 'static> BroadcastReader<T> {
    pub(crate) fn new(world: *mut World, state: *mut BroadcastReaderState) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            state: NonNull::new(state).expect("broadcast reader state pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.iter_all()
    }

    pub fn iter_all(&self) -> std::slice::Iter<'_, T> {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer during system execution.
        let messages = unsafe { self.world.as_ref().read_broadcast::<T>() };
        telemetry::record_event_reader(start.elapsed().as_nanos() as u64, messages.len() as u64);
        messages.iter()
    }

    pub fn iter_new(&mut self) -> std::slice::Iter<'_, T> {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer and param state pointer.
        let state = unsafe { self.state.as_mut() };
        // Safety: extraction guarantees a live world pointer during system execution.
        let (messages, next_sequence) = unsafe {
            self.world
                .as_ref()
                .read_broadcast_since::<T>(state.next_sequence)
        };
        state.next_sequence = next_sequence;
        telemetry::record_event_reader(start.elapsed().as_nanos() as u64, messages.len() as u64);
        messages.iter()
    }
}

pub struct BroadcastWriter<T: 'static> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: 'static> BroadcastWriter<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn send(&mut self, message: T) {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().publish_broadcast(message) };
        telemetry::record_event_writer(start.elapsed().as_nanos() as u64, 1);
    }
}

pub struct QueueReader<T: 'static> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: 'static> QueueReader<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_ref().queue_iter::<T>() }
    }

    pub fn len(&self) -> usize {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_ref().queue_pending_count::<T>() }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn peek(&self) -> Option<&T> {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_ref().queue_peek::<T>() }
    }
}

pub struct QueueWriter<T: 'static> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: 'static> QueueWriter<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn enqueue(&mut self, message: T) -> Result<(), QueueEnqueueError> {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().queue_enqueue(message) }
    }
}

pub struct QueueDrainer<T: 'static> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: 'static> QueueDrainer<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn drain(&mut self) -> Vec<T> {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().queue_drain::<T>() }
    }

    pub fn clear(&mut self) -> usize {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().clear_queue::<T>() }
    }
}

pub struct InputStreamReader<T: 'static> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: 'static> InputStreamReader<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.iter_current()
    }

    pub fn iter_current(&self) -> std::slice::Iter<'_, T> {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer during system execution.
        let inputs = unsafe { self.world.as_ref().read_input_current_tick::<T>() };
        telemetry::record_event_reader(start.elapsed().as_nanos() as u64, inputs.len() as u64);
        inputs.iter()
    }

    pub fn iter_tick(&self, tick: u64) -> std::slice::Iter<'_, T> {
        let start = Instant::now();
        // Safety: extraction guarantees a live world pointer during system execution.
        let inputs = unsafe { self.world.as_ref().read_input_tick::<T>(tick) };
        telemetry::record_event_reader(start.elapsed().as_nanos() as u64, inputs.len() as u64);
        inputs.iter()
    }
}

pub struct InputStreamWriter<T: 'static> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: 'static> InputStreamWriter<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn push_for_tick(&mut self, tick: u64, input: T) -> Result<(), InputStreamPushError> {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().push_input_for_tick::<T>(tick, input) }
    }

    pub fn push_current(&mut self, input: T) -> Result<(), InputStreamPushError> {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().push_input_for_current_tick::<T>(input) }
    }
}

pub struct InputStreamDrainer<T: 'static> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: 'static> InputStreamDrainer<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }

    pub fn drain_tick(&mut self, tick: u64) -> Vec<T> {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().drain_input_tick::<T>(tick) }
    }

    pub fn drain_current(&mut self) -> Vec<T> {
        // Safety: extraction guarantees a live world pointer during system execution.
        unsafe { self.world.as_mut().drain_input_current_tick::<T>() }
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

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "query",
            label: "Query",
            type_name: std::any::type_name::<Self>(),
        }
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

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "query_orphaned",
            label: "QueryOrphaned",
            type_name: std::any::type_name::<Self>(),
        }
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

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "res",
            label: "Res",
            type_name: std::any::type_name::<Self>(),
        }
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

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "res_mut",
            label: "ResMut",
            type_name: std::any::type_name::<Self>(),
        }
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

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "commands",
            label: "Commands",
            type_name: std::any::type_name::<Self>(),
        }
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        _world: *mut World,
        commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(Commands::from_external(commands))
    }
}

impl<'w, T: 'static> SystemParam<'w> for BroadcastReader<T> {
    type State = BroadcastReaderState;

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(BroadcastReaderState::default())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_broadcast_read_named::<T>(std::any::type_name::<T>());
        access
    }

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "broadcast_reader",
            label: "BroadcastReader",
            type_name: std::any::type_name::<Self>(),
        }
    }

    unsafe fn extract(
        state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(BroadcastReader::new(
            world,
            state as *mut BroadcastReaderState,
        ))
    }
}

impl<'w, T: 'static> SystemParam<'w> for BroadcastWriter<T> {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_broadcast_write_named::<T>(std::any::type_name::<T>());
        access
    }

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "broadcast_writer",
            label: "BroadcastWriter",
            type_name: std::any::type_name::<Self>(),
        }
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(BroadcastWriter::new(world))
    }
}

impl<'w, T: 'static> SystemParam<'w> for QueueReader<T> {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_queue_read_named::<T>(std::any::type_name::<T>());
        access
    }

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "queue_reader",
            label: "QueueReader",
            type_name: std::any::type_name::<Self>(),
        }
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(QueueReader::new(world))
    }
}

impl<'w, T: 'static> SystemParam<'w> for QueueWriter<T> {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_queue_write_named::<T>(std::any::type_name::<T>());
        access
    }

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "queue_writer",
            label: "QueueWriter",
            type_name: std::any::type_name::<Self>(),
        }
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(QueueWriter::new(world))
    }
}

impl<'w, T: 'static> SystemParam<'w> for QueueDrainer<T> {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_queue_drain_named::<T>(std::any::type_name::<T>());
        access
    }

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "queue_drainer",
            label: "QueueDrainer",
            type_name: std::any::type_name::<Self>(),
        }
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(QueueDrainer::new(world))
    }
}

impl<'w, T: 'static> SystemParam<'w> for InputStreamReader<T> {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_input_stream_read_named::<T>(std::any::type_name::<T>());
        access
    }

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "input_stream_reader",
            label: "InputStreamReader",
            type_name: std::any::type_name::<Self>(),
        }
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(InputStreamReader::new(world))
    }
}

impl<'w, T: 'static> SystemParam<'w> for InputStreamWriter<T> {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_input_stream_write_named::<T>(std::any::type_name::<T>());
        access
    }

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "input_stream_writer",
            label: "InputStreamWriter",
            type_name: std::any::type_name::<Self>(),
        }
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(InputStreamWriter::new(world))
    }
}

impl<'w, T: 'static> SystemParam<'w> for InputStreamDrainer<T> {
    type State = ();

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }

    fn access(_state: &Self::State) -> QueryAccess {
        let mut access = QueryAccess::default();
        access.add_input_stream_drain_named::<T>(std::any::type_name::<T>());
        access
    }

    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor {
            kind: "input_stream_drainer",
            label: "InputStreamDrainer",
            type_name: std::any::type_name::<Self>(),
        }
    }

    unsafe fn extract(
        _state: &'w mut Self::State,
        world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        Ok(InputStreamDrainer::new(world))
    }
}
