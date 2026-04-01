// Owner: ecs World - Core Entity/Resource Lifecycle
use super::change_tracking::{
    ComponentChangeKind, ComponentChangeRecord, ResourceChangeKind, ResourceChangeRecord,
};
use super::component_indexes::DEFAULT_COMPONENT_INDEX_NAME;
use super::entity_handles::{EntityMut, EntityRef, Mut};
use super::event_channels::{EntityDespawnedEvent, EntitySpawnedEvent};
use super::world_struct::World;
use crate::bundle::Bundle;
use crate::commands::Commands;
use crate::component::{Component, ComponentState, Resource, StatefulComponent};
use crate::entity::{Entity, EntityAllocator};
use crate::errors::{EntityError, ResourceError};
use crate::query::{QueryFilter, QueryOrphanedState, QuerySpec, QueryState};
use crate::storage::ArchetypeRegistry;
use std::any::{Any, TypeId, type_name};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};

impl World {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            alive_entities: BTreeSet::new(),
            component_registry: HashMap::new(),
            next_component_id: 0,
            resources: HashMap::new(),
            event_channels: HashMap::new(),
            event_observers: HashMap::new(),
            event_observer_notifications: Vec::new(),
            component_indexes: RefCell::new(HashMap::new()),
            spatial_indexes: HashMap::new(),
            archetype_registry: ArchetypeRegistry::new(),
            entity_locations: Default::default(),
            change_tick: 0,
            component_change_ticks: HashMap::new(),
            resource_change_ticks: HashMap::new(),
            component_change_log: Vec::new(),
            removed_component_records: HashMap::new(),
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
        self.place_entity_in_empty_archetype(entity);
        bundle
            .insert(self, entity)
            .expect("bundle insert should succeed for new entity");
        self.emit_event(EntitySpawnedEvent { entity });
        entity
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), EntityError> {
        self.ensure_entity_exists(entity)?;
        self.remove_entity_from_spatial_indexes(entity);
        let removed_types = self
            .entity_locations
            .get(entity)
            .and_then(|location| {
                self.archetype_registry
                    .component_types(location.archetype_id)
                    .map(|types| types.to_vec())
            })
            .unwrap_or_default();
        self.remove_entity_from_archetype_tracking(entity);
        self.alive_entities.remove(&entity);
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
        self.archetype_component::<T>(entity)
    }

    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<Mut<'_, T>> {
        if !self.contains(entity) {
            return None;
        }
        self.mark_component_modified_by_id(entity, TypeId::of::<T>(), T::component_name());
        let value = self.archetype_component_mut_untracked::<T>(entity)?;
        Some(Mut { value })
    }

    pub fn component_state<T: StatefulComponent>(&self, entity: Entity) -> Option<ComponentState> {
        if !self.contains(entity) {
            return None;
        }
        let (generation, version) = self.archetype_registry.component_state_by_id(
            entity,
            TypeId::of::<T>(),
            &self.entity_locations,
        )?;
        Some(ComponentState {
            generation,
            version,
        })
    }

    pub fn mark_stateful_changed<T: StatefulComponent>(&mut self, entity: Entity) -> bool {
        if !self.has_component_by_type_id(entity, TypeId::of::<T>()) {
            return false;
        }
        self.mark_component_modified_by_id(entity, TypeId::of::<T>(), T::component_name());
        self.archetype_registry
            .mark_component_stateful_changed_by_id(
                entity,
                TypeId::of::<T>(),
                self.change_tick,
                &self.entity_locations,
            )
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

    pub fn query_state<Q: QuerySpec, F: QueryFilter>(&self) -> QueryState<Q, F> {
        QueryState::new(self)
    }

    pub fn query_orphaned_state<T: Component>(&self) -> QueryOrphanedState<T> {
        QueryOrphanedState::new(self)
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

    pub fn resource_by_type_id(&self, type_id: TypeId) -> Option<&dyn Any> {
        self.resources
            .get(&type_id)
            .map(|resource| resource.as_ref())
    }

    pub fn resource_mut<R: Resource>(&mut self) -> Result<&mut R, ResourceError> {
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
        Ok(value)
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

    #[doc(hidden)]
    pub fn __entity_archetype_location(&self, entity: Entity) -> Option<(usize, usize)> {
        self.entity_locations
            .get(entity)
            .map(|location| (location.archetype_id.index(), location.row))
    }

    #[doc(hidden)]
    pub fn __entity_archetype_component_count(&self, entity: Entity) -> Option<usize> {
        let location = self.entity_locations.get(entity)?;
        self.archetype_registry
            .component_count(location.archetype_id)
    }

    #[doc(hidden)]
    pub fn __entity_component_ticks<T: Component>(&self, entity: Entity) -> Option<(u64, u64)> {
        self.archetype_component_metadata::<T>(entity)
    }

    /// Type-level reporting helper.
    /// This is intentionally separate from `Changed<T>` query semantics, which are driven by
    /// archetype row metadata (`changed_tick`) during query/filter evaluation.
    pub fn component_changed_since<T: Component>(&self, tick: u64) -> bool {
        self.component_change_ticks
            .get(&TypeId::of::<T>())
            .is_some_and(|changed| *changed > tick)
    }

    /// Type-level reporting helper for resources.
    pub fn resource_changed_since<R: Resource>(&self, tick: u64) -> bool {
        self.resource_change_ticks
            .get(&TypeId::of::<R>())
            .is_some_and(|changed| *changed > tick)
    }

    /// Change-history reporting API.
    /// This log is for introspection/reporting and is not used by query/filter change matching.
    pub fn component_changes_since(&self, tick: u64) -> Vec<ComponentChangeRecord> {
        self.component_change_log
            .iter()
            .filter(|change| change.tick > tick)
            .cloned()
            .collect()
    }

    /// Resource change-history reporting API.
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
