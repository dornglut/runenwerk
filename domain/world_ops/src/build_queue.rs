use spatial::ChunkId;
use std::cmp::Ordering;
use std::collections::{BTreeSet, BinaryHeap};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuildQueueClass {
    Interactive,
    Background,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BuildQueueItem {
    pub chunk_id: ChunkId,
    pub queue_class: BuildQueueClass,
    pub priority_score: i64,
    pub starvation_age: u32,
}

impl Ord for BuildQueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority_score
            .cmp(&other.priority_score)
            .then_with(|| self.starvation_age.cmp(&other.starvation_age))
    }
}

impl PartialOrd for BuildQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Default, Clone)]
pub struct BuildQueue {
    pub interactive: BinaryHeap<BuildQueueItem>,
    pub background: BinaryHeap<BuildQueueItem>,
    queued_chunks: BTreeSet<ChunkId>,
}

impl BuildQueue {
    pub fn enqueue(&mut self, item: BuildQueueItem) {
        if self.queued_chunks.contains(&item.chunk_id) {
            return;
        }
        self.queued_chunks.insert(item.chunk_id);
        match item.queue_class {
            BuildQueueClass::Interactive => self.interactive.push(item),
            BuildQueueClass::Background => self.background.push(item),
        }
    }

    pub fn pop_next(&mut self) -> Option<BuildQueueItem> {
        let next = self.interactive.pop().or_else(|| self.background.pop());
        if let Some(item) = next {
            self.queued_chunks.remove(&item.chunk_id);
            return Some(item);
        }
        None
    }

    pub fn contains_chunk(&self, chunk_id: ChunkId) -> bool {
        self.queued_chunks.contains(&chunk_id)
    }

    pub fn queue_depths(&self) -> (usize, usize) {
        (self.interactive.len(), self.background.len())
    }
}
