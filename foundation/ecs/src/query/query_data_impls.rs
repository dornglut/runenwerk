// Owner: Grotto Quest ECS - Query Runtime
use super::access_and_filters::QueryAccess;
use super::store_access::StoreAccess;
use super::traits_and_state::{QueryData, QueryFastCache};
use crate::component::Component;
use crate::entity::Entity;
use crate::world::{TypedStore, World};
use std::any::TypeId;
use std::ptr::null_mut;

impl<T: Component> QueryData for &T {
    type Item<'w> = &'w T;
    type WorldRef<'w> = &'w World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }

    fn supports_fast_path() -> bool {
        true
    }

    fn prepare_fast_cache(world: *mut World, cache: &mut QueryFastCache) -> bool {
        let world_ptr = world as *const World;
        if cache.world_ptr != world_ptr || cache.store0.is_null() {
            cache.world_ptr = world_ptr;
            cache.store0 = unsafe { (&*world).store::<T>() }
                .map(|store| store as *const TypedStore<T> as *mut ())
                .unwrap_or(null_mut());
            cache.store1 = null_mut();
        }
        !cache.store0.is_null()
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        unsafe { (&*world).store::<T>() }.and_then(|store| store.get(entity))
    }

    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        if cache.store0.is_null() {
            return unsafe { Self::fetch(world, entity) };
        }
        let store = unsafe { &*(cache.store0 as *const TypedStore<T>) };
        store.get(entity)
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &*world }
    }
}

impl<T: Component> QueryData for &mut T {
    type Item<'w> = &'w mut T;
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<T>();
    }

    fn supports_fast_path() -> bool {
        true
    }

    fn prepare_fast_cache(world: *mut World, cache: &mut QueryFastCache) -> bool {
        let world_ptr = world as *const World;
        if cache.world_ptr != world_ptr || cache.store0.is_null() {
            cache.world_ptr = world_ptr;
            cache.store0 = unsafe { (&mut *world).store_mut::<T>() }
                .map(|store| store as *mut TypedStore<T> as *mut ())
                .unwrap_or(null_mut());
            cache.store1 = null_mut();
        }
        !cache.store0.is_null()
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        // Safety: query execution ensures exclusive mutable world access for this query form.
        let _ = unsafe { (&mut *world).get_mut::<T>(entity) };
    }

    fn mark_changed_fast(world: *mut World, entity: Entity, _cache: &mut QueryFastCache) {
        let world_mut = unsafe { &mut *world };
        world_mut.mark_component_modified_by_id(entity, TypeId::of::<T>(), T::component_name());
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        unsafe { (&mut *world).store_mut::<T>() }.and_then(|store| store.get_mut(entity))
    }

    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        if cache.store0.is_null() {
            return unsafe { Self::fetch(world, entity) };
        }
        let store = unsafe { &mut *(cache.store0 as *mut TypedStore<T>) };
        store.get_mut(entity)
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<T: Component> QueryData for (Entity, &T) {
    type Item<'w> = (Entity, &'w T);
    type WorldRef<'w> = &'w World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        unsafe { (&*world).store::<T>() }
            .and_then(|store| store.get(entity))
            .map(|value| (entity, value))
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &*world }
    }
}

impl<T: Component> QueryData for (Entity, &mut T) {
    type Item<'w> = (Entity, &'w mut T);
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<T>();
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        // Safety: query execution ensures exclusive mutable world access for this query form.
        let _ = unsafe { (&mut *world).get_mut::<T>(entity) };
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        unsafe { (&mut *world).store_mut::<T>() }
            .and_then(|store| store.get_mut(entity))
            .map(|value| (entity, value))
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<A: Component, B: Component> QueryData for (&A, &B) {
    type Item<'w> = (&'w A, &'w B);
    type WorldRef<'w> = &'w World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<A>();
        access.add_component_read::<B>();
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        let a = unsafe { (&*world).store::<A>() }.and_then(|store| store.get(entity))?;
        let b = unsafe { (&*world).store::<B>() }.and_then(|store| store.get(entity))?;
        Some((a, b))
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &*world }
    }
}

impl<A: Component, B: Component> QueryData for (&mut A, &B) {
    type Item<'w> = (&'w mut A, &'w B);
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<A>();
        access.add_component_read::<B>();
    }

    fn supports_fast_path() -> bool {
        true
    }

    fn prepare_fast_cache(world: *mut World, cache: &mut QueryFastCache) -> bool {
        if TypeId::of::<A>() == TypeId::of::<B>() {
            cache.store0 = null_mut();
            cache.store1 = null_mut();
            return false;
        }

        let world_ptr = world as *const World;
        if cache.world_ptr != world_ptr || cache.store0.is_null() || cache.store1.is_null() {
            cache.world_ptr = world_ptr;
            cache.store0 = unsafe { (&mut *world).store_mut::<A>() }
                .map(|store| store as *mut TypedStore<A> as *mut ())
                .unwrap_or(null_mut());
            cache.store1 = unsafe { (&*world).store::<B>() }
                .map(|store| store as *const TypedStore<B> as *mut ())
                .unwrap_or(null_mut());
        }

        !cache.store0.is_null() && !cache.store1.is_null()
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        // Safety: query execution ensures exclusive mutable world access for this query form.
        let _ = unsafe { (&mut *world).get_mut::<A>(entity) };
    }

    fn mark_changed_fast(world: *mut World, entity: Entity, _cache: &mut QueryFastCache) {
        let world_mut = unsafe { &mut *world };
        world_mut.mark_component_modified_by_id(entity, TypeId::of::<A>(), A::component_name());
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
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

        // Safety: mutable/read query access requires distinct component types.
        Some(unsafe { (&mut *a, &*b) })
    }

    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        if cache.store0.is_null() || cache.store1.is_null() {
            return unsafe { Self::fetch(world, entity) };
        }

        let store_b = unsafe { &*(cache.store1 as *const TypedStore<B>) };
        let b = store_b.get(entity)? as *const B;
        let store_a = unsafe { &mut *(cache.store0 as *mut TypedStore<A>) };
        let a = store_a.get_mut(entity)? as *mut A;

        // Safety: the fast path is only enabled when A and B are distinct component types.
        Some(unsafe { (&mut *a, &*b) })
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<A: Component, B: Component> QueryData for (&A, &mut B) {
    type Item<'w> = (&'w A, &'w mut B);
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<A>();
        access.add_component_write::<B>();
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        // Safety: query execution ensures exclusive mutable world access for this query form.
        let _ = unsafe { (&mut *world).get_mut::<B>(entity) };
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "read/mutable query requires distinct component types",
        );

        let world_mut = unsafe { &mut *world };
        let a = world_mut.store::<A>().and_then(|store| store.get(entity))? as *const A;
        let b = world_mut
            .store_mut::<B>()
            .and_then(|store| store.get_mut(entity))? as *mut B;

        // Safety: read/mutable query access requires distinct component types.
        Some(unsafe { (&*a, &mut *b) })
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<A: Component, B: Component> QueryData for (&mut A, &mut B) {
    type Item<'w> = (&'w mut A, &'w mut B);
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<A>();
        access.add_component_write::<B>();
    }

    fn supports_fast_path() -> bool {
        true
    }

    fn prepare_fast_cache(world: *mut World, cache: &mut QueryFastCache) -> bool {
        if TypeId::of::<A>() == TypeId::of::<B>() {
            cache.store0 = null_mut();
            cache.store1 = null_mut();
            return false;
        }

        let world_ptr = world as *const World;
        if cache.world_ptr != world_ptr || cache.store0.is_null() || cache.store1.is_null() {
            cache.world_ptr = world_ptr;
            cache.store0 = unsafe { (&mut *world).store_mut::<A>() }
                .map(|store| store as *mut TypedStore<A> as *mut ())
                .unwrap_or(null_mut());
            cache.store1 = unsafe { (&mut *world).store_mut::<B>() }
                .map(|store| store as *mut TypedStore<B> as *mut ())
                .unwrap_or(null_mut());
        }

        !cache.store0.is_null() && !cache.store1.is_null()
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        // Safety: query execution ensures exclusive mutable world access for this query form.
        let world_mut = unsafe { &mut *world };
        let _ = world_mut.get_mut::<A>(entity);
        let _ = world_mut.get_mut::<B>(entity);
    }

    fn mark_changed_fast(world: *mut World, entity: Entity, _cache: &mut QueryFastCache) {
        let world_mut = unsafe { &mut *world };
        world_mut.mark_component_modified_by_id(entity, TypeId::of::<A>(), A::component_name());
        world_mut.mark_component_modified_by_id(entity, TypeId::of::<B>(), B::component_name());
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "double mutable query requires distinct component types",
        );

        let world_mut = unsafe { &mut *world };
        let a = world_mut
            .store_mut::<A>()
            .and_then(|store| store.get_mut(entity))? as *mut A;
        let b = world_mut
            .store_mut::<B>()
            .and_then(|store| store.get_mut(entity))? as *mut B;

        // Safety: double mutable query access requires distinct component types.
        Some(unsafe { (&mut *a, &mut *b) })
    }

    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        if cache.store0.is_null() || cache.store1.is_null() {
            return unsafe { Self::fetch(world, entity) };
        }

        let store_a = unsafe { &mut *(cache.store0 as *mut TypedStore<A>) };
        let a = store_a.get_mut(entity)? as *mut A;
        let store_b = unsafe { &mut *(cache.store1 as *mut TypedStore<B>) };
        let b = store_b.get_mut(entity)? as *mut B;

        // Safety: the fast path is only enabled when A and B are distinct component types.
        Some(unsafe { (&mut *a, &mut *b) })
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<T: Component> QueryData for Option<&T> {
    type Item<'w> = Option<&'w T>;
    type WorldRef<'w> = &'w World;

    fn query_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        Some(unsafe { (&*world).store::<T>() }.and_then(|store| store.get(entity)))
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &*world }
    }
}

impl<T: Component> QueryData for Option<&mut T> {
    type Item<'w> = Option<&'w mut T>;
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<T>();
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        let should_mark = unsafe { (&*world).store::<T>() }.and_then(|store| store.get(entity));
        if should_mark.is_some() {
            // Safety: query execution ensures exclusive mutable world access for this query form.
            let _ = unsafe { (&mut *world).get_mut::<T>(entity) };
        }
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        let value =
            unsafe { (&mut *world).store_mut::<T>() }.and_then(|store| store.get_mut(entity));
        Some(value)
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<A: Component, B: Component> QueryData for (&mut A, Option<&B>) {
    type Item<'w> = (&'w mut A, Option<&'w B>);
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<A>();
        access.add_component_read::<B>();
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        // Safety: query execution ensures exclusive mutable world access for this query form.
        let _ = unsafe { (&mut *world).get_mut::<A>(entity) };
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "mutable/optional query requires distinct component types",
        );

        let world_mut = unsafe { &mut *world };
        let b = world_mut
            .store::<B>()
            .and_then(|store| store.get(entity))
            .map(|value| value as *const B);
        let a = world_mut
            .store_mut::<A>()
            .and_then(|store| store.get_mut(entity))? as *mut A;

        // Safety: mutable/optional query access requires distinct component types.
        Some(unsafe { (&mut *a, b.map(|ptr| &*ptr)) })
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<A: Component, B: Component> QueryData for (&A, Option<&B>) {
    type Item<'w> = (&'w A, Option<&'w B>);
    type WorldRef<'w> = &'w World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<A>();
        access.add_component_read::<B>();
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        let world_ref = unsafe { &*world };
        let a = world_ref.store::<A>().and_then(|store| store.get(entity))?;
        let b = world_ref.store::<B>().and_then(|store| store.get(entity));
        Some((a, b))
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &*world }
    }
}

impl<A: Component, B: Component> QueryData for (&A, Option<&mut B>) {
    type Item<'w> = (&'w A, Option<&'w mut B>);
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<A>();
        access.add_component_write::<B>();
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        let should_mark = unsafe { (&*world).store::<B>() }.and_then(|store| store.get(entity));
        if should_mark.is_some() {
            // Safety: query execution ensures exclusive mutable world access for this query form.
            let _ = unsafe { (&mut *world).get_mut::<B>(entity) };
        }
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "read/optional mutable query requires distinct component types",
        );

        let world_mut = unsafe { &mut *world };
        let a = world_mut.store::<A>().and_then(|store| store.get(entity))? as *const A;
        let b = world_mut
            .store_mut::<B>()
            .and_then(|store| store.get_mut(entity))
            .map(|value| value as *mut B);

        // Safety: read/optional mutable query access requires distinct component types.
        Some(unsafe { (&*a, b.map(|ptr| &mut *ptr)) })
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<A: Component, B: Component> QueryData for (&mut A, Option<&mut B>) {
    type Item<'w> = (&'w mut A, Option<&'w mut B>);
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<A>();
        access.add_component_write::<B>();
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        // Safety: query execution ensures exclusive mutable world access for this query form.
        let world_mut = unsafe { &mut *world };
        let _ = world_mut.get_mut::<A>(entity);
        if world_mut.get::<B>(entity).is_some() {
            let _ = world_mut.get_mut::<B>(entity);
        }
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "mutable/optional mutable query requires distinct component types",
        );

        let world_mut = unsafe { &mut *world };
        let b = world_mut
            .store_mut::<B>()
            .and_then(|store| store.get_mut(entity))
            .map(|value| value as *mut B);
        let a = world_mut
            .store_mut::<A>()
            .and_then(|store| store.get_mut(entity))? as *mut A;

        // Safety: mutable/optional mutable query access requires distinct component types.
        Some(unsafe { (&mut *a, b.map(|ptr| &mut *ptr)) })
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<T: Component> QueryData for (Entity, Option<&T>) {
    type Item<'w> = (Entity, Option<&'w T>);
    type WorldRef<'w> = &'w World;

    fn query_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<T>();
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        let value = unsafe { (&*world).store::<T>() }.and_then(|store| store.get(entity));
        Some((entity, value))
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &*world }
    }
}

impl<A: Component, B: Component, C: Component> QueryData for (&A, &B, &C) {
    type Item<'w> = (&'w A, &'w B, &'w C);
    type WorldRef<'w> = &'w World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>(), TypeId::of::<C>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_read::<A>();
        access.add_component_read::<B>();
        access.add_component_read::<C>();
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        let a = unsafe { (&*world).store::<A>() }.and_then(|store| store.get(entity))?;
        let b = unsafe { (&*world).store::<B>() }.and_then(|store| store.get(entity))?;
        let c = unsafe { (&*world).store::<C>() }.and_then(|store| store.get(entity))?;
        Some((a, b, c))
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &*world }
    }
}

impl<A: Component, B: Component, C: Component> QueryData for (&mut A, &B, &C) {
    type Item<'w> = (&'w mut A, &'w B, &'w C);
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>(), TypeId::of::<C>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<A>();
        access.add_component_read::<B>();
        access.add_component_read::<C>();
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        // Safety: query execution ensures exclusive mutable world access for this query form.
        let _ = unsafe { (&mut *world).get_mut::<A>(entity) };
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "mutable/read tuple query requires distinct component types",
        );
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<C>(),
            "mutable/read tuple query requires distinct component types",
        );

        let world_mut = unsafe { &mut *world };
        let b = world_mut.store::<B>().and_then(|store| store.get(entity))? as *const B;
        let c = world_mut.store::<C>().and_then(|store| store.get(entity))? as *const C;
        let a = world_mut
            .store_mut::<A>()
            .and_then(|store| store.get_mut(entity))? as *mut A;

        // Safety: mutable/read tuple query access requires distinct component types.
        Some(unsafe { (&mut *a, &*b, &*c) })
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}

impl<A: Component, B: Component, C: Component> QueryData for (&mut A, &mut B, &C) {
    type Item<'w> = (&'w mut A, &'w mut B, &'w C);
    type WorldRef<'w> = &'w mut World;

    fn query_types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>(), TypeId::of::<C>()]
    }

    fn append_access(access: &mut QueryAccess) {
        access.add_component_write::<A>();
        access.add_component_write::<B>();
        access.add_component_read::<C>();
    }

    fn mark_changed(world: *mut World, entity: Entity) {
        // Safety: query execution ensures exclusive mutable world access for this query form.
        let world_mut = unsafe { &mut *world };
        let _ = world_mut.get_mut::<A>(entity);
        let _ = world_mut.get_mut::<B>(entity);
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "mutable tuple query requires distinct component types",
        );
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<C>(),
            "mutable/read tuple query requires distinct component types",
        );
        assert_ne!(
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            "mutable/read tuple query requires distinct component types",
        );

        let world_mut = unsafe { &mut *world };
        let c = world_mut.store::<C>().and_then(|store| store.get(entity))? as *const C;
        let a = world_mut
            .store_mut::<A>()
            .and_then(|store| store.get_mut(entity))? as *mut A;
        let b = world_mut
            .store_mut::<B>()
            .and_then(|store| store.get_mut(entity))? as *mut B;

        // Safety: mutable/read tuple query access requires distinct component types.
        Some(unsafe { (&mut *a, &mut *b, &*c) })
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}
