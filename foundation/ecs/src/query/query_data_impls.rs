// Owner: Grotto Quest ECS - Query Runtime
use super::access_and_filters::QueryAccess;
use super::store_access::StoreAccess;
use super::traits_and_state::{MutableQueryData, QueryData, ReadOnlyQueryData};
use crate::component::Component;
use crate::entity::Entity;
use crate::world::World;
use std::any::TypeId;

impl<T: Component> QueryData for &T {
    type Item<'w> = &'w T;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }
}

impl<T: Component> ReadOnlyQueryData for &T {
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        world.store::<T>().and_then(|store| store.get(entity))
    }
}

impl<T: Component> QueryData for &mut T {
    type Item<'w> = &'w mut T;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<T>();
    }
}

impl<T: Component> MutableQueryData for &mut T {
    fn mark_changed(world: &mut World, entity: Entity) {
        let _ = world.get_mut::<T>(entity);
    }

    unsafe fn fetch_mut<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        unsafe { (&mut *world).store_mut::<T>() }.and_then(|store| store.get_mut(entity))
    }
}

impl<T: Component> QueryData for (Entity, &T) {
    type Item<'w> = (Entity, &'w T);

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }
}

impl<T: Component> ReadOnlyQueryData for (Entity, &T) {
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        world
            .store::<T>()
            .and_then(|store| store.get(entity).map(|value| (entity, value)))
    }
}

impl<A: Component, B: Component> QueryData for (&A, &B) {
    type Item<'w> = (&'w A, &'w B);

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<A>();
        access.add_component_read::<B>();
    }
}

impl<A: Component, B: Component> ReadOnlyQueryData for (&A, &B) {
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        let a = world.store::<A>().and_then(|store| store.get(entity))?;
        let b = world.store::<B>().and_then(|store| store.get(entity))?;
        Some((a, b))
    }
}

impl<A: Component, B: Component> QueryData for (&mut A, &B) {
    type Item<'w> = (&'w mut A, &'w B);

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<A>();
        access.add_component_read::<B>();
    }
}

impl<A: Component, B: Component> MutableQueryData for (&mut A, &B) {
    fn mark_changed(world: &mut World, entity: Entity) {
        let _ = world.get_mut::<A>(entity);
    }

    unsafe fn fetch_mut<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "mutable/read query requires distinct component types",
        );

        let world_mut = unsafe { &mut *world };
        let b = world_mut.store::<B>().and_then(|store| store.get(entity))? as *const B;
        let a = world_mut
            .store_mut::<A>()
            .and_then(|store| store.get_mut(entity))? as *mut A;

        // Safety: the mutable query contract requires `A` and `B` to be distinct component
        // types, and `QueryIterMut` serializes access to the world for the duration of iteration.
        Some(unsafe { (&mut *a, &*b) })
    }
}
