use super::super::adapters::resources::{
    CaveSectorResource, OperationLogResource, RegionInvalidationJournalResource,
};
use super::super::chunks::lifecycle::{ChunkLifecycleState, WorldChunkRuntimeMapResource};
use super::super::debug::metrics::WorldDebugMetricsResource;
use super::super::plugin::WorldAuthorityState;
use super::super::streaming::interest::WorldStreamingInterestResource;
use crate::plugins::render::features::{
    FeatureContributionStatus, FeatureFallbackPolicy, PreparedCaveFeatureResource,
    PreparedDetailFeatureResource, PreparedProceduralWorldFeatureResource,
    PreparedWindFieldFeatureResource, PreparedWorldFeatureResource,
    world::runtime_cache::WorldRuntimeCacheResource,
};
use crate::plugins::render::frame::{
    PreparedCaveFeatureContribution, PreparedDetailCellContribution,
    PreparedDetailFeatureContribution, PreparedProceduralWorldFeatureContribution,
    PreparedWindFieldFeatureContribution, PreparedWorldChunkContribution,
    PreparedWorldDrawBatchRef, PreparedWorldFeatureContribution, PreparedWorldResidencyIntent,
};
use crate::plugins::render::inspect::{
    RenderDebugTimingsState, RenderRuntimeResourceInspectorState, WorldRuntimeInspectorSnapshot,
};
use crate::runtime::WorldMut;

pub fn prepare_world_feature_contributions_system(mut world: WorldMut) {
    let mut visible_chunks = Vec::<PreparedWorldChunkContribution>::new();
    let mut residency_intents = Vec::<PreparedWorldResidencyIntent>::new();
    let mut detail_cells = Vec::<PreparedDetailCellContribution>::new();
    let mut op_log_count = 0_u64;

    if let Ok(chunks) = world.resource::<WorldChunkRuntimeMapResource>() {
        for record in chunks.by_chunk_id.values() {
            if !matches!(
                record.lifecycle,
                ChunkLifecycleState::Ready
                    | ChunkLifecycleState::Resident
                    | ChunkLifecycleState::Rebuilding
            ) {
                continue;
            }
            let chunk_id = record.chunk_id;
            visible_chunks.push(PreparedWorldChunkContribution {
                chunk_id,
                chunk_revision: record.chunk_revision.0,
                chunk_generation: record.chunk_generation.0,
                draw_batch_ref: PreparedWorldDrawBatchRef { chunk_id },
            });
            residency_intents.push(PreparedWorldResidencyIntent {
                chunk_id,
                priority: if record.gameplay_locked { 1000 } else { 100 },
                hard_pin: record.gameplay_locked,
            });
            detail_cells.push(PreparedDetailCellContribution {
                cell_id: format!(
                    "detail.cell.{}:{}:{}:{}",
                    chunk_id.world_id.0, chunk_id.coord.x, chunk_id.coord.y, chunk_id.coord.z
                ),
                chunk_id,
                instance_count: if record.gameplay_locked { 128 } else { 32 },
            });
        }
    }

    if let Ok(log) = world.resource::<OperationLogResource>() {
        op_log_count = log.operations.len() as u64;
    }

    if let Ok(mut world_feature) = world.resource_mut::<PreparedWorldFeatureResource>() {
        world_feature.status = if visible_chunks.is_empty() {
            FeatureContributionStatus::Stale
        } else {
            FeatureContributionStatus::Ready
        };
        world_feature.fallback_policy = FeatureFallbackPolicy::ReuseLastGood;
        world_feature.payload = PreparedWorldFeatureContribution {
            visible_chunks,
            residency_intents,
        };
    }

    let (visible_sector_ids, scoped_light_volume_count) =
        if let Ok(caves) = world.resource::<CaveSectorResource>() {
            (
                caves.visible_sectors.iter().map(|value| value.0).collect(),
                caves.visible_sectors.len() as u32,
            )
        } else {
            (Vec::new(), 0)
        };

    if let Ok(mut cave_feature) = world.resource_mut::<PreparedCaveFeatureResource>() {
        cave_feature.status = if visible_sector_ids.is_empty() {
            FeatureContributionStatus::Stale
        } else {
            FeatureContributionStatus::Ready
        };
        cave_feature.fallback_policy = FeatureFallbackPolicy::ReuseLastGood;
        cave_feature.payload = PreparedCaveFeatureContribution {
            visible_sector_ids,
            scoped_light_volume_count,
        };
    }

    if let Ok(mut detail_feature) = world.resource_mut::<PreparedDetailFeatureResource>() {
        detail_feature.status = if detail_cells.is_empty() {
            FeatureContributionStatus::Stale
        } else {
            FeatureContributionStatus::Ready
        };
        detail_feature.fallback_policy = FeatureFallbackPolicy::ReuseLastGood;
        detail_feature.payload = PreparedDetailFeatureContribution {
            cells: detail_cells,
        };
    }

    if let Ok(mut procedural_feature) =
        world.resource_mut::<PreparedProceduralWorldFeatureResource>()
    {
        procedural_feature.status = FeatureContributionStatus::Missing;
        procedural_feature.fallback_policy = FeatureFallbackPolicy::SkipFeaturePasses;
        procedural_feature.payload = PreparedProceduralWorldFeatureContribution::default();
    }

    if let Ok(mut wind_feature) = world.resource_mut::<PreparedWindFieldFeatureResource>() {
        wind_feature.status = FeatureContributionStatus::Missing;
        wind_feature.fallback_policy = FeatureFallbackPolicy::SkipFeaturePasses;
        wind_feature.payload = PreparedWindFieldFeatureContribution::default();
    }

    if let Ok(mut metrics) = world.resource_mut::<WorldDebugMetricsResource>() {
        metrics.op_log_count = op_log_count;
    }

    let runtime_inspector_values = match (
        world.resource::<WorldRuntimeCacheResource>(),
        world.resource::<WorldDebugMetricsResource>(),
    ) {
        (Ok(runtime_cache), Ok(world_metrics)) => Some((
            runtime_cache.by_chunk.len(),
            runtime_cache.stale_chunks.len(),
            world_metrics.residency_misses,
            world_metrics.interactive_queue_depth,
            world_metrics.background_queue_depth,
        )),
        _ => None,
    };

    if let (Some((resident, stale, page_miss, interactive, background)), Ok(mut inspector)) = (
        runtime_inspector_values,
        world.resource_mut::<RenderRuntimeResourceInspectorState>(),
    ) {
        inspector.observe_world_runtime(resident, stale, page_miss, interactive, background);
    }

    if let Ok(mut timings) = world.resource_mut::<RenderDebugTimingsState>() {
        timings.observe_world_prepare_sample();
    }

    let streaming_snapshot_values =
        if let Ok(streaming_interest) = world.resource::<WorldStreamingInterestResource>() {
            let mut needs_resync_count = 0_usize;
            let mut max_cursor_lag = 0_u64;
            let mut max_region_sequence_lag = 0_u64;
            for connection in streaming_interest.per_connection.values() {
                if connection.needs_full_resync {
                    needs_resync_count = needs_resync_count.saturating_add(1);
                }
                max_cursor_lag = max_cursor_lag.max(
                    connection
                        .last_sent_cursor
                        .0
                        .saturating_sub(connection.last_ack_cursor.0),
                );
                max_region_sequence_lag = max_region_sequence_lag.max(
                    connection
                        .prepared_region_sequence
                        .saturating_sub(connection.acked_region_sequence),
                );
            }
            Some((
                streaming_interest.per_connection.len(),
                needs_resync_count,
                max_cursor_lag,
                max_region_sequence_lag,
            ))
        } else {
            None
        };

    let region_journal_snapshot_values =
        if let Ok(journal) = world.resource::<RegionInvalidationJournalResource>() {
            Some((
                journal
                    .recent_records
                    .back()
                    .map(|record| record.sequence)
                    .unwrap_or(0),
                journal.recent_records.len(),
            ))
        } else {
            None
        };

    let world_snapshot_values = match (
        world.resource::<WorldChunkRuntimeMapResource>(),
        world.resource::<WorldDebugMetricsResource>(),
        world.resource::<WorldAuthorityState>(),
    ) {
        (Ok(chunk_runtime), Ok(world_metrics), Ok(authority)) => Some((
            chunk_runtime
                .by_chunk_id
                .values()
                .filter(|value| matches!(value.lifecycle, ChunkLifecycleState::Dirty))
                .count(),
            world_metrics.interactive_queue_depth,
            world_metrics.background_queue_depth,
            world_metrics.integrated_build_outputs,
            world_metrics.dropped_stale_build_outputs,
            world_metrics.op_log_count,
            world_metrics.ingress_operations,
            world_metrics.invalidated_chunks,
            world_metrics.collision_queries,
            world_metrics.collision_authority_misses,
            authority.world_revision.0,
            streaming_snapshot_values.unwrap_or((0, 0, 0, 0)),
            region_journal_snapshot_values.unwrap_or((0, 0)),
        )),
        _ => None,
    };

    if let (
        Some((
            chunk_dirty_count,
            queued_interactive,
            queued_background,
            integrated,
            dropped,
            op_log_count,
            ingress_operations,
            invalidated_chunks,
            collision_queries,
            collision_authority_misses,
            world_revision,
            (
                streaming_connection_count,
                streaming_needs_resync_count,
                streaming_max_cursor_lag,
                streaming_max_region_sequence_lag,
            ),
            (region_journal_latest_sequence, region_journal_record_count),
        )),
        Ok(mut snapshot),
    ) = (
        world_snapshot_values,
        world.resource_mut::<WorldRuntimeInspectorSnapshot>(),
    ) {
        snapshot.chunk_dirty_count = chunk_dirty_count;
        snapshot.queued_interactive = queued_interactive;
        snapshot.queued_background = queued_background;
        snapshot.integrated_build_outputs = integrated;
        snapshot.dropped_stale_outputs = dropped;
        snapshot.op_log_count = op_log_count;
        snapshot.ingress_operations = ingress_operations;
        snapshot.invalidated_chunks = invalidated_chunks;
        snapshot.collision_queries = collision_queries;
        snapshot.collision_authority_misses = collision_authority_misses;
        snapshot.world_revision = world_revision;
        snapshot.streaming_connection_count = streaming_connection_count;
        snapshot.streaming_needs_resync_count = streaming_needs_resync_count;
        snapshot.streaming_max_cursor_lag = streaming_max_cursor_lag;
        snapshot.streaming_max_region_sequence_lag = streaming_max_region_sequence_lag;
        snapshot.region_journal_latest_sequence = region_journal_latest_sequence;
        snapshot.region_journal_record_count = region_journal_record_count;
    }
}
