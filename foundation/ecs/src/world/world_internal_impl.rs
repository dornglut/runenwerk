// Owner: ECS World - Internal Mutation and Change Tracking
use super::events_and_indexes::{
    ComponentChangeKind, ComponentChangeRecord, ComponentMeta, ComponentStore,
    EventObserverNotification, ObserverTrigger, ResourceChangeKind, ResourceChangeRecord,
    TypedStore,
};
use super::world_struct::World;
use crate::component::Component;
use crate::entity::Entity;
use crate::errors::EntityError;
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
        let kind = if self
            .store::<T>()
            .is_some_and(|store| store.values.contains_key(&entity))
        {
            ComponentChangeKind::Modified
        } else {
            ComponentChangeKind::Added
        };
        let store = self
            .components
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(TypedStore::<T>::new()));
        let store = store
            .as_any_mut()
            .downcast_mut::<TypedStore<T>>()
            .expect("typed store mismatch");
        store.insert(entity, component);
        self.record_component_change(entity, TypeId::of::<T>(), T::component_name(), kind);
        Ok(())
    }

    #[doc(hidden)]
    pub fn __remove_component<T: Component>(&mut self, entity: Entity) -> Result<T, EntityError> {
        self.ensure_entity_exists(entity)?;
        let value = self
            .store_mut::<T>()
            .and_then(|store| store.remove(entity))
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
        out.clear();
        let mut smallest_store_type = None;
        if required_present.is_empty() {
            out.extend(self.alive_entities.iter().copied());
        } else {
            let mut smallest_store_size = usize::MAX;
            for type_id in required_present {
                let Some(store) = self.components.get(type_id) else {
                    telemetry::record_query_matching(start.elapsed().as_nanos() as u64, 0, 0);
                    return;
                };
                let size = store.entity_count();
                if size < smallest_store_size {
                    smallest_store_size = size;
                    smallest_store_type = Some(*type_id);
                }
            }
            if let Some(type_id) = smallest_store_type {
                if let Some(store) = self.components.get(&type_id) {
                    out.reserve(store.entity_count());
                    store.collect_entities(out);
                }
            }
        }

        let candidate_count = out.len() as u64;
        if candidate_count == 0 {
            telemetry::record_query_matching(start.elapsed().as_nanos() as u64, 0, 0);
            return;
        }

        // Fast path for broad single-component queries without exclusions.
        if excluded.is_empty() && required_present.len() == 1 && smallest_store_type.is_some() {
            telemetry::record_query_matching(
                start.elapsed().as_nanos() as u64,
                candidate_count,
                out.len() as u64,
            );
            return;
        }

        let mut required_stores: Vec<&dyn ComponentStore> = Vec::new();
        for type_id in required_present {
            if Some(*type_id) == smallest_store_type {
                continue;
            }
            let Some(store) = self.components.get(type_id) else {
                out.clear();
                telemetry::record_query_matching(
                    start.elapsed().as_nanos() as u64,
                    candidate_count,
                    0,
                );
                return;
            };
            required_stores.push(store.as_ref());
        }

        let mut excluded_stores: Vec<&dyn ComponentStore> = Vec::new();
        for type_id in excluded {
            if let Some(store) = self.components.get(type_id) {
                excluded_stores.push(store.as_ref());
            }
        }

        if !required_stores.is_empty() || !excluded_stores.is_empty() {
            if excluded_stores.is_empty() {
                match required_stores.len() {
                    0 => {}
                    1 => {
                        let required0 = required_stores[0];
                        out.retain(|entity| required0.contains(*entity));
                    }
                    2 => {
                        let required0 = required_stores[0];
                        let required1 = required_stores[1];
                        out.retain(|entity| {
                            required0.contains(*entity) && required1.contains(*entity)
                        });
                    }
                    _ => {
                        out.retain(|entity| {
                            required_stores.iter().all(|store| store.contains(*entity))
                        });
                    }
                }
            } else if required_stores.is_empty() {
                match excluded_stores.len() {
                    0 => {}
                    1 => {
                        let excluded0 = excluded_stores[0];
                        out.retain(|entity| !excluded0.contains(*entity));
                    }
                    2 => {
                        let excluded0 = excluded_stores[0];
                        let excluded1 = excluded_stores[1];
                        out.retain(|entity| {
                            !excluded0.contains(*entity) && !excluded1.contains(*entity)
                        });
                    }
                    _ => {
                        out.retain(|entity| {
                            excluded_stores.iter().all(|store| !store.contains(*entity))
                        });
                    }
                }
            } else if required_stores.len() == 1 && excluded_stores.len() == 1 {
                let required0 = required_stores[0];
                let excluded0 = excluded_stores[0];
                out.retain(|entity| required0.contains(*entity) && !excluded0.contains(*entity));
            } else {
                out.retain(|entity| {
                    required_stores.iter().all(|store| store.contains(*entity))
                        && excluded_stores.iter().all(|store| !store.contains(*entity))
                });
            }
        }

        if required_present.is_empty() {
            out.retain(|entity| self.contains(*entity));
        }

        telemetry::record_query_matching(
            start.elapsed().as_nanos() as u64,
            candidate_count,
            out.len() as u64,
        );
    }

    fn has_component_by_type_id(&self, entity: Entity, type_id: TypeId) -> bool {
        self.components
            .get(&type_id)
            .is_some_and(|store| store.contains(entity))
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
        self.store::<T>()
            .is_some_and(|store| store.values.contains_key(&entity))
    }

    pub(crate) fn component_changed_for_entity_since<T: Component>(
        &self,
        entity: Entity,
        tick: u64,
    ) -> bool {
        let start = Instant::now();
        let component_type = TypeId::of::<T>();
        let changed = self
            .component_entity_last_changed_ticks
            .get(&component_type)
            .and_then(|ticks| ticks.get(&entity))
            .is_some_and(|last_tick| *last_tick > tick);
        telemetry::record_changed_check(start.elapsed().as_nanos() as u64);
        changed
    }

    pub(crate) fn component_added_for_entity_since<T: Component>(
        &self,
        entity: Entity,
        tick: u64,
    ) -> bool {
        let start = Instant::now();
        let component_type = TypeId::of::<T>();
        let added = self
            .component_entity_last_added_ticks
            .get(&component_type)
            .and_then(|ticks| ticks.get(&entity))
            .is_some_and(|last_tick| *last_tick > tick);
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
        match kind {
            ComponentChangeKind::Added | ComponentChangeKind::Modified => {
                self.component_entity_last_changed_ticks
                    .entry(component_type)
                    .or_default()
                    .insert(entity, self.change_tick);
                if matches!(kind, ComponentChangeKind::Added) {
                    self.component_entity_last_added_ticks
                        .entry(component_type)
                        .or_default()
                        .insert(entity, self.change_tick);
                }
            }
            ComponentChangeKind::Removed => {
                if let Some(changed_ticks) = self
                    .component_entity_last_changed_ticks
                    .get_mut(&component_type)
                {
                    changed_ticks.remove(&entity);
                }
                if let Some(added_ticks) = self
                    .component_entity_last_added_ticks
                    .get_mut(&component_type)
                {
                    added_ticks.remove(&entity);
                }
            }
        }
        self.component_change_log.push(ComponentChangeRecord {
            tick: self.change_tick,
            entity,
            component_type,
            component_name,
            kind,
        });
    }

    pub(crate) fn store<T: Component>(&self) -> Option<&TypedStore<T>> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|store| store.as_any().downcast_ref::<TypedStore<T>>())
    }

    pub(crate) fn store_mut<T: Component>(&mut self) -> Option<&mut TypedStore<T>> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|store| store.as_any_mut().downcast_mut::<TypedStore<T>>())
    }

    fn ensure_component_registered<T: Component>(&mut self) {
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
