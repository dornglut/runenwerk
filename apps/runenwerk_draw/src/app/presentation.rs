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
