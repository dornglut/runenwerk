use super::super::chunks::dirty::ChunkDirtyReasonSet;
use super::super::chunks::lifecycle::{ChunkLifecycleState, WorldChunkRuntimeMapResource};
use super::super::chunks::partition::WorldPartitionConfig;
use super::super::chunks::render_cache_bridge::WorldRenderCacheInvalidationQueueResource;
use super::super::debug::metrics::WorldDebugMetricsResource;
use super::super::ids::{BuildGeneration, ChunkId, ChunkRevision, WorldRevision};
use super::super::plugin::{WorldAuthorityState, WorldRuntimeState};
use super::super::sdf::storage::{RegionSdfSummary, SdfChunkPayload, WorldSdfChunkStoreResource};
use super::jobs::WorldBuildStaleness;
use crate::runtime::{Res, ResMut};
use std::collections::VecDeque;

#[derive(Debug, Clone, ecs::Resource)]
pub struct WorldCompletedBuildOutput {
    pub chunk_id: ChunkId,
    pub target_chunk_revision: ChunkRevision,
    pub target_build_generation: BuildGeneration,
    pub staleness: WorldBuildStaleness,
    pub chunk_payload: SdfChunkPayload,
    pub region_summary: RegionSdfSummary,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct WorldCompletedBuildQueueResource {
    pub outputs: VecDeque<WorldCompletedBuildOutput>,
}

pub fn integrate_completed_build_outputs_system(
    mut completed: ResMut<WorldCompletedBuildQueueResource>,
    mut chunks: ResMut<WorldChunkRuntimeMapResource>,
    mut sdf_store: ResMut<WorldSdfChunkStoreResource>,
    partition: Res<WorldPartitionConfig>,
    mut runtime: ResMut<WorldRuntimeState>,
    mut authority: ResMut<WorldAuthorityState>,
    mut render_cache_invalidation: ResMut<WorldRenderCacheInvalidationQueueResource>,
    mut debug: ResMut<WorldDebugMetricsResource>,
) {
    let mut integrated = 0_u64;
    let mut dropped = 0_u64;

    while let Some(output) = completed.outputs.pop_front() {
        if !matches!(output.staleness, WorldBuildStaleness::Current) {
            dropped = dropped.saturating_add(1);
            continue;
        }

        let record = chunks.ensure_chunk(output.chunk_id);
        if record.pending_build_generation != Some(output.target_build_generation) {
            dropped = dropped.saturating_add(1);
            continue;
        }

        record.chunk_revision = output.target_chunk_revision;
        record.chunk_generation = output.chunk_payload.chunk_generation;
        record.build_generation = output.target_build_generation;
        record.pending_build_generation = None;
        record.dirty_reasons = ChunkDirtyReasonSet::default();
        if !matches!(record.lifecycle, ChunkLifecycleState::Resident) {
            record.lifecycle = ChunkLifecycleState::Ready;
        }

        sdf_store
            .chunks
            .insert(output.chunk_id, output.chunk_payload);
        let region_id = partition.region_id_from_chunk_id(output.chunk_id);
        sdf_store
            .region_summaries
            .insert(region_id, output.region_summary);
        render_cache_invalidation.enqueue(output.chunk_id);
        integrated = integrated.saturating_add(1);
    }

    runtime.integrated_build_outputs = runtime.integrated_build_outputs.saturating_add(integrated);
    runtime.dropped_stale_build_outputs =
        runtime.dropped_stale_build_outputs.saturating_add(dropped);
    if integrated > 0 {
        authority.world_revision =
            WorldRevision(authority.world_revision.0.saturating_add(integrated));
    }
    debug.integrated_build_outputs = debug.integrated_build_outputs.saturating_add(integrated);
    debug.dropped_stale_build_outputs = debug.dropped_stale_build_outputs.saturating_add(dropped);
}
