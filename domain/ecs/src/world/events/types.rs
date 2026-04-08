// Owner: ecs World Events - Event Channel Types
use crate::entity::Entity;
use std::any::{Any, TypeId, type_name};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventObserver {
    pub(super) observer_id: String,
    pub(super) event_type: TypeId,
    pub(super) trigger: ObserverTrigger,
    pub(super) invocations: u64,
}

pub(crate) struct EventChannelStorage {
    pub(super) event_type_name: &'static str,
    events: Box<dyn Any>,
    len_fn: fn(&Box<dyn Any>) -> usize,
    clear_fn: fn(&mut Box<dyn Any>) -> usize,
    pub(super) start_sequence: u64,
    pub(super) next_sequence: u64,
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
            start_sequence: 0,
            next_sequence: 0,
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
        let removed = (self.clear_fn)(&mut self.events);
        self.advance_sequence_for_removed(removed);
        removed
    }

    pub(super) fn events_ref_since<T: 'static>(&self, sequence: u64) -> &[T] {
        let events = self.events_ref::<T>();
        let clamped_sequence = sequence.max(self.start_sequence).min(self.next_sequence);
        let offset = (clamped_sequence.saturating_sub(self.start_sequence)) as usize;
        &events[offset..]
    }

    pub(super) fn advance_sequence_for_removed(&mut self, removed: usize) {
        let removed = removed as u64;
        self.start_sequence = self
            .start_sequence
            .saturating_add(removed)
            .min(self.next_sequence);
    }
}
