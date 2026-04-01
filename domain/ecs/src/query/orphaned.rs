// Owner: Grotto Quest ecs - Removed Component Query
use super::access_and_filters::QueryAccess;
use crate::component::Component;
use crate::entity::Entity;
use crate::world::World;
use std::any::TypeId;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Orphaned<T: Component> {
    entity: Entity,
    tick: u64,
    _marker: PhantomData<T>,
}

impl<T: Component> Orphaned<T> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }
}

pub struct QueryOrphanedState<T: Component> {
    access: QueryAccess,
    scratch: RefCell<Vec<(Entity, u64)>>,
    _marker: PhantomData<T>,
}

impl<T: Component> QueryOrphanedState<T> {
    pub fn new(_world: &World) -> Self {
        let mut access = QueryAccess::default();
        access.add_orphaned_component_read::<T>();
        Self {
            access,
            scratch: RefCell::new(Vec::new()),
            _marker: PhantomData,
        }
    }

    pub fn access(&self) -> &QueryAccess {
        &self.access
    }

    pub fn iter<'w>(&self, world: &'w World) -> impl Iterator<Item = Orphaned<T>> + 'w {
        let mut scratch = self.scratch.borrow_mut();
        world.removed_component_records_current_window(TypeId::of::<T>(), &mut scratch);
        let records = scratch
            .iter()
            .map(|(entity, tick)| Orphaned {
                entity: *entity,
                tick: *tick,
                _marker: PhantomData,
            })
            .collect::<Vec<_>>();
        records.into_iter()
    }
}

pub struct QueryOrphaned<T: Component> {
    world: NonNull<World>,
    state: NonNull<QueryOrphanedState<T>>,
    _marker: PhantomData<T>,
}

impl<T: Component> QueryOrphaned<T> {
    pub(crate) fn new(world: *mut World, state: &mut QueryOrphanedState<T>) -> Self {
        Self {
            world: NonNull::new(world).expect("world pointer must not be null"),
            state: NonNull::from(state),
            _marker: PhantomData,
        }
    }

    pub fn access(&self) -> &QueryAccess {
        unsafe { self.state.as_ref().access() }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = Orphaned<T>> + '_ {
        let world = unsafe { self.world.as_ref() };
        unsafe { self.state.as_ref().iter(world) }
    }
}
