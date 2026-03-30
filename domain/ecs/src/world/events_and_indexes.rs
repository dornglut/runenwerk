// Owner: ecs World - Event and Component Index Types
use super::world_struct::World;
use crate::component::Component;
use crate::entity::Entity;
use std::any::{Any, TypeId, type_name};
use std::collections::BTreeMap;

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
pub(super) struct ComponentMeta {
    pub(super) _id: u32,
    pub(super) name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EventObserver {
    pub(super) observer_id: String,
    pub(super) event_type: TypeId,
    pub(super) trigger: ObserverTrigger,
    pub(super) invocations: u64,
}

pub(super) struct EventChannelStorage {
    pub(super) event_type_name: &'static str,
    events: Box<dyn Any>,
    len_fn: fn(&Box<dyn Any>) -> usize,
    clear_fn: fn(&mut Box<dyn Any>) -> usize,
    pub(super) config: EventChannelConfig,
    pub(super) emitted: u64,
    pub(super) drained: u64,
    pub(super) dropped: u64,
}

impl EventChannelStorage {
    pub(super) fn new<T: 'static>() -> Self {
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

    pub(super) fn events_ref<T: 'static>(&self) -> &[T] {
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

    pub(super) fn events_mut<T: 'static>(&mut self) -> &mut Vec<T> {
        self.events.downcast_mut::<Vec<T>>().unwrap_or_else(|| {
            panic!(
                "event channel type mismatch: stored={} requested={}",
                self.event_type_name,
                type_name::<T>()
            )
        })
    }

    pub(super) fn events_len_any(&self) -> usize {
        (self.len_fn)(&self.events)
    }

    pub(super) fn clear_any(&mut self) -> usize {
        (self.clear_fn)(&mut self.events)
    }
}

pub(super) const DEFAULT_COMPONENT_INDEX_NAME: &str = "__default";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ComponentIndexKey {
    pub(super) component_type: TypeId,
    key_type: TypeId,
    name: String,
}

impl ComponentIndexKey {
    pub(super) fn new(component_type: TypeId, key_type: TypeId, name: impl Into<String>) -> Self {
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

pub(super) trait ComponentIndexStorage {
    fn mark_dirty(&mut self);
    fn rebuild(&mut self, world: &World);
    fn as_any(&self) -> &dyn Any;
}

pub(super) struct ComponentSecondaryIndex<T: Component, K: Ord + Clone + 'static> {
    entries: BTreeMap<K, Vec<Entity>>,
    extractor: fn(&T) -> K,
    dirty: bool,
}

impl<T: Component, K: Ord + Clone + 'static> ComponentSecondaryIndex<T, K> {
    pub(super) fn new(extractor: fn(&T) -> K) -> Self {
        Self {
            entries: BTreeMap::new(),
            extractor,
            dirty: true,
        }
    }

    pub(super) fn entities_for(&self, key: &K) -> Vec<Entity> {
        self.entries.get(key).cloned().unwrap_or_default()
    }

    pub(super) fn first_entity_for(&self, key: &K) -> Option<Entity> {
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
        let mut entities = Vec::new();
        world.matching_entities_into(&[TypeId::of::<T>()], &[], &mut entities);
        for entity in entities {
            let Some(component) = world.get::<T>(entity) else {
                continue;
            };
            let key = (self.extractor)(component);
            self.entries.entry(key).or_default().push(entity);
        }
        self.dirty = false;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
