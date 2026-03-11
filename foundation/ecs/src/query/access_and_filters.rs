// Owner: Grotto Quest ECS - Query Runtime
use crate::component::Component;
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
    component_writes: Vec<QueryTypeAccess>,
    resource_reads: Vec<QueryTypeAccess>,
    resource_writes: Vec<QueryTypeAccess>,
    deferred_structural_mutation: bool,
}

impl QueryAccess {
    pub fn component_reads(&self) -> &[QueryTypeAccess] {
        &self.component_reads
    }

    pub fn component_writes(&self) -> &[QueryTypeAccess] {
        &self.component_writes
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

    pub(super) fn add_component_read<T: Component>(&mut self) {
        push_unique_access(
            &mut self.component_reads,
            QueryTypeAccess::of::<T>(T::component_name()),
        );
    }

    pub(super) fn add_component_write<T: Component>(&mut self) {
        push_unique_access(
            &mut self.component_writes,
            QueryTypeAccess::of::<T>(T::component_name()),
        );
    }
}

pub trait QueryFilter {
    fn configure(required: &mut Vec<TypeId>, excluded: &mut Vec<TypeId>);
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

impl<A: QueryFilter, B: QueryFilter> QueryFilter for (A, B) {
    fn configure(required: &mut Vec<TypeId>, excluded: &mut Vec<TypeId>) {
        A::configure(required, excluded);
        B::configure(required, excluded);
    }
}

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
