use engine::plugins::FixedStepPlugin;
use engine::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Clone, Component, Resource)]
struct StartupProbe(Arc<AtomicBool>);

fn mark_startup(probe: Res<StartupProbe>) {
    probe.0.store(true, Ordering::SeqCst);
}

#[test]
fn windowed_app_rejects_bounded_frame_advancement_before_startup() {
    let startup_ran = Arc::new(AtomicBool::new(false));
    let mut app = App::new();
    app.insert_resource(StartupProbe(startup_ran.clone()));
    app.add_systems(Startup, mark_startup);

    let error = match app.run_for_frames(1) {
        Ok(_) => panic!("windowed bounded advancement must reject"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("requires App::headless()"));
    assert!(!startup_ran.load(Ordering::SeqCst));
}

#[test]
fn windowed_app_rejects_bounded_fixed_step_advancement_before_startup() {
    let startup_ran = Arc::new(AtomicBool::new(false));
    let mut app = App::new();
    app.add_plugin(FixedStepPlugin);
    app.insert_resource(StartupProbe(startup_ran.clone()));
    app.add_systems(Startup, mark_startup);

    let error = match app.run_for_fixed_steps(1) {
        Ok(_) => panic!("windowed bounded fixed-step advancement must reject"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("requires App::headless()"));
    assert!(!startup_ran.load(Ordering::SeqCst));
}

#[derive(Debug, Default, Component, Resource)]
struct LifecycleCounts {
    startups: u32,
    updates: u32,
}

fn count_startup(mut counts: ResMut<LifecycleCounts>) {
    counts.startups += 1;
}

fn count_update(mut counts: ResMut<LifecycleCounts>) {
    counts.updates += 1;
}

#[test]
fn headless_bounded_advancement_is_repeatable_and_startup_remains_one_shot() {
    let mut app = App::headless();
    app.init_resource::<LifecycleCounts>();
    app.add_systems(Startup, count_startup);
    app.add_systems(Update, count_update);

    let app = app
        .run_for_frames(2)
        .expect("first bounded headless run should succeed");
    let app = app
        .run_for_frames(3)
        .expect("second bounded headless run should succeed");

    let counts = app.world().resource::<LifecycleCounts>().unwrap();
    assert_eq!(counts.startups, 1);
    assert_eq!(counts.updates, 5);
}

#[derive(Clone, Component, Resource)]
struct SharedFrameCount(Arc<AtomicUsize>);

fn count_shared_frame(frames: Res<SharedFrameCount>) {
    frames.0.fetch_add(1, Ordering::SeqCst);
}

#[test]
fn headless_run_executes_startup_once_and_one_frame() {
    let startup_ran = Arc::new(AtomicBool::new(false));
    let observed_frames = Arc::new(AtomicUsize::new(0));
    let mut app = App::headless();
    app.insert_resource(StartupProbe(startup_ran.clone()));
    app.insert_resource(SharedFrameCount(observed_frames.clone()));
    app.add_systems(Startup, mark_startup);
    app.add_systems(Update, count_shared_frame);

    app.run().expect("plain headless run should succeed");

    assert!(startup_ran.load(Ordering::SeqCst));
    assert_eq!(observed_frames.load(Ordering::SeqCst), 1);
}

#[test]
fn zero_frame_advancement_runs_startup_without_update() {
    let mut app = App::headless();
    app.init_resource::<LifecycleCounts>();
    app.add_systems(Startup, count_startup);
    app.add_systems(Update, count_update);

    let app = app
        .run_for_frames(0)
        .expect("zero-frame headless advancement should admit and start");

    let counts = app.world().resource::<LifecycleCounts>().unwrap();
    assert_eq!(counts.startups, 1);
    assert_eq!(counts.updates, 0);
}
