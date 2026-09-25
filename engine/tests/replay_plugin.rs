use engine::plugins::{
    ReplayControllerResource, ReplayRecorderResource, ReplayState, ScenePlugin, default_plugins,
};
use engine::prelude::*;

#[test]
fn replay_controls_reject_without_replay_plugin() {
    let mut app = App::headless();

    let error = match app.start_recording() {
        Ok(_) => panic!("Replay controls must reject when ReplayPlugin is absent"),
        Err(error) => error,
    };

    assert!(format!("{error:#}").contains("ReplayPlugin is not installed"));
}

#[test]
fn public_replay_resources_do_not_activate_replay_controls() {
    let mut app = App::headless();
    app.init_resource::<ReplayState>();
    app.init_resource::<ReplayRecorderResource>();
    app.init_resource::<ReplayControllerResource>();

    let error = match app.start_recording() {
        Ok(_) => panic!("Replay resources alone must not activate Replay controls"),
        Err(error) => error,
    };

    assert!(format!("{error:#}").contains("ReplayPlugin is not installed"));
}

#[test]
fn replay_plugin_records_scene_ticks_and_seeks_back_to_a_target_tick() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.start_recording()
        .expect("replay recording should start with ReplayPlugin installed");

    let mut app = app
        .run_for_fixed_steps(60)
        .expect("scene and replay plugins should run");
    let archive = app.stop_recording().expect("recording should stop cleanly");
    assert!(archive.journal.len() >= 60);
    assert!(
        archive
            .checkpoints
            .iter()
            .any(|checkpoint| checkpoint.meta.tick.0 == 0)
    );

    let mut replay_app = App::headless();
    replay_app.add_plugins(default_plugins());
    replay_app.add_plugin(ScenePlugin);
    replay_app
        .load_replay(archive)
        .expect("replay archive should load");
    let report = replay_app
        .seek_tick(60)
        .expect("replay seek should restore the recorded state");
    assert!(report.is_clean());
    assert_eq!(replay_app.current_tick(), 60);
    let scene_state = replay_app
        .world()
        .resource::<SceneRuntimeState>()
        .expect("scene runtime state should be published");
    assert_eq!(scene_state.world_scene_label, "gameplay_stub");
}
