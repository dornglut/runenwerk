use super::super::edits::operation::WorldOperationRecord;
use super::super::ids::{ChunkGeneration, ChunkId, ChunkRevision, WorldOpId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct ChunkHeaderDelta {
    pub chunk_id: ChunkId,
    pub chunk_revision: ChunkRevision,
    pub chunk_generation: ChunkGeneration,
    pub checksum: u64,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct ChunkContentDelta {
    pub chunk_id: ChunkId,
    pub chunk_revision: ChunkRevision,
    pub page_deltas: Vec<Vec<u8>>,
    pub full_payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct OpWindowDelta {
    pub start_exclusive: WorldOpId,
    pub end_inclusive: WorldOpId,
    pub operations: Vec<WorldOperationRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct ChunkResidencyHint {
    pub chunk_id: ChunkId,
    pub relevant_to_client: bool,
    pub gameplay_locked: bool,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldReplicationStateResource {
    pub pending_header_deltas: BTreeMap<ChunkId, ChunkHeaderDelta>,
    pub pending_content_deltas: BTreeMap<ChunkId, ChunkContentDelta>,
    pub pending_op_windows: Vec<OpWindowDelta>,
}
