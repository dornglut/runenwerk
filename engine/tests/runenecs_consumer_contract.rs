use engine::prelude::*;
use runen_ecs::{Commands, Res, ResMut, Resource, SystemSet, WorldMut};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
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
struct FrameProbe(Arc<AtomicUsize>);

#[derive(Resource)]
struct LateSystemProbe(Arc<AtomicUsize>);

fn count_frame(probe: Res<FrameProbe>) {
    probe.0.fetch_add(1, Ordering::SeqCst);
}

fn count_late_system(probe: Res<LateSystemProbe>) {
    probe.0.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn late_system_registration_is_deferred_and_does_not_mutate_topology() {
    let frames = Arc::new(AtomicUsize::new(0));
    let late_runs = Arc::new(AtomicUsize::new(0));
    let mut app = App::headless();
    app.insert_resource(FrameProbe(frames.clone()));
    app.add_systems(Update, count_frame);

    let mut app = app
        .run_for_frames(1)
        .expect("initial bounded run should succeed");
    assert_eq!(frames.load(Ordering::SeqCst), 1);

    app.insert_resource(LateSystemProbe(late_runs.clone()));
    app.add_systems(Update, count_late_system);

    let error = app
        .run_for_frames(1)
        .err()
        .expect("late system registration should reject the next execution");
    let message = format!("{error:#}");
    assert!(message.contains("App composition admission rejected"));
    assert!(message.contains("add_systems"));
    assert_eq!(frames.load(Ordering::SeqCst), 1);
    assert_eq!(late_runs.load(Ordering::SeqCst), 0);
}

struct PluginBuildProbe(Arc<AtomicUsize>);

impl Plugin for PluginBuildProbe {
    fn build(&self, _: &mut App) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn late_plugin_registration_does_not_execute_plugin_builds() {
    let builds = Arc::new(AtomicUsize::new(0));
    let mut app = App::headless()
        .run_for_frames(0)
        .expect("initial Startup should succeed");

    app.add_plugin(PluginBuildProbe(builds.clone()));
    app.add_plugins((
        PluginBuildProbe(builds.clone()),
        PluginBuildProbe(builds.clone()),
    ));
    app.add_boxed_plugin(Box::new(PluginBuildProbe(builds.clone())));

    assert_eq!(builds.load(Ordering::SeqCst), 0);
    let error = app
        .run_for_frames(0)
        .err()
        .expect("late plugin registration should reject the next execution");
    let message = format!("{error:#}");
    assert!(message.contains("add_plugin"));
    assert!(message.contains("add_plugins"));
    assert!(message.contains("add_boxed_plugin"));
    assert_eq!(builds.load(Ordering::SeqCst), 0);
}

#[test]
fn late_fixed_step_plugin_rejection_precedes_capability_check() {
    let mut app = App::headless()
        .run_for_frames(0)
        .expect("initial Startup should succeed");

    app.add_plugin(FixedStepPlugin);

    let error = app
        .run_for_fixed_steps(1)
        .err()
        .expect("late plugin topology must reject before fixed-cadence capability checks");
    let message = format!("{error:#}");
    assert!(message.contains("App composition admission rejected"));
    assert!(message.contains("add_plugin"));
    assert!(!message.contains("requires FixedStepPlugin"));
}

#[test]
fn late_publication_handler_registration_does_not_install_handlers() {
    let product_observations = Arc::new(AtomicUsize::new(0));
    let query_observations = Arc::new(AtomicUsize::new(0));
    let mut app = App::headless()
        .run_for_frames(0)
        .expect("initial Startup should succeed");

    let observed_products = product_observations.clone();
    app.add_product_publication_handler(move |_, _| {
        observed_products.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });
    let observed_queries = query_observations.clone();
    app.add_query_snapshot_publication_handler(move |_, _| {
        observed_queries.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });

    dispatch_product_publication(app.world_mut(), "test")
        .expect("existing product publication handlers should remain usable");
    dispatch_query_snapshot_publication(app.world_mut(), "test")
        .expect("existing query publication handlers should remain usable");
    assert_eq!(product_observations.load(Ordering::SeqCst), 0);
    assert_eq!(query_observations.load(Ordering::SeqCst), 0);

    let error = app
        .run_for_frames(0)
        .err()
        .expect("late publication registration should reject the next execution");
    let message = format!("{error:#}");
    assert!(message.contains("add_product_publication_handler"));
    assert!(message.contains("add_query_snapshot_publication_handler"));
}

#[derive(Resource)]
struct RuntimeValue(u32);

#[derive(Default, Resource)]
struct RuntimeObservations(Vec<u32>);

fn observe_runtime_value(value: Res<RuntimeValue>, mut observations: ResMut<RuntimeObservations>) {
    observations.0.push(value.0);
}

#[test]
fn live_resource_mutation_remains_valid_between_bounded_runs() {
    let mut app = App::headless();
    app.insert_resource(RuntimeValue(1));
    app.init_resource::<RuntimeObservations>();
    app.add_systems(Update, observe_runtime_value);

    let mut app = app
        .run_for_frames(1)
        .expect("first bounded run should succeed");
    app.world_mut()
        .resource_mut::<RuntimeValue>()
        .expect("runtime value should exist")
        .0 = 7;
    let app = app
        .run_for_frames(1)
        .expect("runtime resource mutation must not reopen composition");

    assert_eq!(
        app.world()
            .resource::<RuntimeObservations>()
            .expect("runtime observations should exist")
            .0,
        vec![1, 7]
    );
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

    let query = app.world().query::<&SpawnedByDefaultCommands>();
    assert_eq!(query.iter(app.world()).count(), 1);
}
