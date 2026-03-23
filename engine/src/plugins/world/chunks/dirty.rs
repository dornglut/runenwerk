use super::super::ids::ChunkId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ecs::Resource,
)]
pub enum ChunkDirtyReason {
    Geometry,
    MaterialField,
    Structure,
    Topology,
    ReplicationResync,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ecs::Resource)]
pub struct ChunkDirtyReasonSet {
    pub reasons: BTreeSet<ChunkDirtyReason>,
}

impl ChunkDirtyReasonSet {
    pub fn is_empty(&self) -> bool {
        self.reasons.is_empty()
    }

    pub fn insert(&mut self, reason: ChunkDirtyReason) {
        self.reasons.insert(reason);
    }
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldDirtyChunkMapResource {
    pub by_chunk: BTreeMap<ChunkId, ChunkDirtyReasonSet>,
}

impl WorldDirtyChunkMapResource {
    pub fn mark_dirty(&mut self, chunk_id: ChunkId, reason: ChunkDirtyReason) {
        self.by_chunk.entry(chunk_id).or_default().insert(reason);
    }

    pub fn take_reasons(&mut self, chunk_id: &ChunkId) -> Option<ChunkDirtyReasonSet> {
        self.by_chunk.remove(chunk_id)
    }
}
