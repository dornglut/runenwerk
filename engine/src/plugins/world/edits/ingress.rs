use super::super::chunks::dirty::WorldDirtyChunkMapResource;
use super::super::chunks::partition::WorldPartitionConfig;
use super::super::chunks::render_cache_bridge::WorldRenderCacheInvalidationQueueResource;
use super::super::debug::metrics::WorldDebugMetricsResource;
use super::super::ids::{PlanetId, WorldOpId};
use super::super::{WorldAuthorityState, WorldRuntimeConfig, WorldRuntimeMode};
use super::invalidation::invalidate_dirty_chunks_from_quantized_bounds;
use super::log::WorldOperationLog;
use super::operation::{QuantizedAabb, WorldOperation, WorldOperationRecord};
use super::region_journal::WorldRegionInvalidationJournalResource;
use ecs::World;
use engine_sim::SimulationTick;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WorldEditIngressMeta {
    pub planet_id: PlanetId,
    pub deterministic_seed: u64,
    pub server_tick: SimulationTick,
    pub author_connection_id: Option<u64>,
}

impl Default for WorldEditIngressMeta {
    fn default() -> Self {
        Self {
            planet_id: PlanetId(0),
            deterministic_seed: 0,
            server_tick: SimulationTick::default(),
            author_connection_id: None,
        }
    }
}

pub fn submit_world_operation(
    world: &mut World,
    operation: WorldOperation,
    affected_bounds_q: QuantizedAabb,
    meta: WorldEditIngressMeta,
) -> Option<WorldOpId> {
    if !world_runtime_is_authoritative(world) {
        return None;
    }

    let base_world_revision = world
        .resource::<WorldAuthorityState>()
        .map(|value| value.world_revision)
        .unwrap_or_default();

    let op_id = {
        let op_log = world.resource_mut::<WorldOperationLog>().ok()?;
        op_log.append(WorldOperationRecord {
            op_id: WorldOpId(0),
            base_world_revision,
            planet_id: meta.planet_id,
            operation,
            affected_bounds_q,
            deterministic_seed: meta.deterministic_seed,
            server_tick: meta.server_tick,
            author_connection_id: meta.author_connection_id,
        })
    };

    let partition = world.resource::<WorldPartitionConfig>().ok()?.clone();
    let fixed_point_scale = partition.quantization_scale();
    let touched_chunks = {
        let dirty = world.resource_mut::<WorldDirtyChunkMapResource>().ok()?;
        invalidate_dirty_chunks_from_quantized_bounds(
            dirty,
            &partition,
            affected_bounds_q,
            meta.planet_id,
            fixed_point_scale,
        )
    };
    let invalidated_chunk_count = touched_chunks.len() as u64;

    if let Ok(queue) = world.resource_mut::<WorldRenderCacheInvalidationQueueResource>() {
        queue.enqueue_ingress_bounds(&partition, touched_chunks.clone());
    }
    if let Ok(journal) = world.resource_mut::<WorldRegionInvalidationJournalResource>() {
        journal.append_ingress_record(&partition, touched_chunks, base_world_revision, op_id);
    }

    let op_count = world
        .resource::<WorldOperationLog>()
        .map(|op_log| op_log.operations.len() as u64)
        .ok();
    if let (Some(op_count), Ok(metrics)) =
        (op_count, world.resource_mut::<WorldDebugMetricsResource>())
    {
        metrics.op_log_count = op_count;
        metrics.ingress_operations = metrics.ingress_operations.saturating_add(1);
        metrics.invalidated_chunks = metrics
            .invalidated_chunks
            .saturating_add(invalidated_chunk_count);
    }

    Some(op_id)
}

fn world_runtime_is_authoritative(world: &World) -> bool {
    world
        .resource::<WorldRuntimeConfig>()
        .map(|config| matches!(config.mode, WorldRuntimeMode::ServerAuthoritative))
        .unwrap_or(true)
}
