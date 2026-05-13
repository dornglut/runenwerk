use std::collections::BTreeSet;

use drawing::{
    CanvasCoordinate, CanvasTileId, DrawingTileFormationDiagnostic,
    DrawingTileFormationDiagnosticCode, StrokeToolKind, ratify_drawing_document,
};
use engine::plugins::render::{
    FeatureContributionStatus, PreparedUiFrameResource, UiFrameSubmissionRegistryResource,
};
use engine::runtime::{ProductPublicationRuntimeResource, QuerySnapshotRuntimeResource};
use engine::{BarrierKind, ExecutionBarrier, SceneRuntimeState};
use runenwerk_draw::app::{
    DRAWING_UI_SURFACE_ID, DrawingToolRouteKind, RunenwerkDrawApp, minimal_drawing_document,
};
use runenwerk_draw::runtime::{
    DRAWING_UI_FRAME_PRODUCER_ID, DrawingHostResource, build_headless_app,
    publish_drawing_ink_products, publish_drawing_ink_query_snapshots,
};
use ui_input::{
    Modifiers, PointerButton, PointerDeviceId, PointerEvent, PointerEventKind, PointerLatencyClass,
    PointerPacket, PointerTilt, PointerToolKind, UiInputEvent,
};
use ui_math::{UiPoint, UiVector};
use ui_render_data::UiPrimitive;

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
    let shell_rect_count = rect_primitive_count(app.last_frame());

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
    assert!(
        !app.ink_runtime().preview_products().is_empty(),
        "active preview should be formed from domain ink tile products"
    );
    assert_eq!(
        rect_primitive_count(app.last_frame()),
        shell_rect_count,
        "ink preview must not use the old per-pixel rect projection path"
    );
    assert!(
        product_surface_primitive_count(app.last_frame()) > 0,
        "active preview tile products should be visible before authoritative commit"
    );
}

#[test]
fn pointer_up_commits_preview_stroke_into_authoritative_document() {
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
        1
    );
    assert!(app.ink_runtime().formed_products().is_empty());
    assert!(
        !app.ink_runtime().preview_products().is_empty(),
        "released preview should remain visible until committed ink is accepted"
    );
}

#[test]
fn committed_ink_products_publish_snapshot_and_become_visible_only_after_barriers() {
    let mut app = RunenwerkDrawApp::new();
    let shell_only_count = rect_primitive_count(app.last_frame());
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
        PointerEventKind::Move,
        UiPoint::new(position.x + 12.0, position.y + 4.0),
        UiVector::new(12.0, 4.0),
        Some(PointerButton::Primary),
        Modifiers::default(),
        0,
    )));
    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Up,
        UiPoint::new(position.x + 18.0, position.y + 8.0),
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        0,
    )));

    assert_eq!(app.document().unwrap().strokes.len(), 1);
    assert!(app.ink_runtime().formed_products().is_empty());
    assert!(!app.ink_runtime().preview_products().is_empty());

    let mut publications = ProductPublicationRuntimeResource::default();
    let mut snapshots = QuerySnapshotRuntimeResource::default();
    let publication = publish_drawing_ink_products(
        &mut app,
        &mut publications,
        &barrier(BarrierKind::ProductPublication),
    );
    assert_eq!(publication.published_count, 1);
    assert!(!app.ink_runtime().formed_products().is_empty());
    assert!(app.ink_runtime().accepted_snapshot_ids().is_empty());
    assert!(app.ink_runtime().visible_products().next().is_none());
    let repeated_publication = publish_drawing_ink_products(
        &mut app,
        &mut publications,
        &barrier(BarrierKind::ProductPublication),
    );
    assert_eq!(repeated_publication.published_count, 0);

    let query = publish_drawing_ink_query_snapshots(
        &mut app,
        &mut snapshots,
        &barrier(BarrierKind::QuerySnapshotPublication),
    );
    assert!(query.published_count > 0);
    assert!(!app.ink_runtime().accepted_snapshot_ids().is_empty());
    assert!(app.ink_runtime().preview_products().is_empty());
    assert!(app.ink_runtime().visible_products().next().is_some());

    let size = app.workspace().window_size;
    app.rebuild_frame(size);
    assert_eq!(
        rect_primitive_count(app.last_frame()),
        shell_only_count,
        "accepted ink should no longer expand into UI rect primitives"
    );
    assert_eq!(
        product_surface_primitive_count(app.last_frame()),
        app.ink_runtime().visible_product_count(),
        "accepted current ink tile snapshots should project one product surface per visible tile"
    );
    assert!(
        product_surface_primitive_count(app.last_frame()) > 0,
        "accepted current ink tile snapshots should project visible committed ink"
    );
}

#[test]
fn pointer_release_keeps_last_accepted_ink_visible_before_new_barriers() {
    let mut app = RunenwerkDrawApp::new();
    let position = center_of_canvas(app.workspace().canvas_view.screen_bounds);
    draw_stroke(
        &mut app,
        position,
        UiPoint::new(position.x + 18.0, position.y + 8.0),
    );

    let mut publications = ProductPublicationRuntimeResource::default();
    let mut snapshots = QuerySnapshotRuntimeResource::default();
    publish_drawing_ink_products(
        &mut app,
        &mut publications,
        &barrier(BarrierKind::ProductPublication),
    );
    publish_drawing_ink_query_snapshots(
        &mut app,
        &mut snapshots,
        &barrier(BarrierKind::QuerySnapshotPublication),
    );
    assert!(app.ink_runtime().visible_products().next().is_some());
    let accepted_count = app.ink_runtime().visible_products().count();

    draw_stroke(
        &mut app,
        UiPoint::new(position.x + 24.0, position.y + 16.0),
        UiPoint::new(position.x + 42.0, position.y + 24.0),
    );

    assert_eq!(app.ink_runtime().visible_products().count(), accepted_count);
    assert!(
        !app.ink_runtime().preview_products().is_empty(),
        "new released preview should overlay the last accepted committed ink"
    );
}

#[test]
fn two_committed_strokes_publish_and_remain_drawable() {
    let mut app = RunenwerkDrawApp::new();
    let first_start = screen_point_for_canvas(&app, 512.0, 512.0);
    let first_end = screen_point_for_canvas(&app, 620.0, 560.0);
    let second_start = screen_point_for_canvas(&app, 3_100.0, 3_100.0);
    let second_end = screen_point_for_canvas(&app, 3_240.0, 3_180.0);
    let mut publications = ProductPublicationRuntimeResource::default();
    let mut snapshots = QuerySnapshotRuntimeResource::default();

    draw_stroke(&mut app, first_start, first_end);
    publish_drawing_ink_products(
        &mut app,
        &mut publications,
        &barrier(BarrierKind::ProductPublication),
    );
    publish_drawing_ink_query_snapshots(
        &mut app,
        &mut snapshots,
        &barrier(BarrierKind::QuerySnapshotPublication),
    );
    let first_tiles = visible_tile_ids(&app);
    assert!(!first_tiles.is_empty());

    draw_stroke(&mut app, second_start, second_end);
    assert!(
        first_tiles.is_subset(&visible_tile_ids(&app)),
        "releasing the second stroke must preserve first-stroke visible tiles until new barriers accept"
    );
    publish_drawing_ink_products(
        &mut app,
        &mut publications,
        &barrier(BarrierKind::ProductPublication),
    );
    publish_drawing_ink_query_snapshots(
        &mut app,
        &mut snapshots,
        &barrier(BarrierKind::QuerySnapshotPublication),
    );

    assert_eq!(app.document().expect("document is open").strokes.len(), 2);
    assert!(app.ink_runtime().visible_products().next().is_some());
    assert!(
        first_tiles.is_subset(&visible_tile_ids(&app)),
        "accepting the second stroke should not clear unrelated first-stroke tiles"
    );
    assert!(app.ink_runtime().preview_products().is_empty());
}

#[test]
fn formation_failure_preserves_last_good_visible_ink_and_records_diagnostics() {
    let mut app = RunenwerkDrawApp::new();
    let position = center_of_canvas(app.workspace().canvas_view.screen_bounds);
    draw_stroke(
        &mut app,
        position,
        UiPoint::new(position.x + 18.0, position.y + 8.0),
    );

    let mut publications = ProductPublicationRuntimeResource::default();
    let mut snapshots = QuerySnapshotRuntimeResource::default();
    publish_drawing_ink_products(
        &mut app,
        &mut publications,
        &barrier(BarrierKind::ProductPublication),
    );
    publish_drawing_ink_query_snapshots(
        &mut app,
        &mut snapshots,
        &barrier(BarrierKind::QuerySnapshotPublication),
    );
    assert!(app.ink_runtime().visible_products().next().is_some());
    let accepted_count = app.ink_runtime().visible_product_count();
    app.ink_runtime_mut().record_failed_generation(
        "test.failed.formation".to_string(),
        vec![DrawingTileFormationDiagnostic::blocking(
            DrawingTileFormationDiagnosticCode::InvalidPolicy,
            "test formation failure",
        )],
        true,
    );

    assert_eq!(app.ink_runtime().visible_product_count(), accepted_count);
    assert!(app.ink_runtime().visible_products().next().is_some());
    assert!(app.ink_runtime().preview_products().is_empty());
    assert!(app.ink_runtime().diagnostics().iter().any(|diagnostic| {
        diagnostic.code == DrawingTileFormationDiagnosticCode::InvalidPolicy
    }));
}

#[test]
fn long_stroke_batches_dirty_tiles_instead_of_clearing_canvas() {
    let mut app = RunenwerkDrawApp::new();
    let mut publications = ProductPublicationRuntimeResource::default();
    let mut snapshots = QuerySnapshotRuntimeResource::default();
    let canvas = app.workspace().canvas_view.screen_bounds;
    draw_stroke(
        &mut app,
        UiPoint::new(canvas.x + 1.0, canvas.y + 1.0),
        UiPoint::new(
            canvas.x + canvas.width - 1.0,
            canvas.y + canvas.height - 1.0,
        ),
    );

    assert_eq!(app.document().expect("document is open").strokes.len(), 1);
    assert!(
        app.ink_runtime().dirty_tiles().len() > drawing::DEFAULT_MAX_AFFECTED_INK_TILES,
        "the diagonal canvas stroke should exceed one interactive formation batch"
    );

    let mut batches = 0;
    while !app.ink_runtime().dirty_tiles().is_empty() {
        batches += 1;
        assert!(batches <= 8, "dirty tile batches should drain promptly");
        let report = publish_drawing_ink_products(
            &mut app,
            &mut publications,
            &barrier(BarrierKind::ProductPublication),
        );
        assert!(
            report.published_count > 0 || report.rejected_count == 0,
            "long stroke batches must not reject solely because the whole stroke spans many tiles"
        );
        publish_drawing_ink_query_snapshots(
            &mut app,
            &mut snapshots,
            &barrier(BarrierKind::QuerySnapshotPublication),
        );
    }

    assert!(
        batches > 1,
        "long stroke should be formed as more than one bounded batch"
    );
    assert!(app.ink_runtime().visible_product_count() > 0);
    assert!(app.ink_runtime().diagnostics().iter().all(|diagnostic| {
        diagnostic.code != DrawingTileFormationDiagnosticCode::TooManyAffectedTiles
    }));
    app.rebuild_frame(app.workspace().window_size);
    let product_surface_count = product_surface_primitive_count(app.last_frame());
    assert!(
        product_surface_count > 1,
        "long stroke should render as tile-count-scaled product surfaces"
    );
    assert!(
        product_surface_count <= app.ink_runtime().visible_product_count(),
        "offscreen or clipped tiles may be stored without producing screen primitives"
    );
}

#[test]
fn query_snapshots_wait_for_product_publication() {
    let mut app = RunenwerkDrawApp::new();
    let mut snapshots = QuerySnapshotRuntimeResource::default();

    let report = publish_drawing_ink_query_snapshots(
        &mut app,
        &mut snapshots,
        &barrier(BarrierKind::QuerySnapshotPublication),
    );

    assert_eq!(report.published_count, 0);
    assert!(snapshots.current_snapshots().is_empty());
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

#[test]
fn headless_runtime_installs_visible_shell_render_prerequisites() {
    let app = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run a headless frame");

    app.world()
        .resource::<SceneRuntimeState>()
        .expect("drawing app should install scene runtime state");

    let prepared_ui = app
        .world()
        .resource::<PreparedUiFrameResource>()
        .expect("prepared ui frame resource should exist");
    assert_eq!(prepared_ui.status, FeatureContributionStatus::Ready);

    let prepared_submission = prepared_ui
        .payload
        .submissions
        .iter()
        .find(|submission| submission.producer_id == DRAWING_UI_FRAME_PRODUCER_ID)
        .expect("drawing app should prepare its UI frame for render composition");
    assert_eq!(
        prepared_submission.frame.surfaces[0].id,
        DRAWING_UI_SURFACE_ID
    );
    assert!(prepared_submission.primitive_count_hint() >= 4);
}

fn center_of_canvas(rect: ui_math::UiRect) -> UiPoint {
    UiPoint::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
}

fn screen_point_for_canvas(app: &RunenwerkDrawApp, x: f64, y: f64) -> UiPoint {
    app.workspace()
        .canvas_view
        .canvas_to_screen(CanvasCoordinate::new(x, y))
        .expect("test canvas point should project into the workspace")
}

fn draw_stroke(app: &mut RunenwerkDrawApp, start: UiPoint, end: UiPoint) {
    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Down,
        start,
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        1,
    )));
    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Up,
        end,
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        0,
    )));
}

fn barrier(kind: BarrierKind) -> ExecutionBarrier {
    ExecutionBarrier {
        index: 5,
        phase_index: 0,
        after_wave_index: Some(0),
        kind,
    }
}

fn rect_primitive_count(frame: &ui_render_data::UiFrame) -> usize {
    frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .filter(|primitive| matches!(primitive, UiPrimitive::Rect(_)))
        .count()
}

fn product_surface_primitive_count(frame: &ui_render_data::UiFrame) -> usize {
    frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .filter(|primitive| matches!(primitive, UiPrimitive::ProductSurface(_)))
        .count()
}

fn visible_tile_ids(app: &RunenwerkDrawApp) -> BTreeSet<CanvasTileId> {
    app.ink_runtime()
        .visible_products()
        .map(|product| product.metadata.tile_id)
        .collect()
}
