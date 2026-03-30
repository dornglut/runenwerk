// Owner: Grotto Quest ecs - Query Runtime
use super::access_and_filters::QueryAccess;
use super::traits_and_state::{QueryArchetypeRow, QueryData, QueryFastCache};
use crate::component::Component;
use crate::entity::Entity;
use crate::storage::ArchetypeExecutionBinding;
use crate::world::World;
use std::any::TypeId;

fn required_types_match(required_present: &[TypeId], expected: &[TypeId]) -> bool {
    required_present.len() == expected.len()
        && expected
            .iter()
            .all(|type_id| required_present.contains(type_id))
}

fn collect_rows_from_bindings(
    world: &World,
    bindings: &[ArchetypeExecutionBinding],
    rows: &mut Vec<QueryArchetypeRow>,
) {
    rows.clear();
    for binding in bindings {
        for row in 0..binding.row_count {
            if let Some(entity) = world.archetype_entity_at(binding.archetype_index, row) {
                rows.push(QueryArchetypeRow {
                    entity,
                    archetype_index: binding.archetype_index,
                    row,
                });
            }
        }
    }
    rows.sort_unstable();
}

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
        if cache.world_ptr != world_ptr {
            cache.world_ptr = world_ptr;
            cache.archetype_bindings.clear();
        }
        true
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        unsafe { (&*world).archetype_component::<T>(entity) }
    }

    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        let _ = cache;
        unsafe { Self::fetch(world, entity) }
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
        if cache.world_ptr != world_ptr {
            cache.world_ptr = world_ptr;
            cache.archetype_bindings.clear();
        }
        true
    }

    fn supports_archetype_execution() -> bool {
        true
    }

    fn collect_archetype_rows(
        world: *mut World,
        required_present: &[TypeId],
        excluded: &[TypeId],
        rows: &mut Vec<QueryArchetypeRow>,
        cache: &mut QueryFastCache,
    ) -> bool {
        if !required_types_match(required_present, &[TypeId::of::<T>()]) || !excluded.is_empty() {
            return false;
        }

        let world_ref = unsafe { &*world };
        if !world_ref.matching_archetype_bindings_into(
            required_present,
            excluded,
            &mut cache.archetype_bindings,
        ) {
            return false;
        }

        collect_rows_from_bindings(world_ref, &cache.archetype_bindings, rows);
        true
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
        unsafe { (&mut *world).archetype_component_mut_untracked::<T>(entity) }
    }

    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        let _ = cache;
        unsafe { Self::fetch(world, entity) }
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
        unsafe { (&*world).archetype_component::<T>(entity) }.map(|value| (entity, value))
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
        unsafe { (&mut *world).archetype_component_mut_untracked::<T>(entity) }
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
        let a = unsafe { (&*world).archetype_component::<A>(entity) }?;
        let b = unsafe { (&*world).archetype_component::<B>(entity) }?;
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
            cache.archetype_bindings.clear();
            return false;
        }

        let world_ptr = world as *const World;
        if cache.world_ptr != world_ptr {
            cache.world_ptr = world_ptr;
            cache.archetype_bindings.clear();
        }

        true
    }

    fn supports_archetype_execution() -> bool {
        true
    }

    fn collect_archetype_rows(
        world: *mut World,
        required_present: &[TypeId],
        excluded: &[TypeId],
        rows: &mut Vec<QueryArchetypeRow>,
        cache: &mut QueryFastCache,
    ) -> bool {
        if TypeId::of::<A>() == TypeId::of::<B>()
            || !required_types_match(required_present, &[TypeId::of::<A>(), TypeId::of::<B>()])
            || !excluded.is_empty()
        {
            return false;
        }

        let world_ref = unsafe { &*world };
        if !world_ref.matching_archetype_bindings_into(
            required_present,
            excluded,
            &mut cache.archetype_bindings,
        ) {
            return false;
        }

        collect_rows_from_bindings(world_ref, &cache.archetype_bindings, rows);
        true
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
        let b = world_mut.archetype_component::<B>(entity)? as *const B;
        let a = world_mut.archetype_component_mut_untracked::<A>(entity)? as *mut A;

        // Safety: mutable/read query access requires distinct component types.
        Some(unsafe { (&mut *a, &*b) })
    }

    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        let _ = cache;
        unsafe { Self::fetch(world, entity) }
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
        let a = world_mut.archetype_component::<A>(entity)? as *const A;
        let b = world_mut.archetype_component_mut_untracked::<B>(entity)? as *mut B;

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
            cache.archetype_bindings.clear();
            return false;
        }

        let world_ptr = world as *const World;
        if cache.world_ptr != world_ptr {
            cache.world_ptr = world_ptr;
            cache.archetype_bindings.clear();
        }

        true
    }

    fn supports_archetype_execution() -> bool {
        true
    }

    fn collect_archetype_rows(
        world: *mut World,
        required_present: &[TypeId],
        excluded: &[TypeId],
        rows: &mut Vec<QueryArchetypeRow>,
        cache: &mut QueryFastCache,
    ) -> bool {
        if TypeId::of::<A>() == TypeId::of::<B>()
            || !required_types_match(required_present, &[TypeId::of::<A>(), TypeId::of::<B>()])
            || !excluded.is_empty()
        {
            return false;
        }

        let world_ref = unsafe { &*world };
        if !world_ref.matching_archetype_bindings_into(
            required_present,
            excluded,
            &mut cache.archetype_bindings,
        ) {
            return false;
        }

        collect_rows_from_bindings(world_ref, &cache.archetype_bindings, rows);
        true
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
        let a = world_mut.archetype_component_mut_untracked::<A>(entity)? as *mut A;
        let b = world_mut.archetype_component_mut_untracked::<B>(entity)? as *mut B;

        // Safety: double mutable query access requires distinct component types.
        Some(unsafe { (&mut *a, &mut *b) })
    }

    unsafe fn fetch_fast<'w>(
        world: *mut World,
        entity: Entity,
        cache: &mut QueryFastCache,
    ) -> Option<Self::Item<'w>> {
        let _ = cache;
        unsafe { Self::fetch(world, entity) }
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
        Some(unsafe { (&*world).archetype_component::<T>(entity) })
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
        let should_mark = unsafe { (&*world).archetype_component::<T>(entity) };
        if should_mark.is_some() {
            // Safety: query execution ensures exclusive mutable world access for this query form.
            let _ = unsafe { (&mut *world).get_mut::<T>(entity) };
        }
    }

    unsafe fn fetch<'w>(world: *mut World, entity: Entity) -> Option<Self::Item<'w>> {
        let value = unsafe { (&mut *world).archetype_component_mut_untracked::<T>(entity) };
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
            .archetype_component::<B>(entity)
            .map(|value| value as *const B);
        let a = world_mut.archetype_component_mut_untracked::<A>(entity)? as *mut A;

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
        let a = world_ref.archetype_component::<A>(entity)?;
        let b = world_ref.archetype_component::<B>(entity);
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
        let should_mark = unsafe { (&*world).archetype_component::<B>(entity) };
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
        let a = world_mut.archetype_component::<A>(entity)? as *const A;
        let b = world_mut
            .archetype_component_mut_untracked::<B>(entity)
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
            .archetype_component_mut_untracked::<B>(entity)
            .map(|value| value as *mut B);
        let a = world_mut.archetype_component_mut_untracked::<A>(entity)? as *mut A;

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
        let value = unsafe { (&*world).archetype_component::<T>(entity) };
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
        let a = unsafe { (&*world).archetype_component::<A>(entity) }?;
        let b = unsafe { (&*world).archetype_component::<B>(entity) }?;
        let c = unsafe { (&*world).archetype_component::<C>(entity) }?;
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
        let b = world_mut.archetype_component::<B>(entity)? as *const B;
        let c = world_mut.archetype_component::<C>(entity)? as *const C;
        let a = world_mut.archetype_component_mut_untracked::<A>(entity)? as *mut A;

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
        let c = world_mut.archetype_component::<C>(entity)? as *const C;
        let a = world_mut.archetype_component_mut_untracked::<A>(entity)? as *mut A;
        let b = world_mut.archetype_component_mut_untracked::<B>(entity)? as *mut B;

        // Safety: mutable/read tuple query access requires distinct component types.
        Some(unsafe { (&mut *a, &mut *b, &*c) })
    }

    unsafe fn world_ref<'w>(world: *mut World) -> Self::WorldRef<'w> {
        unsafe { &mut *world }
    }
}
