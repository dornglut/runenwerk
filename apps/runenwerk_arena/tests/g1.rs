use engine::plugins::{InputState, SceneResource};
use engine::prelude::*;
use runenwerk_arena::{
    ArenaPlayer, GameActionSnapshot, GameInputAccumulator, LOCAL_PARTICIPANT_ID,
    LastLocalCommandBatch, ParticipantCommand, ParticipantId, PlayerCommand, PlayerControlState,
    TickCommandBatch, apply_game_commands, build_headless_game_app, player_state_for,
};
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[derive(Debug, Copy, Clone, Component)]
struct Filler;

#[derive(Debug, Default, Component, Resource)]
struct ScriptFrame(u8);

struct ZeroThenTickInputPlugin;

impl Plugin for ZeroThenTickInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FixedTimeConfig { step_seconds: 0.1 });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.init_resource::<ScriptFrame>();
        app.add_systems(
            PreUpdate,
            inject_jump_across_zero_tick_frame
                .after(CoreSet::Time)
                .before(CoreSet::Input),
        );
    }
}

fn inject_jump_across_zero_tick_frame(
    mut time: ResMut<Time>,
    mut input: ResMut<InputState>,
    mut frame: ResMut<ScriptFrame>,
) {
    if frame.0 == 0 {
        time.delta_seconds = 0.0;
        input.handle_keyboard_input(KeyCode::Space, ElementState::Pressed, None);
    } else {
        time.delta_seconds = 0.1;
        input.handle_keyboard_input(KeyCode::Space, ElementState::Released, None);
    }
    frame.0 = frame.0.saturating_add(1);
}

struct BatchedTickInputPlugin {
    key: KeyCode,
}

impl Plugin for BatchedTickInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FixedTimeConfig { step_seconds: 0.1 });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.insert_resource(TestKey(self.key));
        app.add_systems(
            PreUpdate,
            inject_pressed_key_for_batched_ticks
                .after(CoreSet::Time)
                .before(CoreSet::Input),
        );
    }
}

#[derive(Debug, Copy, Clone, Component, Resource)]
struct TestKey(KeyCode);

fn inject_pressed_key_for_batched_ticks(
    mut time: ResMut<Time>,
    mut input: ResMut<InputState>,
    key: Res<TestKey>,
) {
    time.delta_seconds = 0.35;
    input.handle_keyboard_input(key.0, ElementState::Pressed, None);
}

#[test]
fn maintained_game_composes_headlessly_without_scene_or_net() {
    let app = build_headless_game_app()
        .run_for_frames(0)
        .expect("maintained game should compose headlessly");

    assert!(app.world().resource::<SceneResource>().is_err());
    assert!(app.world().resource::<NetworkInboundQueue>().is_err());
}

#[test]
fn app_world_owns_the_gameplay_player() {
    let app = build_headless_game_app()
        .run_for_frames(0)
        .expect("startup should publish the local gameplay player");

    let state = player_state_for(app.world(), LOCAL_PARTICIPANT_ID)
        .expect("local gameplay player should live in the App world");
    assert_eq!(state, PlayerControlState::default());
    assert!(app.world().resource::<SceneResource>().is_err());
}

#[test]
fn one_shot_input_survives_a_frame_with_zero_fixed_ticks() {
    let mut app = build_headless_game_app();
    app.add_plugin(ZeroThenTickInputPlugin);
    let app = app
        .run_for_frames(2)
        .expect("scripted zero-tick then fixed-tick frames should run");

    let state = player_state_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert_eq!(state.jump_request_count, 1);
    assert_eq!(state.applied_command_count, 1);
    assert_eq!(state.last_applied_tick, Some(SimulationTick(1)));
}

#[test]
fn one_action_edge_is_consumed_once_across_multiple_fixed_ticks() {
    let mut app = build_headless_game_app();
    app.add_plugin(BatchedTickInputPlugin {
        key: KeyCode::Space,
    });
    let app = app
        .run_for_frames(1)
        .expect("batched fixed ticks should run");

    let state = player_state_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert_eq!(state.applied_command_count, 3);
    assert_eq!(state.jump_request_count, 1);
    assert_eq!(state.last_applied_tick, Some(SimulationTick(3)));
}

#[test]
fn held_movement_is_available_to_each_fixed_tick() {
    let mut app = build_headless_game_app();
    app.add_plugin(BatchedTickInputPlugin { key: KeyCode::KeyD });
    let app = app
        .run_for_frames(1)
        .expect("held movement should span batched fixed ticks");

    let state = player_state_for(app.world(), LOCAL_PARTICIPANT_ID).unwrap();
    assert_eq!(state.applied_command_count, 3);
    assert_eq!(state.movement_command_count, 3);
    assert_eq!(state.last_command.move_x, 1);
    assert_eq!(state.last_command.move_y, 0);
}

#[test]
fn formed_command_batch_records_current_simulation_tick() {
    let mut app = build_headless_game_app();
    app.add_plugin(BatchedTickInputPlugin { key: KeyCode::KeyD });
    let app = app.run_for_frames(1).unwrap();

    let batch = app
        .world()
        .resource::<LastLocalCommandBatch>()
        .unwrap()
        .0
        .as_ref()
        .expect("fixed ticks should form a command batch");
    assert_eq!(batch.tick, SimulationTick(3));
    assert_eq!(batch.commands[0].participant, LOCAL_PARTICIPANT_ID);
}

#[test]
fn local_single_player_applies_commands_without_net_plugin() {
    let app = build_headless_game_app()
        .run_for_fixed_steps(2)
        .expect("local authority should run without NetPlugin");

    assert!(app.world().resource::<NetworkInboundQueue>().is_err());
    assert_eq!(
        player_state_for(app.world(), LOCAL_PARTICIPANT_ID)
            .unwrap()
            .applied_command_count,
        2
    );
}

#[test]
fn command_application_is_directly_callable_for_an_explicit_tick() {
    let mut world = World::new();
    world
        .spawn((
            ArenaPlayer {
                participant: ParticipantId(7),
            },
            PlayerControlState::default(),
        ))
        .unwrap();
    let batch = TickCommandBatch {
        tick: SimulationTick(9),
        commands: vec![ParticipantCommand {
            participant: ParticipantId(7),
            command: PlayerCommand {
                move_x: -1,
                move_y: 1,
                jump: true,
                interact: true,
            },
        }],
    };

    apply_game_commands(&mut world, SimulationTick(9), &batch).unwrap();

    let state = player_state_for(&world, ParticipantId(7)).unwrap();
    assert_eq!(state.last_applied_tick, Some(SimulationTick(9)));
    assert_eq!(state.last_command, batch.commands[0].command);
    assert_eq!(state.jump_request_count, 1);
    assert_eq!(state.interact_request_count, 1);
}

#[test]
fn identical_tick_command_sequences_produce_identical_game_state() {
    fn world_with_player(filler_first: bool) -> World {
        let mut world = World::new();
        if filler_first {
            world.spawn(Filler).unwrap();
        }
        world
            .spawn((
                ArenaPlayer {
                    participant: ParticipantId(12),
                },
                PlayerControlState::default(),
            ))
            .unwrap();
        if !filler_first {
            world.spawn(Filler).unwrap();
        }
        world
    }

    let sequence = [
        TickCommandBatch {
            tick: SimulationTick(1),
            commands: vec![ParticipantCommand {
                participant: ParticipantId(12),
                command: PlayerCommand {
                    move_x: 1,
                    move_y: 0,
                    jump: true,
                    interact: false,
                },
            }],
        },
        TickCommandBatch {
            tick: SimulationTick(2),
            commands: vec![ParticipantCommand {
                participant: ParticipantId(12),
                command: PlayerCommand {
                    move_x: 1,
                    move_y: -1,
                    jump: false,
                    interact: true,
                },
            }],
        },
    ];

    let mut first = world_with_player(true);
    let mut second = world_with_player(false);
    for batch in &sequence {
        apply_game_commands(&mut first, batch.tick, batch).unwrap();
        apply_game_commands(&mut second, batch.tick, batch).unwrap();
    }

    assert_eq!(
        player_state_for(&first, ParticipantId(12)),
        player_state_for(&second, ParticipantId(12))
    );
}

#[test]
fn participant_identity_does_not_depend_on_ecs_spawn_order() {
    let mut first = World::new();
    first.spawn(Filler).unwrap();
    first
        .spawn((
            ArenaPlayer {
                participant: ParticipantId(44),
            },
            PlayerControlState::default(),
        ))
        .unwrap();

    let mut second = World::new();
    second
        .spawn((
            ArenaPlayer {
                participant: ParticipantId(44),
            },
            PlayerControlState::default(),
        ))
        .unwrap();
    second.spawn(Filler).unwrap();

    let batch = TickCommandBatch {
        tick: SimulationTick(5),
        commands: vec![ParticipantCommand {
            participant: ParticipantId(44),
            command: PlayerCommand {
                move_x: 0,
                move_y: 1,
                jump: false,
                interact: true,
            },
        }],
    };
    apply_game_commands(&mut first, batch.tick, &batch).unwrap();
    apply_game_commands(&mut second, batch.tick, &batch).unwrap();

    assert_eq!(
        player_state_for(&first, ParticipantId(44)),
        player_state_for(&second, ParticipantId(44))
    );
}

#[test]
fn accumulator_separates_held_intent_from_latched_edges() {
    let mut accumulator = GameInputAccumulator::default();
    accumulator.collect(GameActionSnapshot {
        move_right: true,
        jump_pressed: true,
        interact_pressed: true,
        ..GameActionSnapshot::default()
    });

    let first = accumulator.form_command_batch(ParticipantId(1), SimulationTick(1));
    let second = accumulator.form_command_batch(ParticipantId(1), SimulationTick(2));

    assert_eq!(first.commands[0].command.move_x, 1);
    assert!(first.commands[0].command.jump);
    assert!(first.commands[0].command.interact);
    assert_eq!(second.commands[0].command.move_x, 1);
    assert!(!second.commands[0].command.jump);
    assert!(!second.commands[0].command.interact);
}
