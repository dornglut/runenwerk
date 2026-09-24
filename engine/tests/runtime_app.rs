use engine::plugins::input::domain::action;
use engine::plugins::{
    ActionState, FixedStepPlugin, InputFinalizePlugin, SimulationPlugin, TimePlugin, default_plugins,
};
use engine::prelude::*;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[derive(Debug, Copy, Clone, PartialEq, Component, runen_ecs::Resource)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, runen_ecs::Resource)]
struct Velocity {
    x: i32,
    y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Component, runen_ecs::Resource)]
struct FrameCounter(u32);

struct MinimalPlugin;

impl Plugin for MinimalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameCounter>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, movement);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Position { x: 0, y: 0 }, Velocity { x: 2, y: 1 }));
}

fn movement(mut query: Query<(&mut Position, &Velocity)>, mut frames: ResMut<FrameCounter>) {
    for (position, velocity) in query.iter() {
        position.x += velocity.x;
        position.y += velocity.y;
    }
    frames.0 += 1;
}

#[test]
fn app_runs_startup_once_and_updates_each_frame() {
    let mut app = App::headless();
    app.add_plugin(MinimalPlugin);
    let app = app.run_for_frames(3).expect("headless app should run");

    assert_eq!(app.world().resource::<FrameCounter>().unwrap().0, 3);

    let world = app.world();
    let query = world.query::<&Position>();
    let positions: Vec<_> = query.iter(world).copied().collect();
    assert_eq!(positions, vec![Position { x: 6, y: 3 }]);
}

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct StartupSnapshot {
    saw_headless_window: bool,
    saw_title: String,
}

struct ResourceVisibilityPlugin;

impl Plugin for ResourceVisibilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StartupSnapshot>();
        app.add_systems(Startup, capture_startup_resources);
    }
}

fn capture_startup_resources(
    window: Res<WindowState>,
    _input: Res<InputState>,
    mut snapshot: ResMut<StartupSnapshot>,
) {
    snapshot.saw_headless_window = window.is_headless();
    snapshot.saw_title = window.title.clone();
}

#[test]
fn selected_input_capability_is_visible_before_startup() {
    let mut app = App::headless();
    app.set_title("Headless Runtime");
    app.add_plugin(InputFinalizePlugin);
    app.add_plugin(ResourceVisibilityPlugin);
    let app = app
        .run_for_frames(0)
        .expect("startup-only run should succeed");

    let snapshot = app.world().resource::<StartupSnapshot>().unwrap();
    assert!(snapshot.saw_headless_window);
    assert_eq!(snapshot.saw_title, "Headless Runtime");
}

#[test]
fn input_finalize_plugin_installs_owned_input_resources() {
    let mut app = App::headless();
    assert!(app.world().resource::<InputState>().is_err());
    assert!(app.world().resource::<ActionState>().is_err());

    app.add_plugin(InputFinalizePlugin);

    assert!(app.world().resource::<InputState>().is_ok());
    assert!(app.world().resource::<ActionState>().is_ok());
}

#[test]
fn input_bindings_require_selected_input_capability() {
    let mut app = App::headless();
    app.add_input_bindings([(
        "test.action",
        PhysicalKeyIdentity::code("KeyA"),
    )]);

    let error = app
        .run_for_frames(0)
        .expect_err("bindings without input capability should reject App composition");
    assert!(
        format!("{error:#}").contains("InputFinalizePlugin"),
        "composition error should identify the missing input capability: {error:#}"
    );
}

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct OrderLog(Vec<&'static str>);

#[derive(Debug, Copy, Clone)]
struct InputStage;

impl SystemSet for InputStage {
    fn name(&self) -> &'static str {
        "InputStage"
    }
}

struct OrderingPlugin;

impl Plugin for OrderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrderLog>();
        app.add_systems(
            Update,
            (
                ordered_before.before(InputStage),
                ordered_root.in_set(InputStage),
                ordered_after.after(InputStage),
            ),
        );
    }
}

fn ordered_before(mut log: ResMut<OrderLog>) {
    log.0.push("before");
}

fn ordered_root(mut log: ResMut<OrderLog>) {
    log.0.push("root");
}

fn ordered_after(mut log: ResMut<OrderLog>) {
    log.0.push("after");
}

#[test]
fn tuple_system_registration_respects_set_ordering() {
    let mut app = App::headless();
    app.add_plugin(OrderingPlugin);
    let app = app.run_for_frames(1).expect("ordered systems should run");

    assert_eq!(
        app.world().resource::<OrderLog>().unwrap().0,
        vec!["before", "root", "after"]
    );
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, runen_ecs::Resource)]
struct Player;

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct DemoFrames(u32);

struct DemoLogicPlugin;

impl Plugin for DemoLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DemoFrames>();
        app.add_plugins(default_plugins());
        app.add_systems(Startup, setup_demo_player);
        app.add_systems(PreUpdate, inject_demo_input.before(CoreSet::Input));
        app.add_systems(Update, update_demo_title);
    }
}

fn setup_demo_player(mut commands: Commands) {
    commands.spawn((Player, Position { x: 0, y: 0 }));
}

fn inject_demo_input(mut input: ResMut<InputState>, mut frames: ResMut<DemoFrames>) {
    if frames.0 == 0 {
        input.handle_keyboard_input(KeyCode::KeyD, ElementState::Pressed, None);
        input.handle_keyboard_input(KeyCode::Escape, ElementState::Pressed, None);
    }
    frames.0 += 1;
}

fn update_demo_title(
    actions: Res<ActionState>,
    time: Res<Time>,
    mut window: ResMut<WindowState>,
    mut query: Query<&mut Position>,
) {
    let position = query.single().expect("demo should have one position");
    if actions.action_down(action::WORLD_MOVE_RIGHT) {
        position.x += 1;
    }

    window.set_title(format!("x={} dt={:.4}", position.x, time.delta_seconds));
    if actions.action_pressed(action::SYSTEM_TOGGLE_PAUSE_MENU) {
        window.request_close();
    }
}

#[test]
fn demo_style_plugin_updates_title_and_close_state_headlessly() {
    let mut app = App::headless();
    app.add_plugin(DemoLogicPlugin);
    let app = app.run_for_frames(1).expect("demo logic should run");

    let window = app.world().resource::<WindowState>().unwrap();
    assert!(window.close_requested);
    assert!(window.title.contains("x=1"));
    assert!(window.title.contains("dt="));

    let world = app.world();
    let query = world.query::<&Position>();
    let positions: Vec<_> = query.iter(world).copied().collect();
    assert_eq!(positions, vec![Position { x: 1, y: 0 }]);
}

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct FixedScheduleLog(Vec<&'static str>);

struct FixedTickPlugin;

impl Plugin for FixedTickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FixedScheduleLog>();
        app.insert_resource(FixedTimeConfig {
            step_seconds: 1.0 / 60.0,
        });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.add_systems(PreUpdate, log_pre_update);
        app.add_systems(FixedUpdate, log_fixed_update);
        app.add_systems(Update, log_update);
        app.add_systems(FrameEnd, log_frame_end);
    }
}

fn log_pre_update(mut log: ResMut<FixedScheduleLog>) {
    log.0.push("pre");
}

fn log_fixed_update(mut log: ResMut<FixedScheduleLog>) {
    log.0.push("fixed");
}

fn log_update(mut log: ResMut<FixedScheduleLog>) {
    log.0.push("update");
}

fn log_frame_end(mut log: ResMut<FixedScheduleLog>) {
    log.0.push("frame_end");
}

#[test]
fn run_for_fixed_steps_executes_fixed_update_deterministically() {
    let mut app = App::headless();
    app.add_plugins((FixedStepPlugin, SimulationPlugin));
    app.add_plugin(FixedTickPlugin);
    let app = app
        .run_for_fixed_steps(3)
        .expect("fixed-step runner should stop after the requested step count");

    assert_eq!(app.world().resource::<SimulationTick>().unwrap().0, 3);
    assert_eq!(
        app.world().resource::<FixedScheduleLog>().unwrap().0,
        vec![
            "pre",
            "fixed",
            "update",
            "frame_end",
            "pre",
            "fixed",
            "update",
            "frame_end",
            "pre",
            "fixed",
            "update",
            "frame_end",
        ]
    );

    let fixed_state = app.world().resource::<FixedTimeState>().unwrap();
    assert_eq!(fixed_state.steps_ran_last_frame, 1);
    assert_eq!(fixed_state.saturated_frames, 0);
    assert_eq!(fixed_state.total_completed_steps, 3);
}

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct ScriptedDeltaState {
    next_frame: usize,
    fixed_updates: u32,
}

struct ScriptedDeltaPlugin;

impl Plugin for ScriptedDeltaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScriptedDeltaState>();
        app.insert_resource(FixedTimeConfig { step_seconds: 0.1 });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.add_systems(PreUpdate, scripted_delta.after(CoreSet::Time));
        app.add_systems(FixedUpdate, count_fixed_update);
    }
}

fn scripted_delta(mut time: ResMut<Time>, mut state: ResMut<ScriptedDeltaState>) {
    time.delta_seconds = if state.next_frame == 0 { 0.0 } else { 0.35 };
    state.next_frame += 1;
}

fn count_fixed_update(mut state: ResMut<ScriptedDeltaState>) {
    state.fixed_updates += 1;
}

#[test]
fn fixed_step_schedule_supports_zero_and_batched_ticks_per_frame() {
    let mut app = App::headless();
    app.add_plugins((TimePlugin, FixedStepPlugin, SimulationPlugin));
    app.add_plugin(ScriptedDeltaPlugin);
    let app = app
        .run_for_frames(2)
        .expect("scripted fixed-step frames should run");

    let state = app.world().resource::<ScriptedDeltaState>().unwrap();
    assert_eq!(state.fixed_updates, 3);
    assert_eq!(app.world().resource::<SimulationTick>().unwrap().0, 3);

    let fixed_state = app.world().resource::<FixedTimeState>().unwrap();
    assert_eq!(fixed_state.steps_ran_last_frame, 3);
    assert_eq!(fixed_state.saturated_frames, 0);
    assert_eq!(fixed_state.total_completed_steps, 3);
}

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct TickVisibilityLog(Vec<u64>);

struct TickVisibilityPlugin;

impl Plugin for TickVisibilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TickVisibilityLog>();
        app.insert_resource(FixedTimeConfig {
            step_seconds: 1.0 / 60.0,
        });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.add_systems(PreUpdate, force_large_delta.after(CoreSet::Time));
        app.add_systems(FixedUpdate, observe_tick_during_fixed_update);
    }
}

fn force_large_delta(mut time: ResMut<Time>) {
    time.delta_seconds = 0.05;
}

fn observe_tick_during_fixed_update(tick: Res<SimulationTick>, mut log: ResMut<TickVisibilityLog>) {
    log.0.push(tick.0);
}

#[test]
fn simulation_plugin_advances_tick_before_each_fixed_update_step() {
    let mut app = App::headless();
    app.add_plugins((TimePlugin, FixedStepPlugin, SimulationPlugin));
    app.add_plugin(TickVisibilityPlugin);
    let app = app.run_for_frames(1).expect("fixed-step frame should run");

    assert_eq!(app.world().resource::<SimulationTick>().unwrap().0, 3);
    assert_eq!(
        app.world().resource::<TickVisibilityLog>().unwrap().0,
        vec![1, 2, 3]
    );
    assert_eq!(
        app.world()
            .resource::<FixedTimeState>()
            .unwrap()
            .steps_ran_last_frame,
        3
    );
}

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct SaturationFixedStepCounter(u32);

struct SaturationPlugin;

impl Plugin for SaturationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaturationFixedStepCounter>();
        app.insert_resource(FixedTimeConfig {
            step_seconds: 1.0 / 60.0,
        });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 1,
        });
        app.add_systems(PreUpdate, force_saturating_delta.after(CoreSet::Time));
        app.add_systems(FixedUpdate, count_saturation_fixed_step);
    }
}

fn force_saturating_delta(mut time: ResMut<Time>) {
    time.delta_seconds = 0.25;
}

fn count_saturation_fixed_step(mut count: ResMut<SaturationFixedStepCounter>) {
    count.0 += 1;
}

#[test]
fn fixed_step_saturation_tracks_dropped_backlog_when_budget_is_exhausted() {
    let mut app = App::headless();
    app.add_plugins((TimePlugin, FixedStepPlugin, SimulationPlugin));
    app.add_plugin(SaturationPlugin);
    let app = app.run_for_frames(1).expect("saturation frame should run");

    assert_eq!(app.world().resource::<SimulationTick>().unwrap().0, 1);
    assert_eq!(
        app.world()
            .resource::<SaturationFixedStepCounter>()
            .unwrap()
            .0,
        1
    );

    let fixed_state = app.world().resource::<FixedTimeState>().unwrap();
    assert_eq!(fixed_state.steps_ran_last_frame, 1);
    assert_eq!(fixed_state.saturated_frames, 1);
    assert_eq!(fixed_state.accumulator_seconds, 0.0);
    assert_eq!(fixed_state.total_completed_steps, 1);
}

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct RuntimeLifecycleLog(Vec<&'static str>);

struct RuntimeLifecyclePlugin;

impl Plugin for RuntimeLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RuntimeLifecycleLog>();
        app.insert_resource(FixedTimeConfig {
            step_seconds: 1.0 / 60.0,
        });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 1,
        });
        app.add_systems(Startup, lifecycle_log_startup);
        app.add_systems(PreUpdate, lifecycle_log_pre_update);
        app.add_systems(FixedUpdate, lifecycle_log_fixed_update);
        app.add_systems(Update, lifecycle_log_update);
        app.add_systems(RenderPrepare, lifecycle_log_render_prepare);
        app.add_systems(RenderSubmit, lifecycle_log_render_submit);
        app.add_systems(FrameEnd, lifecycle_log_frame_end);
    }
}

fn lifecycle_log_startup(mut log: ResMut<RuntimeLifecycleLog>) {
    log.0.push("startup");
}

fn lifecycle_log_pre_update(mut log: ResMut<RuntimeLifecycleLog>) {
    log.0.push("pre_update");
}

fn lifecycle_log_fixed_update(mut log: ResMut<RuntimeLifecycleLog>) {
    log.0.push("fixed_update");
}

fn lifecycle_log_update(mut log: ResMut<RuntimeLifecycleLog>) {
    log.0.push("update");
}

fn lifecycle_log_render_prepare(mut log: ResMut<RuntimeLifecycleLog>) {
    log.0.push("render_prepare");
}

fn lifecycle_log_render_submit(mut log: ResMut<RuntimeLifecycleLog>) {
    log.0.push("render_submit");
}

fn lifecycle_log_frame_end(mut log: ResMut<RuntimeLifecycleLog>) {
    log.0.push("frame_end");
}

#[test]
fn runtime_lifecycle_runs_startup_then_selected_fixed_cadence_then_frame_order() {
    let mut app = App::headless();
    app.add_plugin(FixedStepPlugin);
    app.add_plugin(RuntimeLifecyclePlugin);
    let app = app.run_for_frames(1).expect("lifecycle frame should run");

    assert_eq!(
        app.world().resource::<RuntimeLifecycleLog>().unwrap().0,
        vec![
            "startup",
            "pre_update",
            "fixed_update",
            "update",
            "render_prepare",
            "render_submit",
            "frame_end",
        ]
    );
}

#[test]
fn app_tracks_scene_registrations_without_legacy_runtime() {
    let mut app = App::headless();
    app.add_scene("engine/tests/fixtures/scene_templates/main_menu.ron");
    app.add_scene_template("engine/tests/fixtures/scene_templates/main_menu.ron");
    app.add_scene_template("engine/tests/fixtures/scene_templates/main_menu.ron");

    assert_eq!(app.registered_scene_count(), 3);

    let catalog = app.world().resource::<SceneCatalog>().unwrap();
    assert_eq!(catalog.len(), 3);
    assert!(catalog.handle("main_menu").is_some());
    assert!(catalog.handle("main_menu_2").is_some());
    assert!(catalog.handle("main_menu_3").is_some());
}

struct ZeroDeltaPlugin;

impl Plugin for ZeroDeltaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FixedTimeConfig {
            step_seconds: 1.0 / 60.0,
        });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.add_systems(PreUpdate, force_zero_delta.after(CoreSet::Time));
    }
}

fn force_zero_delta(mut time: ResMut<Time>) {
    time.delta_seconds = 0.0;
}

#[test]
fn zero_fixed_step_frames_do_not_advance_simulation_tick() {
    let mut app = App::headless();
    app.add_plugins((TimePlugin, FixedStepPlugin, SimulationPlugin));
    app.add_plugin(ZeroDeltaPlugin);
    let app = app
        .run_for_frames(2)
        .expect("zero-delta frames should run without fixed updates");

    assert_eq!(app.world().resource::<SimulationTick>().unwrap().0, 0);
}
