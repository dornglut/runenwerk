// Owner: ecs World - World State and Construction
use super::change_tracking::{
    ComponentChangeRecord, ComponentMeta, RemovedComponentRecord, ResourceChangeRecord,
};
use super::component_indexes::{ComponentIndexKey, ComponentIndexStorage};
use super::events::types::{EventChannelStorage, EventObserver, EventObserverNotification};
use crate::entity::{Entity, EntityAllocator};
use crate::indexing::SpatialIndexStorage;
use crate::storage::{ArchetypeRegistry, EntityLocationMap};
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};

pub struct World {
    pub(super) allocator: EntityAllocator,
    pub(super) alive_entities: BTreeSet<Entity>,

    pub(super) component_type_registry: HashMap<TypeId, ComponentMeta>,
    pub(super) reflected_component_types:
        HashMap<TypeId, crate::reflect::ReflectedComponentRegistration>,
    pub(super) reflected_resource_types:
        HashMap<TypeId, crate::reflect::ReflectedResourceRegistration>,

    pub(super) next_component_id: u32,
    pub(super) resources: HashMap<TypeId, Box<dyn Any>>,

    pub(super) event_channels: HashMap<TypeId, EventChannelStorage>,
    pub(super) event_observers: HashMap<String, EventObserver>,
    pub(super) event_observer_notifications: Vec<EventObserverNotification>,

    pub(super) component_indexes:
        RefCell<HashMap<ComponentIndexKey, Box<dyn ComponentIndexStorage>>>,
    pub(super) spatial_indexes: HashMap<String, Box<dyn SpatialIndexStorage>>,

    pub(super) archetype_registry: ArchetypeRegistry,
    pub(super) entity_locations: EntityLocationMap,

    pub(super) change_tick: u64,
    pub(super) component_change_ticks: HashMap<TypeId, u64>,
    pub(super) resource_change_ticks: HashMap<TypeId, u64>,
    pub(super) component_change_log: Vec<ComponentChangeRecord>,
    pub(super) removed_component_records: HashMap<TypeId, Vec<RemovedComponentRecord>>,
    pub(super) resource_change_log: Vec<ResourceChangeRecord>,
}

impl World {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            alive_entities: BTreeSet::new(),

            component_type_registry: HashMap::new(),
            reflected_component_types: HashMap::new(),
            reflected_resource_types: HashMap::new(),

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
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
