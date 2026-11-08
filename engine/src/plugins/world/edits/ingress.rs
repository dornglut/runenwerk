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
    if touched_chunks.is_empty() {
        return None;
    }

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
        world.insert_resource(WorldRenderCacheInvalidationQueueResource::default());
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
        let journal = world
            .resource::<RegionInvalidationJournalResource>()
            .expect("region journal remains available");
        assert!(journal.recent_records.is_empty());
        assert_eq!(journal.next_sequence, 1);
        let queue = world
            .resource::<WorldRenderCacheInvalidationQueueResource>()
            .expect("optional render queue remains available");
        assert!(queue.pending_records.is_empty());
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

    fn assert_no_edit_publication(world: &World) {
        let operations = world
            .resource::<OperationLogResource>()
            .expect("operation log remains present");
        assert!(operations.operations.is_empty());
        assert!(operations.by_id.is_empty());
        assert_eq!(operations.next_op_id, 0);

        let dirty = world
            .resource::<DirtyChunkMapResource>()
            .expect("dirty map remains present");
        assert!(dirty.by_chunk.is_empty());

        let journal = world
            .resource::<RegionInvalidationJournalResource>()
            .expect("region journal remains present");
        assert!(journal.recent_records.is_empty());
        assert_eq!(journal.next_sequence, 1);
    }

    #[test]
    fn inverted_quantized_bounds_reject_before_any_authoritative_edit() {
        let mut world = configured_world();
        let scale = WorldQuantizationScale::try_new(1024).expect("valid quantization scale");
        let result = submit_world_operation(
            &mut world,
            Operation::Stamp {
                stamp_id: "tests.world-ingress-inverted-bounds".to_string(),
                anchor_q: Default::default(),
                payload: vec![1, 2, 3],
            },
            quantize_aabb([64.0, 0.0, 0.0], [0.0, 0.0, 0.0], scale),
            WorldEditIngressMeta::default(),
        );

        assert_eq!(result, None);
        assert_no_edit_publication(&world);
    }

    #[test]
    fn missing_partition_or_quantization_rejects_without_publication() {
        let mut without_partition = configured_world();
        without_partition
            .remove_resource::<PartitionConfigResource>()
            .expect("configured World has partition");
        assert_eq!(submit_sample(&mut without_partition), None);
        assert_no_edit_publication(&without_partition);

        let mut without_scale = configured_world();
        without_scale
            .remove_resource::<WorldQuantizationScaleResource>()
            .expect("configured World has quantization scale");
        assert_eq!(submit_sample(&mut without_scale), None);
        assert_no_edit_publication(&without_scale);
    }

    #[test]
    fn explicit_read_only_world_rejects_without_publication() {
        let mut world = configured_world();
        world
            .resource_mut::<WorldRuntimeConfig>()
            .expect("configured World has runtime mode")
            .mode = WorldRuntimeMode::ReadOnly;
        assert_eq!(submit_sample(&mut world), None);
        assert_no_edit_publication(&world);
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
        let accepted_id = op_id.expect("accepted id");
        assert_eq!(operations.by_id.get(&accepted_id), Some(&0));
        let journal = world
            .resource::<RegionInvalidationJournalResource>()
            .expect("region journal must be available");
        assert_eq!(
            journal
                .recent_records
                .front()
                .expect("one ingress record")
                .op_id,
            Some(accepted_id)
        );
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
