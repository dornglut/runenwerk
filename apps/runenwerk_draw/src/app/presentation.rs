//! Canvas-first UI frame projection for the drawing app shell and ink product surfaces.

use drawing::{CanvasTileId, DrawingInkTileProduct};
use ui_math::{UiRect, UiSize};
use ui_render_data::{
    ProductSurfaceAlphaMode, ProductSurfacePrimitive, ProductSurfaceTextureBindingSource,
    RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId, UiPaint, UiPrimitive, UiSortKey,
    UiSurface, UiSurfaceId,
};

use crate::app::DrawingWorkspaceProjection;

pub const DRAWING_UI_SURFACE_ID: UiSurfaceId = UiSurfaceId(4_001);
pub const DRAWING_CANVAS_LAYER_ID: UiLayerId = UiLayerId(4_010);

const DRAW_KEY_SOLID: UiDrawKey = UiDrawKey::new(1, None);

pub const DRAWING_INK_TEXTURE_NAMESPACE: &str = "runenwerk.draw.ink";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingInkSurfaceKind {
    Committed,
    Preview,
}

impl DrawingInkSurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Preview => "preview",
        }
    }
}

pub fn build_workspace_frame(workspace: &DrawingWorkspaceProjection) -> UiFrame {
    build_workspace_frame_with_ink(workspace, &[], &[])
}

pub fn build_workspace_frame_with_ink(
    workspace: &DrawingWorkspaceProjection,
    ink_tiles: &[DrawingInkTileProduct],
    preview_tiles: &[DrawingInkTileProduct],
) -> UiFrame {
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
        push_tablet_panel(&mut layer, workspace, 30_000);
    }
    push_rect(
        &mut layer,
        workspace.canvas_view.screen_bounds,
        UiPaint::rgba(0.93, 0.925, 0.9, 1.0),
        3,
    );
    push_ink_surfaces(
        &mut layer,
        workspace,
        ink_tiles,
        DrawingInkSurfaceKind::Committed,
        10,
    );
    push_ink_surfaces(
        &mut layer,
        workspace,
        preview_tiles,
        DrawingInkSurfaceKind::Preview,
        20_000,
    );

    UiFrame::with_surfaces(vec![UiSurface::with_layers(
        DRAWING_UI_SURFACE_ID,
        workspace.window_size,
        vec![layer],
    )])
}

fn push_tablet_panel(
    layer: &mut UiLayer,
    workspace: &DrawingWorkspaceProjection,
    primitive_order_start: u32,
) {
    let panel = workspace.layer_panel_bounds;
    if panel.width <= 0.0 || panel.height <= 0.0 {
        return;
    }
    let state = &workspace.tablet_panel;
    let x = panel.x + 16.0;
    let width = (panel.width - 32.0).max(0.0);
    let mut y = panel.y + 16.0;
    let row_height = 8.0;
    let gap = 12.0;
    let status = if state.warning_count > 0 || state.dropped_samples > 0 {
        UiPaint::rgba(0.86, 0.58, 0.18, 1.0)
    } else if state.active_backend != "winit fallback" {
        UiPaint::rgba(0.16, 0.68, 0.45, 1.0)
    } else {
        UiPaint::rgba(0.31, 0.34, 0.38, 1.0)
    };
    push_rect(
        layer,
        UiRect::new(x, y, width, 12.0),
        UiPaint::rgba(0.13, 0.135, 0.145, 1.0),
        primitive_order_start,
    );
    push_rect(
        layer,
        UiRect::new(x, y, width * 0.72, 12.0),
        status,
        primitive_order_start + 1,
    );
    y += 28.0;

    let sample_rate_fill = (state.sample_rate_hz / 240.0).clamp(0.0, 1.0);
    push_meter(
        layer,
        UiRect::new(x, y, width, row_height),
        sample_rate_fill,
        UiPaint::rgba(0.35, 0.62, 0.93, 1.0),
        primitive_order_start + 2,
    );
    y += row_height + gap;

    let gap_fill = (1.0 - state.max_segment_gap_px / 120.0).clamp(0.0, 1.0);
    push_meter(
        layer,
        UiRect::new(x, y, width, row_height),
        gap_fill,
        UiPaint::rgba(0.47, 0.75, 0.42, 1.0),
        primitive_order_start + 4,
    );
    y += row_height + gap;

    push_toggle(
        layer,
        UiRect::new(x, y, 34.0, 14.0),
        state.pressure_available,
        primitive_order_start + 6,
    );
    push_toggle(
        layer,
        UiRect::new(x + 46.0, y, 34.0, 14.0),
        state.tilt_available,
        primitive_order_start + 8,
    );
    y += 28.0;

    let pressure_fill = ((state.pressure_scale + state.pressure_bias) / 2.0).clamp(0.0, 1.0);
    push_meter(
        layer,
        UiRect::new(x, y, width, row_height),
        pressure_fill,
        UiPaint::rgba(0.78, 0.5, 0.84, 1.0),
        primitive_order_start + 10,
    );
    y += row_height + gap;

    let offset_magnitude = (state.cursor_offset.x * state.cursor_offset.x
        + state.cursor_offset.y * state.cursor_offset.y)
        .sqrt();
    let offset_fill = (offset_magnitude / 64.0).clamp(0.0, 1.0);
    push_meter(
        layer,
        UiRect::new(x, y, width, row_height),
        offset_fill,
        UiPaint::rgba(0.74, 0.64, 0.32, 1.0),
        primitive_order_start + 12,
    );
}

fn push_meter(layer: &mut UiLayer, rect: UiRect, fill: f32, paint: UiPaint, primitive_order: u32) {
    push_rect(
        layer,
        rect,
        UiPaint::rgba(0.13, 0.135, 0.145, 1.0),
        primitive_order,
    );
    push_rect(
        layer,
        UiRect::new(
            rect.x,
            rect.y,
            rect.width * fill.clamp(0.0, 1.0),
            rect.height,
        ),
        paint,
        primitive_order + 1,
    );
}

fn push_toggle(layer: &mut UiLayer, rect: UiRect, enabled: bool, primitive_order: u32) {
    push_rect(
        layer,
        rect,
        UiPaint::rgba(0.13, 0.135, 0.145, 1.0),
        primitive_order,
    );
    let fill = if enabled {
        UiPaint::rgba(0.16, 0.68, 0.45, 1.0)
    } else {
        UiPaint::rgba(0.31, 0.34, 0.38, 1.0)
    };
    push_rect(
        layer,
        UiRect::new(
            rect.x + 2.0,
            rect.y + 2.0,
            rect.width - 4.0,
            rect.height - 4.0,
        ),
        fill,
        primitive_order + 1,
    );
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

fn push_ink_surfaces(
    layer: &mut UiLayer,
    workspace: &DrawingWorkspaceProjection,
    ink_tiles: &[DrawingInkTileProduct],
    surface_kind: DrawingInkSurfaceKind,
    primitive_order_start: u32,
) {
    let mut primitive_order = primitive_order_start;
    for product in ink_tiles {
        let Some(rect) = workspace
            .canvas_view
            .canvas_rect_to_screen(product.metadata.invalidation_bounds)
            .and_then(|rect| rect.intersect(workspace.canvas_view.screen_bounds))
        else {
            continue;
        };
        if rect.width <= 0.0 || rect.height <= 0.0 {
            continue;
        }
        layer.push(UiPrimitive::ProductSurface(ProductSurfacePrimitive::new(
            ProductSurfaceTextureBindingSource::dynamic_texture(
                DRAWING_INK_TEXTURE_NAMESPACE,
                drawing_ink_texture_target_id(surface_kind, product.metadata.tile_id),
            ),
            rect,
            UiRect::new(0.0, 0.0, 1.0, 1.0),
            UiPaint::rgba(1.0, 1.0, 1.0, 1.0),
            ProductSurfaceAlphaMode::Straight,
            UiSortKey::new(0, 0, primitive_order),
        )));
        primitive_order = primitive_order.saturating_add(1);
    }
}

pub fn drawing_ink_texture_target_id(
    surface_kind: DrawingInkSurfaceKind,
    tile_id: CanvasTileId,
) -> String {
    format!(
        "{}.L{}.{}.{}",
        surface_kind.label(),
        tile_id.level.raw(),
        tile_id.x,
        tile_id.y
    )
}

pub fn default_surface_size() -> UiSize {
    UiSize::new(1280.0, 720.0)
}
