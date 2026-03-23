use super::super::ids::ChunkId;
use std::cmp::Ordering;
use std::collections::{BTreeSet, BinaryHeap};

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Resource)]
pub enum WorldBuildQueueClass {
    Interactive,
    Background,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Resource)]
pub struct WorldBuildQueueItem {
    pub chunk_id: ChunkId,
    pub queue_class: WorldBuildQueueClass,
    pub priority_score: i64,
    pub starvation_age: u32,
}

impl Ord for WorldBuildQueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority_score
            .cmp(&other.priority_score)
            .then_with(|| self.starvation_age.cmp(&other.starvation_age))
    }
}

impl PartialOrd for WorldBuildQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Default, ecs::Component, ecs::Resource)]
pub struct WorldBuildQueueResource {
    pub interactive: BinaryHeap<WorldBuildQueueItem>,
    pub background: BinaryHeap<WorldBuildQueueItem>,
    queued_chunks: BTreeSet<ChunkId>,
}

impl WorldBuildQueueResource {
    pub fn enqueue(&mut self, item: WorldBuildQueueItem) {
        if self.queued_chunks.contains(&item.chunk_id) {
            return;
        }
        self.queued_chunks.insert(item.chunk_id);
        match item.queue_class {
            WorldBuildQueueClass::Interactive => self.interactive.push(item),
            WorldBuildQueueClass::Background => self.background.push(item),
        }
    }

    pub fn pop_next(&mut self) -> Option<WorldBuildQueueItem> {
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
