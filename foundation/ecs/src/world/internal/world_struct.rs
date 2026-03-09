// Owner: ECS World - World State and Default Construction
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

