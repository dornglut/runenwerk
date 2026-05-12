use drawing::{StrokeToolKind, ratify_drawing_document};
use engine::plugins::render::UiFrameSubmissionRegistryResource;
use runenwerk_draw::app::{
    DRAWING_UI_SURFACE_ID, DrawingToolRouteKind, RunenwerkDrawApp, minimal_drawing_document,
};
use runenwerk_draw::runtime::{
    DRAWING_UI_FRAME_PRODUCER_ID, DrawingHostResource, build_headless_app,
};
use ui_input::{
    Modifiers, PointerButton, PointerDeviceId, PointerEvent, PointerEventKind, PointerLatencyClass,
    PointerPacket, PointerTilt, PointerToolKind, UiInputEvent,
};
use ui_math::{UiPoint, UiVector};

#[test]
fn minimal_document_opens_as_the_app_shell_document() {
    let document = minimal_drawing_document();
    let report = ratify_drawing_document(&document);
    assert!(report.is_accepted(), "{report:?}");

    let app = RunenwerkDrawApp::new();
    let document = app.document().expect("drawing app should open a document");

    assert_eq!(document.display_name, "Untitled Drawing");
    assert_eq!(document.strokes.len(), 0);
    assert_eq!(document.pending_strokes.len(), 0);
    assert_eq!(app.active_brush_id().raw(), 1);
    assert_eq!(app.active_layer_entry_id().raw(), 1);
    assert!(app.workspace().canvas_area_ratio() > 0.45);
    assert!(!app.last_frame().is_empty());
}

#[test]
fn stylus_input_routes_to_preview_stroke_without_committing_document_truth() {
    let mut app = RunenwerkDrawApp::new();
    let position = center_of_canvas(app.workspace().canvas_view.screen_bounds);

    let packet = PointerPacket::stylus(PointerDeviceId(42), PointerToolKind::Pen)
        .with_timestamp_micros(10_000)
        .with_pressure(0.72)
        .with_tilt(PointerTilt::new(15.0, -20.0))
        .with_twist_degrees(120.0)
        .with_latency_class(PointerLatencyClass::LowLatencyPreview);

    let handled = app.dispatch_input(&UiInputEvent::Pointer(
        PointerEvent::new(
            PointerEventKind::Down,
            position,
            UiVector::ZERO,
            Some(PointerButton::Primary),
            Modifiers::default(),
            1,
        )
        .with_packet(packet),
    ));

    assert!(handled);
    assert_eq!(app.routed_inputs().len(), 1);
    let routed = &app.routed_inputs()[0];
    assert_eq!(routed.route_kind, DrawingToolRouteKind::BeginPreviewStroke);
    assert_eq!(routed.pressure, Some(0.72));
    assert_eq!(routed.twist_degrees, Some(120.0));
    assert_eq!(routed.device_id, Some(PointerDeviceId(42)));
    assert!(routed.low_latency_preview);

    let preview = app
        .preview_stroke()
        .expect("pointer down should create a preview stroke");
    assert!(preview.active);
    assert_eq!(preview.samples.len(), 1);
    assert_eq!(preview.samples[0].pressure, Some(0.72));
    assert_eq!(preview.samples[0].tool_kind, Some(StrokeToolKind::Pen));
    assert_eq!(preview.samples[0].timestamp_micros, Some(10_000));
    assert_eq!(
        app.document()
            .expect("document should remain open")
            .strokes
            .len(),
        0
    );
}

#[test]
fn pointer_up_finishes_preview_stroke_and_leaves_authoritative_document_unchanged() {
    let mut app = RunenwerkDrawApp::new();
    let position = center_of_canvas(app.workspace().canvas_view.screen_bounds);

    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Down,
        position,
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        1,
    )));
    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Up,
        UiPoint::new(position.x + 5.0, position.y + 3.0),
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        0,
    )));

    let preview = app
        .preview_stroke()
        .expect("pointer up should finish the active preview stroke");
    assert!(!preview.active);
    assert_eq!(preview.samples.len(), 2);
    assert_eq!(
        app.document()
            .expect("document should remain open")
            .strokes
            .len(),
        0
    );
}

#[test]
fn headless_runtime_starts_and_submits_canvas_first_frame() {
    let app = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run a headless frame");
    let host = app
        .world()
        .resource::<DrawingHostResource>()
        .expect("drawing host resource should exist");
    assert!(host.app.document().is_some());
    assert!(host.app.workspace().canvas_area_ratio() > 0.45);

    let submissions = app
        .world()
        .resource::<UiFrameSubmissionRegistryResource>()
        .expect("ui frame submission registry should exist");
    let submission = submissions
        .get(&DRAWING_UI_FRAME_PRODUCER_ID)
        .expect("drawing app should submit a UI frame");
    assert_eq!(submission.frame.surfaces[0].id, DRAWING_UI_SURFACE_ID);
    assert!(submission.primitive_count_hint() >= 4);
}

fn center_of_canvas(rect: ui_math::UiRect) -> UiPoint {
    UiPoint::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
}
