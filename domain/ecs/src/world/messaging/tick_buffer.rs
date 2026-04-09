use super::diagnostics::TickBufferKey;
use crate::world::world::World;
use std::any::{Any, TypeId, type_name};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TickBufferConfig {
    pub capacity: Option<usize>,
    pub retain_finalized_ticks: bool,
}

impl Default for TickBufferConfig {
    fn default() -> Self {
        Self {
            capacity: None,
            retain_finalized_ticks: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TickBufferProvenance {
    pub domain: u32,
    pub token: u64,
}

impl TickBufferProvenance {
    pub const UNSPECIFIED: Self = Self {
        domain: 0,
        token: 0,
    };

    pub const fn new(domain: u32, token: u64) -> Self {
        Self { domain, token }
    }
}

impl Default for TickBufferProvenance {
    fn default() -> Self {
        Self::UNSPECIFIED
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct TickBufferMeta {
    pub buffer_key: TickBufferKey,
    pub tick: u64,
    pub sequence: u64,
    pub provenance: TickBufferProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickBufferRecord<T> {
    pub meta: TickBufferMeta,
    pub payload: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickBufferRecordRef<'a, T> {
    pub meta: TickBufferMeta,
    pub payload: &'a T,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TickBufferStats {
    pub buffer_key: TickBufferKey,
    pub pushed: u64,
    pub drained: u64,
    pub dropped: u64,
    pub rejected: u64,
    pub pending_messages: usize,
    pub pending_ticks: usize,
    pub capacity: Option<usize>,
    pub latest_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TickBufferPushError {
    Backpressure {
        buffer_type: &'static str,
        capacity: usize,
    },
    Deduplicated {
        buffer_type: &'static str,
    },
}

type TickBufferDedupHook = Box<dyn Fn(&dyn Any, &dyn Any) -> bool + Send + Sync + 'static>;

pub(crate) struct TickBufferStorage {
    pub(super) buffer_key: TickBufferKey,
    pub(super) buffer_type_name: &'static str,
    buckets: Box<dyn Any>,
    bucket_count_fn: fn(&Box<dyn Any>) -> usize,
    finalize_tick_fn: fn(&mut Box<dyn Any>, &mut BTreeMap<u64, Vec<TickBufferMeta>>, u64) -> usize,
    metadata: BTreeMap<u64, Vec<TickBufferMeta>>,
    dedup_hook: Option<TickBufferDedupHook>,
    pub(super) config: TickBufferConfig,
    pub(super) next_sequence: u64,
    pub(super) pushed: u64,
    pub(super) drained: u64,
    pub(super) dropped: u64,
    pub(super) rejected: u64,
    pub(super) pending_messages: usize,
}

impl TickBufferStorage {
    pub(super) fn new<T: 'static>(buffer_key: TickBufferKey) -> Self {
        fn bucket_count_for<T: 'static>(buckets: &Box<dyn Any>) -> usize {
            buckets
                .downcast_ref::<BTreeMap<u64, Vec<T>>>()
                .unwrap_or_else(|| {
                    panic!(
                        "tick buffer bucket count type mismatch: {}",
                        type_name::<T>()
                    )
                })
                .len()
        }

        fn finalize_tick_for<T: 'static>(
            buckets: &mut Box<dyn Any>,
            metadata: &mut BTreeMap<u64, Vec<TickBufferMeta>>,
            finalized_tick: u64,
        ) -> usize {
            let entries = buckets
                .downcast_mut::<BTreeMap<u64, Vec<T>>>()
                .unwrap_or_else(|| {
                    panic!("tick buffer finalize type mismatch: {}", type_name::<T>())
                });
            let to_remove: Vec<u64> = entries
                .keys()
                .copied()
                .take_while(|tick| *tick <= finalized_tick)
                .collect();
            let mut removed = 0usize;
            for tick in to_remove {
                if let Some(values) = entries.remove(&tick) {
                    removed = removed.saturating_add(values.len());
                }
                metadata.remove(&tick);
            }
            removed
        }

        Self {
            buffer_key,
            buffer_type_name: type_name::<T>(),
            buckets: Box::new(BTreeMap::<u64, Vec<T>>::new()),
            bucket_count_fn: bucket_count_for::<T>,
            finalize_tick_fn: finalize_tick_for::<T>,
            metadata: BTreeMap::new(),
            dedup_hook: None,
            config: TickBufferConfig::default(),
            next_sequence: 0,
            pushed: 0,
            drained: 0,
            dropped: 0,
            rejected: 0,
            pending_messages: 0,
        }
    }

    pub(super) fn buckets_ref<T: 'static>(&self) -> &BTreeMap<u64, Vec<T>> {
        self.buckets
            .downcast_ref::<BTreeMap<u64, Vec<T>>>()
            .unwrap_or_else(|| {
                panic!(
                    "tick buffer type mismatch: stored={} requested={}",
                    self.buffer_type_name,
                    type_name::<T>()
                )
            })
    }

    pub(super) fn buckets_mut<T: 'static>(&mut self) -> &mut BTreeMap<u64, Vec<T>> {
        self.buckets
            .downcast_mut::<BTreeMap<u64, Vec<T>>>()
            .unwrap_or_else(|| {
                panic!(
                    "tick buffer type mismatch: stored={} requested={}",
                    self.buffer_type_name,
                    type_name::<T>()
                )
            })
    }

    pub(super) fn metadata_ref(&self, tick: u64) -> &[TickBufferMeta] {
        self.metadata.get(&tick).map(Vec::as_slice).unwrap_or(&[])
    }

    pub(super) fn metadata_remove(&mut self, tick: u64) -> Vec<TickBufferMeta> {
        self.metadata.remove(&tick).unwrap_or_default()
    }

    pub(super) fn metadata_clear(&mut self) {
        self.metadata.clear();
    }

    pub(super) fn bucket_count_any(&self) -> usize {
        (self.bucket_count_fn)(&self.buckets)
    }

    pub(super) fn finalize_up_to_any(&mut self, finalized_tick: u64) -> usize {
        (self.finalize_tick_fn)(&mut self.buckets, &mut self.metadata, finalized_tick)
    }
}

impl World {
    fn allocate_tick_buffer_key(&mut self) -> TickBufferKey {
        self.next_tick_buffer_key = self.next_tick_buffer_key.saturating_add(1);
        TickBufferKey(self.next_tick_buffer_key)
    }

    pub fn has_tick_buffer<T: 'static>(&self) -> bool {
        self.tick_buffers.contains_key(&TypeId::of::<T>())
    }

    pub fn ensure_tick_buffer<T: 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if self.tick_buffers.contains_key(&type_id) {
            return false;
        }
        let buffer_key = self.allocate_tick_buffer_key();
        self.tick_buffers
            .insert(type_id, TickBufferStorage::new::<T>(buffer_key));
        true
    }

    pub fn configure_tick_buffer<T: 'static>(&mut self, config: TickBufferConfig) {
        let type_id = TypeId::of::<T>();
        if !self.tick_buffers.contains_key(&type_id) {
            let buffer_key = self.allocate_tick_buffer_key();
            self.tick_buffers
                .insert(type_id, TickBufferStorage::new::<T>(buffer_key));
        }
        let buffer = self
            .tick_buffers
            .get_mut(&type_id)
            .expect("tick buffer should exist after ensure");
        buffer.config = config;
    }

    pub fn set_tick_buffer_dedup_hook<T: 'static, F>(&mut self, hook: F)
    where
        F: Fn(&T, &T) -> bool + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        if !self.tick_buffers.contains_key(&type_id) {
            let buffer_key = self.allocate_tick_buffer_key();
            self.tick_buffers
                .insert(type_id, TickBufferStorage::new::<T>(buffer_key));
        }

        let buffer = self
            .tick_buffers
            .get_mut(&type_id)
            .expect("tick buffer should exist after ensure");
        buffer.dedup_hook = Some(Box::new(move |left, right| {
            let Some(left) = left.downcast_ref::<T>() else {
                return false;
            };
            let Some(right) = right.downcast_ref::<T>() else {
                return false;
            };
            hook(left, right)
        }));
    }

    pub fn clear_tick_buffer_dedup_hook<T: 'static>(&mut self) {
        let Some(buffer) = self.tick_buffers.get_mut(&TypeId::of::<T>()) else {
            return;
        };
        buffer.dedup_hook = None;
    }

    pub fn push_buffer_message_for_tick<T: 'static>(
        &mut self,
        tick: u64,
        provenance: TickBufferProvenance,
        message: T,
    ) -> Result<TickBufferMeta, TickBufferPushError> {
        let type_id = TypeId::of::<T>();
        if !self.tick_buffers.contains_key(&type_id) {
            let buffer_key = self.allocate_tick_buffer_key();
            self.tick_buffers
                .insert(type_id, TickBufferStorage::new::<T>(buffer_key));
        }
        let buffer = self
            .tick_buffers
            .get_mut(&type_id)
            .expect("tick buffer should exist after ensure");

        if let Some(capacity) = buffer.config.capacity
            && buffer.pending_messages >= capacity
        {
            buffer.rejected = buffer.rejected.saturating_add(1);
            return Err(TickBufferPushError::Backpressure {
                buffer_type: buffer.buffer_type_name,
                capacity,
            });
        }

        if let Some(hook) = &buffer.dedup_hook
            && let Some(last) = buffer
                .buckets_ref::<T>()
                .get(&tick)
                .and_then(|values| values.last())
            && hook(last as &dyn Any, &message as &dyn Any)
        {
            buffer.dropped = buffer.dropped.saturating_add(1);
            return Err(TickBufferPushError::Deduplicated {
                buffer_type: buffer.buffer_type_name,
            });
        }

        buffer.next_sequence = buffer.next_sequence.saturating_add(1);
        let meta = TickBufferMeta {
            buffer_key: buffer.buffer_key,
            tick,
            sequence: buffer.next_sequence,
            provenance,
        };

        buffer
            .buckets_mut::<T>()
            .entry(tick)
            .or_default()
            .push(message);
        buffer.metadata.entry(tick).or_default().push(meta);
        buffer.pending_messages = buffer.pending_messages.saturating_add(1);
        buffer.pushed = buffer.pushed.saturating_add(1);
        Ok(meta)
    }

    pub fn buffer_messages_at_tick<T: 'static>(&self, tick: u64) -> &[T] {
        self.tick_buffers
            .get(&TypeId::of::<T>())
            .and_then(|buffer| buffer.buckets_ref::<T>().get(&tick).map(Vec::as_slice))
            .unwrap_or(&[])
    }

    pub fn buffer_records_at_tick<T: 'static>(&self, tick: u64) -> Vec<TickBufferRecordRef<'_, T>> {
        let Some(buffer) = self.tick_buffers.get(&TypeId::of::<T>()) else {
            return Vec::new();
        };

        let Some(messages) = buffer.buckets_ref::<T>().get(&tick) else {
            return Vec::new();
        };
        let metadata = buffer.metadata_ref(tick);

        metadata
            .iter()
            .copied()
            .zip(messages.iter())
            .map(|(meta, payload)| TickBufferRecordRef { meta, payload })
            .collect()
    }

    pub fn drain_buffer_messages_at_tick<T: 'static>(&mut self, tick: u64) -> Vec<T> {
        let Some(buffer) = self.tick_buffers.get_mut(&TypeId::of::<T>()) else {
            return Vec::new();
        };

        let drained = buffer.buckets_mut::<T>().remove(&tick).unwrap_or_default();
        buffer.metadata_remove(tick);
        buffer.pending_messages = buffer.pending_messages.saturating_sub(drained.len());
        buffer.drained = buffer.drained.saturating_add(drained.len() as u64);
        drained
    }

    pub fn drain_buffer_records_at_tick<T: 'static>(
        &mut self,
        tick: u64,
    ) -> Vec<TickBufferRecord<T>> {
        let Some(buffer) = self.tick_buffers.get_mut(&TypeId::of::<T>()) else {
            return Vec::new();
        };

        let drained_payload = buffer.buckets_mut::<T>().remove(&tick).unwrap_or_default();
        let drained_meta = buffer.metadata_remove(tick);
        buffer.pending_messages = buffer
            .pending_messages
            .saturating_sub(drained_payload.len());
        buffer.drained = buffer.drained.saturating_add(drained_payload.len() as u64);

        drained_meta
            .into_iter()
            .zip(drained_payload)
            .map(|(meta, payload)| TickBufferRecord { meta, payload })
            .collect()
    }

    pub fn clear_buffer_messages<T: 'static>(&mut self) -> usize {
        let Some(buffer) = self.tick_buffers.get_mut(&TypeId::of::<T>()) else {
            return 0;
        };

        let drained = buffer
            .buckets_mut::<T>()
            .values()
            .map(Vec::len)
            .sum::<usize>();
        buffer.buckets_mut::<T>().clear();
        buffer.metadata_clear();
        buffer.pending_messages = 0;
        buffer.drained = buffer.drained.saturating_add(drained as u64);
        drained
    }

    pub fn buffer_stats<T: 'static>(&self) -> Option<TickBufferStats> {
        self.tick_buffers
            .get(&TypeId::of::<T>())
            .map(|buffer| TickBufferStats {
                buffer_key: buffer.buffer_key,
                pushed: buffer.pushed,
                drained: buffer.drained,
                dropped: buffer.dropped,
                rejected: buffer.rejected,
                pending_messages: buffer.pending_messages,
                pending_ticks: buffer.bucket_count_any(),
                capacity: buffer.config.capacity,
                latest_sequence: buffer.next_sequence,
            })
    }

    pub fn set_current_buffer_tick(&mut self, tick: u64) {
        self.current_buffer_tick = tick;
    }

    pub fn current_buffer_tick(&self) -> u64 {
        self.current_buffer_tick
    }

    pub fn push_buffer_message_for_current_tick<T: 'static>(
        &mut self,
        provenance: TickBufferProvenance,
        message: T,
    ) -> Result<TickBufferMeta, TickBufferPushError> {
        self.push_buffer_message_for_tick::<T>(self.current_buffer_tick, provenance, message)
    }

    pub fn current_buffer_messages<T: 'static>(&self) -> &[T] {
        self.buffer_messages_at_tick::<T>(self.current_buffer_tick)
    }

    pub fn current_buffer_records<T: 'static>(&self) -> Vec<TickBufferRecordRef<'_, T>> {
        self.buffer_records_at_tick::<T>(self.current_buffer_tick)
    }

    pub fn drain_current_buffer_messages<T: 'static>(&mut self) -> Vec<T> {
        self.drain_buffer_messages_at_tick::<T>(self.current_buffer_tick)
    }

    pub fn drain_current_buffer_records<T: 'static>(&mut self) -> Vec<TickBufferRecord<T>> {
        self.drain_buffer_records_at_tick::<T>(self.current_buffer_tick)
    }
}
