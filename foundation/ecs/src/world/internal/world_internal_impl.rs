// Owner: ECS World - Internal Mutation and Change Tracking
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

    pub(crate) fn matching_entities(
        &self,
        query_types: &[TypeId],
        required: &[TypeId],
        excluded: &[TypeId],
    ) -> Vec<Entity> {
        self.alive_entities
            .iter()
            .copied()
            .filter(|entity| {
                query_types
                    .iter()
                    .all(|type_id| self.has_component_by_type_id(*entity, *type_id))
                    && required
                        .iter()
                        .all(|type_id| self.has_component_by_type_id(*entity, *type_id))
                    && excluded
                        .iter()
                        .all(|type_id| !self.has_component_by_type_id(*entity, *type_id))
            })
            .collect()
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

    fn ensure_entity_exists(&self, entity: Entity) -> Result<(), EntityError> {
        if self.contains(entity) {
            Ok(())
        } else {
            Err(EntityError::NoSuchEntity { entity })
        }
    }

    fn has_component_by_type_id(&self, entity: Entity, type_id: TypeId) -> bool {
        self.components
            .get(&type_id)
            .is_some_and(|store| store.contains(entity))
    }

    fn contains_component<T: Component>(&self, entity: Entity) -> bool {
        self.store::<T>()
            .is_some_and(|store| store.contains(entity))
    }

    fn mark_component_type_changed_by_id(&mut self, type_id: TypeId) {
        self.change_tick = self.change_tick.saturating_add(1);
        self.component_change_ticks
            .insert(type_id, self.change_tick);
        self.mark_component_indexes_dirty(type_id);
    }

    fn record_component_change(
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

    fn record_resource_change(
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

    fn trigger_observers(
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

    fn mark_component_indexes_dirty(&mut self, component_type: TypeId) {
        for (index_key, index) in &mut self.component_indexes {
            if index_key.component_type == component_type {
                index.mark_dirty();
            }
        }
    }
}
