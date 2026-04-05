use serde::{Deserialize, Serialize};
use spatial::ChunkId;
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub enum ChunkNavState {
    Unknown,
    Blocked,
    Traversable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct ChunkNavSummary {
    pub chunk_id: ChunkId,
    pub state: ChunkNavState,
    pub clearance_meters: f32,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldNavSummaryResource {
    pub by_chunk: BTreeMap<ChunkId, ChunkNavSummary>,
}

impl WorldNavSummaryResource {
    pub fn summary_for_chunk(&self, chunk_id: ChunkId) -> Option<&ChunkNavSummary> {
        self.by_chunk.get(&chunk_id)
    }
}
