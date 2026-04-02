use super::super::chunks::lifecycle::WorldChunkRuntimeMapResource;
use super::super::edits::log::WorldOperationLog;
use super::super::edits::operation::WorldOperationRecord;
use super::super::edits::region_journal::{
    WorldRegionInvalidationJournalResource, WorldRegionInvalidationSource,
};
use super::super::ids::{
    ChunkGeneration, ChunkId, ChunkRevision, RegionId, WorldOpId, WorldRevision,
};
use super::super::plugin::WorldAuthorityState;
use super::super::sdf::storage::WorldSdfChunkStoreResource;
use crate::runtime::{Res, ResMut};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct RegionInvalidationDelta {
    pub sequence: u64,
    pub source: WorldRegionInvalidationSource,
    pub world_revision: WorldRevision,
    pub op_id: Option<WorldOpId>,
    pub chunk_ids: Vec<ChunkId>,
    pub region_ids: Vec<RegionId>,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldReplicationStateResource {
    pub world_revision: WorldRevision,
    pub next_op_id: WorldOpId,
    pub pending_header_deltas: BTreeMap<ChunkId, ChunkHeaderDelta>,
    pub pending_content_deltas: BTreeMap<ChunkId, ChunkContentDelta>,
    pub pending_op_windows: Vec<OpWindowDelta>,
    pub pending_residency_hints: BTreeMap<ChunkId, ChunkResidencyHint>,
    pub pending_region_invalidations: Vec<RegionInvalidationDelta>,
}

pub fn rebuild_world_replication_state_system(
    chunk_runtime: Res<WorldChunkRuntimeMapResource>,
    sdf_store: Res<WorldSdfChunkStoreResource>,
    op_log: Res<WorldOperationLog>,
    region_invalidation_journal: Res<WorldRegionInvalidationJournalResource>,
    authority: Res<WorldAuthorityState>,
    mut replication: ResMut<WorldReplicationStateResource>,
) {
    replication.world_revision = authority.world_revision;
    replication.next_op_id = WorldOpId(op_log.next_op_id);

    replication.pending_header_deltas.clear();
    replication.pending_residency_hints.clear();
    for record in chunk_runtime.by_chunk_id.values() {
        let checksum = sdf_store
            .chunks
            .get(&record.chunk_id)
            .map(|payload| payload.checksum)
            .unwrap_or(0);
        replication.pending_header_deltas.insert(
            record.chunk_id,
            ChunkHeaderDelta {
                chunk_id: record.chunk_id,
                chunk_revision: record.chunk_revision,
                chunk_generation: record.chunk_generation,
                checksum,
                flags: if record.gameplay_locked { 1 } else { 0 },
            },
        );
        replication.pending_residency_hints.insert(
            record.chunk_id,
            ChunkResidencyHint {
                chunk_id: record.chunk_id,
                relevant_to_client: true,
                gameplay_locked: record.gameplay_locked,
            },
        );
    }

    replication.pending_content_deltas.clear();
    for payload in sdf_store.chunks.values() {
        replication.pending_content_deltas.insert(
            payload.chunk_id,
            ChunkContentDelta {
                chunk_id: payload.chunk_id,
                chunk_revision: payload.chunk_revision,
                page_deltas: Vec::new(),
                full_payload: postcard::to_allocvec(payload).ok(),
            },
        );
    }

    replication.pending_op_windows.clear();
    if let (Some(first), Some(last)) = (op_log.operations.first(), op_log.operations.last()) {
        replication.pending_op_windows.push(OpWindowDelta {
            start_exclusive: WorldOpId(first.op_id.0.saturating_sub(1)),
            end_inclusive: last.op_id,
            operations: op_log.operations.clone(),
        });
    }

    replication.pending_region_invalidations.clear();
    for record in &region_invalidation_journal.recent_records {
        replication
            .pending_region_invalidations
            .push(RegionInvalidationDelta {
                sequence: record.sequence,
                source: record.source,
                world_revision: record.world_revision,
                op_id: record.op_id,
                chunk_ids: record.chunk_ids.iter().copied().collect::<Vec<_>>(),
                region_ids: record.region_ids.iter().copied().collect::<Vec<_>>(),
            });
    }
}
