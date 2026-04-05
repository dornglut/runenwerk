use super::super::adapters::resources::{
    OperationLogResource, RegionInvalidationJournalResource, ReplicationStateResource,
    SdfChunkStoreResource,
};
use super::super::chunks::lifecycle::WorldChunkRuntimeMapResource;
use super::super::plugin::WorldAuthorityState;
use crate::runtime::{Res, ResMut};
use world_ops::{
    ChunkContentDelta, ChunkHeaderDelta, ChunkResidencyHint, OpWindowDelta, OperationId,
    RegionInvalidationDelta,
};

pub fn rebuild_world_replication_state_system(
    chunk_runtime: Res<WorldChunkRuntimeMapResource>,
    sdf_store: Res<SdfChunkStoreResource>,
    op_log: Res<OperationLogResource>,
    region_invalidation_journal: Res<RegionInvalidationJournalResource>,
    authority: Res<WorldAuthorityState>,
    mut replication: ResMut<ReplicationStateResource>,
) {
    replication.world_revision = authority.world_revision;
    replication.next_op_id = OperationId(op_log.next_op_id);

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
            start_exclusive: OperationId(first.op_id.0.saturating_sub(1)),
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
