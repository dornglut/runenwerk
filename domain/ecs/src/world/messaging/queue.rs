use crate::world::world::World;
use std::any::{Any, TypeId, type_name};
use std::collections::VecDeque;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QueueConfig {
    pub capacity: Option<usize>,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self { capacity: None }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QueueStats {
    pub enqueued: u64,
    pub drained: u64,
    pub rejected: u64,
    pub pending: usize,
    pub capacity: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueEnqueueError {
    Backpressure {
        queue_type: &'static str,
        capacity: usize,
    },
}

pub(crate) struct QueueStorage {
    pub(super) queue_type_name: &'static str,
    messages: Box<dyn Any>,
    len_fn: fn(&Box<dyn Any>) -> usize,
    clear_fn: fn(&mut Box<dyn Any>) -> usize,
    pub(super) config: QueueConfig,
    pub(super) enqueued: u64,
    pub(super) drained: u64,
    pub(super) rejected: u64,
}

impl QueueStorage {
    pub(super) fn new<T: 'static>() -> Self {
        fn len_for<T: 'static>(messages: &Box<dyn Any>) -> usize {
            messages
                .downcast_ref::<VecDeque<T>>()
                .unwrap_or_else(|| panic!("queue len type mismatch: {}", type_name::<T>()))
                .len()
        }

        fn clear_for<T: 'static>(messages: &mut Box<dyn Any>) -> usize {
            let queue = messages
                .downcast_mut::<VecDeque<T>>()
                .unwrap_or_else(|| panic!("queue clear type mismatch: {}", type_name::<T>()));
            let removed = queue.len();
            queue.clear();
            removed
        }

        Self {
            queue_type_name: type_name::<T>(),
            messages: Box::new(VecDeque::<T>::new()),
            len_fn: len_for::<T>,
            clear_fn: clear_for::<T>,
            config: QueueConfig::default(),
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
                    "queue type mismatch: stored={} requested={}",
                    self.queue_type_name,
                    type_name::<T>()
                )
            })
    }

    pub(super) fn messages_mut<T: 'static>(&mut self) -> &mut VecDeque<T> {
        self.messages
            .downcast_mut::<VecDeque<T>>()
            .unwrap_or_else(|| {
                panic!(
                    "queue type mismatch: stored={} requested={}",
                    self.queue_type_name,
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
    pub fn has_queue<T: 'static>(&self) -> bool {
        self.queues.contains_key(&TypeId::of::<T>())
    }

    pub fn ensure_queue<T: 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        if self.queues.contains_key(&type_id) {
            return false;
        }
        self.queues.insert(type_id, QueueStorage::new::<T>());
        true
    }

    pub fn configure_queue<T: 'static>(&mut self, config: QueueConfig) {
        let type_id = TypeId::of::<T>();
        let queue = self
            .queues
            .entry(type_id)
            .or_insert_with(QueueStorage::new::<T>);
        queue.config = config;
    }

    pub fn queue_enqueue<T: 'static>(&mut self, message: T) -> Result<(), QueueEnqueueError> {
        let type_id = TypeId::of::<T>();
        let queue = self
            .queues
            .entry(type_id)
            .or_insert_with(QueueStorage::new::<T>);

        if let Some(capacity) = queue.config.capacity {
            if queue.messages_ref::<T>().len() >= capacity {
                queue.rejected = queue.rejected.saturating_add(1);
                return Err(QueueEnqueueError::Backpressure {
                    queue_type: queue.queue_type_name,
                    capacity,
                });
            }
        }

        queue.messages_mut::<T>().push_back(message);
        queue.enqueued = queue.enqueued.saturating_add(1);
        Ok(())
    }

    pub fn queue_iter<T: 'static>(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.queues.get(&TypeId::of::<T>()) {
            Some(queue) => Box::new(queue.messages_ref::<T>().iter()),
            None => Box::new(std::iter::empty()),
        }
    }

    pub fn queue_peek<T: 'static>(&self) -> Option<&T> {
        self.queues
            .get(&TypeId::of::<T>())
            .and_then(|queue| queue.messages_ref::<T>().front())
    }

    pub fn queue_drain<T: 'static>(&mut self) -> Vec<T> {
        let Some(queue) = self.queues.get_mut(&TypeId::of::<T>()) else {
            return Vec::new();
        };

        let mut drained = Vec::with_capacity(queue.messages_ref::<T>().len());
        while let Some(value) = queue.messages_mut::<T>().pop_front() {
            drained.push(value);
        }

        queue.drained = queue.drained.saturating_add(drained.len() as u64);
        drained
    }

    pub fn clear_queue<T: 'static>(&mut self) -> usize {
        let Some(queue) = self.queues.get_mut(&TypeId::of::<T>()) else {
            return 0;
        };
        let removed = queue.clear_any();
        queue.drained = queue.drained.saturating_add(removed as u64);
        removed
    }

    pub fn queue_pending_count<T: 'static>(&self) -> usize {
        self.queues
            .get(&TypeId::of::<T>())
            .map(QueueStorage::messages_len_any)
            .unwrap_or(0)
    }

    pub fn queue_stats<T: 'static>(&self) -> Option<QueueStats> {
        self.queues.get(&TypeId::of::<T>()).map(|queue| QueueStats {
            enqueued: queue.enqueued,
            drained: queue.drained,
            rejected: queue.rejected,
            pending: queue.messages_len_any(),
            capacity: queue.config.capacity,
        })
    }
}
