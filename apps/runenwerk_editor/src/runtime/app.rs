use anyhow::Result;
use engine::plugins::{
    DiagnosticsConfigResource, RenderFlow, RenderPlugin, ScenePlugin, SchedulerDiagnosticsPlugin,
    default_plugins,
};
use engine::prelude::*;
use winit::keyboard::KeyCode;

use crate::runtime::plugin::EditorAppPlugin;
use crate::runtime::resources::EditorViewportRenderState;
use crate::runtime::viewport::{
    VIEWPORT_RESOURCE_OVERLAY, VIEWPORT_RESOURCE_PICKING_IDS, VIEWPORT_RESOURCE_SCENE_COLOR,
};

pub const ACTION_EDITOR_UNDO: &str = "editor.undo";
pub const ACTION_EDITOR_REDO: &str = "editor.redo";
pub const ACTION_EDITOR_TOOL_SELECT: &str = "editor.tool.select";
pub const ACTION_EDITOR_TOOL_TRANSLATE: &str = "editor.tool.translate";

const WINDOW_TITLE: &str = "Runenwerk Editor";
const EDITOR_MAIN_FLOW_ID: &str = "runenwerk.editor.main";
const EDITOR_SURFACE_CLEAR_PASS_ID: &str = "runenwerk.editor.surface.clear";
const EDITOR_VIEWPORT_SCENE_PRODUCT_PASS_ID: &str = "runenwerk.editor.viewport.product.scene";
const EDITOR_VIEWPORT_PICKING_PRODUCT_PASS_ID: &str = "runenwerk.editor.viewport.product.picking";
const EDITOR_VIEWPORT_OVERLAY_PRODUCT_PASS_ID: &str = "runenwerk.editor.viewport.product.overlay";
const EDITOR_MAIN_UI_PASS_ID: &str = "runenwerk.editor.main.ui";
pub const EDITOR_VIEWPORT_SCENE_PRODUCT_SHADER_ID: &str = "editor_viewport_scene_product";

fn configure_app(app: &mut App) {
    app.set_title(WINDOW_TITLE);
    app.add_plugins(default_plugins());
    app.add_plugin(SchedulerDiagnosticsPlugin);
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    register_editor_render_flow(app);
    app.add_plugin(EditorAppPlugin);
    configure_editor_diagnostics(app);

    app.add_input_bindings([
        (ACTION_EDITOR_UNDO, KeyCode::KeyZ),
        (ACTION_EDITOR_REDO, KeyCode::KeyY),
        (ACTION_EDITOR_TOOL_SELECT, KeyCode::Digit1),
        (ACTION_EDITOR_TOOL_TRANSLATE, KeyCode::Digit2),
    ]);
}

fn configure_editor_diagnostics(app: &mut App) {
    if let Ok(config) = app.world_mut().resource_mut::<DiagnosticsConfigResource>() {
        config.adapters.console_enabled = env_flag_or("RUNENWERK_EDITOR_DIAGNOSTICS_CONSOLE", true);
        config.adapters.stdout_enabled = env_flag_or("RUNENWERK_EDITOR_DIAGNOSTICS_STDOUT", true);
        config.adapters.file_json_enabled = env_flag_or("RUNENWERK_EDITOR_DIAGNOSTICS_JSON", false);
    }
}

fn env_flag_or(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

fn register_editor_render_flow(app: &mut App) {
    let flow = RenderFlow::new(EDITOR_MAIN_FLOW_ID)
        .with_state::<EditorViewportRenderState>()
        .with_color_target(VIEWPORT_RESOURCE_SCENE_COLOR)
        .with_color_target(VIEWPORT_RESOURCE_PICKING_IDS)
        .with_color_target(VIEWPORT_RESOURCE_OVERLAY)
        .with_surface_color()
        .fullscreen_pass(EDITOR_SURFACE_CLEAR_PASS_ID)
        .write_surface_color()
        .finish()
        .fullscreen_pass(EDITOR_VIEWPORT_SCENE_PRODUCT_PASS_ID)
        .shader_asset(EDITOR_VIEWPORT_SCENE_PRODUCT_SHADER_ID)
        .uniform_from_state_with_surface(EditorViewportRenderState::compose_scene_product_uniform)
        .write_color_target(VIEWPORT_RESOURCE_SCENE_COLOR)
        .finish()
        .fullscreen_pass(EDITOR_VIEWPORT_PICKING_PRODUCT_PASS_ID)
        .depends_on(EDITOR_VIEWPORT_SCENE_PRODUCT_PASS_ID)
        .write_color_target(VIEWPORT_RESOURCE_PICKING_IDS)
        .finish()
        .fullscreen_pass(EDITOR_VIEWPORT_OVERLAY_PRODUCT_PASS_ID)
        .depends_on(EDITOR_VIEWPORT_PICKING_PRODUCT_PASS_ID)
        .write_color_target(VIEWPORT_RESOURCE_OVERLAY)
        .finish()
        .builtin_ui_composite_pass(EDITOR_MAIN_UI_PASS_ID)
        .depends_on(EDITOR_SURFACE_CLEAR_PASS_ID)
        .depends_on(EDITOR_VIEWPORT_OVERLAY_PRODUCT_PASS_ID)
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
