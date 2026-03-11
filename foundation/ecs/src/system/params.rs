use super::extract::{SystemParam, SystemParamError};
use crate::World;
use crate::component::Component;
use crate::query::{Query, QueryAccess, QueryFilter, QuerySpec, QueryState};
use crate::telemetry;
use crate::world::Commands;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::time::Instant;

pub struct Res<T: Component> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: Component> Res<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }
}

impl<T: Component> Deref for Res<T> {
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

pub struct ResMut<T: Component> {
    world: NonNull<World>,
    _marker: PhantomData<T>,
}

impl<T: Component> ResMut<T> {
    pub(crate) fn new(world: *mut World) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            _marker: PhantomData,
        }
    }
}

impl<T: Component> Deref for ResMut<T> {
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

impl<T: Component> DerefMut for ResMut<T> {
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

impl<'w, T: Component> SystemParam<'w> for Res<T> {
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

impl<'w, T: Component> SystemParam<'w> for ResMut<T> {
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
