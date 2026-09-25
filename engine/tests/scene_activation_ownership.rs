use engine::plugins::scene::set_world_paused;
use engine::plugins::{RenderPlugin, ReplayPlugin, ScenePlugin, default_plugins};
use engine::prelude::App;

fn assert_scene_control_rejected(app: &mut App) {
    let error = set_world_paused(app.world_mut(), true)
        .err()
        .expect("Scene controls must reject when ScenePlugin is absent");
    assert!(format!("{error:#}").contains("ScenePlugin is not installed"));
}

#[test]
fn default_stack_does_not_activate_scene_controls() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());

    assert_scene_control_rejected(&mut app);
}

#[test]
fn replay_shared_scene_substrate_does_not_activate_scene_controls() {
    let mut app = App::headless();
    app.add_plugin(ReplayPlugin);

    assert_scene_control_rejected(&mut app);
}

#[test]
fn render_shared_scene_substrate_does_not_activate_scene_controls() {
    let mut app = App::headless();
    app.add_plugin(RenderPlugin);

    assert_scene_control_rejected(&mut app);
}

#[test]
fn scene_plugin_activation_admits_scene_controls() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);

    set_world_paused(app.world_mut(), true)
        .expect("Scene controls should be admitted after ScenePlugin selection");
}
