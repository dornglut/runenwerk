// Owner: ECS World - World State and Default Construction
use super::events_and_indexes::{
    ComponentChangeRecord, ComponentIndexKey, ComponentIndexStorage, ComponentMeta, ComponentStore,
    EventChannelStorage, EventObserver, EventObserverNotification, ResourceChangeRecord,
};
use crate::entity::{Entity, EntityAllocator};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};

pub struct World {
    pub(super) allocator: EntityAllocator,
    pub(super) alive_entities: BTreeSet<Entity>,
    pub(super) component_registry: HashMap<TypeId, ComponentMeta>,
    pub(super) next_component_id: u32,
    pub(super) components: HashMap<TypeId, Box<dyn ComponentStore>>,
    pub(super) resources: HashMap<TypeId, Box<dyn Any>>,
    pub(super) event_channels: HashMap<TypeId, EventChannelStorage>,
    pub(super) event_observers: HashMap<String, EventObserver>,
    pub(super) event_observer_notifications: Vec<EventObserverNotification>,
    pub(super) component_indexes:
        RefCell<HashMap<ComponentIndexKey, Box<dyn ComponentIndexStorage>>>,
    pub(super) change_tick: u64,
    pub(super) component_change_ticks: HashMap<TypeId, u64>,
    pub(super) component_entity_last_changed_ticks: HashMap<TypeId, HashMap<Entity, u64>>,
    pub(super) component_entity_last_added_ticks: HashMap<TypeId, HashMap<Entity, u64>>,
    pub(super) resource_change_ticks: HashMap<TypeId, u64>,
    pub(super) component_change_log: Vec<ComponentChangeRecord>,
    pub(super) resource_change_log: Vec<ResourceChangeRecord>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
