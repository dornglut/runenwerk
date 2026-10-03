use engine::plugins::{DebugMetricsPlugin, ScenePlugin};
use engine::prelude::{App, SceneRuntimeState};

#[test]
fn bare_app_does_not_install_scene_runtime_state() {
    let app = App::headless();

    assert!(app.world().resource::<SceneRuntimeState>().is_err());
}

#[test]
fn scene_plugin_installs_scene_runtime_state() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);

    assert!(app.world().resource::<SceneRuntimeState>().is_ok());
}

#[test]
fn scene_plugin_preserves_preinserted_scene_runtime_state() {
    let mut app = App::headless();
    let scene = SceneRuntimeState {
        world_scene_label: "configured-scene".to_string(),
        ..Default::default()
    };
    app.insert_resource(scene);

    app.add_plugin(ScenePlugin);

    assert_eq!(
        app.world()
            .resource::<SceneRuntimeState>()
            .unwrap()
            .world_scene_label,
        "configured-scene"
    );
}

#[test]
fn debug_metrics_plugin_does_not_install_scene_runtime_state() {
    let mut app = App::headless();
    app.add_plugin(DebugMetricsPlugin);

    assert!(app.world().resource::<SceneRuntimeState>().is_err());
}
