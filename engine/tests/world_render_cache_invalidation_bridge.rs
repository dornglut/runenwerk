use engine::plugins::render::features::world::runtime_cache::WorldRuntimeCacheResource;
use engine::plugins::world::chunks::dirty::{ChunkDirtyReason, WorldDirtyChunkMapResource};
use engine::plugins::world::chunks::render_cache_bridge::WorldRenderCacheInvalidationQueueResource;
use engine::plugins::world::edits::ingress::{WorldEditIngressMeta, submit_world_operation};
use engine::plugins::world::edits::operation::{WorldOperation, quantize_aabb, quantize_position};
use engine::plugins::world::ids::{ChunkCoord3, ChunkId, PlanetId};
use engine::plugins::world::plugin::{WorldPlugin, WorldRuntimeConfig};
use engine::prelude::{App, SimulationTick};

fn test_stamp_operation(fixed_point_scale: i32) -> WorldOperation {
    WorldOperation::Stamp {
        stamp_id: "tests.world.render-cache-bridge".to_string(),
        anchor_q: quantize_position([0.5, 0.5, 0.5], fixed_point_scale),
        payload: vec![1, 2, 3, 4],
    }
}

fn default_bounds_q(
    fixed_point_scale: i32,
) -> engine::plugins::world::edits::operation::QuantizedAabb {
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
        .resource::<WorldRuntimeConfig>()
        .expect("world runtime config should exist")
        .fixed_point_scale;
    let op_id = submit_world_operation(
        app.world_mut(),
        test_stamp_operation(fixed_point_scale),
        default_bounds_q(fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: PlanetId(0),
            deterministic_seed: 7,
            server_tick: SimulationTick(1),
            author_connection_id: Some(42),
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
    let expected = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });
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
        .resource::<WorldRuntimeConfig>()
        .expect("world runtime config should exist")
        .fixed_point_scale;
    let bounds_q = default_bounds_q(fixed_point_scale);
    for seed in [11_u64, 12_u64] {
        let op_id = submit_world_operation(
            app.world_mut(),
            test_stamp_operation(fixed_point_scale),
            bounds_q,
            WorldEditIngressMeta {
                planet_id: PlanetId(0),
                deterministic_seed: seed,
                server_tick: SimulationTick(seed),
                author_connection_id: None,
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
    let expected = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });
    assert_eq!(
        runtime_cache.stale_chunks.len(),
        1,
        "overlapping edits in the same chunk should dedupe to one stale chunk entry"
    );
    assert!(runtime_cache.stale_chunks.contains(&expected));
}

#[test]
fn integrated_build_output_marks_chunk_stale() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);
    app.world_mut()
        .insert_resource(WorldRuntimeCacheResource::default());

    let target = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 2, y: -1, z: 4 });
    {
        let dirty = app
            .world_mut()
            .resource_mut::<WorldDirtyChunkMapResource>()
            .expect("world dirty map should exist");
        dirty.mark_dirty(target, ChunkDirtyReason::Geometry);
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
        .resource::<WorldRuntimeConfig>()
        .expect("world runtime config should exist")
        .fixed_point_scale;
    let op_id = submit_world_operation(
        app.world_mut(),
        test_stamp_operation(fixed_point_scale),
        default_bounds_q(fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: PlanetId(0),
            deterministic_seed: 77,
            server_tick: SimulationTick(2),
            author_connection_id: None,
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
    let expected = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 0, y: 0, z: 0 });
    assert!(
        queue.pending_chunks.contains(&expected),
        "without render cache resource, pending invalidations must remain queued"
    );
}
