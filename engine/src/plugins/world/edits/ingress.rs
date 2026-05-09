use super::super::adapters::resources::{
    OperationLogResource, PartitionConfigResource, RegionInvalidationJournalResource,
};
use super::super::chunks::DirtyChunkMapResource;
use super::super::chunks::render_cache_bridge::WorldRenderCacheInvalidationQueueResource;
use super::super::debug::metrics::WorldDebugMetricsResource;
use super::super::{WorldAuthorityState, WorldRuntimeConfig, WorldRuntimeMode};
use ecs::World;
use spatial::WorldId;
use world_ops::{
    Operation, OperationId, OperationRecord, QuantizedAabb, dirty_reason_for_operation,
    mark_dirty_chunks_from_quantized_bounds,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WorldEditIngressMeta {
    pub planet_id: WorldId,
    pub deterministic_seed: u64,
}

impl Default for WorldEditIngressMeta {
    fn default() -> Self {
        Self {
            planet_id: WorldId(0),
            deterministic_seed: 0,
        }
    }
}

pub fn submit_world_operation(
    world: &mut World,
    operation: Operation,
    affected_bounds_q: QuantizedAabb,
    meta: WorldEditIngressMeta,
) -> Option<OperationId> {
    if !world_runtime_is_authoritative(world) {
        return None;
    }

    let base_world_revision = world
        .resource::<WorldAuthorityState>()
        .map(|value| value.world_revision)
        .unwrap_or_default();
    let dirty_reason = dirty_reason_for_operation(&operation);

    let op_id = {
        let op_log = world.resource_mut::<OperationLogResource>().ok()?;
        op_log.append(OperationRecord {
            op_id: OperationId(0),
            base_world_revision,
            planet_id: meta.planet_id,
            operation,
            affected_bounds_q,
            deterministic_seed: meta.deterministic_seed,
        })
    };

    let partition = world.resource::<PartitionConfigResource>().ok()?.clone();
    let fixed_point_scale = partition.quantization_scale();
    let touched_chunks = {
        let dirty = world.resource_mut::<DirtyChunkMapResource>().ok()?;
        mark_dirty_chunks_from_quantized_bounds(
            dirty,
            &partition,
            affected_bounds_q,
            meta.planet_id,
            fixed_point_scale,
            dirty_reason,
        )
    };
    let invalidated_chunk_count = touched_chunks.len() as u64;

    if let Ok(queue) = world.resource_mut::<WorldRenderCacheInvalidationQueueResource>() {
        queue.enqueue_ingress_bounds(&partition, touched_chunks.clone());
    }
    if let Ok(journal) = world.resource_mut::<RegionInvalidationJournalResource>() {
        journal.append_ingress_record(&partition, touched_chunks, base_world_revision, op_id);
    }

    let op_count = world
        .resource::<OperationLogResource>()
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
        .map(|config| matches!(config.mode, WorldRuntimeMode::Writable))
        .unwrap_or(true)
}
