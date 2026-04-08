use crate::world::world::World;
use std::any::{Any, TypeId, type_name};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InputStreamConfig {
    pub capacity: Option<usize>,
    pub retain_finalized_ticks: bool,
}

impl Default for InputStreamConfig {
    fn default() -> Self {
        Self {
            capacity: None,
            retain_finalized_ticks: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InputStreamStats {
    pub pushed: u64,
    pub drained: u64,
    pub rejected: u64,
    pub pending_messages: usize,
    pub pending_ticks: usize,
    pub capacity: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputStreamPushError {
    Backpressure {
        stream_type: &'static str,
        capacity: usize,
    },
}

pub(crate) struct InputStreamStorage {
    pub(super) stream_type_name: &'static str,
    buckets: Box<dyn Any>,
    bucket_count_fn: fn(&Box<dyn Any>) -> usize,
    finalize_tick_fn: fn(&mut Box<dyn Any>, u64) -> usize,
    pub(super) config: InputStreamConfig,
    pub(super) pushed: u64,
    pub(super) drained: u64,
    pub(super) rejected: u64,
    pub(super) pending_messages: usize,
}

impl InputStreamStorage {
    pub(super) fn new<T: 'static>() -> Self {
        fn bucket_count_for<T: 'static>(buckets: &Box<dyn Any>) -> usize {
            buckets
                .downcast_ref::<BTreeMap<u64, Vec<T>>>()
                .unwrap_or_else(|| {
                    panic!(
                        "input stream bucket count type mismatch: {}",
                        type_name::<T>()
                    )
                })
                .len()
        }

        fn finalize_tick_for<T: 'static>(buckets: &mut Box<dyn Any>, finalized_tick: u64) -> usize {
            let entries = buckets
                .downcast_mut::<BTreeMap<u64, Vec<T>>>()
                .unwrap_or_else(|| {
                    panic!("input stream finalize type mismatch: {}", type_name::<T>())
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
            }
            removed
        }

        Self {
            stream_type_name: type_name::<T>(),
            buckets: Box::new(BTreeMap::<u64, Vec<T>>::new()),
            bucket_count_fn: bucket_count_for::<T>,
            finalize_tick_fn: finalize_tick_for::<T>,
            config: InputStreamConfig::default(),
            pushed: 0,
            drained: 0,
            rejected: 0,
            pending_messages: 0,
        }
    }

    pub(super) fn buckets_ref<T: 'static>(&self) -> &BTreeMap<u64, Vec<T>> {
        self.buckets
            .downcast_ref::<BTreeMap<u64, Vec<T>>>()
            .unwrap_or_else(|| {
                panic!(
                    "input stream type mismatch: stored={} requested={}",
                    self.stream_type_name,
                    type_name::<T>()
                )
            })
    }

    pub(super) fn buckets_mut<T: 'static>(&mut self) -> &mut BTreeMap<u64, Vec<T>> {
        self.buckets
            .downcast_mut::<BTreeMap<u64, Vec<T>>>()
            .unwrap_or_else(|| {
                panic!(
                    "input stream type mismatch: stored={} requested={}",
                    self.stream_type_name,
                    type_name::<T>()
                )
            })
    }

    pub(super) fn bucket_count_any(&self) -> usize {
        (self.bucket_count_fn)(&self.buckets)
    }

    pub(super) fn finalize_up_to_any(&mut self, finalized_tick: u64) -> usize {
        (self.finalize_tick_fn)(&mut self.buckets, finalized_tick)
    }
}

impl World {
    pub fn has_input_stream<T: 'static>(&self) -> bool {
        self.input_streams.contains_key(&TypeId::of::<T>())
    }

    pub fn ensure_input_stream<T: 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if self.input_streams.contains_key(&type_id) {
            return false;
        }
        self.input_streams
            .insert(type_id, InputStreamStorage::new::<T>());
        true
    }

    pub fn configure_input_stream<T: 'static>(&mut self, config: InputStreamConfig) {
        let type_id = TypeId::of::<T>();
        let stream = self
            .input_streams
            .entry(type_id)
            .or_insert_with(InputStreamStorage::new::<T>);
        stream.config = config;
    }

    pub fn push_input_for_tick<T: 'static>(
        &mut self,
        tick: u64,
        input: T,
    ) -> Result<(), InputStreamPushError> {
        let type_id = TypeId::of::<T>();
        let stream = self
            .input_streams
            .entry(type_id)
            .or_insert_with(InputStreamStorage::new::<T>);

        if let Some(capacity) = stream.config.capacity {
            if stream.pending_messages >= capacity {
                stream.rejected = stream.rejected.saturating_add(1);
                return Err(InputStreamPushError::Backpressure {
                    stream_type: stream.stream_type_name,
                    capacity,
                });
            }
        }

        stream
            .buckets_mut::<T>()
            .entry(tick)
            .or_default()
            .push(input);
        stream.pending_messages = stream.pending_messages.saturating_add(1);
        stream.pushed = stream.pushed.saturating_add(1);
        Ok(())
    }

    pub fn read_input_tick<T: 'static>(&self, tick: u64) -> &[T] {
        self.input_streams
            .get(&TypeId::of::<T>())
            .and_then(|stream| stream.buckets_ref::<T>().get(&tick).map(Vec::as_slice))
            .unwrap_or(&[])
    }

    pub fn drain_input_tick<T: 'static>(&mut self, tick: u64) -> Vec<T> {
        let Some(stream) = self.input_streams.get_mut(&TypeId::of::<T>()) else {
            return Vec::new();
        };

        let drained = stream.buckets_mut::<T>().remove(&tick).unwrap_or_default();
        stream.pending_messages = stream.pending_messages.saturating_sub(drained.len());
        stream.drained = stream.drained.saturating_add(drained.len() as u64);
        drained
    }

    pub fn clear_input_stream<T: 'static>(&mut self) -> usize {
        let Some(stream) = self.input_streams.get_mut(&TypeId::of::<T>()) else {
            return 0;
        };

        let drained = stream
            .buckets_mut::<T>()
            .values()
            .map(Vec::len)
            .sum::<usize>();
        stream.buckets_mut::<T>().clear();
        stream.pending_messages = 0;
        stream.drained = stream.drained.saturating_add(drained as u64);
        drained
    }

    pub fn input_stream_stats<T: 'static>(&self) -> Option<InputStreamStats> {
        self.input_streams
            .get(&TypeId::of::<T>())
            .map(|stream| InputStreamStats {
                pushed: stream.pushed,
                drained: stream.drained,
                rejected: stream.rejected,
                pending_messages: stream.pending_messages,
                pending_ticks: stream.bucket_count_any(),
                capacity: stream.config.capacity,
            })
    }

    pub fn set_current_input_tick(&mut self, tick: u64) {
        self.current_input_tick = tick;
    }

    pub fn current_input_tick(&self) -> u64 {
        self.current_input_tick
    }

    pub fn push_input_for_current_tick<T: 'static>(
        &mut self,
        input: T,
    ) -> Result<(), InputStreamPushError> {
        self.push_input_for_tick::<T>(self.current_input_tick, input)
    }

    pub fn read_input_current_tick<T: 'static>(&self) -> &[T] {
        self.read_input_tick::<T>(self.current_input_tick)
    }

    pub fn drain_input_current_tick<T: 'static>(&mut self) -> Vec<T> {
        self.drain_input_tick::<T>(self.current_input_tick)
    }
}
