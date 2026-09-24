use engine::plugins::{DebugMetricsPlugin, ScenePlugin};
use engine::prelude::*;

#[test]
fn bare_app_does_not_install_scene_overlay_viewport_state() {
    let app = App::headless();

    assert!(app.world().resource::<SceneOverlayViewportState>().is_err());
}

#[test]
fn scene_plugin_installs_scene_overlay_viewport_state() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);

    let viewport = app
        .world()
        .resource::<SceneOverlayViewportState>()
        .expect("ScenePlugin should provide scene overlay viewport state");
    assert_eq!(viewport.screen_size, (1280.0, 720.0));
    assert_eq!(viewport.scale, 1.0);
}

#[test]
fn headless_scene_startup_uses_primary_presentation_metrics() {
    let mut app = App::headless();
    app.insert_resource(PrimaryPresentationMetricsResource::new((960, 540), 1.5));
    app.add_plugin(ScenePlugin);
    let app = app
        .run_for_frames(0)
        .expect("headless Scene startup should use logical presentation metrics");

    let viewport = app
        .world()
        .resource::<SceneOverlayViewportState>()
        .expect("ScenePlugin should publish its overlay viewport");
    assert_eq!(viewport.screen_size, (960.0, 540.0));
    assert_eq!(viewport.scale, 1.5);
}

#[test]
fn scene_plugin_preserves_preinserted_scene_overlay_viewport_state() {
    let mut app = App::headless();
    let expected = SceneOverlayViewportState {
        screen_size: (960.0, 540.0),
        scale: 1.5,
    };
    app.insert_resource(expected.clone());
    app.add_plugin(ScenePlugin);

    let viewport = app
        .world()
        .resource::<SceneOverlayViewportState>()
        .expect("preinserted viewport state should remain present");
    assert_eq!(viewport, &expected);
}

#[test]
fn debug_metrics_plugin_does_not_install_scene_overlay_viewport_state() {
    let mut app = App::headless();
    app.add_plugin(DebugMetricsPlugin);

    assert!(app.world().resource::<SceneOverlayViewportState>().is_err());
}
