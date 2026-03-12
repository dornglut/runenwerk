// Owner: ECS World - Internal Mutation and Change Tracking
use super::events_and_indexes::{
    ComponentChangeKind, ComponentChangeRecord, ComponentMeta, EventObserverNotification,
    ObserverTrigger, ResourceChangeKind, ResourceChangeRecord,
};
use super::world_struct::World;
use crate::component::Component;
use crate::entity::Entity;
use crate::errors::EntityError;
use crate::storage::ArchetypeExecutionBinding;
use crate::telemetry;
use std::any::{TypeId, type_name};
use std::time::Instant;

impl World {
    #[doc(hidden)]
    pub fn __register_component<T: Component>(&mut self) {
        self.ensure_component_registered::<T>();
    }

    #[doc(hidden)]
    pub fn __insert_component<T: Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), EntityError> {
        self.ensure_entity_exists(entity)?;
        self.ensure_component_registered::<T>();
        let kind = if self.contains_component::<T>(entity) {
            ComponentChangeKind::Modified
        } else {
            ComponentChangeKind::Added
        };
        let component_type = TypeId::of::<T>();
        self.record_component_change(entity, component_type, T::component_name(), kind);
        let tick = self.change_tick;
        let inserted = if matches!(kind, ComponentChangeKind::Added) {
            self.archetype_registry.add_component::<T>(
                entity,
                component,
                tick,
                &mut self.entity_locations,
            )
        } else {
            self.archetype_registry.update_component::<T>(
                entity,
                component,
                tick,
                &self.entity_locations,
            )
        };
        assert!(
            inserted,
            "archetype component insert/update must succeed for live entity"
        );
        Ok(())
    }

    #[doc(hidden)]
    pub fn __remove_component<T: Component>(&mut self, entity: Entity) -> Result<T, EntityError> {
        self.ensure_entity_exists(entity)?;
        let value = self
            .archetype_registry
            .remove_component::<T>(entity, &mut self.entity_locations)
            .ok_or(EntityError::MissingComponent {
                entity,
                component: type_name::<T>(),
            })?;
        self.record_component_change(
            entity,
            TypeId::of::<T>(),
            T::component_name(),
            ComponentChangeKind::Removed,
        );
        Ok(value)
    }

    pub(crate) fn matching_entities_into(
        &self,
        required_present: &[TypeId],
        excluded: &[TypeId],
        out: &mut Vec<Entity>,
    ) {
        let start = Instant::now();
        let _ = self
            .archetype_registry
            .collect_matching_entities(required_present, excluded, out);
        let count = out.len() as u64;
        telemetry::record_query_matching(start.elapsed().as_nanos() as u64, count, count);
    }

    pub(crate) fn matching_archetype_bindings_into(
        &self,
        required_present: &[TypeId],
        excluded: &[TypeId],
        out: &mut Vec<ArchetypeExecutionBinding>,
    ) -> bool {
        self.archetype_registry
            .collect_matching_bindings(required_present, excluded, out)
    }

    pub(crate) fn archetype_entity_at(&self, archetype_index: usize, row: usize) -> Option<Entity> {
        self.archetype_registry.entity_at(archetype_index, row)
    }

    pub(super) fn place_entity_in_empty_archetype(&mut self, entity: Entity) {
        self.archetype_registry
            .set_entity_components(entity, &[], &mut self.entity_locations);
    }

    pub(super) fn remove_entity_from_archetype_tracking(&mut self, entity: Entity) {
        let _ = self
            .archetype_registry
            .remove_entity(entity, &mut self.entity_locations);
    }

    fn has_component_by_type_id(&self, entity: Entity, type_id: TypeId) -> bool {
        let Some(location) = self.entity_locations.get(entity) else {
            return false;
        };
        self.archetype_registry
            .component_types(location.archetype_id)
            .is_some_and(|component_types| component_types.binary_search(&type_id).is_ok())
    }

    pub(crate) fn entity_matches_component_constraints(
        &self,
        entity: Entity,
        required_present: &[TypeId],
        excluded: &[TypeId],
    ) -> bool {
        self.contains(entity)
            && required_present
                .iter()
                .all(|type_id| self.has_component_by_type_id(entity, *type_id))
            && excluded
                .iter()
                .all(|type_id| !self.has_component_by_type_id(entity, *type_id))
    }

    pub(super) fn contains_component<T: Component>(&self, entity: Entity) -> bool {
        self.has_component_by_type_id(entity, TypeId::of::<T>())
    }

    pub(crate) fn component_changed_for_entity_since<T: Component>(
        &self,
        entity: Entity,
        tick: u64,
    ) -> bool {
        let start = Instant::now();
        let changed = self
            .archetype_component_metadata::<T>(entity)
            .is_some_and(|(_added_tick, changed_tick)| changed_tick > tick);
        telemetry::record_changed_check(start.elapsed().as_nanos() as u64);
        changed
    }

    pub(crate) fn component_added_for_entity_since<T: Component>(
        &self,
        entity: Entity,
        tick: u64,
    ) -> bool {
        let start = Instant::now();
        let added = self
            .archetype_component_metadata::<T>(entity)
            .is_some_and(|(added_tick, _changed_tick)| added_tick > tick);
        telemetry::record_added_check(start.elapsed().as_nanos() as u64);
        added
    }

    pub(crate) fn mark_component_modified_by_id(
        &mut self,
        entity: Entity,
        component_type: TypeId,
        component_name: &'static str,
    ) {
        self.record_component_change(
            entity,
            component_type,
            component_name,
            ComponentChangeKind::Modified,
        );
        let _ = self.archetype_registry.mark_component_changed_by_id(
            entity,
            component_type,
            self.change_tick,
            &self.entity_locations,
        );
    }

    fn mark_component_type_changed_by_id(&mut self, type_id: TypeId) {
        self.change_tick = self.change_tick.saturating_add(1);
        self.component_change_ticks
            .insert(type_id, self.change_tick);
        self.mark_component_indexes_dirty(type_id);
    }

    pub(super) fn record_component_change(
        &mut self,
        entity: Entity,
        component_type: TypeId,
        component_name: &'static str,
        kind: ComponentChangeKind,
    ) {
        self.mark_component_type_changed_by_id(component_type);
        self.component_change_log.push(ComponentChangeRecord {
            tick: self.change_tick,
            entity,
            component_type,
            component_name,
            kind,
        });
    }

    pub(crate) fn archetype_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let ptr = self
            .archetype_registry
            .component_ptr::<T>(entity, &self.entity_locations)?;
        // Safety: pointers are produced by archetype-owned typed columns and remain valid.
        Some(unsafe { &*ptr })
    }

    pub(crate) fn archetype_component_mut_untracked<T: Component>(
        &mut self,
        entity: Entity,
    ) -> Option<&mut T> {
        let ptr = self
            .archetype_registry
            .component_mut_ptr::<T>(entity, &self.entity_locations)?;
        // Safety: pointers are produced by archetype-owned typed columns and remain valid.
        Some(unsafe { &mut *ptr })
    }

    pub(crate) fn archetype_component_metadata<T: Component>(
        &self,
        entity: Entity,
    ) -> Option<(u64, u64)> {
        let metadata = self
            .archetype_registry
            .component_metadata::<T>(entity, &self.entity_locations)?;
        Some((metadata.added_tick, metadata.changed_tick))
    }

    fn ensure_component_registered<T: Component>(&mut self) {
        self.archetype_registry.register_component_type::<T>();
        self.component_registry
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                let id = self.next_component_id;
                self.next_component_id = self.next_component_id.saturating_add(1);
                ComponentMeta {
                    _id: id,
                    name: T::component_name(),
                }
            });
    }

    pub(super) fn ensure_entity_exists(&self, entity: Entity) -> Result<(), EntityError> {
        if self.contains(entity) {
            Ok(())
        } else {
            Err(EntityError::NoSuchEntity { entity })
        }
    }

    pub(super) fn record_resource_change(
        &mut self,
        resource_type: TypeId,
        resource_name: &'static str,
        kind: ResourceChangeKind,
    ) {
        self.change_tick = self.change_tick.saturating_add(1);
        self.resource_change_ticks
            .insert(resource_type, self.change_tick);
        self.resource_change_log.push(ResourceChangeRecord {
            tick: self.change_tick,
            resource_type,
            resource_name,
            kind,
        });
    }

    pub(super) fn trigger_observers(
        &mut self,
        event_type: TypeId,
        event_type_name: &'static str,
        trigger: ObserverTrigger,
        event_count: usize,
    ) {
        for observer in self.event_observers.values_mut() {
            if observer.event_type != event_type || observer.trigger != trigger {
                continue;
            }
            observer.invocations = observer.invocations.saturating_add(1);
            self.event_observer_notifications
                .push(EventObserverNotification {
                    observer_id: observer.observer_id.clone(),
                    trigger: trigger.clone(),
                    event_type: event_type_name,
                    event_count,
                });
        }
    }

    pub(super) fn mark_component_indexes_dirty(&mut self, component_type: TypeId) {
        let mut indexes = self.component_indexes.borrow_mut();
        for (index_key, index) in indexes.iter_mut() {
            if index_key.component_type == component_type {
                index.mark_dirty();
            }
        }
    }
}
