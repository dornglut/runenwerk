use engine::WindowState;
use engine::plugins::render::{
    EditorPickingResultResource, EditorPickingTarget, ShaderRegistryResource, UiFontAtlasResource,
    UiFrameRoute, UiFrameSubmission, UiFrameSubmissionOrder, UiFrameSubmissionRegistryResource,
};
use engine::runtime::{Res, ResMut};
use scene::LocalTransform;
use ui_math::UiRect;
use ui_render_data::{
    RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId, UiPaint, UiPrimitive, UiSortKey,
    UiSurface, UiSurfaceId,
};

use crate::editor_runtime::EditorPrimitive;
use crate::runtime::app::EDITOR_VIEWPORT_SDF_SHADER_ID;
use crate::runtime::resources::{
    EditorHostResource, EditorViewportDebugStage, EditorViewportRenderState, effective_shell_scale,
    scaled_shell_theme,
};

const EDITOR_SHELL_UI_PRODUCER_ID: &str = "editor.shell";
const DEBUG_HARDCODED_UI_FRAME_ENV: &str = "RUNENWERK_EDITOR_DEBUG_UI_FRAME";
const VIEWPORT_DEBUG_STAGE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_DEBUG_STAGE";
const VIEWPORT_ROOT_OPAQUE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_ROOT_OPAQUE";
const VIEWPORT_BRANCH_TRACE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_BRANCH_TRACE";

pub fn submit_editor_frame_system(
    window: Res<WindowState>,
    mut host: ResMut<EditorHostResource>,
    mut viewport_render: ResMut<EditorViewportRenderState>,
    atlas: Res<UiFontAtlasResource>,
    picking: Res<EditorPickingResultResource>,
    shader_registry: Res<ShaderRegistryResource>,
    mut submissions: ResMut<UiFrameSubmissionRegistryResource>,
) {
    let bounds = window_bounds(&window);
    let shell_scale = effective_shell_scale(window.scale_factor);
    let EditorHostResource {
        app,
        shell_state,
        theme,
    } = &mut *host;
    let shell_theme = scaled_shell_theme(theme, window.scale_factor);
    let (expression_source_version, frame) = if debug_hardcoded_ui_frame_enabled() {
        let expression = editor_shell::ShellUiExpressionFrame::new(
            app.runtime().current_scene_reality_version(),
            build_debug_frame(bounds),
        );
        (
            expression.metadata.source_version,
            expression.into_ui_frame(),
        )
    } else {
        let expression =
            app.build_shell_expression_frame(shell_state, bounds, &shell_theme, &*atlas);
        (
            expression.metadata.source_version,
            expression.into_ui_frame(),
        )
    };
    let viewport_bounds = viewport_bounds(
        shell_state.last_tree(),
        shell_state.last_bounds(),
        shell_state.runtime(),
    )
    .unwrap_or(bounds);
    let viewport_bounds_changed =
        populate_viewport_render_state(app, &mut viewport_render, viewport_bounds);
    let viewport_valid = viewport_is_valid(viewport_bounds);
    let shader_loaded = shader_registry.revision_for(EDITOR_VIEWPORT_SDF_SHADER_ID) > 0;
    let debug_stage = viewport_debug_stage();
    let root_background_opaque = root_background_opaque_enabled();
    viewport_render.update_visibility_diagnostics(viewport_valid, shader_loaded);
    let debug_stage_changed = viewport_render.set_debug_stage(debug_stage);
    let root_probe_changed = viewport_render.set_root_background_opaque(root_background_opaque);
    let shell_scale_changed = viewport_render.set_effective_shell_scale(shell_scale);
    let contradiction_active =
        picking_hits_entity_or_component(&picking) && viewport_render.scene_should_be_invisible();
    let should_report_contradiction =
        viewport_render.should_report_visibility_contradiction(contradiction_active);
    let branch_trace_enabled = viewport_branch_trace_enabled();
    let branch_trace_snapshot = if branch_trace_enabled || should_report_contradiction {
        Some(
            viewport_render
                .branch_trace_snapshot((window.size_px.0.max(1), window.size_px.1.max(1))),
        )
    } else {
        None
    };

    if app.debug_logs_enabled() {
        if shell_scale_changed && viewport_render.should_report_scale_change() {
            app.append_console_line(format!(
                "[ui] shell scale={:.3} window_scale={:.3} expression_version={}",
                shell_scale, window.scale_factor, expression_source_version.0
            ));
        }

        if viewport_bounds_changed && viewport_render.should_report_bounds_change() {
            app.append_console_line(format!(
                "[viewport] bounds=({:.1},{:.1},{:.1},{:.1})",
                viewport_bounds.x, viewport_bounds.y, viewport_bounds.width, viewport_bounds.height
            ));
            if viewport_bounds.width <= f32::EPSILON || viewport_bounds.height <= f32::EPSILON {
                app.append_console_line(
                    "[viewport] warning: viewport canvas bounds are zero-sized".to_string(),
                );
            }
        }

        if root_probe_changed
            || debug_stage_changed
            || viewport_render.should_report_debug_state_change()
        {
            app.append_console_line(format!(
                "[viewport] root-occlusion={} debug-stage={} viewport_valid={} shader_loaded={} primitive_visible={}",
                if viewport_render.root_background_opaque { "opaque" } else { "transparent" },
                viewport_render.debug_stage.label(),
                viewport_render.viewport_valid,
                viewport_render.shader_loaded,
                viewport_render.has_primitive,
            ));
        }
    }

    if branch_trace_enabled
        && let Some(snapshot) = branch_trace_snapshot
        && viewport_render.should_report_branch_trace_change(snapshot)
    {
        app.append_console_line(format!("[viewport.branch] {}", snapshot.summary_line()));
    }

    if should_report_contradiction {
        let mut line = format!(
            "[viewport] contradiction: analytic picking hit while render-state indicates invisible ({})",
            contradiction_reasons(&viewport_render)
        );
        if let Some(snapshot) = branch_trace_snapshot {
            line.push_str(" | ");
            line.push_str(&snapshot.summary_line());
        }
        app.append_console_line(line);
    }

    submissions.replace(
        UiFrameSubmission::new(EDITOR_SHELL_UI_PRODUCER_ID)
            .with_route(UiFrameRoute::Screen)
            .with_order(UiFrameSubmissionOrder::new(10, 0))
            .with_frame(frame),
    );
}

fn debug_hardcoded_ui_frame_enabled() -> bool {
    std::env::var(DEBUG_HARDCODED_UI_FRAME_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn viewport_debug_stage() -> EditorViewportDebugStage {
    std::env::var(VIEWPORT_DEBUG_STAGE_ENV)
        .map(|value| EditorViewportDebugStage::from_env_value(&value))
        .unwrap_or(EditorViewportDebugStage::Scene)
}

fn root_background_opaque_enabled() -> bool {
    std::env::var(VIEWPORT_ROOT_OPAQUE_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn viewport_branch_trace_enabled() -> bool {
    std::env::var(VIEWPORT_BRANCH_TRACE_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn build_debug_frame(bounds: UiRect) -> UiFrame {
    let mut layer = UiLayer::new(UiLayerId(0));
    let debug_rect = UiRect::new(
        24.0,
        24.0,
        (bounds.width - 48.0).clamp(80.0, 420.0),
        (bounds.height - 48.0).clamp(80.0, 160.0),
    );
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        debug_rect,
        8.0,
        UiPaint::rgba(0.17, 0.58, 0.36, 0.96),
        UiDrawKey::new(0, None),
        UiSortKey::new(0, 0, 0),
    )));

    UiFrame::with_surfaces(vec![UiSurface::with_layers(
        UiSurfaceId(0),
        bounds.size(),
        vec![layer],
    )])
}

fn window_bounds(window: &WindowState) -> UiRect {
    let width = window.size_px.0.max(1) as f32;
    let height = window.size_px.1.max(1) as f32;
    UiRect::new(0.0, 0.0, width, height)
}

fn viewport_is_valid(bounds: UiRect) -> bool {
    bounds.width > f32::EPSILON && bounds.height > f32::EPSILON
}

fn picking_hits_entity_or_component(picking: &EditorPickingResultResource) -> bool {
    matches!(
        picking.hit.target,
        EditorPickingTarget::Entity(_) | EditorPickingTarget::ComponentHandle { .. }
    )
}

fn contradiction_reasons(state: &EditorViewportRenderState) -> String {
    let mut reasons = Vec::new();
    if state.debug_stage != EditorViewportDebugStage::Scene {
        reasons.push("debug-stage");
    }
    if !state.viewport_valid {
        reasons.push("invalid viewport");
    }
    if !state.shader_loaded {
        reasons.push("fallback shader");
    }
    if !state.has_primitive {
        reasons.push("missing primitive");
    }
    if reasons.is_empty() {
        reasons.push("unknown");
    }
    reasons.join(", ")
}

fn viewport_bounds(
    tree: Option<&editor_shell::UiTree>,
    bounds: Option<UiRect>,
    runtime: &editor_shell::UiRuntime,
) -> Option<UiRect> {
    let tree = tree?;
    let bounds = bounds?;
    let layouts = runtime.compute_layout(tree, bounds);
    layouts
        .get(&editor_shell::VIEWPORT_CANVAS_WIDGET_ID)
        .map(|layout| layout.bounds)
}

fn populate_viewport_render_state(
    app: &crate::editor_app::RunenwerkEditorApp,
    render_state: &mut EditorViewportRenderState,
    viewport_bounds: UiRect,
) -> bool {
    let bounds_changed = render_state.set_viewport_bounds((
        viewport_bounds.x,
        viewport_bounds.y,
        viewport_bounds.width,
        viewport_bounds.height,
    ));

    let runtime = app.runtime();
    if let Some((transform, primitive)) = selected_or_first_editor_primitive(runtime) {
        render_state.set_primitive(transform.translation, primitive);
    } else {
        render_state.clear_primitive();
    }

    bounds_changed
}

fn selected_or_first_editor_primitive(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
) -> Option<(LocalTransform, EditorPrimitive)> {
    if let Some(selected) = runtime.selected_entity()
        && let Some(result) = entity_primitive(runtime, selected)
    {
        return Some(result);
    }

    runtime
        .document()
        .entity_ids()
        .find_map(|entity| entity_primitive(runtime, entity))
}

fn entity_primitive(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
    entity: editor_core::EntityId,
) -> Option<(LocalTransform, EditorPrimitive)> {
    let ecs_entity = runtime.ids().resolve_entity(entity)?;
    let transform = runtime.world().get::<LocalTransform>(ecs_entity).copied()?;
    let primitive = runtime
        .world()
        .get::<EditorPrimitive>(ecs_entity)
        .copied()?;
    Some((transform, primitive))
}
