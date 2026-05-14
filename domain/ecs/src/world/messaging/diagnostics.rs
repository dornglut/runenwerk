use crate::world::World;
use std::any::TypeId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BroadcastKey(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WorkQueueKey(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TickBufferKey(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BroadcastDiagnosticsSnapshot {
    pub key: BroadcastKey,
    pub stream_type: &'static str,
    pub emitted: u64,
    pub drained: u64,
    pub dropped: u64,
    pub pending: usize,
    pub consumer_reads: u64,
    pub consumer_lagged_reads: u64,
    pub consumer_missed_messages: u64,
    pub consumer_lag_latest: u64,
    pub consumer_lag_max: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkQueueDiagnosticsSnapshot {
    pub key: WorkQueueKey,
    pub work_queue_type: &'static str,
    pub enqueued: u64,
    pub drained: u64,
    pub rejected: u64,
    pub pending: usize,
    pub capacity: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TickBufferDiagnosticsSnapshot {
    pub key: TickBufferKey,
    pub buffer_type: &'static str,
    pub pushed: u64,
    pub drained: u64,
    pub dropped: u64,
    pub rejected: u64,
    pub pending_messages: usize,
    pub pending_ticks: usize,
    pub capacity: Option<usize>,
    pub latest_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessagingDiagnosticsSnapshot {
    pub broadcasts: Vec<BroadcastDiagnosticsSnapshot>,
    pub work_queues: Vec<WorkQueueDiagnosticsSnapshot>,
    pub tick_buffers: Vec<TickBufferDiagnosticsSnapshot>,
}

impl World {
    pub fn broadcast_key<T: 'static>(&self) -> Option<BroadcastKey> {
        self.broadcast_streams
            .get(&TypeId::of::<T>())
            .map(|stream| stream.stream_key)
    }

    pub fn work_queue_key<T: 'static>(&self) -> Option<WorkQueueKey> {
        self.work_queues
            .get(&TypeId::of::<T>())
            .map(|queue| queue.work_queue_key)
    }

    pub fn tick_buffer_key<T: 'static>(&self) -> Option<TickBufferKey> {
        self.tick_buffers
            .get(&TypeId::of::<T>())
            .map(|buffer| buffer.buffer_key)
    }

    pub fn messaging_diagnostics_snapshot(&self) -> MessagingDiagnosticsSnapshot {
        let mut broadcasts = self
            .broadcast_streams
            .values()
            .map(|stream| BroadcastDiagnosticsSnapshot {
                key: stream.stream_key,
                stream_type: stream.stream_type_name,
                emitted: stream.emitted,
                drained: stream.drained,
                dropped: stream.dropped,
                pending: stream.messages_len_any(),
                consumer_reads: stream.consumer_reads,
                consumer_lagged_reads: stream.consumer_lagged_reads,
                consumer_missed_messages: stream.consumer_missed_messages,
                consumer_lag_latest: stream.consumer_lag_latest,
                consumer_lag_max: stream.consumer_lag_max,
            })
            .collect::<Vec<_>>();
        broadcasts.sort_by_key(|entry| entry.key);

        let mut work_queues = self
            .work_queues
            .values()
            .map(|queue| WorkQueueDiagnosticsSnapshot {
                key: queue.work_queue_key,
                work_queue_type: queue.work_queue_type_name,
                enqueued: queue.enqueued,
                drained: queue.drained,
                rejected: queue.rejected,
                pending: queue.messages_len_any(),
                capacity: queue.config.capacity,
            })
            .collect::<Vec<_>>();
        work_queues.sort_by_key(|entry| entry.key);

        let mut tick_buffers = self
            .tick_buffers
            .values()
            .map(|buffer| TickBufferDiagnosticsSnapshot {
                key: buffer.buffer_key,
                buffer_type: buffer.buffer_type_name,
                pushed: buffer.pushed,
                drained: buffer.drained,
                dropped: buffer.dropped,
                rejected: buffer.rejected,
                pending_messages: buffer.pending_messages,
                pending_ticks: buffer.bucket_count_any(),
                capacity: buffer.config.capacity,
                latest_sequence: buffer.next_sequence,
            })
            .collect::<Vec<_>>();
        tick_buffers.sort_by_key(|entry| entry.key);

        MessagingDiagnosticsSnapshot {
            broadcasts,
            work_queues,
            tick_buffers,
        }
    }
}
