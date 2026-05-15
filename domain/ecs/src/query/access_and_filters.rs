// Owner: Grotto Quest ecs - Query Runtime
use crate::component::{Component, Resource};
use crate::entity::Entity;
use crate::world::World;
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
    orphaned_component_reads: Vec<QueryTypeAccess>,
    component_writes: Vec<QueryTypeAccess>,
    resource_reads: Vec<QueryTypeAccess>,
    resource_writes: Vec<QueryTypeAccess>,
    broadcast_reads: Vec<QueryTypeAccess>,
    broadcast_writes: Vec<QueryTypeAccess>,
    work_queue_reads: Vec<QueryTypeAccess>,
    work_queue_writes: Vec<QueryTypeAccess>,
    work_queue_drains: Vec<QueryTypeAccess>,
    tick_buffer_reads: Vec<QueryTypeAccess>,
    tick_buffer_writes: Vec<QueryTypeAccess>,
    tick_buffer_drains: Vec<QueryTypeAccess>,
    deferred_structural_mutation: bool,
}

impl QueryAccess {
    pub fn structural_mutation() -> Self {
        Self {
            deferred_structural_mutation: true,
            ..Self::default()
        }
    }

    pub fn component_reads(&self) -> &[QueryTypeAccess] {
        &self.component_reads
    }

    pub fn component_writes(&self) -> &[QueryTypeAccess] {
        &self.component_writes
    }

    pub fn orphaned_component_reads(&self) -> &[QueryTypeAccess] {
        &self.orphaned_component_reads
    }

    pub fn resource_reads(&self) -> &[QueryTypeAccess] {
        &self.resource_reads
    }

    pub fn resource_writes(&self) -> &[QueryTypeAccess] {
        &self.resource_writes
    }

    pub fn with_component_read<T: Component>(mut self) -> Self {
        self.add_component_read::<T>();
        self
    }

    pub fn with_component_write<T: Component>(mut self) -> Self {
        self.add_component_write::<T>();
        self
    }

    pub fn with_orphaned_component_read<T: Component>(mut self) -> Self {
        self.add_orphaned_component_read::<T>();
        self
    }

    pub fn with_resource_read<T: Resource>(mut self) -> Self {
        self.add_resource_read::<T>();
        self
    }

    pub fn with_resource_write<T: Resource>(mut self) -> Self {
        self.add_resource_write::<T>();
        self
    }

    pub fn deferred_structural_mutation(&self) -> bool {
        self.deferred_structural_mutation
    }

    pub fn broadcast_reads(&self) -> &[QueryTypeAccess] {
        &self.broadcast_reads
    }

    pub fn broadcast_writes(&self) -> &[QueryTypeAccess] {
        &self.broadcast_writes
    }

    pub fn work_queue_reads(&self) -> &[QueryTypeAccess] {
        &self.work_queue_reads
    }

    pub fn work_queue_writes(&self) -> &[QueryTypeAccess] {
        &self.work_queue_writes
    }

    pub fn work_queue_drains(&self) -> &[QueryTypeAccess] {
        &self.work_queue_drains
    }

    pub fn tick_buffer_reads(&self) -> &[QueryTypeAccess] {
        &self.tick_buffer_reads
    }

    pub fn tick_buffer_writes(&self) -> &[QueryTypeAccess] {
        &self.tick_buffer_writes
    }

    pub fn tick_buffer_drains(&self) -> &[QueryTypeAccess] {
        &self.tick_buffer_drains
    }

    pub(crate) fn add_component_read<T: Component>(&mut self) {
        push_unique_access(
            &mut self.component_reads,
            QueryTypeAccess::of::<T>(T::component_name()),
        );
    }

    pub(crate) fn add_component_write<T: Component>(&mut self) {
        push_unique_access(
            &mut self.component_writes,
            QueryTypeAccess::of::<T>(T::component_name()),
        );
    }

    pub(crate) fn add_orphaned_component_read<T: Component>(&mut self) {
        push_unique_access(
            &mut self.orphaned_component_reads,
            QueryTypeAccess::of::<T>(T::component_name()),
        );
    }

    pub(crate) fn add_resource_read<T: Resource>(&mut self) {
        push_unique_access(
            &mut self.resource_reads,
            QueryTypeAccess::of::<T>(T::resource_name()),
        );
    }

    pub(crate) fn add_resource_write<T: Resource>(&mut self) {
        push_unique_access(
            &mut self.resource_writes,
            QueryTypeAccess::of::<T>(T::resource_name()),
        );
    }

    pub(crate) fn add_broadcast_read_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.broadcast_reads, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn add_broadcast_write_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.broadcast_writes, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn add_work_queue_read_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.work_queue_reads, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn add_work_queue_write_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.work_queue_writes, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn add_work_queue_drain_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.work_queue_drains, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn add_tick_buffer_read_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.tick_buffer_reads, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn add_tick_buffer_write_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.tick_buffer_writes, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn add_tick_buffer_drain_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.tick_buffer_drains, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn set_deferred_structural_mutation(&mut self) {
        self.deferred_structural_mutation = true;
    }

    /// Extends this access set with another access set.
    ///
    /// Composite `SystemParam` implementations use this as the canonical way to
    /// preserve child access semantics while reporting one grouped parameter.
    pub fn extend(&mut self, other: Self) {
        for access in other.component_reads {
            push_unique_access(&mut self.component_reads, access);
        }
        for access in other.orphaned_component_reads {
            push_unique_access(&mut self.orphaned_component_reads, access);
        }
        for access in other.component_writes {
            push_unique_access(&mut self.component_writes, access);
        }
        for access in other.resource_reads {
            push_unique_access(&mut self.resource_reads, access);
        }
        for access in other.resource_writes {
            push_unique_access(&mut self.resource_writes, access);
        }
        for access in other.broadcast_reads {
            push_unique_access(&mut self.broadcast_reads, access);
        }
        for access in other.broadcast_writes {
            push_unique_access(&mut self.broadcast_writes, access);
        }
        for access in other.work_queue_reads {
            push_unique_access(&mut self.work_queue_reads, access);
        }
        for access in other.work_queue_writes {
            push_unique_access(&mut self.work_queue_writes, access);
        }
        for access in other.work_queue_drains {
            push_unique_access(&mut self.work_queue_drains, access);
        }
        for access in other.tick_buffer_reads {
            push_unique_access(&mut self.tick_buffer_reads, access);
        }
        for access in other.tick_buffer_writes {
            push_unique_access(&mut self.tick_buffer_writes, access);
        }
        for access in other.tick_buffer_drains {
            push_unique_access(&mut self.tick_buffer_drains, access);
        }
        self.deferred_structural_mutation |= other.deferred_structural_mutation;
    }
}

pub trait QueryFilter {
    fn configure(required: &mut Vec<TypeId>, excluded: &mut Vec<TypeId>);

    fn append_access(_access: &mut QueryAccess) {}

    fn needs_tick_filter() -> bool {
        false
    }

    fn matches_entity(_world: &World, _entity: Entity, _since_tick: u64) -> bool {
        true
    }
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

pub struct Changed<T: Component>(PhantomData<T>);
pub struct Added<T: Component>(PhantomData<T>);

impl<T: Component> QueryFilter for Changed<T> {
    fn configure(required: &mut Vec<TypeId>, _excluded: &mut Vec<TypeId>) {
        push_unique_type(required, TypeId::of::<T>());
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }

    fn needs_tick_filter() -> bool {
        true
    }

    fn matches_entity(world: &World, entity: Entity, since_tick: u64) -> bool {
        world.component_changed_for_entity_since::<T>(entity, since_tick)
    }
}

impl<T: Component> QueryFilter for Added<T> {
    fn configure(required: &mut Vec<TypeId>, _excluded: &mut Vec<TypeId>) {
        push_unique_type(required, TypeId::of::<T>());
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }

    fn needs_tick_filter() -> bool {
        true
    }

    fn matches_entity(world: &World, entity: Entity, since_tick: u64) -> bool {
        world.component_added_for_entity_since::<T>(entity, since_tick)
    }
}

impl<A: QueryFilter, B: QueryFilter> QueryFilter for (A, B) {
    fn configure(required: &mut Vec<TypeId>, excluded: &mut Vec<TypeId>) {
        A::configure(required, excluded);
        B::configure(required, excluded);
    }

    fn append_access(access: &mut QueryAccess) {
        A::append_access(access);
        B::append_access(access);
    }

    fn needs_tick_filter() -> bool {
        A::needs_tick_filter() || B::needs_tick_filter()
    }

    fn matches_entity(world: &World, entity: Entity, since_tick: u64) -> bool {
        A::matches_entity(world, entity, since_tick) && B::matches_entity(world, entity, since_tick)
    }
}

macro_rules! impl_query_filter_tuple {
    ($(($($name:ident),+)),+ $(,)?) => {
        $(
            impl<$($name: QueryFilter,)+> QueryFilter for ($($name,)+) {
                fn configure(required: &mut Vec<TypeId>, excluded: &mut Vec<TypeId>) {
                    $(
                        $name::configure(required, excluded);
                    )+
                }

                fn append_access(access: &mut QueryAccess) {
                    $(
                        $name::append_access(access);
                    )+
                }

                fn needs_tick_filter() -> bool {
                    false $(|| $name::needs_tick_filter())+
                }

                fn matches_entity(world: &World, entity: Entity, since_tick: u64) -> bool {
                    true $(
                        && $name::matches_entity(world, entity, since_tick)
                    )+
                }
            }
        )+
    };
}

impl_query_filter_tuple!((A, B, C), (A, B, C, D), (A, B, C, D, E), (A, B, C, D, E, F));

impl<T: Component> Default for With<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> With<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component> Default for Without<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> Without<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component> Default for Changed<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> Changed<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component> Default for Added<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> Added<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub(super) fn push_unique_type(target: &mut Vec<TypeId>, type_id: TypeId) {
    if !target.contains(&type_id) {
        target.push(type_id);
    }
}

fn push_unique_access(target: &mut Vec<QueryTypeAccess>, access: QueryTypeAccess) {
    if !target.iter().any(|entry| entry.type_id == access.type_id) {
        target.push(access);
    }
}
