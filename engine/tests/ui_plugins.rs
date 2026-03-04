use engine::plugins::{ScenePlugin, UiInputPlugin, UiRenderPlugin};
use engine::prelude::*;

#[test]
fn ui_plugins_populate_overlay_draw_list_when_overlay_is_visible() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);
    app.add_plugin(UiInputPlugin);
    app.add_plugin(UiRenderPlugin);
    app.world_mut()
        .resource_mut::<InputState>()
        .expect("input state should exist")
        .toggle_pause_menu = true;

    let app = app.run_for_frames(1).expect("ui plugins should run");
    let scene = app
        .world()
        .resource::<SceneRuntimeState>()
        .expect("scene state should exist");
    assert!(scene.overlay_visible);

    let overlay = app
        .world()
        .resource::<UiOverlayState>()
        .expect("ui overlay state should exist");
    assert!(!overlay.draw_list.commands.is_empty());
}

#[test]
fn ui_input_plugin_marks_overlay_consumed_when_editor_mode_is_toggled() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);
    app.add_plugin(UiInputPlugin);
    app.add_plugin(UiRenderPlugin);
    {
        let mut input = app
            .world_mut()
            .resource_mut::<InputState>()
            .expect("input state should exist");
        input.toggle_pause_menu = true;
        input.toggle_ui_editor_mode = true;
    }

    let app = app.run_for_frames(1).expect("ui plugins should run");
    let input = app
        .world()
        .resource::<InputState>()
        .expect("input state should exist");
    assert!(input.overlay_consumed);
}
