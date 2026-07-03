use ui_math::{UiPoint, UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, RectPrimitive, StrokePrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId,
    UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};

use super::{
    base_controls_surface2d_report, Surface2DProofRenderFrame, Surface2DProofRenderSummary,
    Surface2DProofReport,
};

pub fn base_controls_surface2d_proof_frame() -> Surface2DProofRenderFrame {
    surface2d_report_to_frame(base_controls_surface2d_report())
}

pub fn surface2d_report_to_frame(report: Surface2DProofReport) -> Surface2DProofRenderFrame {
    let mut primitives = Vec::new();
    let mut order = 0_u32;
    let viewport = UiRect::new(32.0, 32.0, 640.0, 420.0);
    primitives.push(background(viewport, &mut order));
    primitives.extend(grid(viewport, &mut order));
    primitives.push(selection_outline(
        UiRect::new(104.0, 120.0, 192.0, 144.0),
        &mut order,
    ));
    primitives.push(diagnostic_overlay(
        viewport.inset(ui_math::UiInsets::all(8.0)),
        &mut order,
    ));
    let primitive_count = primitives.len();
    let mut surface = UiSurface::new(UiSurfaceId(16), UiSize::new(704.0, 484.0));
    surface.push_layer(UiLayer::with_primitives(UiLayerId(0), primitives));
    let summary = render_summary(&report, primitive_count);
    Surface2DProofRenderFrame {
        proof_id: report.proof_id,
        frame: UiFrame::with_surfaces(vec![surface]),
        summary,
    }
}

fn render_summary(report: &Surface2DProofReport, primitive_count: usize) -> Surface2DProofRenderSummary {
    Surface2DProofRenderSummary {
        descriptor_rows: report.descriptor_evidence.len(),
        transform_rows: report.transform_evidence.len(),
        navigation_rows: report.navigation_evidence.len(),
        hover_rows: report.hover_evidence.len(),
        selection_rows: report.selection_evidence.len(),
        pointer_capture_rows: report.pointer_capture_evidence.len(),
        gesture_rows: report.gesture_evidence.len(),
        accessibility_input_rows: report.accessibility_input_evidence.len(),
        budget_rows: report.budget_evidence.len(),
        diagnostic_rows: report.diagnostic_evidence.len(),
        catalog_rows: report.catalog_projection_evidence.len(),
        inspection_rows: report.inspection_projection_evidence.len(),
        primitive_count,
        has_background: true,
        has_grid: true,
        has_selection_outline: true,
        has_diagnostic_overlay: true,
        boundary_clean: report.boundary_counters.clean(),
    }
}

fn background(rect: UiRect, order: &mut u32) -> UiPrimitive {
    RectPrimitive::new(
        rect,
        6.0,
        UiPaint::rgba(0.06, 0.07, 0.08, 1.0),
        UiDrawKey::new(1600, None),
        sort_key(order),
    )
    .into()
}

fn grid(rect: UiRect, order: &mut u32) -> Vec<UiPrimitive> {
    let mut primitives = Vec::new();
    for x in [
        rect.x + 80.0,
        rect.x + 160.0,
        rect.x + 240.0,
        rect.x + 320.0,
        rect.x + 400.0,
        rect.x + 480.0,
        rect.x + 560.0,
    ] {
        primitives.push(
            StrokePrimitive::new(
                [UiPoint::new(x, rect.y), UiPoint::new(x, rect.y + rect.height)],
                1.0,
                UiPaint::rgba(0.18, 0.2, 0.22, 1.0),
                UiDrawKey::new(1601, None),
                sort_key(order),
            )
            .with_clip(rect)
            .into(),
        );
    }
    for y in [
        rect.y + 70.0,
        rect.y + 140.0,
        rect.y + 210.0,
        rect.y + 280.0,
        rect.y + 350.0,
    ] {
        primitives.push(
            StrokePrimitive::new(
                [UiPoint::new(rect.x, y), UiPoint::new(rect.x + rect.width, y)],
                1.0,
                UiPaint::rgba(0.18, 0.2, 0.22, 1.0),
                UiDrawKey::new(1602, None),
                sort_key(order),
            )
            .with_clip(rect)
            .into(),
        );
    }
    primitives
}

fn selection_outline(rect: UiRect, order: &mut u32) -> UiPrimitive {
    BorderPrimitive::new(
        rect,
        0.0,
        2.0,
        UiPaint::rgba(0.7, 0.9, 1.0, 1.0),
        UiDrawKey::new(1603, None),
        sort_key(order),
    )
    .into()
}

fn diagnostic_overlay(rect: UiRect, order: &mut u32) -> UiPrimitive {
    BorderPrimitive::new(
        rect,
        6.0,
        2.0,
        UiPaint::rgba(1.0, 0.55, 0.2, 1.0),
        UiDrawKey::new(1604, None),
        sort_key(order),
    )
    .into()
}

fn sort_key(order: &mut u32) -> UiSortKey {
    let key = UiSortKey::new(0, 0, *order);
    *order += 1;
    key
}
