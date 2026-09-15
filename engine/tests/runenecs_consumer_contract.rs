use engine::prelude::*;
use runen_ecs::{Commands, Res, ResMut, Resource, SystemSet, WorldMut};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Resource)]
struct StartupFlag(Arc<AtomicBool>);

fn mark_startup(flag: Res<StartupFlag>) {
    flag.0.store(true, Ordering::SeqCst);
}

fn invalid_system(_: WorldMut, _: Res<StartupFlag>) {}

#[test]
fn app_rejects_registration_errors_before_startup() {
    let startup_ran = Arc::new(AtomicBool::new(false));
    let mut app = App::headless();
    app.insert_resource(StartupFlag(startup_ran.clone()));
    app.add_systems(Startup, mark_startup);
    app.add_systems(Update, invalid_system.on_invoker_thread());

    let error = app
        .run_for_frames(1)
        .err()
        .expect("invalid composition should be rejected");

    let message = format!("{error:#}");
    assert!(message.contains("App composition admission rejected"));
    assert!(message.contains("Update"));
    assert!(message.contains("conflicting param borrows"));
    assert!(!startup_ran.load(Ordering::SeqCst));
}

#[derive(Copy, Clone)]
struct MissingSet;

impl SystemSet for MissingSet {
    fn name(&self) -> &'static str {
        "MissingSet"
    }
}

fn topology_probe() {}

#[test]
fn app_rejects_invalid_topology_before_startup() {
    let startup_ran = Arc::new(AtomicBool::new(false));
    let mut app = App::headless();
    app.insert_resource(StartupFlag(startup_ran.clone()));
    app.add_systems(Startup, mark_startup);
    app.add_systems(Update, topology_probe.after(MissingSet));

    let error = app
        .run_for_frames(1)
        .err()
        .expect("invalid topology should be rejected");

    let message = format!("{error:#}");
    assert!(message.contains("App topology admission rejected"));
    assert!(message.contains("MissingSet"));
    assert!(!startup_ran.load(Ordering::SeqCst));
}

#[derive(Resource)]
struct Produced(u32);

#[derive(Default, Resource)]
struct Consumed(u32);

fn produce_startup_resource(mut world: WorldMut) {
    world.insert_resource(Produced(42));
}

fn consume_startup_resource(produced: Res<Produced>, mut consumed: ResMut<Consumed>) {
    consumed.0 = produced.0;
}

#[test]
fn startup_can_admit_and_publish_a_resource_for_update() {
    let mut app = App::headless();
    app.init_resource::<Consumed>();
    app.add_systems(Startup, produce_startup_resource.on_invoker_thread());
    app.add_systems(Update, consume_startup_resource);

    let app = app
        .run_for_frames(1)
        .expect("valid startup and update composition should run");

    assert_eq!(app.world().resource::<Consumed>().unwrap().0, 42);
}

#[derive(Component)]
struct SpawnedByDefaultCommands;

fn spawn_with_default_commands(mut commands: Commands) {
    commands.spawn(SpawnedByDefaultCommands);
}

#[test]
fn ordinary_commands_use_the_transfer_safe_default() {
    let mut app = App::headless();
    app.add_systems(Startup, spawn_with_default_commands);

    let app = app
        .run_for_frames(0)
        .expect("ordinary Commands should not require a local wrapper");

    let query = app.world().query_state::<&SpawnedByDefaultCommands, ()>();
    assert_eq!(query.iter(app.world()).count(), 1);
}
