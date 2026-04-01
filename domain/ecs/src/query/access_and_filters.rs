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
    deferred_structural_mutation: bool,
}

impl QueryAccess {
    pub fn structural_mutation() -> Self {
        let mut access = Self::default();
        access.deferred_structural_mutation = true;
        access
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

    pub fn deferred_structural_mutation(&self) -> bool {
        self.deferred_structural_mutation
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

    pub(crate) fn add_resource_read_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.resource_reads, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn add_resource_write_named<T: 'static>(&mut self, name: &'static str) {
        push_unique_access(&mut self.resource_writes, QueryTypeAccess::of::<T>(name));
    }

    pub(crate) fn set_deferred_structural_mutation(&mut self) {
        self.deferred_structural_mutation = true;
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

impl<T: Component> Changed<T> {
    pub fn new() -> Self {
        Self(PhantomData)
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
