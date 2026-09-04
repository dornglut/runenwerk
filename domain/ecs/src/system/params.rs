use super::extract::{SystemParam, SystemParamContext, SystemParamError};
use crate::Commands;
use crate::World;
use crate::component::{Component, Resource};
use crate::query::{
    Query, QueryAccess, QueryFilter, QueryOrphaned, QueryOrphanedState, QuerySpec, QueryState,
};
use crate::telemetry;
use crate::world::messaging::{TickBufferProvenance, TickBufferPushError, WorkQueueEnqueueError};
use crate::world::{MessagingCapability, ResourceCapability, ResourceMutationCapability};
use scheduler::system::ParamSlotDescriptor;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::time::Instant;

pub struct Res<'world, T: Resource> {
    value: NonNull<T>,
    _marker: PhantomData<&'world T>,
}
pub type ResView<'world, T> = Res<'world, T>;
impl<'world, T: Resource> Res<'world, T> {
    pub(crate) fn new(capability: ResourceCapability<'world, T>) -> Self {
        Self {
            value: capability.value(),
            _marker: PhantomData,
        }
    }
}
impl<'world, T: Resource> Deref for Res<'world, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.value.as_ref() }
    }
}

pub struct ResMut<'world, T: Resource> {
    value: NonNull<T>,
    mutation: ResourceMutationCapability<'world>,
    _marker: PhantomData<&'world mut T>,
}
impl<'world, T: Resource> ResMut<'world, T> {
    pub(crate) fn new(capability: ResourceCapability<'world, T>) -> Self {
        Self {
            value: capability.value(),
            mutation: capability
                .mutation()
                .expect("mutable resource capability must track mutation"),
            _marker: PhantomData,
        }
    }
}
impl<'world, T: Resource> Deref for ResMut<'world, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.value.as_ref() }
    }
}
impl<'world, T: Resource> DerefMut for ResMut<'world, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.mutation.mark_modified::<T>();
        unsafe { self.value.as_mut() }
    }
}

#[derive(Debug, Default)]
pub struct BroadcastReaderState {
    next_sequence: u64,
}
pub struct BroadcastReader<'world, 'state, T: 'static> {
    messaging: MessagingCapability<'world>,
    state: NonNull<BroadcastReaderState>,
    _marker: PhantomData<(&'state mut BroadcastReaderState, T)>,
}
impl<'world, 'state, T: 'static> BroadcastReader<'world, 'state, T> {
    pub(crate) fn new(
        messaging: MessagingCapability<'world>,
        state: &mut BroadcastReaderState,
    ) -> Self {
        Self {
            messaging,
            state: NonNull::from(state),
            _marker: PhantomData,
        }
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.iter_all()
    }
    pub fn iter_all(&self) -> std::slice::Iter<'_, T> {
        let start = Instant::now();
        let messages = self.messaging.broadcast_read::<T>();
        telemetry::record_event_reader(start.elapsed().as_nanos() as u64, messages.len() as u64);
        messages.iter()
    }
    pub fn iter_new(&mut self) -> std::slice::Iter<'_, T> {
        let start = Instant::now();
        let state = unsafe { self.state.as_mut() };
        let (messages, next) = self
            .messaging
            .broadcast_read_since::<T>(state.next_sequence);
        state.next_sequence = next;
        telemetry::record_event_reader(start.elapsed().as_nanos() as u64, messages.len() as u64);
        messages.iter()
    }
}

pub struct BroadcastWriter<'world, T: 'static> {
    messaging: MessagingCapability<'world>,
    _marker: PhantomData<T>,
}
impl<'world, T: 'static> BroadcastWriter<'world, T> {
    pub(crate) fn new(messaging: MessagingCapability<'world>) -> Self {
        Self {
            messaging,
            _marker: PhantomData,
        }
    }
    pub fn send(&mut self, message: T) {
        let start = Instant::now();
        self.messaging.broadcast_publish(message);
        telemetry::record_event_writer(start.elapsed().as_nanos() as u64, 1);
    }
}

pub struct WorkQueueReader<'world, T: 'static> {
    messaging: MessagingCapability<'world>,
    _marker: PhantomData<T>,
}
impl<'world, T: 'static> WorkQueueReader<'world, T> {
    pub(crate) fn new(messaging: MessagingCapability<'world>) -> Self {
        Self {
            messaging,
            _marker: PhantomData,
        }
    }
    pub fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        self.messaging.work_queue_iter::<T>()
    }
    pub fn len(&self) -> usize {
        self.messaging.work_queue_len::<T>()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn peek(&self) -> Option<&T> {
        self.messaging.work_queue_peek::<T>()
    }
}
pub struct WorkQueueWriter<'world, T: 'static> {
    messaging: MessagingCapability<'world>,
    _marker: PhantomData<T>,
}
impl<'world, T: 'static> WorkQueueWriter<'world, T> {
    pub(crate) fn new(messaging: MessagingCapability<'world>) -> Self {
        Self {
            messaging,
            _marker: PhantomData,
        }
    }
    pub fn enqueue(&mut self, message: T) -> Result<(), WorkQueueEnqueueError> {
        self.messaging.work_queue_enqueue(message)
    }
}
pub struct WorkQueueDrainer<'world, T: 'static> {
    messaging: MessagingCapability<'world>,
    _marker: PhantomData<T>,
}
impl<'world, T: 'static> WorkQueueDrainer<'world, T> {
    pub(crate) fn new(messaging: MessagingCapability<'world>) -> Self {
        Self {
            messaging,
            _marker: PhantomData,
        }
    }
    pub fn drain(&mut self) -> Vec<T> {
        self.messaging.work_queue_drain::<T>()
    }
    pub fn clear(&mut self) -> usize {
        self.messaging.work_queue_clear::<T>()
    }
}

pub struct TickBufferReader<'world, T: 'static> {
    messaging: MessagingCapability<'world>,
    _marker: PhantomData<T>,
}
impl<'world, T: 'static> TickBufferReader<'world, T> {
    pub(crate) fn new(messaging: MessagingCapability<'world>) -> Self {
        Self {
            messaging,
            _marker: PhantomData,
        }
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.iter_current()
    }
    pub fn iter_current(&self) -> std::slice::Iter<'_, T> {
        self.messaging.current_buffer_messages::<T>().iter()
    }
    pub fn iter_tick(&self, tick: u64) -> std::slice::Iter<'_, T> {
        self.messaging.buffer_messages_at_tick::<T>(tick).iter()
    }
}
pub struct TickBufferWriter<'world, T: 'static> {
    messaging: MessagingCapability<'world>,
    _marker: PhantomData<T>,
}
impl<'world, T: 'static> TickBufferWriter<'world, T> {
    pub(crate) fn new(messaging: MessagingCapability<'world>) -> Self {
        Self {
            messaging,
            _marker: PhantomData,
        }
    }
    pub fn push_for_tick(&mut self, tick: u64, input: T) -> Result<(), TickBufferPushError> {
        self.messaging
            .push_buffer_message(tick, TickBufferProvenance::UNSPECIFIED, input)
            .map(|_| ())
    }
    pub fn push_current(&mut self, input: T) -> Result<(), TickBufferPushError> {
        let tick = self.messaging.current_buffer_tick();
        self.push_for_tick(tick, input)
    }
}
pub struct TickBufferDrainer<'world, T: 'static> {
    messaging: MessagingCapability<'world>,
    _marker: PhantomData<T>,
}
impl<'world, T: 'static> TickBufferDrainer<'world, T> {
    pub(crate) fn new(messaging: MessagingCapability<'world>) -> Self {
        Self {
            messaging,
            _marker: PhantomData,
        }
    }
    pub fn drain_tick(&mut self, tick: u64) -> Vec<T> {
        self.messaging.drain_buffer::<T>(tick)
    }
    pub fn drain_current(&mut self) -> Vec<T> {
        self.messaging
            .drain_buffer::<T>(self.messaging.current_buffer_tick())
    }
}

unsafe impl<'param, 'cached, Q, F> SystemParam for Query<'param, 'cached, Q, F>
where
    Q: QuerySpec + 'static,
    F: QueryFilter + 'static,
{
    type State = QueryState<Q, F>;
    type Item<'world, 'state> = Query<'world, 'state, Q, F>;
    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(QueryState::new(world))
    }
    fn access(state: &Self::State) -> QueryAccess {
        state.access().clone()
    }
    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor::leaf("query", "Query", std::any::type_name::<Self>())
    }
    unsafe fn extract<'world, 'state>(
        state: &'state mut Self::State,
        context: SystemParamContext<'world>,
    ) -> Result<Self::Item<'world, 'state>, SystemParamError> {
        Ok(Query::new(context.query(), state))
    }
}
unsafe impl<'param, 'cached, T: Component + 'static> SystemParam
    for QueryOrphaned<'param, 'cached, T>
{
    type State = QueryOrphanedState<T>;
    type Item<'world, 'state> = QueryOrphaned<'world, 'state, T>;
    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(QueryOrphanedState::new(world))
    }
    fn access(state: &Self::State) -> QueryAccess {
        state.access().clone()
    }
    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor::leaf(
            "query_orphaned",
            "QueryOrphaned",
            std::any::type_name::<Self>(),
        )
    }
    unsafe fn extract<'world, 'state>(
        state: &'state mut Self::State,
        context: SystemParamContext<'world>,
    ) -> Result<Self::Item<'world, 'state>, SystemParamError> {
        Ok(QueryOrphaned::new(context.query(), state))
    }
}
unsafe impl<'param, T: Resource + 'static> SystemParam for Res<'param, T> {
    type State = ();
    type Item<'world, 'state> = Res<'world, T>;
    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError> {
        world.resource::<T>()?;
        Ok(())
    }
    fn access(_: &Self::State) -> QueryAccess {
        QueryAccess::default().with_resource_read::<T>()
    }
    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor::leaf("res", "Res", std::any::type_name::<Self>())
    }
    unsafe fn extract<'world, 'state>(
        _: &'state mut Self::State,
        context: SystemParamContext<'world>,
    ) -> Result<Self::Item<'world, 'state>, SystemParamError> {
        Ok(Res::new(context.resource::<T>()?))
    }
}
unsafe impl<'param, T: Resource + 'static> SystemParam for ResMut<'param, T> {
    type State = ();
    type Item<'world, 'state> = ResMut<'world, T>;
    fn init_state(world: &mut World) -> Result<Self::State, SystemParamError> {
        world.resource::<T>()?;
        Ok(())
    }
    fn access(_: &Self::State) -> QueryAccess {
        QueryAccess::default().with_resource_write::<T>()
    }
    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor::leaf("res_mut", "ResMut", std::any::type_name::<Self>())
    }
    unsafe fn extract<'world, 'state>(
        _: &'state mut Self::State,
        context: SystemParamContext<'world>,
    ) -> Result<Self::Item<'world, 'state>, SystemParamError> {
        Ok(ResMut::new(context.resource_mut::<T>()?))
    }
}
unsafe impl<'param> SystemParam for Commands<'param> {
    type State = ();
    type Item<'world, 'state> = Commands<'world>;
    fn init_state(_: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(())
    }
    fn access(_: &Self::State) -> QueryAccess {
        QueryAccess::structural_mutation()
    }
    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor::leaf("commands", "Commands", std::any::type_name::<Self>())
    }
    unsafe fn extract<'world, 'state>(
        _: &'state mut Self::State,
        context: SystemParamContext<'world>,
    ) -> Result<Self::Item<'world, 'state>, SystemParamError> {
        Ok(context.commands())
    }
}

macro_rules! impl_tuple_system_param {
    ($(($index:tt, $param:ident)),+ $(,)?) => {
        unsafe impl<$($param: SystemParam),+> SystemParam for ($($param,)+) {
            type State = ($($param::State,)+); type Item<'world, 'state> = ($($param::Item<'world, 'state>,)+);
            fn init_state(world: &mut World) -> Result<Self::State, SystemParamError> { Ok(($($param::init_state(world)?,)+)) }
            fn access(state: &Self::State) -> QueryAccess { let mut access = QueryAccess::default(); $(access.extend($param::access(&state.$index));)+ access }
            fn slot_descriptor() -> ParamSlotDescriptor { ParamSlotDescriptor::group("tuple", "Tuple", std::any::type_name::<Self>(), vec![$(ParamSlotDescriptor::named_child(stringify!($index), $param::slot_descriptor()),)+]) }
            unsafe fn extract<'world, 'state>(state: &'state mut Self::State, context: SystemParamContext<'world>) -> Result<Self::Item<'world, 'state>, SystemParamError> { Ok(($((unsafe { $param::extract(&mut state.$index, context) })?,)+)) }
        }
    };
}
impl_tuple_system_param!((0, A), (1, B));
impl_tuple_system_param!((0, A), (1, B), (2, C));
impl_tuple_system_param!((0, A), (1, B), (2, C), (3, D));
impl_tuple_system_param!((0, A), (1, B), (2, C), (3, D), (4, E));
impl_tuple_system_param!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F));
impl_tuple_system_param!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G));
impl_tuple_system_param!(
    (0, A),
    (1, B),
    (2, C),
    (3, D),
    (4, E),
    (5, F),
    (6, G),
    (7, H)
);

macro_rules! impl_message_simple {
    ($ty:ident, $access:ident, $kind:literal) => {
        unsafe impl<'param, T: 'static> SystemParam for $ty<'param, T> {
            type State = ();
            type Item<'world, 'state> = $ty<'world, T>;
            fn init_state(_: &mut World) -> Result<Self::State, SystemParamError> {
                Ok(())
            }
            fn access(_: &Self::State) -> QueryAccess {
                let mut a = QueryAccess::default();
                a.$access::<T>(std::any::type_name::<T>());
                a
            }
            fn slot_descriptor() -> ParamSlotDescriptor {
                ParamSlotDescriptor::leaf($kind, stringify!($ty), std::any::type_name::<Self>())
            }
            unsafe fn extract<'world, 'state>(
                _: &'state mut Self::State,
                context: SystemParamContext<'world>,
            ) -> Result<Self::Item<'world, 'state>, SystemParamError> {
                Ok($ty::new(context.messaging()))
            }
        }
    };
}
impl_message_simple!(
    BroadcastWriter,
    add_broadcast_write_named,
    "broadcast_writer"
);
impl_message_simple!(
    WorkQueueReader,
    add_work_queue_read_named,
    "work_queue_reader"
);
impl_message_simple!(
    WorkQueueWriter,
    add_work_queue_write_named,
    "work_queue_writer"
);
impl_message_simple!(
    WorkQueueDrainer,
    add_work_queue_drain_named,
    "work_queue_drainer"
);
impl_message_simple!(
    TickBufferReader,
    add_tick_buffer_read_named,
    "tick_buffer_reader"
);
impl_message_simple!(
    TickBufferWriter,
    add_tick_buffer_write_named,
    "tick_buffer_writer"
);
impl_message_simple!(
    TickBufferDrainer,
    add_tick_buffer_drain_named,
    "tick_buffer_drainer"
);

unsafe impl<'param, 'cached, T: 'static> SystemParam for BroadcastReader<'param, 'cached, T> {
    type State = BroadcastReaderState;
    type Item<'world, 'state> = BroadcastReader<'world, 'state, T>;
    fn init_state(_: &mut World) -> Result<Self::State, SystemParamError> {
        Ok(Default::default())
    }
    fn access(_: &Self::State) -> QueryAccess {
        let mut a = QueryAccess::default();
        a.add_broadcast_read_named::<T>(std::any::type_name::<T>());
        a
    }
    fn slot_descriptor() -> ParamSlotDescriptor {
        ParamSlotDescriptor::leaf(
            "broadcast_reader",
            "BroadcastReader",
            std::any::type_name::<Self>(),
        )
    }
    unsafe fn extract<'world, 'state>(
        state: &'state mut Self::State,
        context: SystemParamContext<'world>,
    ) -> Result<Self::Item<'world, 'state>, SystemParamError> {
        Ok(BroadcastReader::new(context.messaging(), state))
    }
}
