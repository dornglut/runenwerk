use crate::component_registry::ComponentRegistry;
use crate::{
    AnyStorage, Archetype, ArchetypeKey, Component, ComponentKey, EntityAllocator, EntityHandle,
    Resource,
};
use std::any::type_name;
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};
use tracing::{debug, warn};

mod archetypes;
mod components;
mod spawn;

pub use components::MutQueryBuilder;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OverflowPolicy {
    DropOldest,
    DropNewest,
    Panic,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventLifetime {
    FrameTransient,
    Manual,
    Persistent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventTracingPolicy {
    Disabled,
    OnOverflow,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EventChannelConfig {
    pub capacity: Option<usize>,
    pub overflow: OverflowPolicy,
    pub lifetime: EventLifetime,
    pub tracing: EventTracingPolicy,
}

impl Default for EventChannelConfig {
    fn default() -> Self {
        Self {
            capacity: None,
            overflow: OverflowPolicy::DropOldest,
            lifetime: EventLifetime::Manual,
            tracing: EventTracingPolicy::Disabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObserverTrigger {
    OnEmit,
    OnDrain,
    EndOfFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventObserverNotification {
    pub observer_id: String,
    pub trigger: ObserverTrigger,
    pub event_type: &'static str,
    pub event_count: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EventChannelStats {
    pub emitted: u64,
    pub drained: u64,
    pub dropped: u64,
    pub pending: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ComponentChangeKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentChangeRecord {
    pub tick: u64,
    pub entity: EntityHandle,
    pub component_type: TypeId,
    pub component_name: String,
    pub kind: ComponentChangeKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResourceChangeKind {
    Inserted,
    Modified,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceChangeRecord {
    pub tick: u64,
    pub resource_type: TypeId,
    pub resource_name: String,
    pub kind: ResourceChangeKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EntitySpawnedEvent {
    pub entity: EntityHandle,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EntityDespawnedEvent {
    pub entity: EntityHandle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EventObserver {
    observer_id: String,
    event_type: TypeId,
    trigger: ObserverTrigger,
    invocations: u64,
}

struct EventChannelStorage {
    event_type_name: &'static str,
    events: Box<dyn Any>,
    len_fn: fn(&Box<dyn Any>) -> usize,
    clear_fn: fn(&mut Box<dyn Any>) -> usize,
    config: EventChannelConfig,
    emitted: u64,
    drained: u64,
    dropped: u64,
}

impl EventChannelStorage {
    fn new<T: 'static>() -> Self {
        fn len_for<T: 'static>(events: &Box<dyn Any>) -> usize {
            events
                .downcast_ref::<Vec<T>>()
                .unwrap_or_else(|| panic!("event channel len type mismatch: {}", type_name::<T>()))
                .len()
        }

        fn clear_for<T: 'static>(events: &mut Box<dyn Any>) -> usize {
            let buffer = events.downcast_mut::<Vec<T>>().unwrap_or_else(|| {
                panic!("event channel clear type mismatch: {}", type_name::<T>())
            });
            let removed = buffer.len();
            buffer.clear();
            removed
        }

        Self {
            event_type_name: type_name::<T>(),
            events: Box::new(Vec::<T>::new()),
            len_fn: len_for::<T>,
            clear_fn: clear_for::<T>,
            config: EventChannelConfig::default(),
            emitted: 0,
            drained: 0,
            dropped: 0,
        }
    }

    fn events_ref<T: 'static>(&self) -> &[T] {
        self.events
            .downcast_ref::<Vec<T>>()
            .map(Vec::as_slice)
            .unwrap_or_else(|| {
                panic!(
                    "event channel type mismatch: stored={} requested={}",
                    self.event_type_name,
                    type_name::<T>()
                )
            })
    }

    fn events_mut<T: 'static>(&mut self) -> &mut Vec<T> {
        self.events.downcast_mut::<Vec<T>>().unwrap_or_else(|| {
            panic!(
                "event channel type mismatch: stored={} requested={}",
                self.event_type_name,
                type_name::<T>()
            )
        })
    }

    fn pending_len<T: 'static>(&self) -> usize {
        self.events_ref::<T>().len()
    }

    fn events_len_any(&self) -> usize {
        (self.len_fn)(&self.events)
    }

    fn clear_any(&mut self) -> usize {
        (self.clear_fn)(&mut self.events)
    }
}

const DEFAULT_COMPONENT_INDEX_NAME: &str = "__default";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ComponentIndexKey {
    component_type: TypeId,
    key_type: TypeId,
    name: String,
}

impl ComponentIndexKey {
    fn new(component_type: TypeId, key_type: TypeId, name: impl Into<String>) -> Self {
        let mut name = name.into();
        name = name.trim().to_string();
        if name.is_empty() {
            name = DEFAULT_COMPONENT_INDEX_NAME.to_string();
        }
        Self {
            component_type,
            key_type,
            name,
        }
    }
}

trait ComponentIndexStorage {
    fn mark_dirty(&mut self);
    fn rebuild(&mut self, world: &World);
    fn as_any(&self) -> &dyn Any;
}

struct ComponentSecondaryIndex<T: 'static, K: Ord + Clone + 'static> {
    entries: BTreeMap<K, Vec<EntityHandle>>,
    extractor: fn(&T) -> K,
    dirty: bool,
}

impl<T: 'static, K: Ord + Clone + 'static> ComponentSecondaryIndex<T, K> {
    fn new(extractor: fn(&T) -> K) -> Self {
        Self {
            entries: BTreeMap::new(),
            extractor,
            dirty: true,
        }
    }

    fn entities_for(&self, key: &K) -> Vec<EntityHandle> {
        self.entries.get(key).cloned().unwrap_or_default()
    }

    fn first_entity_for(&self, key: &K) -> Option<EntityHandle> {
        self.entries
            .get(key)
            .and_then(|entities| entities.first())
            .copied()
    }
}

impl<T: 'static, K: Ord + Clone + 'static> ComponentIndexStorage for ComponentSecondaryIndex<T, K> {
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn rebuild(&mut self, world: &World) {
        if !self.dirty {
            return;
        }

        self.entries.clear();
        for entity in world.entities_with::<T>() {
            if let Some(component) = world.get_component::<T>(entity) {
                let key = (self.extractor)(component);
                self.entries.entry(key).or_default().push(entity);
            }
        }
        for entities in self.entries.values_mut() {
            entities.sort_by_key(|entity| (entity.id, entity.generation));
            entities.dedup();
        }
        self.dirty = false;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The ECS world, containing archetypes and entity-component mapping.
pub struct World {
    pub entity_allocator: EntityAllocator,
    pub archetypes: HashMap<ArchetypeKey, Archetype>,
    pub entity_locations: HashMap<EntityHandle, (ArchetypeKey, usize)>,
    pub component_registry: ComponentRegistry,
    resources: HashMap<TypeId, Box<dyn Any>>,
    event_channels: HashMap<TypeId, EventChannelStorage>,
    event_observers: HashMap<String, EventObserver>,
    event_observer_notifications: Vec<EventObserverNotification>,
    component_indexes: HashMap<ComponentIndexKey, Box<dyn ComponentIndexStorage>>,
    change_tick: u64,
    component_change_ticks: HashMap<TypeId, u64>,
    resource_change_ticks: HashMap<TypeId, u64>,
    component_change_log: Vec<ComponentChangeRecord>,
    resource_change_log: Vec<ResourceChangeRecord>,
}

impl World {
    /// Create a new empty world.
    pub fn new() -> Self {
        Self {
            entity_allocator: EntityAllocator::new(),
            archetypes: HashMap::new(),
            entity_locations: HashMap::new(),
            component_registry: ComponentRegistry::new(),
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

    /// Register a component type `T` with the world.
    pub fn register_component<T: Component>(&mut self) -> ComponentKey {
        self.register_component_named::<T>(T::component_name())
    }

    /// Register `T` only if it has not already been registered.
    pub fn ensure_component_registered<T: Component>(&mut self) -> ComponentKey {
        let type_id = TypeId::of::<T>();
        if let Some(key) = self.component_registry.get_key_by_type(type_id) {
            return key.clone();
        }

        self.register_component::<T>()
    }

    /// Register a component type `T` with an explicit display name.
    pub fn register_component_named<T: Component>(
        &mut self,
        name: impl Into<String>,
    ) -> ComponentKey {
        let key = self.component_registry.register::<T>(name);
        debug!("registered component '{}'", key.name);
        key
    }

    /// Allocate a new entity handle.
    pub fn allocate_entity(&mut self) -> EntityHandle {
        let entity = self.entity_allocator.allocate();
        debug!(?entity, "Allocated entity");
        entity
    }

    /// Get or create an archetype for a set of component keys.
    pub fn get_or_create_archetype(&mut self, keys: &[ComponentKey]) -> &mut Archetype {
        let key = ArchetypeKey::new(keys.to_vec());

        self.archetypes.entry(key.clone()).or_insert_with(|| {
            let mut constructors: HashMap<TypeId, fn() -> Box<dyn AnyStorage>> = HashMap::new();
            for component_key in keys {
                if let Some(constructor) = self.component_registry.get_constructor(component_key) {
                    constructors.insert(component_key.type_id, *constructor);
                } else {
                    panic!("Component {} not registered", component_key.name);
                }
            }

            Archetype::new(keys.to_vec(), &constructors)
        })
    }

    /// Insert or replace a world resource.
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> Option<R> {
        let previous = self
            .resources
            .insert(TypeId::of::<R>(), Box::new(resource))
            .and_then(|prev| prev.downcast::<R>().ok().map(|boxed| *boxed));
        self.mark_resource_changed_with_kind(
            TypeId::of::<R>(),
            type_name::<R>(),
            if previous.is_some() {
                ResourceChangeKind::Modified
            } else {
                ResourceChangeKind::Inserted
            },
        );
        previous
    }

    /// Returns true if a resource of type `R` exists.
    pub fn has_resource<R: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }

    /// Borrow an immutable world resource.
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.resources
            .get(&TypeId::of::<R>())
            .and_then(|res| res.downcast_ref::<R>())
    }

    /// Borrow a mutable world resource.
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        let type_id = TypeId::of::<R>();
        if !self.resources.contains_key(&type_id) {
            return None;
        }
        self.mark_resource_changed_with_kind(
            type_id,
            type_name::<R>(),
            ResourceChangeKind::Modified,
        );
        self.resources
            .get_mut(&type_id)
            .and_then(|res| res.downcast_mut::<R>())
    }

    /// Remove and return a world resource.
    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        let removed = self
            .resources
            .remove(&TypeId::of::<R>())
            .and_then(|res| res.downcast::<R>().ok().map(|boxed| *boxed));
        if removed.is_some() {
            self.mark_resource_changed_with_kind(
                TypeId::of::<R>(),
                type_name::<R>(),
                ResourceChangeKind::Removed,
            );
        }
        removed
    }

    /// Returns true if an event channel for `T` exists.
    pub fn has_event_channel<T: 'static>(&self) -> bool {
        self.event_channels.contains_key(&TypeId::of::<T>())
    }

    /// Ensure an event channel exists for `T`.
    ///
    /// This is optional for call sites, because `emit_event` auto-creates channels.
    pub fn ensure_event_channel<T: 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if self.event_channels.contains_key(&type_id) {
            return false;
        }
        self.event_channels
            .insert(type_id, EventChannelStorage::new::<T>());
        true
    }

    /// Configure event channel behavior for `T`.
    ///
    /// If the channel does not yet exist, it is created.
    pub fn configure_event_channel<T: 'static>(&mut self, config: EventChannelConfig) {
        let type_id = TypeId::of::<T>();
        let channel = self
            .event_channels
            .entry(type_id)
            .or_insert_with(EventChannelStorage::new::<T>);
        channel.config = config;
    }

    /// Emit an event for type `T`.
    ///
    /// Channels are created automatically on first emit.
    pub fn emit_event<T: 'static>(&mut self, event: T) {
        let type_id = TypeId::of::<T>();
        let (event_type_name, emitted_count) = {
            let channel = self
                .event_channels
                .entry(type_id)
                .or_insert_with(EventChannelStorage::new::<T>);
            let config = channel.config;
            let event_type_name = channel.event_type_name;
            let events = channel.events_mut::<T>();
            let before = events.len();
            let mut dropped = false;

            match config.capacity {
                None => {
                    events.push(event);
                }
                Some(capacity) => {
                    if capacity == 0 {
                        dropped = true;
                        if matches!(config.overflow, OverflowPolicy::Panic) {
                            panic!("event channel overflow for {event_type_name} with capacity=0");
                        }
                    } else if before < capacity {
                        events.push(event);
                    } else {
                        match config.overflow {
                            OverflowPolicy::DropOldest => {
                                events.remove(0);
                                events.push(event);
                                dropped = true;
                            }
                            OverflowPolicy::DropNewest => {
                                dropped = true;
                            }
                            OverflowPolicy::Panic => {
                                panic!(
                                    "event channel overflow for {event_type_name} at capacity={capacity}"
                                );
                            }
                        }
                    }
                }
            }

            channel.emitted = channel.emitted.saturating_add(1);
            if dropped {
                channel.dropped = channel.dropped.saturating_add(1);
                if matches!(config.tracing, EventTracingPolicy::OnOverflow) {
                    warn!(
                        event_type = event_type_name,
                        capacity = ?config.capacity,
                        "event channel overflow"
                    );
                }
            }

            (event_type_name, if dropped { 0 } else { 1 })
        };
        if emitted_count > 0 {
            self.trigger_observers(
                type_id,
                event_type_name,
                ObserverTrigger::OnEmit,
                emitted_count,
            );
        }
    }

    /// Borrow pending events for `T` without consuming them.
    pub fn read_events<T: 'static>(&self) -> &[T] {
        self.event_channels
            .get(&TypeId::of::<T>())
            .map(|channel| channel.events_ref::<T>())
            .unwrap_or(&[])
    }

    /// Drain and return all pending events for `T`.
    pub fn drain_events<T: 'static>(&mut self) -> Vec<T> {
        let type_id = TypeId::of::<T>();
        let (drained, event_type_name, drained_count) = {
            let Some(channel) = self.event_channels.get_mut(&type_id) else {
                return Vec::new();
            };
            let event_type_name = channel.event_type_name;
            let drained = std::mem::take(channel.events_mut::<T>());
            let drained_count = drained.len();
            if drained_count > 0 {
                channel.drained = channel.drained.saturating_add(drained_count as u64);
            }
            (drained, event_type_name, drained_count)
        };
        if drained_count > 0 {
            self.trigger_observers(
                type_id,
                event_type_name,
                ObserverTrigger::OnDrain,
                drained_count,
            );
        }
        drained
    }

    /// Clear pending events for `T`, returning number removed.
    pub fn clear_events<T: 'static>(&mut self) -> usize {
        let type_id = TypeId::of::<T>();
        let Some(channel) = self.event_channels.get_mut(&type_id) else {
            return 0;
        };
        let events = channel.events_mut::<T>();
        let removed = events.len();
        events.clear();
        if removed > 0 {
            channel.drained = channel.drained.saturating_add(removed as u64);
        }
        removed
    }

    /// Number of pending events for `T`.
    pub fn event_count<T: 'static>(&self) -> usize {
        self.event_channels
            .get(&TypeId::of::<T>())
            .map(|channel| channel.pending_len::<T>())
            .unwrap_or(0)
    }

    /// Return channel stats for `T` if the channel exists.
    pub fn event_channel_stats<T: 'static>(&self) -> Option<EventChannelStats> {
        self.event_channels
            .get(&TypeId::of::<T>())
            .map(|channel| EventChannelStats {
                emitted: channel.emitted,
                drained: channel.drained,
                dropped: channel.dropped,
                pending: channel.pending_len::<T>(),
            })
    }

    /// Register or replace an observer for typed events.
    ///
    /// Returns true when created, false when replaced.
    pub fn observe_events<T: 'static>(
        &mut self,
        observer_id: impl Into<String>,
        trigger: ObserverTrigger,
    ) -> bool {
        let observer_id = observer_id.into();
        let created = !self.event_observers.contains_key(&observer_id);
        self.event_observers.insert(
            observer_id.clone(),
            EventObserver {
                observer_id,
                event_type: TypeId::of::<T>(),
                trigger,
                invocations: 0,
            },
        );
        created
    }

    /// Remove an observer by id.
    pub fn remove_event_observer(&mut self, observer_id: &str) -> bool {
        self.event_observers.remove(observer_id).is_some()
    }

    /// Returns invocation count for an observer id.
    pub fn event_observer_invocations(&self, observer_id: &str) -> Option<u64> {
        self.event_observers
            .get(observer_id)
            .map(|observer| observer.invocations)
    }

    /// Drain observer notifications emitted by event triggers.
    pub fn drain_event_observer_notifications(&mut self) -> Vec<EventObserverNotification> {
        std::mem::take(&mut self.event_observer_notifications)
    }

    /// Advance event lifecycle policies at end of frame.
    ///
    /// - Triggers `EndOfFrame` observers for channels with pending events.
    /// - Clears channels configured as `FrameTransient`.
    pub fn finish_event_frame(&mut self) {
        let mut end_of_frame_triggers: Vec<(TypeId, &'static str, usize)> = Vec::new();
        for (key, channel) in &mut self.event_channels {
            let pending = channel.events_len_any();
            if pending > 0 {
                end_of_frame_triggers.push((*key, channel.event_type_name, pending));
            }

            if matches!(channel.config.lifetime, EventLifetime::FrameTransient) && pending > 0 {
                let removed = channel.clear_any();
                channel.drained = channel.drained.saturating_add(removed as u64);
            }
        }

        for (event_type, event_type_name, pending) in end_of_frame_triggers {
            self.trigger_observers(
                event_type,
                event_type_name,
                ObserverTrigger::EndOfFrame,
                pending,
            );
        }
    }

    /// Drain and transform events for `T` using `map`.
    pub fn drain_events_map<T: 'static, U, F>(&mut self, map: F) -> Vec<U>
    where
        F: FnMut(T) -> U,
    {
        self.drain_events::<T>().into_iter().map(map).collect()
    }

    /// Drain events for `T`, keeping only items where `predicate` returns true.
    pub fn drain_events_filter<T: 'static, F>(&mut self, mut predicate: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        self.drain_events::<T>()
            .into_iter()
            .filter(|event| predicate(event))
            .collect()
    }

    /// Register or replace a named secondary index for component `T` keyed by `K`.
    ///
    /// Returns true when a new index was created and false when an existing one was replaced.
    pub fn register_component_index_named<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        extractor: fn(&T) -> K,
    ) -> bool {
        let key = ComponentIndexKey::new(TypeId::of::<T>(), TypeId::of::<K>(), name);
        let replaced = self
            .component_indexes
            .insert(
                key,
                Box::new(ComponentSecondaryIndex::<T, K>::new(extractor)),
            )
            .is_some();
        self.mark_component_indexes_dirty(TypeId::of::<T>());
        !replaced
    }

    /// Register or replace the default secondary index for `(T, K)`.
    pub fn register_component_index<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        extractor: fn(&T) -> K,
    ) -> bool {
        self.register_component_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, extractor)
    }

    /// Ensure a named secondary index exists for component `T` keyed by `K`.
    ///
    /// Returns true when created, false when it already existed.
    pub fn ensure_component_index_named<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        extractor: fn(&T) -> K,
    ) -> bool {
        let key = ComponentIndexKey::new(TypeId::of::<T>(), TypeId::of::<K>(), name);
        if self.component_indexes.contains_key(&key) {
            return false;
        }
        self.component_indexes.insert(
            key,
            Box::new(ComponentSecondaryIndex::<T, K>::new(extractor)),
        );
        self.mark_component_indexes_dirty(TypeId::of::<T>());
        true
    }

    /// Ensure the default secondary index exists for `(T, K)`.
    pub fn ensure_component_index<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        extractor: fn(&T) -> K,
    ) -> bool {
        self.ensure_component_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, extractor)
    }

    /// Remove a named secondary index for component `T` keyed by `K`.
    pub fn remove_component_index_named<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
    ) -> bool {
        let key = ComponentIndexKey::new(TypeId::of::<T>(), TypeId::of::<K>(), name);
        self.component_indexes.remove(&key).is_some()
    }

    /// Remove the default secondary index for `(T, K)`.
    pub fn remove_component_index<T: 'static, K: Ord + Clone + 'static>(&mut self) -> bool {
        self.remove_component_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME)
    }

    /// Returns true if a named index for `(T, K)` is present.
    pub fn has_component_index_named<T: 'static, K: Ord + Clone + 'static>(
        &self,
        name: impl Into<String>,
    ) -> bool {
        let key = ComponentIndexKey::new(TypeId::of::<T>(), TypeId::of::<K>(), name);
        self.component_indexes.contains_key(&key)
    }

    /// Returns true if the default index for `(T, K)` is present.
    pub fn has_component_index<T: 'static, K: Ord + Clone + 'static>(&self) -> bool {
        self.has_component_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME)
    }

    /// Return all entities with component `T` matching a named indexed `key`.
    pub fn find_entities_by_index_named<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Vec<EntityHandle> {
        let index_key = ComponentIndexKey::new(TypeId::of::<T>(), TypeId::of::<K>(), name);
        let Some(mut index) = self.component_indexes.remove(&index_key) else {
            return Vec::new();
        };
        index.rebuild(self);
        let entities = index
            .as_any()
            .downcast_ref::<ComponentSecondaryIndex<T, K>>()
            .map(|index| index.entities_for(key))
            .unwrap_or_default();
        self.component_indexes.insert(index_key, index);
        entities
    }

    /// Return all entities with component `T` matching the default indexed `key`.
    pub fn find_entities_by_index<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Vec<EntityHandle> {
        self.find_entities_by_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, key)
    }

    /// Return the first entity with component `T` matching a named indexed `key`.
    pub fn find_entity_by_index_named<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Option<EntityHandle> {
        let index_key = ComponentIndexKey::new(TypeId::of::<T>(), TypeId::of::<K>(), name);
        let Some(mut index) = self.component_indexes.remove(&index_key) else {
            return None;
        };
        index.rebuild(self);
        let entity = index
            .as_any()
            .downcast_ref::<ComponentSecondaryIndex<T, K>>()
            .and_then(|index| index.first_entity_for(key));
        self.component_indexes.insert(index_key, index);
        entity
    }

    /// Return the first entity with component `T` matching the default indexed `key`.
    pub fn find_entity_by_index<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Option<EntityHandle> {
        self.find_entity_by_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, key)
    }

    /// Return the first component `T` matching a named indexed `key`.
    pub fn find_component_by_index_named<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Option<&T> {
        let entity = self.find_entity_by_index_named::<T, K>(name, key)?;
        self.get_component::<T>(entity)
    }

    /// Return the first component `T` matching the default indexed `key`.
    pub fn find_component_by_index<T: 'static, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Option<&T> {
        self.find_component_by_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, key)
    }

    /// Monotonic world change tick.
    pub fn current_change_tick(&self) -> u64 {
        self.change_tick
    }

    /// Last change tick for component type `T`, if any.
    pub fn component_last_changed_tick<T: 'static>(&self) -> Option<u64> {
        self.component_change_ticks.get(&TypeId::of::<T>()).copied()
    }

    /// Last change tick for resource type `R`, if any.
    pub fn resource_last_changed_tick<R: Resource>(&self) -> Option<u64> {
        self.resource_change_ticks.get(&TypeId::of::<R>()).copied()
    }

    /// True when component `T` changed after `tick`.
    pub fn component_changed_since<T: 'static>(&self, tick: u64) -> bool {
        self.component_last_changed_tick::<T>()
            .is_some_and(|changed| changed > tick)
    }

    /// True when resource `R` changed after `tick`.
    pub fn resource_changed_since<R: Resource>(&self, tick: u64) -> bool {
        self.resource_last_changed_tick::<R>()
            .is_some_and(|changed| changed > tick)
    }

    /// Return component change records with tick strictly greater than `tick`.
    pub fn component_changes_since(&self, tick: u64) -> Vec<ComponentChangeRecord> {
        self.component_change_log
            .iter()
            .filter(|change| change.tick > tick)
            .cloned()
            .collect()
    }

    /// Return resource change records with tick strictly greater than `tick`.
    pub fn resource_changes_since(&self, tick: u64) -> Vec<ResourceChangeRecord> {
        self.resource_change_log
            .iter()
            .filter(|change| change.tick > tick)
            .cloned()
            .collect()
    }

    pub(crate) fn mark_component_added_for_entity(
        &mut self,
        entity: EntityHandle,
        component_type: TypeId,
        component_name: impl Into<String>,
    ) {
        self.record_component_change(
            entity,
            component_type,
            component_name,
            ComponentChangeKind::Added,
        );
    }

    pub(crate) fn mark_component_modified_for_entity(
        &mut self,
        entity: EntityHandle,
        component_type: TypeId,
        component_name: impl Into<String>,
    ) {
        self.record_component_change(
            entity,
            component_type,
            component_name,
            ComponentChangeKind::Modified,
        );
    }

    pub(crate) fn mark_component_removed_for_entity(
        &mut self,
        entity: EntityHandle,
        component_type: TypeId,
        component_name: impl Into<String>,
    ) {
        self.record_component_change(
            entity,
            component_type,
            component_name,
            ComponentChangeKind::Removed,
        );
    }

    fn record_component_change(
        &mut self,
        entity: EntityHandle,
        component_type: TypeId,
        component_name: impl Into<String>,
        kind: ComponentChangeKind,
    ) {
        let tick = self.next_change_tick();
        self.component_change_ticks.insert(component_type, tick);
        self.mark_component_indexes_dirty(component_type);
        self.component_change_log.push(ComponentChangeRecord {
            tick,
            entity,
            component_type,
            component_name: component_name.into(),
            kind,
        });
    }

    fn mark_resource_changed_with_kind(
        &mut self,
        resource_type: TypeId,
        resource_name: impl Into<String>,
        kind: ResourceChangeKind,
    ) {
        let tick = self.next_change_tick();
        self.resource_change_ticks.insert(resource_type, tick);
        self.resource_change_log.push(ResourceChangeRecord {
            tick,
            resource_type,
            resource_name: resource_name.into(),
            kind,
        });
    }

    fn mark_component_indexes_dirty(&mut self, component_type: TypeId) {
        for (index_key, index) in &mut self.component_indexes {
            if index_key.component_type == component_type {
                index.mark_dirty();
            }
        }
    }

    fn next_change_tick(&mut self) -> u64 {
        self.change_tick = self.change_tick.saturating_add(1);
        self.change_tick
    }

    fn trigger_observers(
        &mut self,
        event_type: TypeId,
        event_type_name: &'static str,
        trigger: ObserverTrigger,
        event_count: usize,
    ) {
        if event_count == 0 {
            return;
        }

        let mut matching_observers: Vec<String> = self
            .event_observers
            .iter()
            .filter_map(|(observer_id, observer)| {
                if observer.event_type == event_type && observer.trigger == trigger {
                    Some(observer_id.clone())
                } else {
                    None
                }
            })
            .collect();
        matching_observers.sort();

        for observer_id in matching_observers {
            let Some(observer) = self.event_observers.get_mut(&observer_id) else {
                continue;
            };
            observer.invocations = observer.invocations.saturating_add(1);
            self.event_observer_notifications
                .push(EventObserverNotification {
                    observer_id: observer.observer_id.clone(),
                    trigger: observer.trigger.clone(),
                    event_type: event_type_name,
                    event_count,
                });
        }
    }
}
