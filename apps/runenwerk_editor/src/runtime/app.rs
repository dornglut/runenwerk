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
    EDITOR_MAIN_FLOW_ID, EDITOR_VIEWPORT_SCENE_PRODUCT_UNIFORM_ID,
    VIEWPORT_TARGET_ALIAS_MATERIAL_PREVIEW, VIEWPORT_TARGET_ALIAS_OVERLAY,
    VIEWPORT_TARGET_ALIAS_PICKING_IDS, VIEWPORT_TARGET_ALIAS_SCENE_COLOR,
};

pub const ACTION_EDITOR_UNDO: &str = "editor.undo";
pub const ACTION_EDITOR_REDO: &str = "editor.redo";
pub const ACTION_EDITOR_TOOL_SELECT: &str = "editor.tool.select";
pub const ACTION_EDITOR_TOOL_TRANSLATE: &str = "editor.tool.translate";
pub const ACTION_EDITOR_TOOL_ROTATE: &str = "editor.tool.rotate";
pub const ACTION_EDITOR_TOOL_SCALE: &str = "editor.tool.scale";
pub const ACTION_EDITOR_VIEWPORT_FOCUS: &str = "editor.viewport.focus_selected";
pub const ACTION_EDITOR_VIEWPORT_TOOL_RADIAL: &str = "editor.viewport.tool_radial";

const WINDOW_TITLE: &str = "Runenwerk Editor";
const EDITOR_SURFACE_CLEAR_PASS_ID: &str = "runenwerk.editor.surface.clear";
const EDITOR_VIEWPORT_SCENE_PRODUCT_PASS_ID: &str = "runenwerk.editor.viewport.product.scene";
const EDITOR_VIEWPORT_PICKING_PRODUCT_PASS_ID: &str = "runenwerk.editor.viewport.product.picking";
const EDITOR_VIEWPORT_OVERLAY_PRODUCT_PASS_ID: &str = "runenwerk.editor.viewport.product.overlay";
const EDITOR_MAIN_UI_PASS_ID: &str = "runenwerk.editor.main.ui";
pub const EDITOR_MATERIAL_PREVIEW_FLOW_ID: &str = "runenwerk.editor.material.preview.flow";
const EDITOR_MATERIAL_PREVIEW_PASS_ID: &str = "runenwerk.editor.material.preview.pass";
pub const EDITOR_VIEWPORT_SCENE_PRODUCT_SHADER_ID: &str = "editor_viewport_scene_product";
pub const EDITOR_VIEWPORT_PICKING_PRODUCT_SHADER_ID: &str = "editor_viewport_picking_product";
pub const EDITOR_VIEWPORT_OVERLAY_PRODUCT_SHADER_ID: &str = "editor_viewport_overlay_product";
pub const EDITOR_MATERIAL_PREVIEW_SHADER_ID: &str = "editor_material_preview_generated";
const EDITOR_VIEWPORT_BACKGROUND_CLEAR: [f32; 4] = [0.09, 0.10, 0.12, 1.0];
const EDITOR_VIEWPORT_TRANSPARENT_CLEAR: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

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
        (ACTION_EDITOR_TOOL_ROTATE, KeyCode::Digit3),
        (ACTION_EDITOR_TOOL_SCALE, KeyCode::Digit4),
        (ACTION_EDITOR_VIEWPORT_FOCUS, KeyCode::KeyF),
        (ACTION_EDITOR_VIEWPORT_TOOL_RADIAL, KeyCode::Tab),
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
    let (flow, scene_product_uniform) = RenderFlow::new(EDITOR_MAIN_FLOW_ID)
        .with_state::<EditorViewportRenderState>()
        .uniform_buffer::<crate::runtime::resources::EditorViewportSceneProductUniform>(
            EDITOR_VIEWPORT_SCENE_PRODUCT_UNIFORM_ID,
        );
    let flow = flow
        .with_color_target_alias(VIEWPORT_TARGET_ALIAS_SCENE_COLOR)
        .with_color_target_alias(VIEWPORT_TARGET_ALIAS_PICKING_IDS)
        .with_color_target_alias(VIEWPORT_TARGET_ALIAS_OVERLAY)
        .with_surface_color()
        .fullscreen_pass(EDITOR_SURFACE_CLEAR_PASS_ID)
        .main_surface_only()
        .clear_color(EDITOR_VIEWPORT_BACKGROUND_CLEAR)
        .write_surface_color()
        .finish()
        .fullscreen_pass(EDITOR_VIEWPORT_SCENE_PRODUCT_PASS_ID)
        .offscreen_products_only()
        .material_scene_shader_asset(EDITOR_VIEWPORT_SCENE_PRODUCT_SHADER_ID)
        .clear_color(EDITOR_VIEWPORT_BACKGROUND_CLEAR)
        .uniform_from_state_with_surface_to(
            scene_product_uniform.clone(),
            EditorViewportRenderState::compose_scene_product_uniform,
        )
        .write_target_alias(VIEWPORT_TARGET_ALIAS_SCENE_COLOR)
        .finish()
        .fullscreen_pass(EDITOR_VIEWPORT_PICKING_PRODUCT_PASS_ID)
        .offscreen_products_only()
        .depends_on(EDITOR_VIEWPORT_SCENE_PRODUCT_PASS_ID)
        .shader_asset(EDITOR_VIEWPORT_PICKING_PRODUCT_SHADER_ID)
        .uniform_from_state_with_surface_to(
            scene_product_uniform.clone(),
            EditorViewportRenderState::compose_scene_product_uniform,
        )
        .write_target_alias(VIEWPORT_TARGET_ALIAS_PICKING_IDS)
        .finish()
        .fullscreen_pass(EDITOR_VIEWPORT_OVERLAY_PRODUCT_PASS_ID)
        .offscreen_products_only()
        .depends_on(EDITOR_VIEWPORT_PICKING_PRODUCT_PASS_ID)
        .shader_asset(EDITOR_VIEWPORT_OVERLAY_PRODUCT_SHADER_ID)
        .clear_color(EDITOR_VIEWPORT_TRANSPARENT_CLEAR)
        .uniform_from_state_with_surface_to(
            scene_product_uniform,
            EditorViewportRenderState::compose_scene_product_uniform,
        )
        .write_target_alias(VIEWPORT_TARGET_ALIAS_OVERLAY)
        .finish()
        .builtin_ui_composite_pass(EDITOR_MAIN_UI_PASS_ID)
        .main_surface_only()
        .depends_on(EDITOR_SURFACE_CLEAR_PASS_ID)
        .depends_on(EDITOR_VIEWPORT_OVERLAY_PRODUCT_PASS_ID)
        .finish()
        .validate()
        .expect("editor render flow should validate");
    app.add_render_flow(flow);

    let material_preview_flow = RenderFlow::new(EDITOR_MATERIAL_PREVIEW_FLOW_ID)
        .with_color_target_alias(VIEWPORT_TARGET_ALIAS_MATERIAL_PREVIEW)
        .fullscreen_pass(EDITOR_MATERIAL_PREVIEW_PASS_ID)
        .offscreen_products_only()
        .shader_asset(EDITOR_MATERIAL_PREVIEW_SHADER_ID)
        .for_feature(engine::plugins::render::MATERIAL_RENDER_FEATURE_ID)
        .write_target_alias(VIEWPORT_TARGET_ALIAS_MATERIAL_PREVIEW)
        .finish()
        .validate()
        .expect("editor material preview render flow should validate");
    app.add_render_flow(material_preview_flow);
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
