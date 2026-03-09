// Owner: ECS World - Core Entity/Resource Lifecycle
impl World {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            alive_entities: BTreeSet::new(),
            component_registry: HashMap::new(),
            next_component_id: 0,
            components: HashMap::new(),
            resources: HashMap::new(),
            event_channels: HashMap::new(),
            event_observers: HashMap::new(),
            event_observer_notifications: Vec::new(),
            component_indexes: HashMap::new(),
            change_tick: 0,
            component_change_ticks: HashMap::new(),
            resource_change_ticks: HashMap::new(),
            component_change_log: Vec::new(),
            resource_change_log: Vec::new(),
        }
    }

    pub fn commands(&self) -> Commands {
        Commands::new()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.alive_entities.contains(&entity)
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        B::register(self);
        let entity = self.allocator.allocate();
        self.alive_entities.insert(entity);
        bundle
            .insert(self, entity)
            .expect("bundle insert should succeed for new entity");
        self.emit_event(EntitySpawnedEvent { entity });
        entity
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), EntityError> {
        self.ensure_entity_exists(entity)?;
        self.alive_entities.remove(&entity);
        let mut removed_types = Vec::new();
        for (type_id, store) in &mut self.components {
            if store.remove_entity(entity) {
                removed_types.push(*type_id);
            }
        }
        self.allocator.free(entity);
        for type_id in removed_types {
            let component_name = self
                .component_registry
                .get(&type_id)
                .map(|meta| meta.name)
                .unwrap_or("unknown_component");
            self.record_component_change(
                entity,
                type_id,
                component_name,
                ComponentChangeKind::Removed,
            );
        }
        self.emit_event(EntityDespawnedEvent { entity });
        Ok(())
    }

    pub fn insert<B: Bundle>(&mut self, entity: Entity, bundle: B) -> Result<(), EntityError> {
        self.ensure_entity_exists(entity)?;
        B::register(self);
        bundle.insert(self, entity)
    }

    pub fn remove<B: Bundle>(&mut self, entity: Entity) -> Result<B, EntityError> {
        self.ensure_entity_exists(entity)?;
        B::remove(self, entity)
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.contains(entity) {
            return None;
        }
        self.store::<T>()
            .and_then(|store| store.values.get(&entity))
    }

    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<Mut<'_, T>> {
        if !self.contains(entity) {
            return None;
        }
        self.record_component_change(
            entity,
            TypeId::of::<T>(),
            T::component_name(),
            ComponentChangeKind::Modified,
        );
        let value = self
            .store_mut::<T>()
            .and_then(|store| store.values.get_mut(&entity))?;
        Some(Mut { value })
    }

    pub fn require<T: Component>(&self, entity: Entity) -> Result<&T, EntityError> {
        self.get::<T>(entity).ok_or(EntityError::MissingComponent {
            entity,
            component: type_name::<T>(),
        })
    }

    pub fn require_mut<T: Component>(&mut self, entity: Entity) -> Result<Mut<'_, T>, EntityError> {
        self.get_mut::<T>(entity)
            .ok_or(EntityError::MissingComponent {
                entity,
                component: type_name::<T>(),
            })
    }

    pub fn entity(&self, entity: Entity) -> Result<EntityRef<'_>, EntityError> {
        self.ensure_entity_exists(entity)?;
        Ok(EntityRef {
            world: self,
            entity,
        })
    }

    pub fn entity_mut(&mut self, entity: Entity) -> Result<EntityMut<'_>, EntityError> {
        self.ensure_entity_exists(entity)?;
        Ok(EntityMut {
            world: self,
            entity,
        })
    }

    pub fn query<Q: QueryData>(&self) -> QueryBorrow<'_, Q> {
        QueryBorrow::new(self)
    }

    pub fn query_mut<Q: QueryData>(&mut self) -> QueryBorrowMut<'_, Q> {
        QueryBorrowMut::new(self)
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        let type_id = TypeId::of::<R>();
        let kind = if self.resources.contains_key(&type_id) {
            ResourceChangeKind::Modified
        } else {
            ResourceChangeKind::Inserted
        };
        self.resources.insert(type_id, Box::new(resource));
        self.record_resource_change(type_id, type_name::<R>(), kind);
    }

    pub fn has_resource<R: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }

    pub fn resource<R: Resource>(&self) -> Result<&R, ResourceError> {
        self.resources
            .get(&TypeId::of::<R>())
            .and_then(|res| res.downcast_ref::<R>())
            .ok_or(ResourceError::Missing {
                resource: type_name::<R>(),
            })
    }

    pub fn resource_mut<R: Resource>(&mut self) -> Result<ResMut<'_, R>, ResourceError> {
        let type_id = TypeId::of::<R>();
        if !self.resources.contains_key(&type_id) {
            return Err(ResourceError::Missing {
                resource: type_name::<R>(),
            });
        }
        self.record_resource_change(type_id, type_name::<R>(), ResourceChangeKind::Modified);
        let value = self
            .resources
            .get_mut(&type_id)
            .and_then(|res| res.downcast_mut::<R>())
            .ok_or(ResourceError::Missing {
                resource: type_name::<R>(),
            })?;
        Ok(ResMut { value })
    }

    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        let type_id = TypeId::of::<R>();
        let removed = self
            .resources
            .remove(&type_id)
            .and_then(|res| res.downcast::<R>().ok().map(|boxed| *boxed));
        if removed.is_some() {
            self.record_resource_change(type_id, type_name::<R>(), ResourceChangeKind::Removed);
        }
        removed
    }

    pub fn current_change_tick(&self) -> u64 {
        self.change_tick
    }

    pub fn component_changed_since<T: Component>(&self, tick: u64) -> bool {
        self.component_change_ticks
            .get(&TypeId::of::<T>())
            .is_some_and(|changed| *changed > tick)
    }

    pub fn resource_changed_since<R: Resource>(&self, tick: u64) -> bool {
        self.resource_change_ticks
            .get(&TypeId::of::<R>())
            .is_some_and(|changed| *changed > tick)
    }

    pub fn component_changes_since(&self, tick: u64) -> Vec<ComponentChangeRecord> {
        self.component_change_log
            .iter()
            .filter(|change| change.tick > tick)
            .cloned()
            .collect()
    }

    pub fn resource_changes_since(&self, tick: u64) -> Vec<ResourceChangeRecord> {
        self.resource_change_log
            .iter()
            .filter(|change| change.tick > tick)
            .cloned()
            .collect()
    }

    pub fn ensure_component_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        extractor: fn(&T) -> K,
    ) -> bool {
        self.ensure_component_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, extractor)
    }
}
