use ui_math::{UiRect, UiSize};
use ui_render_data::{BorderPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId, UiPaint, UiSortKey, UiSurface, UiSurfaceId};
use ui_static_mount::UiStaticMountReport;

#[test]
fn base_controls_overlay_layering_static_mount_accepts_renderer_neutral_frame() {
    let mut surface = UiSurface::new(UiSurfaceId(13), UiSize::new(640.0, 360.0));
    surface.push_layer(UiLayer::with_primitives(
        UiLayerId(0),
        vec![
            RectPrimitive::new(
                UiRect::new(0.0, 0.0, 640.0, 360.0),
                0.0,
                UiPaint::rgba(0.1, 0.1, 0.12, 1.0),
                UiDrawKey::new(1301, None),
                UiSortKey::new(0, 0, 0),
            )
            .into(),
            BorderPrimitive::new(
                UiRect::new(16.0, 16.0, 608.0, 328.0),
                2.0,
                1.0,
                UiPaint::WHITE,
                UiDrawKey::new(1302, None),
                UiSortKey::new(0, 0, 1),
            )
            .into(),
        ],
    ));

    let report = UiStaticMountReport::from_frame(UiFrame::with_surfaces(vec![surface]));

    assert!(report.passed(), "static overlay proof frame should mount: {:?}", report.diagnostics());
    let summary = &report.mounted_frame().expect("mounted frame").summary;
    assert_eq!(summary.surface_count, 1);
    assert!(summary.has_rect_primitive);
    assert!(summary.has_border_primitive);
    assert!(summary.draw_order_stable);
}
