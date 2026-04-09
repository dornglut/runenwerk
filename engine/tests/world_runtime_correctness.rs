use engine::plugins::world::adapters::resources::{
    OperationLogResource, PartitionConfigResource, SdfChunkStoreResource,
};
use engine::plugins::world::chunks::DirtyChunkMapResource;
use engine::plugins::world::chunks::lifecycle::{
    ChunkLifecycleState, WorldChunkRuntimeMapResource, WorldChunkRuntimeRecord,
};
use engine::plugins::world::edits::ingress::{WorldEditIngressMeta, submit_world_operation};
use engine::plugins::world::plugin::{
    WorldAuthorityState, WorldPlugin, WorldRuntimeConfig, WorldRuntimeMode, WorldRuntimeState,
};
use engine::plugins::world::{
    build::integration::{WorldCompletedBuildOutput, WorldCompletedBuildQueueResource},
    build::jobs::WorldBuildStaleness,
};
use engine::prelude::{App, AuthorityRole};
use engine_sim::SimulationTick;
use spatial::{ChunkCoord3, ChunkId, WorldId};
use world_ops::{
    BrushShape, BuildGeneration, ChunkGeneration, ChunkRevision, DirtyReason, Operation,
    quantize_aabb, quantize_position,
};
use world_sdf::{RegionSdfSummary, SdfChunkPayload};

#[test]
fn dirty_chunk_without_runtime_record_is_bootstrapped_and_built() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let chunk_id = ChunkId::new(WorldId(0), ChunkCoord3 { x: 2, y: -1, z: 4 });
    {
        let dirty = app
            .world_mut()
            .resource_mut::<DirtyChunkMapResource>()
            .expect("world dirty map should be available");
        dirty.mark_dirty(chunk_id, DirtyReason::Geometry);
    }

    let app = app
        .run_for_ticks(1)
        .expect("world plugin systems should run for one fixed tick");

    let chunk_runtime = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime map should be available");
    assert!(
        chunk_runtime.by_chunk_id.contains_key(&chunk_id),
        "dirty chunk should create a runtime record before build dispatch"
    );

    let sdf_store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("sdf store should be available");
    assert!(
        sdf_store.chunks.contains_key(&chunk_id),
        "dirty chunk should produce an integrated chunk payload in the same tick"
    );

    let authority = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("world authority state should be available");
    assert!(
        authority.world_revision.0 > 0,
        "world revision should advance when build outputs integrate"
    );
}

#[test]
fn ingress_rejects_operations_in_client_replica_mode() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);
    {
        let world_runtime = app
            .world_mut()
            .resource_mut::<WorldRuntimeConfig>()
            .expect("world runtime config should exist");
        world_runtime.mode = WorldRuntimeMode::ReadOnly;
    }

    let op_id = submit_world_operation(
        app.world_mut(),
        Operation::CsgAdd {
            brush: BrushShape::Sphere {
                center_q: quantize_position([0.0, 0.0, 0.0], 1024),
                radius_q: 256,
            },
            material_channel: 1,
        },
        quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1024),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 404,
        },
    );
    assert!(
        op_id.is_none(),
        "client-replica world runtime mode must reject world edit ingress mutations"
    );

    let op_log = app
        .world()
        .resource::<OperationLogResource>()
        .expect("world operation log should exist");
    assert!(
        op_log.operations.is_empty(),
        "rejected ingress should not append world operation records"
    );
    let dirty = app
        .world()
        .resource::<DirtyChunkMapResource>()
        .expect("dirty map should exist");
    assert!(
        dirty.by_chunk.is_empty(),
        "rejected ingress should not mutate dirty chunk invalidation state"
    );
}

#[test]
fn world_runtime_mode_tracks_authority_role() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let initial_mode = app
        .world()
        .resource::<WorldRuntimeConfig>()
        .expect("world runtime config should exist")
        .mode;
    assert_eq!(
        initial_mode,
        WorldRuntimeMode::Writable,
        "local default authority should initialize world runtime in authoritative mode"
    );

    app.set_authority_role(AuthorityRole::Client);
    let client_mode = app
        .world()
        .resource::<WorldRuntimeConfig>()
        .expect("world runtime config should exist")
        .mode;
    assert_eq!(
        client_mode,
        WorldRuntimeMode::ReadOnly,
        "set_authority_role(Client) should immediately switch world runtime mode"
    );

    app.set_authority_role(AuthorityRole::Server);
    let server_mode = app
        .world()
        .resource::<WorldRuntimeConfig>()
        .expect("world runtime config should exist")
        .mode;
    assert_eq!(
        server_mode,
        WorldRuntimeMode::Writable,
        "set_authority_role(Server) should switch world runtime mode back to authoritative"
    );
}

#[test]
fn ingress_invalidation_uses_partition_quantization_scale() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);
    {
        let mut partition = app
            .world_mut()
            .resource_mut::<PartitionConfigResource>()
            .expect("world partition config should be available");
        partition.fixed_point_scale = 1;
    }

    let op_id = submit_world_operation(
        app.world_mut(),
        Operation::Stamp {
            stamp_id: "tests.partition-scale-ingress".to_string(),
            anchor_q: Default::default(),
            payload: vec![1, 2, 3],
        },
        quantize_aabb([40.0, 0.0, 0.0], [40.0, 0.0, 0.0], 1),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 77,
        },
    );
    assert!(op_id.is_some(), "world ingress should append operation");

    let dirty = app
        .world()
        .resource::<DirtyChunkMapResource>()
        .expect("world dirty map should be available");
    let expected_chunk = ChunkId::new(WorldId(0), ChunkCoord3 { x: 1, y: 0, z: 0 });
    assert!(
        dirty.by_chunk.contains_key(&expected_chunk),
        "ingress invalidation must dequantize bounds using world partition quantization scale"
    );
}

#[test]
fn world_revision_advances_only_for_integrated_outputs() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let chunk_id = ChunkId::new(WorldId(0), ChunkCoord3 { x: 1, y: 1, z: 1 });
    {
        let dirty = app
            .world_mut()
            .resource_mut::<DirtyChunkMapResource>()
            .expect("world dirty map should be available");
        dirty.mark_dirty(chunk_id, DirtyReason::Geometry);
    }

    let mut app = app
        .run_for_ticks(1)
        .expect("first dirty build should integrate");
    let revision_after_integrate = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist")
        .world_revision
        .0;
    assert!(revision_after_integrate > 0);

    let next_tick = app
        .world()
        .resource::<SimulationTick>()
        .expect("simulation tick should exist")
        .0
        .saturating_add(1);
    app = app
        .run_for_ticks(next_tick)
        .expect("idle fixed tick should not change world revision");
    let revision_after_idle = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist")
        .world_revision
        .0;
    assert_eq!(
        revision_after_idle, revision_after_integrate,
        "world revision must remain stable without accepted integrations"
    );

    {
        let mut runtime_chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        let record = runtime_chunks.ensure_chunk(chunk_id);
        record.lifecycle = ChunkLifecycleState::Rebuilding;
        record.pending_build_generation = Some(BuildGeneration(9));
    }
    {
        let mut completed = app
            .world_mut()
            .resource_mut::<WorldCompletedBuildQueueResource>()
            .expect("completed queue should exist");
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id,
            target_chunk_revision: ChunkRevision(99),
            target_build_generation: BuildGeneration(8),
            staleness: WorldBuildStaleness::Current,
            chunk_payload: SdfChunkPayload {
                chunk_id,
                chunk_revision: ChunkRevision(99),
                ..SdfChunkPayload::default()
            },
            region_summary: RegionSdfSummary::default(),
        });
    }

    let next_tick = app
        .world()
        .resource::<SimulationTick>()
        .expect("simulation tick should exist")
        .0
        .saturating_add(1);
    app = app
        .run_for_ticks(next_tick)
        .expect("stale output should be dropped without revision bump");
    let revision_after_stale = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist")
        .world_revision
        .0;
    assert_eq!(
        revision_after_stale, revision_after_integrate,
        "world revision must not advance for dropped stale outputs"
    );
    let runtime = app
        .world()
        .resource::<WorldRuntimeState>()
        .expect("runtime state should exist");
    assert!(
        runtime.dropped_stale_build_outputs > 0,
        "stale output path should increment dropped count"
    );
}

#[test]
fn dirty_reasons_while_rebuilding_are_preserved_for_followup_build() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let chunk_id = ChunkId::new(WorldId(0), ChunkCoord3 { x: 3, y: 2, z: -1 });
    {
        let mut runtime_chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        runtime_chunks.by_chunk_id.insert(
            chunk_id,
            WorldChunkRuntimeRecord {
                chunk_id,
                lifecycle: ChunkLifecycleState::Rebuilding,
                chunk_revision: ChunkRevision(4),
                chunk_generation: Default::default(),
                build_generation: BuildGeneration(4),
                dirty_reasons: Default::default(),
                pending_build_generation: Some(BuildGeneration(5)),
                gameplay_locked: false,
            },
        );
    }
    {
        let dirty = app
            .world_mut()
            .resource_mut::<DirtyChunkMapResource>()
            .expect("world dirty map should be available");
        dirty.mark_dirty(chunk_id, DirtyReason::Geometry);
    }
    {
        let mut completed = app
            .world_mut()
            .resource_mut::<WorldCompletedBuildQueueResource>()
            .expect("completed queue should exist");
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id,
            target_chunk_revision: ChunkRevision(5),
            target_build_generation: BuildGeneration(5),
            staleness: WorldBuildStaleness::Current,
            chunk_payload: SdfChunkPayload {
                chunk_id,
                chunk_revision: ChunkRevision(5),
                chunk_generation: ChunkGeneration(5),
                ..SdfChunkPayload::default()
            },
            region_summary: RegionSdfSummary::default(),
        });
    }

    let app = app
        .run_for_ticks(1)
        .expect("integration should preserve rebuild-time dirty reasons");
    let runtime_chunks = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime should exist");
    let record = runtime_chunks
        .by_chunk_id
        .get(&chunk_id)
        .expect("runtime record should remain present");
    assert!(
        matches!(record.lifecycle, ChunkLifecycleState::Dirty),
        "chunk must re-enter Dirty lifecycle when new dirty reasons arrive during rebuild"
    );
    assert!(
        !record.dirty_reasons.is_empty(),
        "dirty reasons merged during rebuild must be preserved for follow-up dispatch"
    );
    assert!(
        record.pending_build_generation.is_none(),
        "accepted integration clears pending generation before follow-up rebuild"
    );
}

#[test]
fn stamp_operation_produces_authoritative_chunk_payload() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let op_id = submit_world_operation(
        app.world_mut(),
        Operation::Stamp {
            stamp_id: "tests.stamp-authority".to_string(),
            anchor_q: Default::default(),
            payload: vec![9, 9, 9],
        },
        quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1024),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 101,
        },
    );
    assert!(
        op_id.is_some(),
        "stamp operation should be accepted by ingress"
    );

    let app = app
        .run_for_ticks(1)
        .expect("stamp operation should build and integrate");
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("sdf store should exist after integration");
    let chunk_id = ChunkId::new(WorldId(0), ChunkCoord3::default());
    let payload = store
        .chunks
        .get(&chunk_id)
        .expect("stamp build should produce chunk payload");

    assert!(
        !payload.page_table.is_empty(),
        "stamp operation should produce non-empty authoritative payload content"
    );
    assert!(
        payload
            .page_table
            .values()
            .flat_map(|page| page.bricks.values())
            .any(|brick| brick.metadata.occupancy_mask != 0),
        "stamp payload should carry occupied brick metadata for collision authority"
    );
}

#[test]
fn material_field_edit_preserves_existing_chunk_solidity() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let add_op = submit_world_operation(
        app.world_mut(),
        Operation::CsgAdd {
            brush: BrushShape::Sphere {
                center_q: Default::default(),
                radius_q: 128,
            },
            material_channel: 1,
        },
        quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1024),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 201,
        },
    );
    assert!(add_op.is_some(), "add operation should be accepted");

    let mut app = app
        .run_for_ticks(1)
        .expect("initial csg add should integrate into chunk payload");

    let edit_op = submit_world_operation(
        app.world_mut(),
        Operation::MaterialFieldEdit {
            bounds_q: quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1024),
            channel_mask: 0b0100,
            payload: vec![1],
        },
        quantize_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1024),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 202,
        },
    );
    assert!(edit_op.is_some(), "material field edit should be accepted");

    app = app
        .run_for_ticks(2)
        .expect("material field edit should rebuild payload without topology loss");
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("sdf store should exist");
    let chunk_id = ChunkId::new(WorldId(0), ChunkCoord3::default());
    let payload = store
        .chunks
        .get(&chunk_id)
        .expect("chunk payload should remain present after material edit");
    let brick_masks = payload
        .page_table
        .values()
        .flat_map(|page| page.bricks.values())
        .map(|brick| brick.metadata.material_channel_mask)
        .collect::<Vec<_>>();

    assert!(
        !brick_masks.is_empty(),
        "material edit should preserve occupied payload pages after csg add"
    );
    assert!(
        brick_masks.iter().any(|mask| (mask & 0b0100) != 0),
        "material edit mask should be reflected in authoritative payload metadata"
    );
}

#[test]
fn integration_drops_output_when_payload_revision_contract_mismatches() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let chunk_id = ChunkId::new(WorldId(0), ChunkCoord3 { x: 6, y: 0, z: -2 });
    {
        let mut runtime_chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        runtime_chunks.by_chunk_id.insert(
            chunk_id,
            WorldChunkRuntimeRecord {
                chunk_id,
                lifecycle: ChunkLifecycleState::Rebuilding,
                chunk_revision: ChunkRevision(10),
                chunk_generation: ChunkGeneration(10),
                build_generation: BuildGeneration(10),
                dirty_reasons: Default::default(),
                pending_build_generation: Some(BuildGeneration(11)),
                gameplay_locked: false,
            },
        );
    }
    {
        let mut completed = app
            .world_mut()
            .resource_mut::<WorldCompletedBuildQueueResource>()
            .expect("completed queue should exist");
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id,
            target_chunk_revision: ChunkRevision(11),
            target_build_generation: BuildGeneration(11),
            staleness: WorldBuildStaleness::Current,
            chunk_payload: SdfChunkPayload {
                chunk_id,
                chunk_revision: ChunkRevision(10),
                chunk_generation: ChunkGeneration(11),
                ..SdfChunkPayload::default()
            },
            region_summary: RegionSdfSummary::default(),
        });
    }

    let app = app
        .run_for_ticks(1)
        .expect("integration should reject mismatched payload revision contract");
    let authority = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist");
    assert_eq!(
        authority.world_revision.0, 0,
        "authority revision must not advance when integration rejects malformed output"
    );
    let runtime = app
        .world()
        .resource::<WorldRuntimeState>()
        .expect("runtime state should exist");
    assert!(
        runtime.dropped_stale_build_outputs > 0,
        "malformed output should be counted as dropped"
    );
    let runtime_chunks = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("chunk runtime should exist");
    let record = runtime_chunks
        .by_chunk_id
        .get(&chunk_id)
        .expect("runtime record should remain present");
    assert_eq!(
        record.pending_build_generation,
        Some(BuildGeneration(11)),
        "rejected integration must keep pending generation unchanged"
    );
}

#[test]
fn integration_drops_output_when_payload_chunk_id_contract_mismatches() {
    let mut app = App::headless();
    app.add_plugin(WorldPlugin);

    let chunk_id = ChunkId::new(WorldId(0), ChunkCoord3 { x: -4, y: 1, z: 3 });
    let wrong_chunk_id = ChunkId::new(WorldId(0), ChunkCoord3 { x: -3, y: 1, z: 3 });
    {
        let mut runtime_chunks = app
            .world_mut()
            .resource_mut::<WorldChunkRuntimeMapResource>()
            .expect("chunk runtime should exist");
        runtime_chunks.by_chunk_id.insert(
            chunk_id,
            WorldChunkRuntimeRecord {
                chunk_id,
                lifecycle: ChunkLifecycleState::Rebuilding,
                chunk_revision: ChunkRevision(2),
                chunk_generation: ChunkGeneration(2),
                build_generation: BuildGeneration(2),
                dirty_reasons: Default::default(),
                pending_build_generation: Some(BuildGeneration(3)),
                gameplay_locked: false,
            },
        );
    }
    {
        let mut completed = app
            .world_mut()
            .resource_mut::<WorldCompletedBuildQueueResource>()
            .expect("completed queue should exist");
        completed.outputs.push_back(WorldCompletedBuildOutput {
            chunk_id,
            target_chunk_revision: ChunkRevision(3),
            target_build_generation: BuildGeneration(3),
            staleness: WorldBuildStaleness::Current,
            chunk_payload: SdfChunkPayload {
                chunk_id: wrong_chunk_id,
                chunk_revision: ChunkRevision(3),
                chunk_generation: ChunkGeneration(3),
                ..SdfChunkPayload::default()
            },
            region_summary: RegionSdfSummary::default(),
        });
    }

    let app = app
        .run_for_ticks(1)
        .expect("integration should reject mismatched payload chunk-id contract");
    let authority = app
        .world()
        .resource::<WorldAuthorityState>()
        .expect("authority should exist");
    assert_eq!(
        authority.world_revision.0, 0,
        "authority revision must not advance when payload chunk-id contract mismatches"
    );
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("sdf store should exist");
    assert!(
        !store.chunks.contains_key(&chunk_id),
        "rejected integration must not publish malformed payload into authoritative chunk store"
    );
}
