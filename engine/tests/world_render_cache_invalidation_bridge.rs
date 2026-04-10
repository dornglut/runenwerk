use engine::plugins::render::features::world::runtime_cache::WorldRuntimeCacheResource;
use engine::plugins::world::adapters::resources::PartitionConfigResource;
use engine::plugins::world::chunks::DirtyChunkMapResource;
use engine::plugins::world::chunks::render_cache_bridge::{
    WorldRenderCacheInvalidationQueueResource, WorldRenderInvalidationSource,
};
use engine::plugins::world::edits::ingress::{WorldEditIngressMeta, submit_world_operation};
use engine::plugins::world::plugin::WorldPlugin;
use engine::prelude::{App, SimulationTick};
use spatial::{ChunkCoord3, ChunkId, WorldId};
use world_ops::{DirtyReason, Operation, QuantizedAabb, quantize_aabb, quantize_position};

fn test_stamp_operation(fixed_point_scale: i32) -> Operation {
    Operation::Stamp {
        stamp_id: "tests.world.render-cache-bridge".to_string(),
        anchor_q: quantize_position([0.5, 0.5, 0.5], fixed_point_scale),
        payload: vec![1, 2, 3, 4],
    }
}

fn default_bounds_q(fixed_point_scale: i32) -> QuantizedAabb {
    quantize_aabb([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], fixed_point_scale)
}

#[test]
fn ingress_bounds_marks_render_cache_stale_next_fixed_tick() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);
    app.world_mut()
        .insert_resource(WorldRuntimeCacheResource::default());

    let fixed_point_scale = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("world partition config should exist")
        .quantization_scale();
    let op_id = submit_world_operation(
        app.world_mut(),
        test_stamp_operation(fixed_point_scale),
        default_bounds_q(fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 7,
        },
    );
    assert!(op_id.is_some(), "ingress should append an operation");

    let app = app
        .run_for_ticks(1)
        .expect("world systems should process one fixed tick");

    let runtime_cache = app
        .world()
        .resource::<WorldRuntimeCacheResource>()
        .expect("world runtime cache should exist");
    let expected = ChunkId::new(WorldId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });
    assert!(
        runtime_cache.stale_chunks.contains(&expected),
        "expected ingress invalidation to mark target chunk stale"
    );
}

#[test]
fn duplicate_edits_same_chunk_dedupe_invalidation() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);
    app.world_mut()
        .insert_resource(WorldRuntimeCacheResource::default());

    let fixed_point_scale = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("world partition config should exist")
        .quantization_scale();
    let bounds_q = default_bounds_q(fixed_point_scale);
    for seed in [11_u64, 12_u64] {
        let op_id = submit_world_operation(
            app.world_mut(),
            test_stamp_operation(fixed_point_scale),
            bounds_q,
            WorldEditIngressMeta {
                planet_id: WorldId(0),
                deterministic_seed: seed,
            },
        );
        assert!(op_id.is_some(), "ingress should append operation {seed}");
    }

    let app = app
        .run_for_ticks(1)
        .expect("world systems should process one fixed tick");

    let runtime_cache = app
        .world()
        .resource::<WorldRuntimeCacheResource>()
        .expect("world runtime cache should exist");
    let expected = ChunkId::new(WorldId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });
    assert_eq!(
        runtime_cache.stale_chunks.len(),
        1,
        "overlapping edits in the same chunk should dedupe to one stale chunk entry"
    );
    assert!(runtime_cache.stale_chunks.contains(&expected));
}

#[test]
fn multi_chunk_bounds_invalidation_marks_all_touched_chunks_stale() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);
    app.world_mut()
        .insert_resource(WorldRuntimeCacheResource::default());

    let fixed_point_scale = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("world partition config should exist")
        .quantization_scale();
    let op_id = submit_world_operation(
        app.world_mut(),
        test_stamp_operation(fixed_point_scale),
        quantize_aabb([0.0, 0.0, 0.0], [40.0, 1.0, 1.0], fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 99,
        },
    );
    assert!(op_id.is_some(), "ingress should append an operation");

    let app = app
        .run_for_ticks(1)
        .expect("world systems should process one fixed tick");

    let runtime_cache = app
        .world()
        .resource::<WorldRuntimeCacheResource>()
        .expect("world runtime cache should exist");
    let chunk_a = ChunkId::new(WorldId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });
    let chunk_b = ChunkId::new(WorldId(0), ChunkCoord3 { x: 1, y: 0, z: 0 });
    assert!(
        runtime_cache.stale_chunks.contains(&chunk_a),
        "expected lower bound chunk to be marked stale"
    );
    assert!(
        runtime_cache.stale_chunks.contains(&chunk_b),
        "expected upper bound chunk to be marked stale"
    );
}

#[test]
fn integrated_build_output_marks_chunk_stale() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);
    app.world_mut()
        .insert_resource(WorldRuntimeCacheResource::default());

    let target = ChunkId::new(WorldId(0), ChunkCoord3 { x: 2, y: -1, z: 4 });
    {
        let dirty = app
            .world_mut()
            .resource_mut::<DirtyChunkMapResource>()
            .expect("world dirty map should exist");
        dirty.mark_dirty(target, DirtyReason::Geometry);
    }

    let app = app
        .run_for_ticks(1)
        .expect("dirty->build->integrate path should run for one fixed tick");

    let runtime_cache = app
        .world()
        .resource::<WorldRuntimeCacheResource>()
        .expect("world runtime cache should exist");
    assert!(
        runtime_cache.stale_chunks.contains(&target),
        "integrated build output should enqueue and flush stale mark for rebuilt chunk"
    );
}

#[test]
fn bridge_does_not_drop_queue_without_render_cache_resource() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let fixed_point_scale = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("world partition config should exist")
        .quantization_scale();
    let op_id = submit_world_operation(
        app.world_mut(),
        test_stamp_operation(fixed_point_scale),
        default_bounds_q(fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 77,
        },
    );
    assert!(op_id.is_some(), "ingress should append an operation");

    let app = app
        .run_for_ticks(1)
        .expect("missing render cache resource must not fail world runtime");

    let queue = app
        .world()
        .resource::<WorldRenderCacheInvalidationQueueResource>()
        .expect("render-cache invalidation queue should exist");
    let expected = ChunkId::new(WorldId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });
    let record = queue
        .pending_records
        .front()
        .expect("pending invalidation record should be present");
    assert!(
        record.chunk_ids.contains(&expected),
        "without render cache resource, pending invalidations must remain queued"
    );
    assert_eq!(
        record.chunk_bounds.min,
        ChunkCoord3 { x: 0, y: 0, z: 0 },
        "recorded chunk bounds should preserve ingress invalidation extent"
    );
    assert_eq!(
        record.chunk_bounds.max,
        ChunkCoord3 { x: 0, y: 0, z: 0 },
        "single-chunk ingress invalidation should have tight chunk bounds"
    );
    assert!(
        matches!(record.source, WorldRenderInvalidationSource::EditIngress),
        "ingress invalidation should publish an ingress-sourced bridge record"
    );
    let partition = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("world partition config should exist");
    let expected_region = partition.region_id_from_chunk_id(expected);
    assert!(
        record.region_ids.contains(&expected_region),
        "bridge record should carry touched region ids alongside chunk invalidation"
    );
}

#[test]
fn missing_render_cache_then_recreate_flushes_pending_invalidation() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let fixed_point_scale = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("world partition config should exist")
        .quantization_scale();
    let op_id = submit_world_operation(
        app.world_mut(),
        test_stamp_operation(fixed_point_scale),
        default_bounds_q(fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 121,
        },
    );
    assert!(op_id.is_some(), "ingress should append an operation");

    let mut app = app
        .run_for_ticks(1)
        .expect("missing render cache resource must keep invalidations queued");

    let queued_before = app
        .world()
        .resource::<WorldRenderCacheInvalidationQueueResource>()
        .expect("render-cache invalidation queue should exist");
    assert!(
        !queued_before.pending_records.is_empty(),
        "pending invalidations should remain queued without render cache resource"
    );

    app.world_mut()
        .insert_resource(WorldRuntimeCacheResource::default());
    let next_tick = app
        .world()
        .resource::<SimulationTick>()
        .expect("simulation tick should exist")
        .0
        .saturating_add(1);
    let app = app
        .run_for_ticks(next_tick)
        .expect("pending invalidations should flush after render cache resource is restored");

    let runtime_cache = app
        .world()
        .resource::<WorldRuntimeCacheResource>()
        .expect("world runtime cache should exist");
    let expected = ChunkId::new(WorldId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });
    assert!(
        runtime_cache.stale_chunks.contains(&expected),
        "restored render cache should receive stale mark from queued invalidation records"
    );
    let queue = app
        .world()
        .resource::<WorldRenderCacheInvalidationQueueResource>()
        .expect("render-cache invalidation queue should exist");
    assert!(
        queue.pending_records.is_empty(),
        "queue should clear after successful flush into render cache"
    );
}

#[test]
fn integrated_build_without_render_cache_enqueues_build_sourced_record() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let target = ChunkId::new(WorldId(0), ChunkCoord3 { x: 2, y: -1, z: 4 });
    {
        let dirty = app
            .world_mut()
            .resource_mut::<DirtyChunkMapResource>()
            .expect("world dirty map should exist");
        dirty.mark_dirty(target, DirtyReason::Geometry);
    }

    let app = app
        .run_for_ticks(1)
        .expect("dirty->build->integrate path should run with queue preserved");

    let queue = app
        .world()
        .resource::<WorldRenderCacheInvalidationQueueResource>()
        .expect("render-cache invalidation queue should exist");
    let build_record = queue
        .pending_records
        .iter()
        .find(|record| record.chunk_ids.contains(&target))
        .expect("build integration should enqueue invalidation record for rebuilt chunk");
    assert!(
        matches!(
            build_record.source,
            WorldRenderInvalidationSource::BuildIntegration
        ),
        "integrated build output should publish a build-integration invalidation record"
    );
    let partition = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("world partition config should exist");
    let expected_region = partition.region_id_from_chunk_id(target);
    assert!(
        build_record.region_ids.contains(&expected_region),
        "build-sourced bridge records should carry region ids for downstream consumers"
    );
}
