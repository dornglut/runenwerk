use super::diagnostics::WorkQueueKey;
use crate::world::World;
use std::any::{Any, TypeId, type_name};
use std::collections::VecDeque;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct WorkQueueConfig {
    pub capacity: Option<usize>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WorkQueueStats {
    pub enqueued: u64,
    pub drained: u64,
    pub rejected: u64,
    pub pending: usize,
    pub capacity: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkQueueEnqueueError {
    Backpressure {
        work_queue_type: &'static str,
        capacity: usize,
    },
}

pub(crate) struct WorkQueueStorage {
    pub(super) work_queue_key: WorkQueueKey,
    pub(super) work_queue_type_name: &'static str,
    messages: Box<dyn Any>,
    len_fn: fn(&Box<dyn Any>) -> usize,
    clear_fn: fn(&mut Box<dyn Any>) -> usize,
    pub(super) config: WorkQueueConfig,
    pub(super) enqueued: u64,
    pub(super) drained: u64,
    pub(super) rejected: u64,
}

impl WorkQueueStorage {
    pub(super) fn new<T: 'static>(work_queue_key: WorkQueueKey) -> Self {
        fn len_for<T: 'static>(messages: &Box<dyn Any>) -> usize {
            messages
                .downcast_ref::<VecDeque<T>>()
                .unwrap_or_else(|| panic!("work queue len type mismatch: {}", type_name::<T>()))
                .len()
        }

        fn clear_for<T: 'static>(messages: &mut Box<dyn Any>) -> usize {
            let queue = messages
                .downcast_mut::<VecDeque<T>>()
                .unwrap_or_else(|| panic!("work queue clear type mismatch: {}", type_name::<T>()));
            let removed = queue.len();
            queue.clear();
            removed
        }

        Self {
            work_queue_key,
            work_queue_type_name: type_name::<T>(),
            messages: Box::new(VecDeque::<T>::new()),
            len_fn: len_for::<T>,
            clear_fn: clear_for::<T>,
            config: WorkQueueConfig::default(),
            enqueued: 0,
            drained: 0,
            rejected: 0,
        }
    }

    pub(super) fn messages_ref<T: 'static>(&self) -> &VecDeque<T> {
        self.messages
            .downcast_ref::<VecDeque<T>>()
            .unwrap_or_else(|| {
                panic!(
                    "work queue type mismatch: stored={} requested={}",
                    self.work_queue_type_name,
                    type_name::<T>()
                )
            })
    }

    pub(super) fn messages_mut<T: 'static>(&mut self) -> &mut VecDeque<T> {
        self.messages
            .downcast_mut::<VecDeque<T>>()
            .unwrap_or_else(|| {
                panic!(
                    "work queue type mismatch: stored={} requested={}",
                    self.work_queue_type_name,
                    type_name::<T>()
                )
            })
    }

    pub(super) fn messages_len_any(&self) -> usize {
        (self.len_fn)(&self.messages)
    }

    pub(super) fn clear_any(&mut self) -> usize {
        (self.clear_fn)(&mut self.messages)
    }
}

impl World {
    fn allocate_work_queue_key(&mut self) -> WorkQueueKey {
        self.next_work_queue_key = self.next_work_queue_key.saturating_add(1);
        WorkQueueKey(self.next_work_queue_key)
    }

    pub fn has_work_queue<T: 'static>(&self) -> bool {
        self.work_queues.contains_key(&TypeId::of::<T>())
    }

    pub fn ensure_work_queue<T: 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if self.work_queues.contains_key(&type_id) {
            return false;
        }
        let work_queue_key = self.allocate_work_queue_key();
        self.work_queues
            .insert(type_id, WorkQueueStorage::new::<T>(work_queue_key));
        true
    }

    pub fn configure_work_queue<T: 'static>(&mut self, config: WorkQueueConfig) {
        let type_id = TypeId::of::<T>();
        let work_queue_key = self.allocate_work_queue_key();
        let queue = self
            .work_queues
            .entry(type_id)
            .or_insert_with(|| WorkQueueStorage::new::<T>(work_queue_key));
        queue.config = config;
    }

    pub fn work_queue_enqueue<T: 'static>(
        &mut self,
        message: T,
    ) -> Result<(), WorkQueueEnqueueError> {
        let type_id = TypeId::of::<T>();
        if !self.work_queues.contains_key(&type_id) {
            let work_queue_key = self.allocate_work_queue_key();
            self.work_queues
                .insert(type_id, WorkQueueStorage::new::<T>(work_queue_key));
        }
        let queue = self
            .work_queues
            .get_mut(&type_id)
            .expect("queue should exist after ensure");

        if let Some(capacity) = queue.config.capacity
            && queue.messages_ref::<T>().len() >= capacity
        {
            queue.rejected = queue.rejected.saturating_add(1);
            return Err(WorkQueueEnqueueError::Backpressure {
                work_queue_type: queue.work_queue_type_name,
                capacity,
            });
        }

        queue.messages_mut::<T>().push_back(message);
        queue.enqueued = queue.enqueued.saturating_add(1);
        Ok(())
    }

    pub fn work_queue_iter<T: 'static>(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.work_queues.get(&TypeId::of::<T>()) {
            Some(queue) => Box::new(queue.messages_ref::<T>().iter()),
            None => Box::new(std::iter::empty()),
        }
    }

    pub fn work_queue_peek<T: 'static>(&self) -> Option<&T> {
        self.work_queues
            .get(&TypeId::of::<T>())
            .and_then(|queue| queue.messages_ref::<T>().front())
    }

    pub fn work_queue_drain<T: 'static>(&mut self) -> Vec<T> {
        let Some(queue) = self.work_queues.get_mut(&TypeId::of::<T>()) else {
            return Vec::new();
        };

        let mut drained = Vec::with_capacity(queue.messages_ref::<T>().len());
        while let Some(value) = queue.messages_mut::<T>().pop_front() {
            drained.push(value);
        }

        queue.drained = queue.drained.saturating_add(drained.len() as u64);
        drained
    }

    pub fn clear_work_queue<T: 'static>(&mut self) -> usize {
        let Some(queue) = self.work_queues.get_mut(&TypeId::of::<T>()) else {
            return 0;
        };
        let removed = queue.clear_any();
        queue.drained = queue.drained.saturating_add(removed as u64);
        removed
    }

    pub fn work_queue_pending_count<T: 'static>(&self) -> usize {
        self.work_queues
            .get(&TypeId::of::<T>())
            .map(WorkQueueStorage::messages_len_any)
            .unwrap_or(0)
    }

    pub fn work_queue_stats<T: 'static>(&self) -> Option<WorkQueueStats> {
        self.work_queues
            .get(&TypeId::of::<T>())
            .map(|queue| WorkQueueStats {
                enqueued: queue.enqueued,
                drained: queue.drained,
                rejected: queue.rejected,
                pending: queue.messages_len_any(),
                capacity: queue.config.capacity,
            })
    }
}
