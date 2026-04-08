use engine::WindowState;
use engine::plugins::render::{
    UiFrameRoute, UiFrameSubmission, UiFrameSubmissionOrder, UiFrameSubmissionRegistryResource,
};
use engine::runtime::{Res, ResMut};
use ui_math::UiRect;
use ui_render_data::{
    RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId, UiPaint, UiPrimitive, UiSortKey,
    UiSurface, UiSurfaceId,
};

use crate::runtime::resources::EditorHostResource;

const EDITOR_SHELL_UI_PRODUCER_ID: &str = "editor.shell";
const DEBUG_HARDCODED_UI_FRAME_ENV: &str = "RUNENWERK_EDITOR_DEBUG_UI_FRAME";

pub fn submit_editor_frame_system(
    window: Res<WindowState>,
    mut host: ResMut<EditorHostResource>,
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
        app.build_shell_frame(shell_state, bounds, theme)
    };

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
