use engine::plugins::InputState;
use engine::plugins::world::adapters::SdfChunkStoreResource;
use engine::prelude::*;
use runenwerk_arena::{
    ARENA_PLAYER_SPAWN, ArenaHazardConfig, ArenaPlayer, GameActionSnapshot, GameCommandError,
    GameInputAccumulator, LOCAL_PARTICIPANT_ID, LastLocalCommandBatch, MAX_ARENA_HEALTH,
    ParticipantCommand, ParticipantId, PlayerCommand, PlayerControlState, PlayerPhysicalHistory,
    PlayerVitals, TickCommandBatch, apply_game_commands, build_headless_game_app,
    player_physical_history_for, player_state_for, player_vitals_for,
};
use winit::event::ElementState;
use winit::keyboard::KeyCode;

fn integrated_game() -> App {
    build_headless_game_app()
        .run_for_fixed_steps(1)
        .expect("maintained arena must integrate the WorldSDF product at the first tick")
}

fn command_batch(tick: u64, commands: &[(ParticipantId, PlayerCommand)]) -> TickCommandBatch {
    TickCommandBatch {
        tick: SimulationTick(tick),
        commands: commands
            .iter()
            .map(|(participant, command)| ParticipantCommand {
                participant: *participant,
                command: *command,
            })
            .collect(),
    }
}

fn apply(app: &mut App, tick: u64, command: PlayerCommand) {
    let batch = command_batch(tick, &[(LOCAL_PARTICIPANT_ID, command)]);
    apply_game_commands(app.world_mut(), batch.tick, &batch)
        .expect("bounded gameplay command must apply");
}

fn control(app: &App) -> PlayerControlState {
    player_state_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap()
}

fn history(app: &App) -> PlayerPhysicalHistory {
    player_physical_history_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap()
}

fn vitals(app: &App) -> PlayerVitals {
    player_vitals_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap()
}

fn set_local_pose(app: &mut App, position: [f32; 3]) {
    let query = app
        .world()
        .query::<(&ArenaPlayer, &mut PlayerPhysicalHistory)>();
    let (_, state) = query
        .iter(app.world_mut())
        .find(|(player, _)| player.participant == LOCAL_PARTICIPANT_ID)
        .expect("maintained gameplay player");
    *state = PlayerPhysicalHistory::spawned(position);
}

fn defeat_at_hazard(app: &mut App) {
    set_local_pose(app, [2.5, 1.0, 2.5]);
    for tick in 2..=62 {
        apply(app, tick, PlayerCommand::default());
    }
    assert!(vitals(app).is_defeated());
    assert_eq!(vitals(app).last_hazard_hit_tick, Some(SimulationTick(62)));
}

#[test]
fn startup_initializes_single_gameplay_health_source_without_visual_or_net_runtime() {
    let app = integrated_game();
    assert_eq!(vitals(&app), PlayerVitals::default());
    assert_eq!(vitals(&app).health, MAX_ARENA_HEALTH);
    assert!(!vitals(&app).is_defeated());
    assert_eq!(history(&app).previous.position, ARENA_PLAYER_SPAWN);
    assert_eq!(control(&app).last_applied_tick, Some(SimulationTick(1)));
    assert_eq!(app.registered_scene_count(), 0);
    assert!(app.world().resource::<NetworkInboundQueue>().is_err());
    assert!(
        app.world()
            .resource::<LastLocalCommandBatch>()
            .unwrap()
            .0
            .is_some()
    );
}

#[test]
fn maintained_world_movement_enters_one_game_owned_hazard_and_applies_one_damage() {
    let mut app = integrated_game();
    for tick in 2..=30 {
        apply(
            &mut app,
            tick,
            PlayerCommand {
                move_x: 1,
                move_y: 1,
                ..PlayerCommand::default()
            },
        );
    }
    assert_eq!(vitals(&app).health, MAX_ARENA_HEALTH - 1);
    assert!(vitals(&app).last_hazard_hit_tick.is_some());
    assert!(history(&app).current.position[0] > ARENA_PLAYER_SPAWN[0]);
}

#[test]
fn harmless_motion_and_airborne_contact_outside_the_trigger_do_not_damage() {
    let mut app = integrated_game();
    for tick in 2..=8 {
        apply(
            &mut app,
            tick,
            PlayerCommand {
                move_x: 1,
                ..PlayerCommand::default()
            },
        );
    }
    assert_eq!(vitals(&app).health, MAX_ARENA_HEALTH);

    set_local_pose(&mut app, [2.5, 2.0, 2.5]);
    apply(&mut app, 9, PlayerCommand::default());
    assert_eq!(vitals(&app).health, MAX_ARENA_HEALTH);
}

#[test]
fn hazard_cooldown_is_tick_targeted_and_continues_through_reentry() {
    let mut app = integrated_game();
    set_local_pose(&mut app, [2.5, 1.0, 2.5]);
    apply(&mut app, 2, PlayerCommand::default());
    assert_eq!(vitals(&app).health, 2);
    for tick in 3..32 {
        if tick == 8 {
            set_local_pose(&mut app, [1.0, 1.0, 1.0]);
        } else if tick == 9 {
            set_local_pose(&mut app, [2.5, 1.0, 2.5]);
        }
        apply(&mut app, tick, PlayerCommand::default());
        assert_eq!(vitals(&app).health, 2, "tick {tick}");
    }
    apply(&mut app, 32, PlayerCommand::default());
    assert_eq!(vitals(&app).health, 1);
    assert_eq!(vitals(&app).last_hazard_hit_tick, Some(SimulationTick(32)));
}

#[test]
fn defeat_stops_motion_interaction_effects_and_additional_hazard_damage() {
    let mut app = integrated_game();
    defeat_at_hazard(&mut app);
    let before = history(&app);
    apply(
        &mut app,
        63,
        PlayerCommand {
            move_x: 1,
            jump: true,
            interact: true,
            ..PlayerCommand::default()
        },
    );
    assert!(vitals(&app).is_defeated());
    assert_eq!(vitals(&app).health, 0);
    assert_eq!(vitals(&app).last_hazard_hit_tick, Some(SimulationTick(62)));
    assert_eq!(history(&app).current, before.current);
    assert_eq!(history(&app).previous, before.current);
    assert_eq!(
        control(&app).interact_request_count,
        1,
        "request audit remains observable"
    );
}

#[test]
fn restart_is_atomic_and_resets_full_physical_history_without_same_tick_damage() {
    let mut app = integrated_game();
    defeat_at_hazard(&mut app);
    let commands_before = control(&app).applied_command_count;
    apply(
        &mut app,
        63,
        PlayerCommand {
            restart: true,
            move_x: 1,
            jump: true,
            interact: true,
            ..PlayerCommand::default()
        },
    );
    assert_eq!(vitals(&app), PlayerVitals::default());
    assert_eq!(
        history(&app),
        PlayerPhysicalHistory::spawned(ARENA_PLAYER_SPAWN)
    );
    assert_eq!(control(&app).applied_command_count, commands_before + 1);
    apply(
        &mut app,
        64,
        PlayerCommand {
            move_x: 1,
            ..PlayerCommand::default()
        },
    );
    assert!(history(&app).current.position[0] > ARENA_PLAYER_SPAWN[0]);
    assert_eq!(vitals(&app).health, MAX_ARENA_HEALTH);
}

#[test]
fn healthy_restart_intent_is_not_a_teleport_and_does_not_skip_movement() {
    let mut app = integrated_game();
    apply(
        &mut app,
        2,
        PlayerCommand {
            restart: true,
            move_x: 1,
            ..PlayerCommand::default()
        },
    );
    assert_eq!(vitals(&app), PlayerVitals::default());
    assert!(history(&app).current.position[0] > ARENA_PLAYER_SPAWN[0]);
}

#[test]
fn failed_spawn_readiness_preserves_all_gameplay_state_and_control_counters() {
    let mut app = integrated_game();
    defeat_at_hazard(&mut app);
    app.world_mut()
        .resource_mut::<SdfChunkStoreResource>()
        .unwrap()
        .chunks
        .clear();
    let (before_health, before_motion, before_control) =
        (vitals(&app), history(&app), control(&app));
    let batch = command_batch(
        63,
        &[(
            LOCAL_PARTICIPANT_ID,
            PlayerCommand {
                restart: true,
                ..PlayerCommand::default()
            },
        )],
    );
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::SpawnUnavailable(LOCAL_PARTICIPANT_ID))
    );
    assert_eq!(vitals(&app), before_health);
    assert_eq!(history(&app), before_motion);
    assert_eq!(control(&app), before_control);
}

#[test]
fn missing_collision_truth_never_damages_from_unadmitted_travel() {
    let mut app = integrated_game();
    app.world_mut()
        .resource_mut::<SdfChunkStoreResource>()
        .unwrap()
        .chunks
        .clear();
    let before = history(&app);
    app.world_mut()
        .resource_mut::<runenwerk_arena::ArenaMovementConfig>()
        .unwrap()
        .movement_speed = 100.0;
    apply(
        &mut app,
        2,
        PlayerCommand {
            move_x: 1,
            move_y: 1,
            ..PlayerCommand::default()
        },
    );
    assert_eq!(history(&app).current, before.current);
    assert_eq!(vitals(&app), PlayerVitals::default());
}

#[test]
fn non_increasing_or_mismatched_tick_cannot_apply_damage_twice() {
    let mut app = integrated_game();
    set_local_pose(&mut app, [2.5, 1.0, 2.5]);
    apply(&mut app, 2, PlayerCommand::default());
    let before = (vitals(&app), history(&app), control(&app));
    let batch = command_batch(2, &[(LOCAL_PARTICIPANT_ID, PlayerCommand::default())]);
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::NonIncreasingTick {
            participant: LOCAL_PARTICIPANT_ID,
            last: SimulationTick(2),
            attempted: SimulationTick(2),
        })
    );
    assert_eq!(
        apply_game_commands(app.world_mut(), SimulationTick(3), &batch),
        Err(GameCommandError::TickMismatch {
            expected: SimulationTick(3),
            batch: SimulationTick(2),
        })
    );
    assert_eq!((vitals(&app), history(&app), control(&app)), before);
}

#[test]
fn invalid_hazard_and_vitals_fail_before_any_mutation() {
    let mut app = integrated_game();
    let before = (vitals(&app), history(&app), control(&app));
    app.world_mut()
        .resource_mut::<ArenaHazardConfig>()
        .unwrap()
        .radius = f32::NAN;
    let batch = command_batch(2, &[(LOCAL_PARTICIPANT_ID, PlayerCommand::default())]);
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::InvalidHazard)
    );
    assert_eq!((vitals(&app), history(&app), control(&app)), before);

    *app.world_mut().resource_mut::<ArenaHazardConfig>().unwrap() = ArenaHazardConfig::default();
    let query = app.world().query::<(&ArenaPlayer, &mut PlayerVitals)>();
    let (_, vitals) = query
        .iter(app.world_mut())
        .find(|(player, _)| player.participant == LOCAL_PARTICIPANT_ID)
        .unwrap();
    vitals.health = MAX_ARENA_HEALTH + 1;
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::InvalidVitals(LOCAL_PARTICIPANT_ID))
    );
    assert_eq!(control(&app), before.2);
    assert_eq!(history(&app), before.1);
}

#[test]
fn later_participant_missing_vitals_rejects_entire_command_batch() {
    let mut app = integrated_game();
    let other = ParticipantId(17);
    app.world_mut()
        .spawn((
            ArenaPlayer { participant: other },
            PlayerControlState::default(),
            PlayerPhysicalHistory::spawned(ARENA_PLAYER_SPAWN),
        ))
        .unwrap();
    let before = (vitals(&app), history(&app), control(&app));
    let batch = command_batch(
        2,
        &[
            (
                LOCAL_PARTICIPANT_ID,
                PlayerCommand {
                    move_x: 1,
                    ..PlayerCommand::default()
                },
            ),
            (other, PlayerCommand::default()),
        ],
    );
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::MissingVitals(other))
    );
    assert_eq!((vitals(&app), history(&app), control(&app)), before);
}

#[test]
fn restart_edge_is_retained_across_zero_ticks_then_consumed_only_once() {
    let mut accumulator = GameInputAccumulator::default();
    accumulator.collect(GameActionSnapshot {
        restart_pressed: true,
        move_right: true,
        ..GameActionSnapshot::default()
    });
    assert!(accumulator.latched_restart());
    let first = accumulator.form_command_batch(LOCAL_PARTICIPANT_ID, SimulationTick(11));
    let second = accumulator.form_command_batch(LOCAL_PARTICIPANT_ID, SimulationTick(12));
    assert!(first.commands[0].command.restart);
    assert!(!second.commands[0].command.restart);
    assert_eq!(first.commands[0].command.move_x, 1);
    assert_eq!(second.commands[0].command.move_x, 1);
}

#[test]
fn replay_remains_identical_across_entity_and_command_order() {
    #[derive(Debug, Copy, Clone, Component)]
    struct Filler;

    let mut first = integrated_game();
    let mut second = integrated_game();
    first.world_mut().spawn(Filler).unwrap();
    let other = ParticipantId(17);
    for app in [&mut first, &mut second] {
        app.world_mut()
            .spawn((
                ArenaPlayer { participant: other },
                PlayerControlState::default(),
                PlayerPhysicalHistory::spawned(ARENA_PLAYER_SPAWN),
                PlayerVitals::default(),
            ))
            .unwrap();
    }
    second.world_mut().spawn(Filler).unwrap();
    // Different unrelated entity and command participant order cannot affect replay.
    for tick in 2..=42 {
        let inputs = [
            (
                LOCAL_PARTICIPANT_ID,
                PlayerCommand {
                    move_x: 1,
                    move_y: 1,
                    ..PlayerCommand::default()
                },
            ),
            (other, PlayerCommand::default()),
        ];
        let mut reversed = inputs;
        reversed.reverse();
        let first_batch = command_batch(tick, &inputs);
        let second_batch = command_batch(tick, &reversed);
        apply_game_commands(first.world_mut(), first_batch.tick, &first_batch).unwrap();
        apply_game_commands(second.world_mut(), second_batch.tick, &second_batch).unwrap();
    }
    for participant in [LOCAL_PARTICIPANT_ID, other] {
        assert_eq!(
            player_state_for(first.world(), participant)
                .unwrap()
                .last_applied_tick,
            player_state_for(second.world(), participant)
                .unwrap()
                .last_applied_tick
        );
        assert_eq!(
            player_physical_history_for(first.world(), participant),
            player_physical_history_for(second.world(), participant)
        );
        assert_eq!(
            player_vitals_for(first.world(), participant),
            player_vitals_for(second.world(), participant)
        );
    }
}

#[test]
fn occupied_spawn_is_not_accepted_as_a_restart_destination() {
    let mut app = integrated_game();
    defeat_at_hazard(&mut app);
    app.world_mut()
        .resource_mut::<runenwerk_arena::ArenaMovementConfig>()
        .unwrap()
        .character
        .radius = 1.0;
    let before = (vitals(&app), history(&app), control(&app));
    let batch = command_batch(
        63,
        &[(
            LOCAL_PARTICIPANT_ID,
            PlayerCommand {
                restart: true,
                ..PlayerCommand::default()
            },
        )],
    );
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::SpawnUnavailable(LOCAL_PARTICIPANT_ID))
    );
    assert_eq!((vitals(&app), history(&app), control(&app)), before);
}

#[test]
fn bad_character_radius_and_future_damage_tick_are_fail_closed() {
    let mut app = integrated_game();
    let before = (vitals(&app), history(&app), control(&app));
    let batch = command_batch(2, &[(LOCAL_PARTICIPANT_ID, PlayerCommand::default())]);
    app.world_mut()
        .resource_mut::<runenwerk_arena::ArenaMovementConfig>()
        .unwrap()
        .character
        .radius = f32::NAN;
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::InvalidCharacterRadius(
            LOCAL_PARTICIPANT_ID
        ))
    );
    assert_eq!((vitals(&app), history(&app), control(&app)), before);

    app.world_mut()
        .resource_mut::<runenwerk_arena::ArenaMovementConfig>()
        .unwrap()
        .character
        .radius = 0.25;
    let world = app.world_mut();
    let query = world.query::<(&ArenaPlayer, &mut PlayerVitals)>();
    let (_, vitals) = query
        .iter(world)
        .find(|(player, _)| player.participant == LOCAL_PARTICIPANT_ID)
        .unwrap();
    vitals.health = 2;
    vitals.last_hazard_hit_tick = Some(SimulationTick(2));
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::HazardTickRegression(LOCAL_PARTICIPANT_ID))
    );
    assert_eq!(control(&app), before.2);
    assert_eq!(history(&app), before.1);
}

#[test]
fn later_invalid_restart_preserves_prior_participant_update_atomically() {
    let mut app = integrated_game();
    let other = ParticipantId(17);
    app.world_mut()
        .spawn((
            ArenaPlayer { participant: other },
            PlayerControlState::default(),
            PlayerPhysicalHistory::spawned([2.5, 1.0, 2.5]),
            PlayerVitals {
                health: 0,
                last_hazard_hit_tick: Some(SimulationTick(1)),
            },
        ))
        .unwrap();
    app.world_mut()
        .resource_mut::<runenwerk_arena::ArenaMovementConfig>()
        .unwrap()
        .character
        .radius = 1.0;
    let before = (vitals(&app), history(&app), control(&app));
    let batch = command_batch(
        2,
        &[
            (
                LOCAL_PARTICIPANT_ID,
                PlayerCommand {
                    move_x: 1,
                    ..PlayerCommand::default()
                },
            ),
            (
                other,
                PlayerCommand {
                    restart: true,
                    ..PlayerCommand::default()
                },
            ),
        ],
    );
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::SpawnUnavailable(other))
    );
    assert_eq!((vitals(&app), history(&app), control(&app)), before);
    assert_eq!(
        player_state_for(app.world(), other),
        Some(PlayerControlState::default())
    );
}

#[test]
fn ambiguous_player_identity_rejects_mutation_and_fails_closed_on_health_read() {
    let mut app = integrated_game();
    let before = (vitals(&app), history(&app), control(&app));
    app.world_mut()
        .spawn((
            ArenaPlayer {
                participant: LOCAL_PARTICIPANT_ID,
            },
            PlayerControlState::default(),
            PlayerPhysicalHistory::spawned(ARENA_PLAYER_SPAWN),
            PlayerVitals::default(),
        ))
        .unwrap();
    assert_eq!(player_vitals_for(app.world(), LOCAL_PARTICIPANT_ID), None);
    let batch = command_batch(2, &[(LOCAL_PARTICIPANT_ID, PlayerCommand::default())]);
    assert_eq!(
        apply_game_commands(app.world_mut(), batch.tick, &batch),
        Err(GameCommandError::DuplicateParticipant(LOCAL_PARTICIPANT_ID))
    );
    let query = app.world().query::<(&ArenaPlayer, &PlayerControlState)>();
    let controls: Vec<_> = query
        .iter(app.world())
        .filter(|(player, _)| player.participant == LOCAL_PARTICIPANT_ID)
        .map(|(_, state)| *state)
        .collect();
    assert_eq!(controls.len(), 2);
    assert!(controls.contains(&before.2));
    assert!(controls.contains(&PlayerControlState::default()));
}

#[derive(Debug, Copy, Clone, Default, Component, Resource)]
struct RestartScriptFrame(u8);

#[derive(Debug, Default, Component, Resource)]
struct ObservedRestartCommands(Vec<(SimulationTick, bool)>);

struct RestartInputFrameScript;

impl Plugin for RestartInputFrameScript {
    fn build(&self, app: &mut App) {
        app.insert_resource(FixedTimeConfig { step_seconds: 0.1 });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.init_resource::<RestartScriptFrame>();
        app.init_resource::<ObservedRestartCommands>();
        app.add_systems(
            PreUpdate,
            inject_restart_across_zero_tick_frame
                .after(CoreSet::Time)
                .before(CoreSet::Input),
        );
        app.add_systems(
            FixedUpdate,
            observe_formed_restart_commands.after(CoreSet::Simulation),
        );
    }
}

fn inject_restart_across_zero_tick_frame(
    mut time: ResMut<Time>,
    mut input: ResMut<InputState>,
    mut frame: ResMut<RestartScriptFrame>,
) {
    if frame.0 == 0 {
        time.delta_seconds = 0.0;
        input.handle_keyboard_input(KeyCode::KeyR, ElementState::Pressed, None);
    } else {
        time.delta_seconds = 0.2;
        input.handle_keyboard_input(KeyCode::KeyR, ElementState::Released, None);
    }
    frame.0 = frame.0.saturating_add(1);
}

fn observe_formed_restart_commands(
    batch: Res<LastLocalCommandBatch>,
    mut recorded: ResMut<ObservedRestartCommands>,
) {
    if let Some(ref batch) = batch.0 {
        recorded
            .0
            .push((batch.tick, batch.commands[0].command.restart));
    }
}

#[test]
fn actual_restart_key_survives_zero_tick_frame_and_fires_once_during_two_tick_catchup() {
    let mut app = build_headless_game_app();
    app.add_plugin(RestartInputFrameScript);
    let app = app
        .run_for_frames(2)
        .expect("scripted KeyR ingress and two headless fixed steps must execute");

    let observed = app
        .world()
        .resource::<ObservedRestartCommands>()
        .expect("game-owned tick batch observation should exist");
    assert_eq!(
        observed.0,
        vec![(SimulationTick(1), true), (SimulationTick(2), false)]
    );
    assert_eq!(control(&app).applied_command_count, 2);
    assert_eq!(vitals(&app), PlayerVitals::default());
    assert!(
        !app.world()
            .resource::<GameInputAccumulator>()
            .expect("game input accumulator should persist")
            .latched_restart()
    );
}
