//! Canvas-first UI frame projection for the drawing app shell and ink product surfaces.

use drawing::{CanvasTileId, DrawingInkTileProduct, ProductQualityClass};
use ui_math::{UiPoint, UiRect, UiSize};
use ui_render_data::{
    ProductSurfaceAlphaMode, ProductSurfacePrimitive, ProductSurfaceTextureBindingSource,
    RectPrimitive, StrokePrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId, UiPaint, UiPrimitive,
    UiSortKey, UiSurface, UiSurfaceId,
};

use crate::app::{DrawingPreviewStroke, DrawingWorkspaceProjection};

pub const DRAWING_UI_SURFACE_ID: UiSurfaceId = UiSurfaceId(4_001);
pub const DRAWING_CANVAS_LAYER_ID: UiLayerId = UiLayerId(4_010);

const DRAW_KEY_SOLID: UiDrawKey = UiDrawKey::new(1, None);

pub const DRAWING_INK_TEXTURE_NAMESPACE: &str = "runenwerk.draw.ink";
const IMMEDIATE_STROKE_PRIMITIVE_ORDER: u32 = 25_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DrawingInkSurfaceKind {
    Committed,
    Preview,
    GpuCommitted,
    GpuPreview,
}

impl DrawingInkSurfaceKind {
    pub fn gpu_variant(self) -> Self {
        match self {
            Self::Committed | Self::GpuCommitted => Self::GpuCommitted,
            Self::Preview | Self::GpuPreview => Self::GpuPreview,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Preview => "preview",
            Self::GpuCommitted => "gpu.committed",
            Self::GpuPreview => "gpu.preview",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DrawingInkSurfaceProjection<'a> {
    pub product: &'a DrawingInkTileProduct,
    pub surface_kind: DrawingInkSurfaceKind,
}

#[derive(Debug, Clone, Copy)]
pub struct DrawingImmediateStrokeProjection<'a> {
    pub stroke: &'a DrawingPreviewStroke,
    pub width_px: f32,
}

pub fn build_workspace_frame(workspace: &DrawingWorkspaceProjection) -> UiFrame {
    build_workspace_frame_with_ink(workspace, &[], &[])
}

pub fn build_workspace_frame_with_ink(
    workspace: &DrawingWorkspaceProjection,
    ink_tiles: &[DrawingInkTileProduct],
    preview_tiles: &[DrawingInkTileProduct],
) -> UiFrame {
    let ink_tiles = ink_tiles.iter().collect::<Vec<_>>();
    let preview_tiles = preview_tiles.iter().collect::<Vec<_>>();
    build_workspace_frame_with_ink_refs_and_stroke(workspace, &ink_tiles, &preview_tiles, None)
}

pub fn build_workspace_frame_with_ink_and_stroke(
    workspace: &DrawingWorkspaceProjection,
    ink_tiles: &[DrawingInkTileProduct],
    preview_tiles: &[DrawingInkTileProduct],
    immediate_stroke: Option<DrawingImmediateStrokeProjection<'_>>,
) -> UiFrame {
    let ink_tiles = ink_tiles.iter().collect::<Vec<_>>();
    let preview_tiles = preview_tiles.iter().collect::<Vec<_>>();
    build_workspace_frame_with_ink_refs_and_stroke(
        workspace,
        &ink_tiles,
        &preview_tiles,
        immediate_stroke,
    )
}

pub(crate) fn build_workspace_frame_with_ink_surface_refs_and_stroke(
    workspace: &DrawingWorkspaceProjection,
    ink_tiles: &[DrawingInkSurfaceProjection<'_>],
    preview_tiles: &[DrawingInkSurfaceProjection<'_>],
    immediate_stroke: Option<DrawingImmediateStrokeProjection<'_>>,
) -> UiFrame {
    let mut layer = UiLayer::new(DRAWING_CANVAS_LAYER_ID);
    push_workspace_base(&mut layer, workspace);
    push_projected_ink_surfaces(&mut layer, workspace, ink_tiles, 10);
    push_projected_ink_surfaces(&mut layer, workspace, preview_tiles, 20_000);
    if let Some(stroke) = immediate_stroke {
        push_immediate_stroke(&mut layer, workspace, stroke);
    }

    UiFrame::with_surfaces(vec![UiSurface::with_layers(
        DRAWING_UI_SURFACE_ID,
        workspace.window_size,
        vec![layer],
    )])
}

pub(crate) fn build_workspace_frame_with_ink_refs_and_stroke(
    workspace: &DrawingWorkspaceProjection,
    ink_tiles: &[&DrawingInkTileProduct],
    preview_tiles: &[&DrawingInkTileProduct],
    immediate_stroke: Option<DrawingImmediateStrokeProjection<'_>>,
) -> UiFrame {
    let mut layer = UiLayer::new(DRAWING_CANVAS_LAYER_ID);
    push_workspace_base(&mut layer, workspace);
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
    if let Some(stroke) = immediate_stroke {
        push_immediate_stroke(&mut layer, workspace, stroke);
    }

    UiFrame::with_surfaces(vec![UiSurface::with_layers(
        DRAWING_UI_SURFACE_ID,
        workspace.window_size,
        vec![layer],
    )])
}

fn push_workspace_base(layer: &mut UiLayer, workspace: &DrawingWorkspaceProjection) {
    push_rect(
        layer,
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
        layer,
        workspace.toolbar_bounds,
        UiPaint::rgba(0.095, 0.1, 0.108, 1.0),
        1,
    );
    if workspace.layer_panel_bounds.width > 0.0 {
        push_rect(
            layer,
            workspace.layer_panel_bounds,
            UiPaint::rgba(0.09, 0.092, 0.098, 1.0),
            2,
        );
        push_tablet_panel(layer, workspace, 30_000);
    }
    push_rect(
        layer,
        workspace.canvas_view.screen_bounds,
        UiPaint::rgba(0.93, 0.925, 0.9, 1.0),
        3,
    );
}

fn push_immediate_stroke(
    layer: &mut UiLayer,
    workspace: &DrawingWorkspaceProjection,
    projection: DrawingImmediateStrokeProjection<'_>,
) {
    let points = projection
        .stroke
        .samples
        .iter()
        .filter_map(|sample| workspace.canvas_view.canvas_to_screen(sample.position))
        .collect::<Vec<UiPoint>>();
    if points.is_empty() {
        return;
    }

    layer.push(UiPrimitive::Stroke(
        StrokePrimitive::new(
            points,
            projection.width_px.max(1.0),
            UiPaint::rgba(0.04, 0.035, 0.03, 1.0),
            DRAW_KEY_SOLID,
            UiSortKey::new(0, 0, IMMEDIATE_STROKE_PRIMITIVE_ORDER),
        )
        .with_clip(workspace.canvas_view.screen_bounds),
    ));
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
    ink_tiles: &[&DrawingInkTileProduct],
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
                drawing_ink_texture_target_id(
                    surface_kind,
                    product.metadata.quality_class,
                    product.metadata.tile_id,
                ),
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

fn push_projected_ink_surfaces(
    layer: &mut UiLayer,
    workspace: &DrawingWorkspaceProjection,
    ink_tiles: &[DrawingInkSurfaceProjection<'_>],
    primitive_order_start: u32,
) {
    let mut primitive_order = primitive_order_start;
    for projection in ink_tiles {
        push_ink_surface(
            layer,
            workspace,
            projection.product,
            projection.surface_kind,
            primitive_order,
        );
        primitive_order = primitive_order.saturating_add(1);
    }
}

fn push_ink_surface(
    layer: &mut UiLayer,
    workspace: &DrawingWorkspaceProjection,
    product: &DrawingInkTileProduct,
    surface_kind: DrawingInkSurfaceKind,
    primitive_order: u32,
) {
    let Some(rect) = workspace
        .canvas_view
        .canvas_rect_to_screen(product.metadata.invalidation_bounds)
        .and_then(|rect| rect.intersect(workspace.canvas_view.screen_bounds))
    else {
        return;
    };
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    layer.push(UiPrimitive::ProductSurface(ProductSurfacePrimitive::new(
        ProductSurfaceTextureBindingSource::dynamic_texture(
            DRAWING_INK_TEXTURE_NAMESPACE,
            drawing_ink_texture_target_id(
                surface_kind,
                product.metadata.quality_class,
                product.metadata.tile_id,
            ),
        ),
        rect,
        UiRect::new(0.0, 0.0, 1.0, 1.0),
        UiPaint::rgba(1.0, 1.0, 1.0, 1.0),
        ProductSurfaceAlphaMode::Straight,
        UiSortKey::new(0, 0, primitive_order),
    )));
}

pub fn drawing_ink_texture_target_id(
    surface_kind: DrawingInkSurfaceKind,
    quality_class: ProductQualityClass,
    tile_id: CanvasTileId,
) -> String {
    format!(
        "{}.{}.L{}.{}.{}",
        surface_kind.label(),
        quality_class.cache_token(),
        tile_id.level.raw(),
        tile_id.x,
        tile_id.y
    )
}

pub fn default_surface_size() -> UiSize {
    UiSize::new(1280.0, 720.0)
}
