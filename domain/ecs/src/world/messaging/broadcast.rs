use super::diagnostics::BroadcastKey;
use crate::entity::Entity;
use crate::world::world::World;
use std::any::{Any, TypeId, type_name};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BroadcastOverflowPolicy {
    DropOldest,
    DropNewest,
    Panic,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BroadcastLifetime {
    FrameTransient,
    Manual,
    Persistent,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BroadcastTracingPolicy {
    Disabled,
    OnOverflow,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BroadcastStreamConfig {
    pub capacity: Option<usize>,
    pub overflow: BroadcastOverflowPolicy,
    pub lifetime: BroadcastLifetime,
    pub tracing: BroadcastTracingPolicy,
}

impl Default for BroadcastStreamConfig {
    fn default() -> Self {
        Self {
            capacity: None,
            overflow: BroadcastOverflowPolicy::DropOldest,
            lifetime: BroadcastLifetime::Manual,
            tracing: BroadcastTracingPolicy::Disabled,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BroadcastStreamStats {
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
pub enum BroadcastObserverTrigger {
    OnPublish,
    OnDrain,
    EndOfFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastObserverNotification {
    pub observer_id: String,
    pub trigger: BroadcastObserverTrigger,
    pub stream_type: &'static str,
    pub message_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BroadcastObserver {
    pub(super) observer_id: String,
    pub(super) stream_type: TypeId,
    pub(super) trigger: BroadcastObserverTrigger,
    pub(super) invocations: u64,
}

pub(crate) struct BroadcastStreamStorage {
    pub(super) stream_key: BroadcastKey,
    pub(super) stream_type_name: &'static str,
    messages: Box<dyn Any>,
    len_fn: fn(&Box<dyn Any>) -> usize,
    clear_fn: fn(&mut Box<dyn Any>) -> usize,
    pub(super) start_sequence: u64,
    pub(super) next_sequence: u64,
    pub(super) config: BroadcastStreamConfig,
    pub(super) emitted: u64,
    pub(super) drained: u64,
    pub(super) dropped: u64,
}

impl BroadcastStreamStorage {
    pub(super) fn new<T: 'static>(stream_key: BroadcastKey) -> Self {
        fn len_for<T: 'static>(messages: &Box<dyn Any>) -> usize {
            messages
                .downcast_ref::<Vec<T>>()
                .unwrap_or_else(|| {
                    panic!("broadcast stream len type mismatch: {}", type_name::<T>())
                })
                .len()
        }

        fn clear_for<T: 'static>(messages: &mut Box<dyn Any>) -> usize {
            let buffer = messages.downcast_mut::<Vec<T>>().unwrap_or_else(|| {
                panic!("broadcast stream clear type mismatch: {}", type_name::<T>())
            });
            let removed = buffer.len();
            buffer.clear();
            removed
        }

        Self {
            stream_key,
            stream_type_name: type_name::<T>(),
            messages: Box::new(Vec::<T>::new()),
            len_fn: len_for::<T>,
            clear_fn: clear_for::<T>,
            start_sequence: 0,
            next_sequence: 0,
            config: BroadcastStreamConfig::default(),
            emitted: 0,
            drained: 0,
            dropped: 0,
        }
    }

    pub(super) fn messages_ref<T: 'static>(&self) -> &[T] {
        self.messages
            .downcast_ref::<Vec<T>>()
            .map(Vec::as_slice)
            .unwrap_or_else(|| {
                panic!(
                    "broadcast stream type mismatch: stored={} requested={}",
                    self.stream_type_name,
                    type_name::<T>()
                )
            })
    }

    pub(super) fn messages_mut<T: 'static>(&mut self) -> &mut Vec<T> {
        self.messages.downcast_mut::<Vec<T>>().unwrap_or_else(|| {
            panic!(
                "broadcast stream type mismatch: stored={} requested={}",
                self.stream_type_name,
                type_name::<T>()
            )
        })
    }

    pub(super) fn messages_len_any(&self) -> usize {
        (self.len_fn)(&self.messages)
    }

    pub(super) fn clear_any(&mut self) -> usize {
        let removed = (self.clear_fn)(&mut self.messages);
        self.advance_sequence_for_removed(removed);
        removed
    }

    pub(super) fn messages_ref_since<T: 'static>(&self, sequence: u64) -> &[T] {
        let messages = self.messages_ref::<T>();
        let clamped_sequence = sequence.max(self.start_sequence).min(self.next_sequence);
        let offset = (clamped_sequence.saturating_sub(self.start_sequence)) as usize;
        &messages[offset..]
    }

    pub(super) fn advance_sequence_for_removed(&mut self, removed: usize) {
        let removed = removed as u64;
        self.start_sequence = self
            .start_sequence
            .saturating_add(removed)
            .min(self.next_sequence);
    }
}

impl World {
    fn allocate_broadcast_key(&mut self) -> BroadcastKey {
        self.next_broadcast_key = self.next_broadcast_key.saturating_add(1);
        BroadcastKey(self.next_broadcast_key)
    }

    pub fn has_broadcast_stream<T: 'static>(&self) -> bool {
        self.broadcast_streams.contains_key(&TypeId::of::<T>())
    }

    pub fn ensure_broadcast_stream<T: 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if self.broadcast_streams.contains_key(&type_id) {
            return false;
        }
        let stream_key = self.allocate_broadcast_key();
        self.broadcast_streams
            .insert(type_id, BroadcastStreamStorage::new::<T>(stream_key));
        true
    }

    pub fn configure_broadcast_stream<T: 'static>(&mut self, config: BroadcastStreamConfig) {
        let type_id = TypeId::of::<T>();
        let stream_key = self.allocate_broadcast_key();
        let stream = self
            .broadcast_streams
            .entry(type_id)
            .or_insert_with(|| BroadcastStreamStorage::new::<T>(stream_key));
        stream.config = config;
    }

    pub fn publish_broadcast<T: 'static>(&mut self, message: T) {
        let type_id = TypeId::of::<T>();
        if !self.broadcast_streams.contains_key(&type_id) {
            let stream_key = self.allocate_broadcast_key();
            self.broadcast_streams
                .insert(type_id, BroadcastStreamStorage::new::<T>(stream_key));
        }
        let (stream_type_name, emitted_count) = {
            let stream = self
                .broadcast_streams
                .get_mut(&type_id)
                .expect("broadcast stream should exist after ensure");
            let config = stream.config;
            let stream_type_name = stream.stream_type_name;
            let mut dropped = false;
            let mut accepted = false;
            let mut removed_from_front = 0usize;

            {
                let messages = stream.messages_mut::<T>();
                let before = messages.len();
                match config.capacity {
                    None => {
                        messages.push(message);
                        accepted = true;
                    }
                    Some(capacity) => {
                        if capacity == 0 {
                            dropped = true;
                            if matches!(config.overflow, BroadcastOverflowPolicy::Panic) {
                                panic!(
                                    "broadcast stream overflow for {stream_type_name} with capacity=0"
                                );
                            }
                        } else if before < capacity {
                            messages.push(message);
                            accepted = true;
                        } else {
                            match config.overflow {
                                BroadcastOverflowPolicy::DropOldest => {
                                    messages.remove(0);
                                    removed_from_front = 1;
                                    messages.push(message);
                                    dropped = true;
                                    accepted = true;
                                }
                                BroadcastOverflowPolicy::DropNewest => {
                                    dropped = true;
                                }
                                BroadcastOverflowPolicy::Panic => {
                                    panic!(
                                        "broadcast stream overflow for {stream_type_name} at capacity={capacity}"
                                    );
                                }
                            }
                        }
                    }
                }
            }

            if removed_from_front > 0 {
                stream.advance_sequence_for_removed(removed_from_front);
            }

            stream.emitted = stream.emitted.saturating_add(1);
            if dropped {
                stream.dropped = stream.dropped.saturating_add(1);
            }
            if accepted {
                stream.next_sequence = stream.next_sequence.saturating_add(1);
            }

            (stream_type_name, usize::from(accepted))
        };

        if emitted_count > 0 {
            self.trigger_broadcast_observers(
                type_id,
                stream_type_name,
                BroadcastObserverTrigger::OnPublish,
                emitted_count,
            );
        }
    }

    pub fn read_broadcast<T: 'static>(&self) -> &[T] {
        self.broadcast_streams
            .get(&TypeId::of::<T>())
            .map(|stream| stream.messages_ref::<T>())
            .unwrap_or(&[])
    }

    pub(crate) fn read_broadcast_since<T: 'static>(&self, sequence: u64) -> (&[T], u64) {
        let Some(stream) = self.broadcast_streams.get(&TypeId::of::<T>()) else {
            return (&[], 0);
        };
        (
            stream.messages_ref_since::<T>(sequence),
            stream.next_sequence,
        )
    }

    pub fn drain_broadcast_admin<T: 'static>(&mut self) -> Vec<T> {
        let type_id = TypeId::of::<T>();
        let (drained, stream_type_name, drained_count) = {
            let Some(stream) = self.broadcast_streams.get_mut(&type_id) else {
                return Vec::new();
            };
            let stream_type_name = stream.stream_type_name;
            let drained = std::mem::take(stream.messages_mut::<T>());
            let drained_count = drained.len();
            if drained_count > 0 {
                stream.advance_sequence_for_removed(drained_count);
                stream.drained = stream.drained.saturating_add(drained_count as u64);
            }
            (drained, stream_type_name, drained_count)
        };

        if drained_count > 0 {
            self.trigger_broadcast_observers(
                type_id,
                stream_type_name,
                BroadcastObserverTrigger::OnDrain,
                drained_count,
            );
        }

        drained
    }

    pub fn clear_broadcast_admin<T: 'static>(&mut self) -> usize {
        let Some(stream) = self.broadcast_streams.get_mut(&TypeId::of::<T>()) else {
            return 0;
        };
        let removed = stream.clear_any();
        stream.drained = stream.drained.saturating_add(removed as u64);
        removed
    }

    pub fn broadcast_pending_count<T: 'static>(&self) -> usize {
        self.broadcast_streams
            .get(&TypeId::of::<T>())
            .map(|stream| stream.messages_ref::<T>().len())
            .unwrap_or(0)
    }

    pub fn broadcast_stats<T: 'static>(&self) -> Option<BroadcastStreamStats> {
        self.broadcast_streams
            .get(&TypeId::of::<T>())
            .map(|stream| BroadcastStreamStats {
                emitted: stream.emitted,
                drained: stream.drained,
                dropped: stream.dropped,
                pending: stream.messages_ref::<T>().len(),
            })
    }

    pub fn observe_broadcast<T: 'static>(
        &mut self,
        observer_id: impl Into<String>,
        trigger: BroadcastObserverTrigger,
    ) -> bool {
        let observer_id = observer_id.into();
        let created = !self.broadcast_observers.contains_key(&observer_id);
        self.broadcast_observers.insert(
            observer_id.clone(),
            BroadcastObserver {
                observer_id,
                stream_type: TypeId::of::<T>(),
                trigger,
                invocations: 0,
            },
        );
        created
    }

    pub fn remove_broadcast_observer(&mut self, observer_id: &str) -> bool {
        self.broadcast_observers.remove(observer_id).is_some()
    }

    pub fn broadcast_observer_invocations(&self, observer_id: &str) -> Option<u64> {
        self.broadcast_observers
            .get(observer_id)
            .map(|observer| observer.invocations)
    }

    pub fn drain_broadcast_observer_notifications(&mut self) -> Vec<BroadcastObserverNotification> {
        std::mem::take(&mut self.broadcast_observer_notifications)
    }

    pub fn drain_broadcast_map<T: 'static, U, F>(&mut self, map: F) -> Vec<U>
    where
        F: FnMut(T) -> U,
    {
        self.drain_broadcast_admin::<T>()
            .into_iter()
            .map(map)
            .collect()
    }

    pub fn drain_broadcast_filter<T: 'static, F>(&mut self, mut predicate: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        self.drain_broadcast_admin::<T>()
            .into_iter()
            .filter(|message| predicate(message))
            .collect()
    }

    pub(crate) fn trigger_broadcast_observers(
        &mut self,
        stream_type: TypeId,
        stream_type_name: &'static str,
        trigger: BroadcastObserverTrigger,
        message_count: usize,
    ) {
        for observer in self.broadcast_observers.values_mut() {
            if observer.stream_type != stream_type || observer.trigger != trigger {
                continue;
            }

            observer.invocations = observer.invocations.saturating_add(1);
            self.broadcast_observer_notifications
                .push(BroadcastObserverNotification {
                    observer_id: observer.observer_id.clone(),
                    trigger: trigger.clone(),
                    stream_type: stream_type_name,
                    message_count,
                });
        }
    }
}
