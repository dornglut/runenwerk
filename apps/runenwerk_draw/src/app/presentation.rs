//! Canvas-first UI frame projection for the drawing app shell.

use ui_math::{UiRect, UiSize};
use ui_render_data::{
    RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId, UiPaint, UiPrimitive, UiSortKey,
    UiSurface, UiSurfaceId,
};

use crate::app::DrawingWorkspaceProjection;

pub const DRAWING_UI_SURFACE_ID: UiSurfaceId = UiSurfaceId(4_001);
pub const DRAWING_CANVAS_LAYER_ID: UiLayerId = UiLayerId(4_010);

const DRAW_KEY_SOLID: UiDrawKey = UiDrawKey::new(1, None);

pub fn build_workspace_frame(workspace: &DrawingWorkspaceProjection) -> UiFrame {
    let mut layer = UiLayer::new(DRAWING_CANVAS_LAYER_ID);
    push_rect(
        &mut layer,
        UiRect::new(
            0.0,
            0.0,
            workspace.window_size.width,
            workspace.window_size.height,
        ),
        UiPaint::rgba(0.055, 0.058, 0.062, 1.0),
        0,
    );
    push_rect(
        &mut layer,
        workspace.toolbar_bounds,
        UiPaint::rgba(0.095, 0.1, 0.108, 1.0),
        1,
    );
    if workspace.layer_panel_bounds.width > 0.0 {
        push_rect(
            &mut layer,
            workspace.layer_panel_bounds,
            UiPaint::rgba(0.09, 0.092, 0.098, 1.0),
            2,
        );
    }
    push_rect(
        &mut layer,
        workspace.canvas_view.screen_bounds,
        UiPaint::rgba(0.93, 0.925, 0.9, 1.0),
        3,
    );

    UiFrame::with_surfaces(vec![UiSurface::with_layers(
        DRAWING_UI_SURFACE_ID,
        workspace.window_size,
        vec![layer],
    )])
}

fn push_rect(layer: &mut UiLayer, rect: UiRect, paint: UiPaint, primitive_order: u32) {
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        rect,
        0.0,
        paint,
        DRAW_KEY_SOLID,
        UiSortKey::new(0, 0, primitive_order),
    )));
}

pub fn default_surface_size() -> UiSize {
    UiSize::new(1280.0, 720.0)
}
