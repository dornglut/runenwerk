use engine::plugins::world::adapters::{PartitionConfigResource, SdfChunkStoreResource};
use engine::prelude::*;
use runen_spatial::{ChunkCoord3, ChunkId, GridPartitionConfig, WorldId};
use runenwerk_arena::{
    ARENA_PLAYER_SPAWN, ArenaMovementConfig, ArenaPlayer, GameCommandError, LOCAL_PARTICIPANT_ID,
    ParticipantCommand, PlayerCommand, PlayerPhysicalHistory, TickCommandBatch, apply_game_commands,
    build_headless_game_app, player_physical_history_for, player_state_for,
};
use world_ops::{ChunkGeneration, ChunkRevision};
use world_sdf::SdfChunkPayload;

fn partition() -> GridPartitionConfig {
    GridPartitionConfig::try_new(1.0, [8, 8, 8]).expect("test partition is valid")
}

fn chunk0() -> ChunkId {
    ChunkId::new(WorldId::new(0), ChunkCoord3::default())
}

fn clear_payload(chunk_id: ChunkId) -> SdfChunkPayload {
    SdfChunkPayload {
        chunk_id,
        chunk_revision: ChunkRevision::default(),
        chunk_generation: ChunkGeneration::default(),
        page_table: Default::default(),
        hierarchy_revision: 0,
        checksum: 0,
    }
}

fn configure_partition(app: &mut App) {
    *app.world_mut()
        .resource_mut::<PartitionConfigResource>()
        .expect("WorldPlugin should install partition configuration") =
        PartitionConfigResource(partition());
}

fn configure_clear_chunk(app: &mut App) {
    configure_partition(app);
    app.world_mut()
        .resource_mut::<SdfChunkStoreResource>()
        .expect("WorldPlugin should install the SDF store")
        .chunks
        .insert(chunk0(), clear_payload(chunk0()));
}

fn local_batch(tick: u64, command: PlayerCommand) -> TickCommandBatch {
    TickCommandBatch {
        tick: SimulationTick(tick),
        commands: vec![ParticipantCommand {
            participant: LOCAL_PARTICIPANT_ID,
            command,
        }],
    }
}

#[test]
fn maintained_fixed_tick_updates_previous_and_current_physical_state_headlessly() {
    let app = build_headless_game_app()
        .run_for_fixed_steps(1)
        .expect("maintained headless game should execute its physical command path");

    let history = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID)
        .expect("maintained player should own physical history");
    assert_eq!(history.previous.position, ARENA_PLAYER_SPAWN);
    assert!(!history.previous.grounded);
    assert!(history.current.grounded);
    assert!(history.current.support_normal.is_some());
    assert_eq!(app.registered_scene_count(), 0);
    assert!(app.world().resource::<NetworkInboundQueue>().is_err());
}

#[test]
fn game_owned_jump_policy_flows_through_shared_command_application() {
    let mut app = build_headless_game_app()
        .run_for_fixed_steps(1)
        .expect("maintained arena should integrate and ground the player before jump");
    let before = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID)
        .expect("maintained player should have physical state");
    assert!(before.current.grounded);

    let batch = local_batch(
        2,
        PlayerCommand {
            jump: true,
            ..PlayerCommand::default()
        },
    );
    apply_game_commands(app.world_mut(), batch.tick, &batch).unwrap();

    let history = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert!(history.current.velocity[1] > 0.0);
    assert!(history.current.position[1] > history.previous.position[1]);
    assert!(!history.current.grounded);
}

#[test]
fn missing_first_collision_payload_commits_no_physical_movement() {
    let mut app = build_headless_game_app()
        .run_for_frames(0)
        .expect("startup should spawn the maintained player");
    configure_partition(&mut app);
    let before = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();

    let batch = local_batch(
        1,
        PlayerCommand {
            move_x: 1,
            ..PlayerCommand::default()
        },
    );
    apply_game_commands(app.world_mut(), batch.tick, &batch).unwrap();

    let after = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert_eq!(after.current, before.current);
    assert_eq!(after.previous, before.current);
}

#[test]
fn missing_later_support_payload_discards_partial_candidate() {
    let mut app = build_headless_game_app()
        .run_for_frames(0)
        .expect("startup should spawn the maintained player");
    configure_clear_chunk(&mut app);
    {
        let movement = app
            .world_mut()
            .resource_mut::<ArenaMovementConfig>()
            .expect("arena should own movement tuning");
        movement.character.radius = 0.05;
        movement.character.support_probe_distance = 0.4;
    }

    {
        let world = app.world_mut();
        let query = world.query::<(&ArenaPlayer, &mut PlayerPhysicalHistory)>();
        let (_, history) = query
            .iter(world)
            .find(|(player, _)| player.participant == LOCAL_PARTICIPANT_ID)
            .unwrap();
        history.current.position = [0.25, 0.2, 0.25];
        history.current.velocity = [0.0; 3];
        history.current.grounded = false;
        history.current.support_normal = None;
        history.previous = history.current;
    }
    let before = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();

    let batch = local_batch(
        1,
        PlayerCommand {
            move_x: 1,
            ..PlayerCommand::default()
        },
    );
    apply_game_commands(app.world_mut(), batch.tick, &batch).unwrap();

    let after = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert_eq!(after.current, before.current);
    assert_eq!(after.previous, before.current);
}

#[test]
fn arena_player_without_physical_history_is_rejected_instead_of_bypassing_physics() {
    let mut world = World::new();
    world
        .spawn((
            ArenaPlayer {
                participant: LOCAL_PARTICIPANT_ID,
            },
            runenwerk_arena::PlayerControlState::default(),
        ))
        .unwrap();

    let batch = local_batch(1, PlayerCommand::default());
    let error = apply_game_commands(&mut world, batch.tick, &batch)
        .expect_err("all command participants must own physical state after P1");
    assert_eq!(
        error,
        runenwerk_arena::GameCommandError::MissingPhysicalState(LOCAL_PARTICIPANT_ID)
    );
}

#[test]
fn duplicate_participant_commands_are_rejected_before_any_mutation() {
    let mut app = build_headless_game_app()
        .run_for_frames(0)
        .expect("startup should spawn the maintained player");
    configure_clear_chunk(&mut app);
    let before_physics = player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    let before_control = player_state_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();

    let batch = TickCommandBatch {
        tick: SimulationTick(1),
        commands: vec![
            ParticipantCommand {
                participant: LOCAL_PARTICIPANT_ID,
                command: PlayerCommand {
                    move_x: 1,
                    ..PlayerCommand::default()
                },
            },
            ParticipantCommand {
                participant: LOCAL_PARTICIPANT_ID,
                command: PlayerCommand {
                    move_y: 1,
                    ..PlayerCommand::default()
                },
            },
        ],
    };

    let error = apply_game_commands(app.world_mut(), batch.tick, &batch)
        .expect_err("one participant may contribute at most one command per tick batch");
    assert_eq!(
        error,
        GameCommandError::DuplicateCommandParticipant(LOCAL_PARTICIPANT_ID)
    );
    assert_eq!(
        player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID),
        Some(before_physics)
    );
    assert_eq!(
        player_state_for(app.world(), LOCAL_PARTICIPANT_ID),
        Some(before_control)
    );
}
