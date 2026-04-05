use crate::{
    ChunkGeneration, ChunkRevision, OperationId, OperationRecord, RegionInvalidationSource,
    WorldRevision,
};
use serde::{Deserialize, Serialize};
use spatial::{ChunkId, RegionId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkHeaderDelta {
    pub chunk_id: ChunkId,
    pub chunk_revision: ChunkRevision,
    pub chunk_generation: ChunkGeneration,
    pub checksum: u64,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkContentDelta {
    pub chunk_id: ChunkId,
    pub chunk_revision: ChunkRevision,
    pub page_deltas: Vec<Vec<u8>>,
    pub full_payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpWindowDelta {
    pub start_exclusive: OperationId,
    pub end_inclusive: OperationId,
    pub operations: Vec<OperationRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkResidencyHint {
    pub chunk_id: ChunkId,
    pub relevant_to_client: bool,
    pub gameplay_locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionInvalidationDelta {
    pub sequence: u64,
    pub source: RegionInvalidationSource,
    pub world_revision: WorldRevision,
    pub op_id: Option<OperationId>,
    pub chunk_ids: Vec<ChunkId>,
    pub region_ids: Vec<RegionId>,
}

#[derive(Debug, Clone, Default)]
pub struct ReplicationState {
    pub world_revision: WorldRevision,
    pub next_op_id: OperationId,
    pub pending_header_deltas: BTreeMap<ChunkId, ChunkHeaderDelta>,
    pub pending_content_deltas: BTreeMap<ChunkId, ChunkContentDelta>,
    pub pending_op_windows: Vec<OpWindowDelta>,
    pub pending_residency_hints: BTreeMap<ChunkId, ChunkResidencyHint>,
    pub pending_region_invalidations: Vec<RegionInvalidationDelta>,
}
