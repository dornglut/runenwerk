use super::super::adapters::resources::{
    OperationLogResource, RegionInvalidationJournalResource, ReplicationStateResource,
    SdfChunkStoreResource,
};
use super::super::chunks::lifecycle::WorldChunkRuntimeMapResource;
use super::super::plugin::WorldAuthorityState;
use crate::runtime::WorldMut;
use ecs::{ChangeExtractionFilter, OwnerState, ResourceTypeKey};
use std::collections::BTreeSet;
use world_ops::{
    ChunkContentDelta, ChunkHeaderDelta, ChunkResidencyHint, OpWindowDelta, OperationId,
    RegionInvalidationDelta,
};

#[derive(Debug, Copy, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldReplicationExtractionCursor {
    pub last_tick: u64,
    pub last_frame: u64,
}

pub fn rebuild_world_replication_state_system(mut world: WorldMut) {
    let current_tick = world.current_change_tick();
    let current_frame = world.current_frame_index();

    let previous_cursor = world
        .resource::<WorldReplicationExtractionCursor>()
        .copied()
        .unwrap_or_default();

    let tracked_resource_keys = [
        world.resource_type_key::<WorldChunkRuntimeMapResource>(),
        world.resource_type_key::<SdfChunkStoreResource>(),
        world.resource_type_key::<OperationLogResource>(),
        world.resource_type_key::<RegionInvalidationJournalResource>(),
    ]
    .into_iter()
    .flatten()
    .collect::<BTreeSet<_>>();

    let tracked_resource_keys_ref = &tracked_resource_keys;
    let resource_key_filter = |key: ResourceTypeKey| tracked_resource_keys_ref.contains(&key);
    let allows_owner =
        |owner: OwnerState| matches!(owner, OwnerState::Unowned | OwnerState::WorldOwned);
    let component_ownership_filter =
        |_: ecs::Entity, owner: OwnerState, _: ecs::ComponentTypeKey| allows_owner(owner);
    let resource_ownership_filter = |_: ResourceTypeKey, owner: OwnerState| allows_owner(owner);

    let extraction = world.extract_structural_deltas(
        ecs::ChangeExtractionWindow {
            tick_start_exclusive: previous_cursor.last_tick,
            tick_end_inclusive: current_tick,
            frame_start_exclusive: u64::MAX,
            frame_end_inclusive: u64::MAX,
        },
        ChangeExtractionFilter {
            component_key_filter: None,
            resource_key_filter: Some(&resource_key_filter),
            component_ownership_filter: Some(&component_ownership_filter),
            resource_ownership_filter: Some(&resource_ownership_filter),
            interest_filter: None,
        },
    );

    if extraction.is_empty() {
        if let Ok(cursor) = world.resource_mut::<WorldReplicationExtractionCursor>() {
            cursor.last_tick = current_tick;
            cursor.last_frame = current_frame;
        }
        return;
    }

    let world_revision = world
        .resource::<WorldAuthorityState>()
        .map(|authority| authority.world_revision)
        .unwrap_or_default();

    let next_op_id = world
        .resource::<OperationLogResource>()
        .map(|op_log| OperationId(op_log.next_op_id))
        .unwrap_or_default();

    let chunk_runtime_records = world
        .resource::<WorldChunkRuntimeMapResource>()
        .map(|chunk_runtime| {
            chunk_runtime
                .by_chunk_id
                .values()
                .map(|record| {
                    (
                        record.chunk_id,
                        record.chunk_revision,
                        record.chunk_generation,
                        record.gameplay_locked,
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let chunk_payloads = world
        .resource::<SdfChunkStoreResource>()
        .map(|sdf_store| sdf_store.chunks.values().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    let op_records = world
        .resource::<OperationLogResource>()
        .map(|op_log| op_log.operations.clone())
        .unwrap_or_default();

    let region_records = world
        .resource::<RegionInvalidationJournalResource>()
        .map(|journal| journal.recent_records.clone())
        .unwrap_or_default();

    let chunk_checksum_by_id = chunk_payloads
        .iter()
        .map(|payload| (payload.chunk_id, payload.checksum))
        .collect::<std::collections::BTreeMap<_, _>>();

    let pending_header_deltas = chunk_runtime_records
        .iter()
        .map(
            |(chunk_id, chunk_revision, chunk_generation, gameplay_locked)| {
                (
                    *chunk_id,
                    ChunkHeaderDelta {
                        chunk_id: *chunk_id,
                        chunk_revision: *chunk_revision,
                        chunk_generation: *chunk_generation,
                        checksum: chunk_checksum_by_id.get(chunk_id).copied().unwrap_or(0),
                        flags: if *gameplay_locked { 1 } else { 0 },
                    },
                )
            },
        )
        .collect::<std::collections::BTreeMap<_, _>>();

    let pending_residency_hints = chunk_runtime_records
        .iter()
        .map(|(chunk_id, _, _, gameplay_locked)| {
            (
                *chunk_id,
                ChunkResidencyHint {
                    chunk_id: *chunk_id,
                    relevant_to_viewer: true,
                    gameplay_locked: *gameplay_locked,
                },
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    let pending_content_deltas = chunk_payloads
        .iter()
        .map(|payload| {
            (
                payload.chunk_id,
                ChunkContentDelta {
                    chunk_id: payload.chunk_id,
                    chunk_revision: payload.chunk_revision,
                    page_deltas: Vec::new(),
                    full_payload: postcard::to_allocvec(payload).ok(),
                },
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    let pending_op_windows =
        if let (Some(first), Some(last)) = (op_records.first(), op_records.last()) {
            vec![OpWindowDelta {
                start_exclusive: OperationId(first.op_id.0.saturating_sub(1)),
                end_inclusive: last.op_id,
                operations: op_records,
            }]
        } else {
            Vec::new()
        };

    let pending_region_invalidations = region_records
        .iter()
        .map(|record| RegionInvalidationDelta {
            sequence: record.sequence,
            source: record.source,
            world_revision: record.world_revision,
            op_id: record.op_id,
            chunk_ids: record.chunk_ids.iter().copied().collect::<Vec<_>>(),
            region_ids: record.region_ids.iter().copied().collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    if let Ok(replication) = world.resource_mut::<ReplicationStateResource>() {
        replication.world_revision = world_revision;
        replication.next_op_id = next_op_id;
        replication.pending_header_deltas = pending_header_deltas;
        replication.pending_residency_hints = pending_residency_hints;
        replication.pending_content_deltas = pending_content_deltas;
        replication.pending_op_windows = pending_op_windows;
        replication.pending_region_invalidations = pending_region_invalidations;
    }

    if let Ok(cursor) = world.resource_mut::<WorldReplicationExtractionCursor>() {
        cursor.last_tick = current_tick;
        cursor.last_frame = current_frame;
    }
}
