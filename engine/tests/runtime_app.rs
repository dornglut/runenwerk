use engine::prelude::*;

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
