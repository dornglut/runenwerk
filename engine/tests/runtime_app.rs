use engine::plugins::default_runtime_plugins;
use engine::prelude::*;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[derive(Debug, Copy, Clone, PartialEq, Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
struct Velocity {
    x: i32,
    y: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
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
    for (position, velocity) in query.iter_mut() {
        position.x += velocity.x;
        position.y += velocity.y;
    }
    frames.0 += 1;
}

#[test]
fn typed_app_runs_startup_once_and_updates_each_frame() {
    let mut app = App::new();
    app.add_plugin(MinimalPlugin);
    let app = app.run_for_frames(3).expect("headless app should run");

    assert_eq!(app.world().resource::<FrameCounter>().unwrap().0, 3);

    let positions: Vec<_> = app.world().query::<&Position>().iter().copied().collect();
    assert_eq!(positions, vec![Position { x: 6, y: 3 }]);
}

#[derive(Debug, Default)]
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
    _time: Res<Time>,
    _input: Res<InputState>,
    mut snapshot: ResMut<StartupSnapshot>,
) {
    snapshot.saw_headless_window = window.is_headless();
    snapshot.saw_title = window.title.clone();
}

#[test]
fn headless_run_exposes_builtin_runtime_resources_before_startup() {
    let mut app = App::headless();
    app.set_title("Headless Runtime");
    app.add_plugin(ResourceVisibilityPlugin);
    let app = app
        .run_for_frames(0)
        .expect("startup-only run should succeed");

    let snapshot = app.world().resource::<StartupSnapshot>().unwrap();
    assert!(snapshot.saw_headless_window);
    assert_eq!(snapshot.saw_title, "Headless Runtime");
}

#[derive(Debug, Default)]
struct OrderLog(Vec<&'static str>);

#[derive(Debug, Copy, Clone)]
struct InputStage;

impl SystemSet for InputStage {
    fn name() -> &'static str {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
struct Player;

#[derive(Debug, Default)]
struct DemoFrames(u32);

struct DemoLogicPlugin;

impl Plugin for DemoLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DemoFrames>();
        app.add_plugins(default_runtime_plugins());
        app.add_systems(Startup, setup_demo_player);
        app.add_systems(
            Update,
            (
                inject_demo_input.in_set(CoreSet::Input),
                update_demo_title.after(CoreSet::Input).after(CoreSet::Time),
            ),
        );
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
    input: Res<InputState>,
    time: Res<Time>,
    mut window: ResMut<WindowState>,
    mut query: Query<&mut Position>,
) {
    let position = query.single_mut().expect("demo should have one position");
    if input.world_move_right {
        position.x += 1;
    }

    window.set_title(format!("x={} dt={:.4}", position.x, time.delta_seconds));
    if input.toggle_pause_menu {
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

    let positions: Vec<_> = app.world().query::<&Position>().iter().copied().collect();
    assert_eq!(positions, vec![Position { x: 1, y: 0 }]);
}
