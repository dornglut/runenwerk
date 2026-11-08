use super::super::adapters::resources::{
    OperationLogResource, PartitionConfigResource, RegionInvalidationJournalResource,
    WorldQuantizationScaleResource,
};
use super::super::chunks::DirtyChunkMapResource;
use super::super::chunks::render_cache_bridge::WorldRenderCacheInvalidationQueueResource;
use super::super::debug::metrics::WorldDebugMetricsResource;
use super::super::{WorldAuthorityState, WorldRuntimeConfig, WorldRuntimeMode};
use runen_ecs::World;
use runen_spatial::WorldId;
use world_ops::{
    Operation, OperationId, OperationRecord, QuantizedAabb, dirty_reason_for_operation,
    touched_chunks_from_quantized_bounds,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WorldEditIngressMeta {
    pub planet_id: WorldId,
    pub deterministic_seed: u64,
}

impl Default for WorldEditIngressMeta {
    fn default() -> Self {
        Self {
            planet_id: WorldId::new(0),
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

    let base_world_revision = world.resource::<WorldAuthorityState>().ok()?.world_revision;
    let dirty_reason = dirty_reason_for_operation(&operation);
    let partition = world.resource::<PartitionConfigResource>().ok()?.clone();
    let quantization_scale = **world.resource::<WorldQuantizationScaleResource>().ok()?;
    let touched_chunks = touched_chunks_from_quantized_bounds(
        &partition,
        affected_bounds_q,
        meta.planet_id,
        quantization_scale,
    )
    .ok()?;

    // The operation, dirtiness, and region invalidation journal form one accepted edit.
    // Missing required owners must reject before the operation ID is committed.
    if world.resource::<OperationLogResource>().is_err()
        || world.resource::<DirtyChunkMapResource>().is_err()
        || world
            .resource::<RegionInvalidationJournalResource>()
            .is_err()
    {
        return None;
    }

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

    {
        let dirty = world.resource_mut::<DirtyChunkMapResource>().ok()?;
        for chunk_id in touched_chunks.iter().copied() {
            dirty.mark_dirty(chunk_id, dirty_reason);
        }
    }
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
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use world_ops::{WorldQuantizationScale, quantize_aabb};

    fn configured_world() -> World {
        let mut world = World::new();
        world.insert_resource(WorldRuntimeConfig {
            mode: WorldRuntimeMode::Writable,
        });
        world.insert_resource(PartitionConfigResource::default());
        world.insert_resource(WorldQuantizationScaleResource(
            WorldQuantizationScale::try_new(1024).expect("valid test quantization scale"),
        ));
        world.insert_resource(OperationLogResource::default());
        world.insert_resource(DirtyChunkMapResource::default());
        world.insert_resource(RegionInvalidationJournalResource::default());
        world.insert_resource(WorldAuthorityState::default());
        world
    }

    fn submit_sample(world: &mut World) -> Option<OperationId> {
        let scale = WorldQuantizationScale::try_new(1024).expect("valid test quantization scale");
        submit_world_operation(
            world,
            Operation::Stamp {
                stamp_id: "tests.world-ingress-preflight".to_string(),
                anchor_q: Default::default(),
                payload: vec![1, 2, 3],
            },
            quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], scale),
            WorldEditIngressMeta::default(),
        )
    }

    #[test]
    fn missing_world_authority_rejects_without_mutating_operation_or_dirty_state() {
        let mut world = configured_world();
        world
            .remove_resource::<WorldRuntimeConfig>()
            .expect("configured world must have authority");

        assert_eq!(submit_sample(&mut world), None);
        let operations = world
            .resource::<OperationLogResource>()
            .expect("operation log must remain available");
        assert!(operations.operations.is_empty());
        assert!(operations.by_id.is_empty());
        assert_eq!(operations.next_op_id, 0);
        assert!(
            world
                .resource::<DirtyChunkMapResource>()
                .expect("dirty map must remain available")
                .by_chunk
                .is_empty()
        );
    }

    #[test]
    fn missing_dirty_map_rejects_before_operation_log_append() {
        let mut world = configured_world();
        world
            .remove_resource::<DirtyChunkMapResource>()
            .expect("configured world must have dirty map");

        assert_eq!(submit_sample(&mut world), None);
        let operations = world
            .resource::<OperationLogResource>()
            .expect("operation log must remain available");
        assert!(operations.operations.is_empty());
        assert!(operations.by_id.is_empty());
        assert_eq!(operations.next_op_id, 0);
    }

    #[test]
    fn missing_operation_log_rejects_before_dirty_mutation() {
        let mut world = configured_world();
        world
            .remove_resource::<OperationLogResource>()
            .expect("configured world must have operation log");

        assert_eq!(submit_sample(&mut world), None);
        assert!(
            world
                .resource::<DirtyChunkMapResource>()
                .expect("dirty map must remain available")
                .by_chunk
                .is_empty()
        );
    }

    #[test]
    fn missing_world_revision_state_rejects_before_operation_append() {
        let mut world = configured_world();
        world
            .remove_resource::<WorldAuthorityState>()
            .expect("configured world must have revision authority");

        assert_eq!(submit_sample(&mut world), None);
        let operations = world
            .resource::<OperationLogResource>()
            .expect("operation log must remain available");
        assert!(operations.operations.is_empty());
        assert_eq!(operations.next_op_id, 0);
        assert!(
            world
                .resource::<DirtyChunkMapResource>()
                .expect("dirty map must remain available")
                .by_chunk
                .is_empty()
        );
    }

    #[test]
    fn missing_region_journal_rejects_before_operation_or_dirty_mutation() {
        let mut world = configured_world();
        world
            .remove_resource::<RegionInvalidationJournalResource>()
            .expect("configured world must have region journal");

        assert_eq!(submit_sample(&mut world), None);
        let operations = world
            .resource::<OperationLogResource>()
            .expect("operation log must remain available");
        assert!(operations.operations.is_empty());
        assert_eq!(operations.next_op_id, 0);
        assert!(
            world
                .resource::<DirtyChunkMapResource>()
                .expect("dirty map must remain available")
                .by_chunk
                .is_empty()
        );
    }

    #[test]
    fn writable_ingress_remains_valid_without_optional_render_cache_resources() {
        let mut world = configured_world();
        let op_id = submit_sample(&mut world);
        assert!(op_id.is_some(), "valid World edits must remain admitted");
        let operations = world
            .resource::<OperationLogResource>()
            .expect("operation log must remain available");
        assert_eq!(operations.operations.len(), 1);
        assert_eq!(operations.by_id.get(&op_id.expect("accepted id")), Some(&0));
        assert_eq!(
            world
                .resource::<RegionInvalidationJournalResource>()
                .expect("region journal must be available")
                .recent_records
                .len(),
            1,
            "accepted edit must reach the required region journal"
        );
        assert!(
            !world
                .resource::<DirtyChunkMapResource>()
                .expect("dirty map must remain available")
                .by_chunk
                .is_empty()
        );
    }
}
