use crate::bundle::Bundle;
use crate::component::Component;
use crate::entity::{Entity, EntityAllocator};
use crate::errors::{CommandError, EntityError, ResourceError};
use crate::query::{QueryBorrow, QueryBorrowMut, QueryData};
use crate::resource::Resource;
use std::any::{Any, TypeId, type_name};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::{Deref, DerefMut};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EventChannelStats {
    pub emitted: u64,
    pub drained: u64,
    pub dropped: u64,
    pub pending: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EntitySpawnedEvent {
    pub entity: Entity,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EntityDespawnedEvent {
    pub entity: Entity,
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
pub enum ComponentChangeKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentChangeRecord {
    pub tick: u64,
    pub entity: Entity,
    pub component_type: TypeId,
    pub component_name: &'static str,
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
    pub resource_name: &'static str,
    pub kind: ResourceChangeKind,
}

#[derive(Debug)]
struct ComponentMeta {
    _id: u32,
    name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EventObserver {
    observer_id: String,
    event_type: TypeId,
    trigger: ObserverTrigger,
    invocations: u64,
}

trait ComponentStore {
    fn remove_entity(&mut self, entity: Entity) -> bool;
    fn contains(&self, entity: Entity) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub(crate) struct TypedStore<T: Component> {
    pub(crate) values: BTreeMap<Entity, T>,
}

impl<T: Component> TypedStore<T> {
    fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    fn insert(&mut self, entity: Entity, value: T) {
        self.values.insert(entity, value);
    }

    fn remove(&mut self, entity: Entity) -> Option<T> {
        self.values.remove(&entity)
    }
}

impl<T: Component> ComponentStore for TypedStore<T> {
    fn remove_entity(&mut self, entity: Entity) -> bool {
        self.values.remove(&entity).is_some()
    }

    fn contains(&self, entity: Entity) -> bool {
        self.values.contains_key(&entity)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
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

struct ComponentSecondaryIndex<T: Component, K: Ord + Clone + 'static> {
    entries: BTreeMap<K, Vec<Entity>>,
    extractor: fn(&T) -> K,
    dirty: bool,
}

impl<T: Component, K: Ord + Clone + 'static> ComponentSecondaryIndex<T, K> {
    fn new(extractor: fn(&T) -> K) -> Self {
        Self {
            entries: BTreeMap::new(),
            extractor,
            dirty: true,
        }
    }

    fn entities_for(&self, key: &K) -> Vec<Entity> {
        self.entries.get(key).cloned().unwrap_or_default()
    }

    fn first_entity_for(&self, key: &K) -> Option<Entity> {
        self.entries
            .get(key)
            .and_then(|entities| entities.first())
            .copied()
    }
}

impl<T: Component, K: Ord + Clone + 'static> ComponentIndexStorage
    for ComponentSecondaryIndex<T, K>
{
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn rebuild(&mut self, world: &World) {
        if !self.dirty {
            return;
        }

        self.entries.clear();
        let Some(store) = world.store::<T>() else {
            self.dirty = false;
            return;
        };
        for (entity, component) in &store.values {
            let key = (self.extractor)(component);
            self.entries.entry(key).or_default().push(*entity);
        }
        self.dirty = false;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Mut<'a, T> {
    value: &'a mut T,
}

impl<'a, T> Deref for Mut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for Mut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

pub struct Res<'a, T> {
    value: &'a T,
}

impl<'a, T> Deref for Res<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

pub struct ResMut<'a, T> {
    value: &'a mut T,
}

impl<'a, T> Deref for ResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

pub struct EntityRef<'w> {
    world: &'w World,
    entity: Entity,
}

impl<'w> EntityRef<'w> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn contains<T: Component>(&self) -> bool {
        self.world.contains_component::<T>(self.entity)
    }

    pub fn get<T: Component>(&self) -> Option<&T> {
        self.world.get::<T>(self.entity)
    }

    pub fn require<T: Component>(&self) -> Result<&T, EntityError> {
        self.world.require::<T>(self.entity)
    }
}

pub struct EntityMut<'w> {
    world: &'w mut World,
    entity: Entity,
}

impl<'w> EntityMut<'w> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn contains<T: Component>(&self) -> bool {
        self.world.contains_component::<T>(self.entity)
    }

    pub fn get<T: Component>(&self) -> Option<&T> {
        self.world.get::<T>(self.entity)
    }

    pub fn get_mut<T: Component>(&mut self) -> Option<Mut<'_, T>> {
        self.world.get_mut::<T>(self.entity)
    }

    pub fn require<T: Component>(&self) -> Result<&T, EntityError> {
        self.world.require::<T>(self.entity)
    }

    pub fn require_mut<T: Component>(&mut self) -> Result<Mut<'_, T>, EntityError> {
        self.world.require_mut::<T>(self.entity)
    }

    pub fn insert<B: Bundle>(&mut self, bundle: B) -> Result<(), EntityError> {
        self.world.insert(self.entity, bundle)
    }

    pub fn remove<B: Bundle>(&mut self) -> Result<B, EntityError> {
        self.world.remove::<B>(self.entity)
    }

    pub fn despawn(self) -> Result<(), EntityError> {
        self.world.despawn(self.entity)
    }
}

trait WorldCommand {
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), CommandError>;
}

impl<F> WorldCommand for F
where
    F: FnOnce(&mut World) -> Result<(), CommandError> + 'static,
{
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), CommandError> {
        (*self)(world)
    }
}

pub struct Commands {
    queue: Vec<Box<dyn WorldCommand>>,
}

impl Commands {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn spawn<B: Bundle + 'static>(&mut self, bundle: B) {
        self.queue.push(Box::new(move |world: &mut World| {
            world.spawn(bundle);
            Ok(())
        }));
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queue.push(Box::new(move |world: &mut World| {
            world.despawn(entity)?;
            Ok(())
        }));
    }

    pub fn insert<B: Bundle + 'static>(&mut self, entity: Entity, bundle: B) {
        self.queue.push(Box::new(move |world: &mut World| {
            world.insert(entity, bundle)?;
            Ok(())
        }));
    }

    pub fn remove<B: Bundle + 'static>(&mut self, entity: Entity) {
        self.queue.push(Box::new(move |world: &mut World| {
            let _: B = world.remove(entity)?;
            Ok(())
        }));
    }

    pub fn apply(self, world: &mut World) -> Result<(), CommandError> {
        for command in self.queue {
            command.apply(world)?;
        }
        Ok(())
    }
}

pub struct World {
    allocator: EntityAllocator,
    alive_entities: BTreeSet<Entity>,
    component_registry: HashMap<TypeId, ComponentMeta>,
    next_component_id: u32,
    components: HashMap<TypeId, Box<dyn ComponentStore>>,
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

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

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

    pub fn ensure_component_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        extractor: fn(&T) -> K,
    ) -> bool {
        self.__register_component::<T>();
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

    pub fn find_entity_by_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Option<Entity> {
        self.find_entity_by_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, key)
    }

    pub fn find_entity_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Option<Entity> {
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

    pub fn find_entities_by_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Vec<Entity> {
        self.find_entities_by_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, key)
    }

    pub fn find_entities_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Vec<Entity> {
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

    pub fn find_component_by_index<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        key: &K,
    ) -> Option<&T> {
        self.find_component_by_index_named::<T, K>(DEFAULT_COMPONENT_INDEX_NAME, key)
    }

    pub fn find_component_by_index_named<T: Component, K: Ord + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        key: &K,
    ) -> Option<&T> {
        let entity = self.find_entity_by_index_named::<T, K>(name, key)?;
        self.get::<T>(entity)
    }

    pub fn has_event_channel<T: 'static>(&self) -> bool {
        self.event_channels.contains_key(&TypeId::of::<T>())
    }

    pub fn ensure_event_channel<T: 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if self.event_channels.contains_key(&type_id) {
            return false;
        }
        self.event_channels
            .insert(type_id, EventChannelStorage::new::<T>());
        true
    }

    pub fn configure_event_channel<T: 'static>(&mut self, config: EventChannelConfig) {
        let type_id = TypeId::of::<T>();
        let channel = self
            .event_channels
            .entry(type_id)
            .or_insert_with(EventChannelStorage::new::<T>);
        channel.config = config;
    }

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
            let mut accepted = false;

            match config.capacity {
                None => {
                    events.push(event);
                    accepted = true;
                }
                Some(capacity) => {
                    if capacity == 0 {
                        dropped = true;
                        if matches!(config.overflow, OverflowPolicy::Panic) {
                            panic!("event channel overflow for {event_type_name} with capacity=0");
                        }
                    } else if before < capacity {
                        events.push(event);
                        accepted = true;
                    } else {
                        match config.overflow {
                            OverflowPolicy::DropOldest => {
                                events.remove(0);
                                events.push(event);
                                dropped = true;
                                accepted = true;
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
            }

            (event_type_name, usize::from(accepted))
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

    pub fn read_events<T: 'static>(&self) -> &[T] {
        self.event_channels
            .get(&TypeId::of::<T>())
            .map(|channel| channel.events_ref::<T>())
            .unwrap_or(&[])
    }

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

    pub fn clear_events<T: 'static>(&mut self) -> usize {
        let Some(channel) = self.event_channels.get_mut(&TypeId::of::<T>()) else {
            return 0;
        };
        let removed = channel.events_mut::<T>().len();
        channel.events_mut::<T>().clear();
        channel.drained = channel.drained.saturating_add(removed as u64);
        removed
    }

    pub fn event_count<T: 'static>(&self) -> usize {
        self.event_channels
            .get(&TypeId::of::<T>())
            .map(|channel| channel.events_ref::<T>().len())
            .unwrap_or(0)
    }

    pub fn event_channel_stats<T: 'static>(&self) -> Option<EventChannelStats> {
        self.event_channels
            .get(&TypeId::of::<T>())
            .map(|channel| EventChannelStats {
                emitted: channel.emitted,
                drained: channel.drained,
                dropped: channel.dropped,
                pending: channel.events_ref::<T>().len(),
            })
    }

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

    pub fn remove_event_observer(&mut self, observer_id: &str) -> bool {
        self.event_observers.remove(observer_id).is_some()
    }

    pub fn event_observer_invocations(&self, observer_id: &str) -> Option<u64> {
        self.event_observers
            .get(observer_id)
            .map(|observer| observer.invocations)
    }

    pub fn drain_event_observer_notifications(&mut self) -> Vec<EventObserverNotification> {
        std::mem::take(&mut self.event_observer_notifications)
    }

    pub fn drain_events_map<T: 'static, U, F>(&mut self, map: F) -> Vec<U>
    where
        F: FnMut(T) -> U,
    {
        self.drain_events::<T>().into_iter().map(map).collect()
    }

    pub fn drain_events_filter<T: 'static, F>(&mut self, mut predicate: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        self.drain_events::<T>()
            .into_iter()
            .filter(|event| predicate(event))
            .collect()
    }

    pub fn finish_event_frame(&mut self) {
        let mut end_of_frame_triggers: Vec<(TypeId, &'static str, usize)> = Vec::new();
        for (type_id, channel) in &mut self.event_channels {
            let pending = channel.events_len_any();
            if pending > 0 {
                end_of_frame_triggers.push((*type_id, channel.event_type_name, pending));
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
