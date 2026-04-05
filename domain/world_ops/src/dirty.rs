use serde::{Deserialize, Serialize};
use spatial::ChunkId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DirtyReason {
    Geometry,
    MaterialField,
    Structure,
    Topology,
    ReplicationResync,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DirtyReasonSet {
    pub reasons: BTreeSet<DirtyReason>,
}

impl DirtyReasonSet {
    pub fn is_empty(&self) -> bool {
        self.reasons.is_empty()
    }

    pub fn insert(&mut self, reason: DirtyReason) {
        self.reasons.insert(reason);
    }

    pub fn merge_from(&mut self, other: DirtyReasonSet) {
        self.reasons.extend(other.reasons);
    }
}

#[derive(Debug, Clone, Default)]
pub struct DirtyChunkMap {
    pub by_chunk: BTreeMap<ChunkId, DirtyReasonSet>,
}

impl DirtyChunkMap {
    pub fn mark_dirty(&mut self, chunk_id: ChunkId, reason: DirtyReason) {
        self.by_chunk.entry(chunk_id).or_default().insert(reason);
    }

    pub fn take_reasons(&mut self, chunk_id: &ChunkId) -> Option<DirtyReasonSet> {
        self.by_chunk.remove(chunk_id)
    }
}
