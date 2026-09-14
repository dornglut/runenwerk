use engine::plugins::{InputFinalizePlugin, ScenePlugin, TimePlugin};
use engine::prelude::*;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[test]
fn ui_plugins_populate_overlay_state_when_overlay_is_visible() {
    let mut app = App::headless();
    app.add_plugin(TimePlugin);
    app.add_plugin(InputFinalizePlugin);
    app.add_plugin(ScenePlugin);
    app.world_mut()
        .resource_mut::<InputState>()
        .expect("input state should exist")
        .handle_keyboard_input(KeyCode::Escape, ElementState::Pressed, None);

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
    assert!(overlay.scale > 0.0);
    assert!(overlay.screen_size.0 > 0.0 && overlay.screen_size.1 > 0.0);
}
