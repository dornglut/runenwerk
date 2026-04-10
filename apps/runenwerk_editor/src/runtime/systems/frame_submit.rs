use engine::WindowState;
use engine::plugins::render::{
    UiFontAtlasResource, UiFrameRoute, UiFrameSubmission, UiFrameSubmissionOrder,
    UiFrameSubmissionRegistryResource,
};
use engine::runtime::{Res, ResMut};
use scene::LocalTransform;
use ui_math::UiRect;
use ui_render_data::{
    RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId, UiPaint, UiPrimitive, UiSortKey,
    UiSurface, UiSurfaceId,
};

use crate::editor_runtime::EditorPrimitive;
use crate::runtime::resources::{EditorHostResource, EditorViewportRenderState};

const EDITOR_SHELL_UI_PRODUCER_ID: &str = "editor.shell";
const DEBUG_HARDCODED_UI_FRAME_ENV: &str = "RUNENWERK_EDITOR_DEBUG_UI_FRAME";

pub fn submit_editor_frame_system(
    window: Res<WindowState>,
    mut host: ResMut<EditorHostResource>,
    mut viewport_render: ResMut<EditorViewportRenderState>,
    atlas: Res<UiFontAtlasResource>,
    mut submissions: ResMut<UiFrameSubmissionRegistryResource>,
) {
    let bounds = window_bounds(&window);
    let EditorHostResource {
        app,
        shell_state,
        theme,
    } = &mut *host;
    let frame = if debug_hardcoded_ui_frame_enabled() {
        build_debug_frame(bounds)
    } else {
        app.build_shell_frame(shell_state, bounds, theme, &*atlas)
    };
    let viewport_bounds = viewport_bounds(
        shell_state.last_tree(),
        shell_state.last_bounds(),
        shell_state.runtime(),
    )
    .unwrap_or(bounds);
    populate_viewport_render_state(app, &mut viewport_render, viewport_bounds);

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

fn viewport_bounds(
    tree: Option<&editor_shell::UiTree>,
    bounds: Option<UiRect>,
    runtime: &editor_shell::UiRuntime,
) -> Option<UiRect> {
    let tree = tree?;
    let bounds = bounds?;
    let layouts = runtime.compute_layout(tree, bounds);
    layouts
        .get(&editor_shell::VIEWPORT_PANEL_WIDGET_ID)
        .map(|layout| layout.bounds)
}

fn populate_viewport_render_state(
    app: &crate::editor_app::RunenwerkEditorApp,
    render_state: &mut EditorViewportRenderState,
    viewport_bounds: UiRect,
) {
    render_state.set_viewport_bounds((
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
