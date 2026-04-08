use anyhow::Result;
use engine::plugins::{RenderFlow, RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::*;
use winit::keyboard::KeyCode;

use crate::runtime::plugin::EditorAppPlugin;

pub const ACTION_EDITOR_UNDO: &str = "editor.undo";
pub const ACTION_EDITOR_REDO: &str = "editor.redo";
pub const ACTION_EDITOR_TOOL_SELECT: &str = "editor.tool.select";
pub const ACTION_EDITOR_TOOL_TRANSLATE: &str = "editor.tool.translate";

const WINDOW_TITLE: &str = "Runenwerk Editor";
const EDITOR_MAIN_FLOW_ID: &str = "runenwerk.editor.main";
const EDITOR_MAIN_UI_PASS_ID: &str = "runenwerk.editor.main.ui";

fn configure_app(app: &mut App) {
    app.set_title(WINDOW_TITLE);
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    register_editor_render_flow(app);
    app.add_plugin(EditorAppPlugin);

    app.add_input_bindings([
        (ACTION_EDITOR_UNDO, KeyCode::KeyZ),
        (ACTION_EDITOR_REDO, KeyCode::KeyY),
        (ACTION_EDITOR_TOOL_SELECT, KeyCode::Digit1),
        (ACTION_EDITOR_TOOL_TRANSLATE, KeyCode::Digit2),
    ]);
}

fn register_editor_render_flow(app: &mut App) {
    let flow = RenderFlow::new(EDITOR_MAIN_FLOW_ID)
        .with_surface_color()
        .with_builtin_ui()
        .builtin_ui_composite_pass(EDITOR_MAIN_UI_PASS_ID)
        .finish()
        .validate()
        .expect("editor render flow should validate");
    app.add_render_flow(flow);
}

pub fn build_headless_app() -> App {
    let mut app = App::headless();
    configure_app(&mut app);
    app
}

pub fn run() -> Result<()> {
    let mut app = App::new();
    configure_app(&mut app);
    app.run()
}
