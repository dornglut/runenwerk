use engine::plugins::world::adapters::{
    CollisionQueryServiceResource, PartitionConfigResource, SdfChunkStoreResource,
};
use engine::plugins::world::build::{
    WorldCompletedBuildQueueResource, WorldRuntimeSdfProductCatalogResource,
};
use engine::plugins::world::chunks::lifecycle::WorldChunkRuntimeMapResource;
use engine::prelude::*;
use runen_spatial::WorldId;
use runenwerk_arena::{
    ARENA_FIELD_PRODUCT_ID, ARENA_PLAYER_SPAWN, LOCAL_PARTICIPANT_ID, ParticipantCommand,
    PlayerCommand, TickCommandBatch, apply_game_commands, arena_chunk_id, build_headless_game_app,
    player_physical_history_for,
};

fn started_app() -> App {
    build_headless_game_app()
        .run_for_frames(0)
        .expect("maintained arena startup should succeed")
}

fn integrated_app() -> App {
    build_headless_game_app()
        .run_for_fixed_steps(1)
        .expect("first fixed tick should integrate the maintained arena")
}

fn signed_distance(app: &App, position: [f32; 3]) -> f32 {
    let partition = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("WorldPlugin should install partition configuration");
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("WorldPlugin should install the SDF store");
    let collision = app
        .world()
        .resource::<CollisionQueryServiceResource>()
        .expect("WorldPlugin should install collision queries");
    collision
        .sample_signed_distance(partition, store, WorldId::new(0), position)
        .expect("arena sample position should be valid")
        .expect("arena chunk payload should exist")
        .distance
}

fn local_batch(tick: u64, move_x: i8) -> TickCommandBatch {
    TickCommandBatch {
        tick: SimulationTick(tick),
        commands: vec![ParticipantCommand {
            participant: LOCAL_PARTICIPANT_ID,
            command: PlayerCommand {
                move_x,
                ..PlayerCommand::default()
            },
        }],
    }
}

#[test]
fn maintained_startup_enqueues_arena_through_world_runtime_intake() {
    let app = started_app();
    let partition = app
        .world()
        .resource::<PartitionConfigResource>()
        .expect("arena partition should exist");
    assert_eq!(partition.chunk_edge_meters(), 4.0);

    let completed = app
        .world()
        .resource::<WorldCompletedBuildQueueResource>()
        .expect("world runtime intake queue should exist");
    assert_eq!(completed.outputs.len(), 1);

    let chunks = app
        .world()
        .resource::<WorldChunkRuntimeMapResource>()
        .expect("world chunk runtime map should exist");
    let record = chunks
        .by_chunk_id
        .get(&arena_chunk_id())
        .expect("arena chunk should be registered with world runtime");
    assert!(record.pending_build_generation.is_some());

    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("arena SDF store should exist");
    assert!(
        store.chunks.is_empty(),
        "startup must not bypass World BuildIntegrate by writing the SDF store directly"
    );

    let products = app
        .world()
        .resource::<WorldRuntimeSdfProductCatalogResource>()
        .expect("runtime SDF product catalog should exist");
    let descriptor = products
        .products()
        .values()
        .find(|descriptor| descriptor.product_id == ARENA_FIELD_PRODUCT_ID)
        .expect("maintained arena should retain its field product identity");
    assert_eq!(descriptor.payload_refs.len(), 1);
    assert_eq!(descriptor.payload_refs[0].chunk_id, arena_chunk_id());
}

#[test]
fn first_fixed_tick_integrates_arena_through_world_authority() {
    let app = integrated_app();
    let store = app
        .world()
        .resource::<SdfChunkStoreResource>()
        .expect("arena SDF store should exist after integration");
    assert_eq!(store.chunks.len(), 1);
    assert!(store.chunks.contains_key(&arena_chunk_id()));

    let completed = app
        .world()
        .resource::<WorldCompletedBuildQueueResource>()
        .expect("world runtime intake queue should exist");
    assert!(completed.outputs.is_empty());
}

#[test]
fn maintained_arena_collision_truth_contains_floor_clear_interior_and_four_walls() {
    let app = integrated_app();

    assert!(signed_distance(&app, [1.0, 0.25, 1.0]) < 0.0);
    assert!(signed_distance(&app, [1.0, 0.75, 1.0]) > 0.0);

    for wall in [
        [0.25, 1.0, 1.0],
        [3.75, 1.0, 1.0],
        [1.0, 1.0, 0.25],
        [1.0, 1.0, 3.75],
    ] {
        assert!(signed_distance(&app, wall) < 0.0, "{wall:?}");
    }
}

#[test]
fn maintained_player_spawns_in_clear_runtime_arena_interior() {
    let app = integrated_app();
    let history = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID)
        .expect("maintained player should exist");
    assert_eq!(history.current.position, ARENA_PLAYER_SPAWN);
    assert!(signed_distance(&app, history.current.position) > 0.0);
}

#[test]
fn ordinary_fixed_tick_grounds_player_against_installed_floor() {
    let app = build_headless_game_app()
        .run_for_fixed_steps(1)
        .expect("maintained game should advance against its own arena payload");
    let history = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID)
        .expect("maintained player should retain physical history");

    assert!(history.current.grounded);
    assert!(history.current.support_normal.is_some());
    assert!(history.current.position[1] <= ARENA_PLAYER_SPAWN[1] + 0.001);
}

#[test]
fn movement_command_advances_player_through_shared_p1_path_without_test_payload_injection() {
    let mut app = integrated_app();
    let before = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();

    let batch = local_batch(2, 1);
    apply_game_commands(app.world_mut(), batch.tick, &batch).unwrap();

    let after = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert!(after.current.position[0] > before.current.position[0]);
}

#[test]
fn repeated_movement_stops_at_boundary_without_crossing_into_missing_payload_space() {
    let mut app = integrated_app();

    for tick in 2..=121_u64 {
        let batch = local_batch(tick, -1);
        apply_game_commands(app.world_mut(), batch.tick, &batch).unwrap();
    }

    let history = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert!(history.current.position[0] >= 0.49);
    assert!(history.current.position[0] < ARENA_PLAYER_SPAWN[0]);
}
