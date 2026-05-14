use std::collections::BTreeSet;

use drawing::{
    CanvasCoordinate, CanvasTileId, DrawingTileFormationDiagnostic,
    DrawingTileFormationDiagnosticCode, StrokeToolKind, ratify_drawing_document,
};
use engine::plugins::render::{
    FeatureContributionStatus, PreparedUiFrameResource, RenderDynamicTextureUploadRegistryResource,
    UiFrameSubmissionRegistryResource,
};
use engine::plugins::{InputState, TouchInputPhase};
use engine::runtime::{
    ProductPublicationRuntimeResource, QuerySnapshotRuntimeResource, RuntimeJobExecutorConfig,
    RuntimeJobExecutorResource, RuntimeProductCacheResource,
};
use engine::{BarrierKind, ExecutionBarrier, SceneRuntimeState};
use native_tablet_input::{
    NativeTabletBackendHealth, NativeTabletBackendKind, NativeTabletFrameResource,
    NativeTabletPacket, NativeTabletSample, map_native_tablet_packet,
};
use runenwerk_draw::app::{
    DRAWING_UI_SURFACE_ID, DrawingToolRouteKind, RunenwerkDrawApp, minimal_drawing_document,
};
use runenwerk_draw::runtime::{
    DRAWING_UI_FRAME_PRODUCER_ID, DrawingHostResource, build_headless_app,
    process_drawing_preview_ink_jobs, publish_drawing_ink_products,
    publish_drawing_ink_products_with_executor_and_cache, publish_drawing_ink_query_snapshots,
};
use ui_input::{
    Modifiers, PointerButton, PointerContactState, PointerDeviceId, PointerEvent, PointerEventKind,
    PointerLatencyClass, PointerPacket, PointerSample, PointerSampleRole, PointerTilt,
    PointerToolKind, UiInputEvent,
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
    assert!(app.ink_runtime().preview_products().is_empty());
    assert_eq!(
        app.ink_runtime().last_preview_dirty_tile_count(),
        0,
        "dispatch_input must not form preview ink tiles synchronously"
    );
    assert!(
        stroke_primitive_count(app.last_frame()) > 0,
        "active preview should use an immediate stroke primitive"
    );
    assert_eq!(
        rect_primitive_count(app.last_frame()),
        shell_rect_count,
        "ink preview must not use the old per-pixel rect projection path"
    );
    assert_eq!(
        product_surface_primitive_count(app.last_frame()),
        0,
        "preview tile catch-up must not run on the input hot path"
    );
}

#[test]
fn coalesced_pointer_samples_append_ordered_preview_samples_before_current_sample() {
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

    let packet = PointerPacket::stylus(PointerDeviceId(7), PointerToolKind::Pen)
        .with_coalesced_samples([
            PointerSample::new(
                PointerSampleRole::Coalesced,
                UiPoint::new(position.x + 4.0, position.y + 1.0),
                UiVector::new(4.0, 1.0),
            )
            .with_timestamp_micros(10_100)
            .with_pressure(0.4),
            PointerSample::new(
                PointerSampleRole::Coalesced,
                UiPoint::new(position.x + 8.0, position.y + 3.0),
                UiVector::new(4.0, 2.0),
            )
            .with_timestamp_micros(10_200)
            .with_pressure(0.6),
        ])
        .with_timestamp_micros(10_300)
        .with_pressure(0.8);

    app.dispatch_input(&UiInputEvent::Pointer(
        PointerEvent::new(
            PointerEventKind::Move,
            UiPoint::new(position.x + 12.0, position.y + 6.0),
            UiVector::new(4.0, 3.0),
            Some(PointerButton::Primary),
            Modifiers::default(),
            0,
        )
        .with_packet(packet),
    ));

    let preview = app
        .preview_stroke()
        .expect("move should update the active preview stroke");
    assert_eq!(
        preview.samples.len(),
        4,
        "down sample plus two coalesced samples plus the current move sample should be retained"
    );
    assert_eq!(preview.samples[0].sequence, 1);
    assert_eq!(preview.samples[1].sequence, 2);
    assert_eq!(preview.samples[2].sequence, 3);
    assert_eq!(preview.samples[3].sequence, 4);
    assert_eq!(preview.samples[1].timestamp_micros, Some(10_100));
    assert_eq!(preview.samples[2].timestamp_micros, Some(10_200));
    assert_eq!(preview.samples[3].timestamp_micros, Some(10_300));
    assert_eq!(preview.samples[1].pressure, Some(0.4));
    assert_eq!(preview.samples[2].pressure, Some(0.6));
    assert_eq!(preview.samples[3].pressure, Some(0.8));
    assert_eq!(app.routed_inputs()[1].coalesced_sample_count, 2);
    assert_eq!(app.routed_inputs()[1].coalesced_samples.len(), 2);
}

#[test]
fn active_stroke_continues_outside_canvas_and_commits_on_outside_release() {
    let mut app = RunenwerkDrawApp::new();
    let start = screen_point_for_canvas(&app, 64.0, 64.0);
    let outside = app
        .workspace()
        .canvas_view
        .canvas_to_screen(CanvasCoordinate::new(-128.0, -96.0))
        .expect("unbounded canvas point should project to screen space");
    let reenter = screen_point_for_canvas(&app, 128.0, 128.0);

    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Down,
        start,
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        1,
    )));
    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Move,
        outside,
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        0,
    )));
    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Move,
        reenter,
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        0,
    )));
    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Up,
        outside,
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        0,
    )));

    assert_eq!(
        app.document()
            .expect("document should remain open")
            .strokes
            .len(),
        1,
        "release outside the canvas should commit the captured stroke"
    );
    let preview = app
        .preview_stroke()
        .expect("captured release should keep the released preview available");
    assert!(!preview.active);
    assert_eq!(preview.samples.len(), 4);
    assert!(
        preview.samples[1].position.x < 0.0 && preview.samples[1].position.y < 0.0,
        "the outside-canvas move should remain part of the same stroke"
    );
    assert_eq!(
        app.routed_inputs()[1].route_kind,
        DrawingToolRouteKind::UpdatePreviewStroke
    );
    assert_eq!(
        app.routed_inputs()[3].route_kind,
        DrawingToolRouteKind::EndPreviewStroke
    );
}

#[test]
fn window_touch_history_routes_as_ordered_preview_samples() {
    let mut runtime = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run its startup frame");
    let (start, mid, end) = {
        let host = runtime
            .world()
            .resource::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        (
            screen_point_for_canvas(&host.app, 800.0, 800.0),
            screen_point_for_canvas(&host.app, 900.0, 920.0),
            screen_point_for_canvas(&host.app, 1_000.0, 980.0),
        )
    };
    {
        let input = runtime
            .world_mut()
            .resource_mut::<InputState>()
            .expect("input state should exist");
        input.handle_touch_input(TouchInputPhase::Started, 91, start.x, start.y, Some(0.4));
        input.handle_touch_input(TouchInputPhase::Moved, 91, mid.x, mid.y, Some(0.6));
        input.handle_touch_input(TouchInputPhase::Moved, 91, end.x, end.y, Some(0.8));
    }

    runtime = runtime
        .run_for_frames(1)
        .expect("touch history route frame should run");

    let host = runtime
        .world()
        .resource::<DrawingHostResource>()
        .expect("drawing host resource should exist");
    let preview = host
        .app
        .preview_stroke()
        .expect("touch input should create a preview stroke");
    assert_eq!(
        preview.samples.len(),
        3,
        "touch history should be routed as down plus coalesced move samples"
    );
    assert_eq!(preview.samples[0].pressure, Some(0.4));
    assert_eq!(preview.samples[1].pressure, Some(0.6));
    assert_eq!(preview.samples[2].pressure, Some(0.8));
    assert_eq!(preview.samples[0].sequence, 1);
    assert_eq!(preview.samples[1].sequence, 2);
    assert_eq!(preview.samples[2].sequence, 3);
    assert_eq!(host.app.routed_inputs().len(), 2);
    assert_eq!(host.app.routed_inputs()[1].coalesced_sample_count, 1);
}

#[test]
fn native_tablet_move_burst_routes_as_one_coalesced_preview_update() {
    let mut runtime = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run its startup frame");
    let positions = {
        let host = runtime
            .world()
            .resource::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        [
            screen_point_for_canvas(&host.app, 600.0, 600.0),
            screen_point_for_canvas(&host.app, 650.0, 620.0),
            screen_point_for_canvas(&host.app, 700.0, 650.0),
            screen_point_for_canvas(&host.app, 760.0, 700.0),
        ]
    };

    {
        let native_frame = runtime
            .world_mut()
            .resource_mut::<NativeTabletFrameResource>()
            .expect("native tablet frame resource should exist");
        native_frame.events.push(stylus_pointer_event(
            PointerEventKind::Down,
            positions[0],
            10_000,
            0.3,
        ));
        native_frame.events.push(stylus_pointer_event(
            PointerEventKind::Move,
            positions[1],
            10_100,
            0.5,
        ));
        native_frame.events.push(stylus_pointer_event(
            PointerEventKind::Move,
            positions[2],
            10_200,
            0.7,
        ));
        native_frame.events.push(stylus_pointer_event(
            PointerEventKind::Move,
            positions[3],
            10_300,
            0.9,
        ));
    }

    runtime = runtime
        .run_for_frames(1)
        .expect("native tablet move burst route frame should run");

    let host = runtime
        .world()
        .resource::<DrawingHostResource>()
        .expect("drawing host resource should exist");
    assert_eq!(
        host.app.routed_inputs().len(),
        2,
        "same-frame native move bursts should be routed as one coalesced move"
    );
    assert_eq!(host.app.routed_inputs()[1].coalesced_sample_count, 2);
    let preview = host
        .app
        .preview_stroke()
        .expect("native burst should start a preview stroke");
    assert_eq!(preview.samples.len(), 4);
    assert_eq!(preview.samples[1].timestamp_micros, Some(10_100));
    assert_eq!(preview.samples[2].timestamp_micros, Some(10_200));
    assert_eq!(preview.samples[3].timestamp_micros, Some(10_300));
    assert_eq!(preview.samples[1].pressure, Some(0.5));
    assert_eq!(preview.samples[2].pressure, Some(0.7));
    assert_eq!(preview.samples[3].pressure, Some(0.9));
}

#[test]
fn native_tablet_events_route_before_winit_touch_fallback() {
    let mut runtime = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run its startup frame");
    let (native_start, fallback_start) = {
        let host = runtime
            .world()
            .resource::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        (
            screen_point_for_canvas(&host.app, 700.0, 700.0),
            screen_point_for_canvas(&host.app, 1_000.0, 1_000.0),
        )
    };
    {
        let native_frame = runtime
            .world_mut()
            .resource_mut::<NativeTabletFrameResource>()
            .expect("native tablet frame resource should exist");
        let packet = NativeTabletPacket::windows_pointer(
            501,
            PointerEventKind::Down,
            native_start,
            UiVector::ZERO,
        )
        .with_pressure(0.7);
        native_frame
            .events
            .push(map_native_tablet_packet(&packet).event);
    }
    {
        let input = runtime
            .world_mut()
            .resource_mut::<InputState>()
            .expect("input state should exist");
        input.handle_touch_input(
            TouchInputPhase::Started,
            91,
            fallback_start.x,
            fallback_start.y,
            Some(0.2),
        );
    }

    runtime = runtime
        .run_for_frames(1)
        .expect("native tablet input route frame should run");

    let host = runtime
        .world()
        .resource::<DrawingHostResource>()
        .expect("drawing host resource should exist");
    assert_eq!(host.app.routed_inputs().len(), 1);
    assert_eq!(
        host.app.routed_inputs()[0].device_id,
        Some(PointerDeviceId(501)),
        "native tablet packet should route and suppress duplicate fallback touch for the frame"
    );
}

#[test]
fn active_native_contact_suppresses_fallback_without_new_native_samples() {
    let mut runtime = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run its startup frame");
    let fallback_start = {
        let host = runtime
            .world()
            .resource::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        screen_point_for_canvas(&host.app, 1_000.0, 1_000.0)
    };
    {
        let native_frame = runtime
            .world_mut()
            .resource_mut::<NativeTabletFrameResource>()
            .expect("native tablet frame resource should exist");
        native_frame.active_native_contact = true;
    }
    {
        let input = runtime
            .world_mut()
            .resource_mut::<InputState>()
            .expect("input state should exist");
        input.handle_touch_input(
            TouchInputPhase::Started,
            92,
            fallback_start.x,
            fallback_start.y,
            Some(0.3),
        );
    }

    runtime = runtime
        .run_for_frames(1)
        .expect("native active-contact suppression frame should run");

    let host = runtime
        .world()
        .resource::<DrawingHostResource>()
        .expect("drawing host resource should exist");
    assert!(
        host.app.routed_inputs().is_empty(),
        "winit fallback input should not duplicate an active native stylus contact"
    );
}

#[test]
fn native_tablet_coalesced_samples_become_ordered_preview_samples() {
    let mut runtime = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run its startup frame");
    let (start, c1, c2, current) = {
        let host = runtime
            .world()
            .resource::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        (
            screen_point_for_canvas(&host.app, 600.0, 600.0),
            screen_point_for_canvas(&host.app, 650.0, 620.0),
            screen_point_for_canvas(&host.app, 700.0, 650.0),
            screen_point_for_canvas(&host.app, 760.0, 700.0),
        )
    };
    {
        let native_frame = runtime
            .world_mut()
            .resource_mut::<NativeTabletFrameResource>()
            .expect("native tablet frame resource should exist");
        let down =
            NativeTabletPacket::windows_pointer(502, PointerEventKind::Down, start, UiVector::ZERO)
                .with_pressure(0.4);
        let movement = NativeTabletPacket::windows_pointer(
            502,
            PointerEventKind::Move,
            current,
            UiVector::new(current.x - c2.x, current.y - c2.y),
        )
        .with_pressure(0.9)
        .with_timestamp_micros(40)
        .with_coalesced_samples([
            NativeTabletSample::new(c1, UiVector::new(c1.x - start.x, c1.y - start.y))
                .with_timestamp_micros(20)
                .with_pressure(0.5),
            NativeTabletSample::new(c2, UiVector::new(c2.x - c1.x, c2.y - c1.y))
                .with_timestamp_micros(30)
                .with_pressure(0.7),
        ]);
        native_frame
            .events
            .push(map_native_tablet_packet(&down).event);
        native_frame
            .events
            .push(map_native_tablet_packet(&movement).event);
    }

    runtime = runtime
        .run_for_frames(1)
        .expect("native tablet coalesced route frame should run");

    let host = runtime
        .world()
        .resource::<DrawingHostResource>()
        .expect("drawing host resource should exist");
    let preview = host
        .app
        .preview_stroke()
        .expect("native input should start a preview stroke");
    assert_eq!(preview.samples.len(), 4);
    assert_eq!(preview.samples[0].sequence, 1);
    assert_eq!(preview.samples[1].sequence, 2);
    assert_eq!(preview.samples[2].sequence, 3);
    assert_eq!(preview.samples[3].sequence, 4);
    assert_eq!(preview.samples[1].pressure, Some(0.5));
    assert_eq!(preview.samples[2].pressure, Some(0.7));
    assert_eq!(preview.samples[3].pressure, Some(0.9));
}

#[test]
fn native_tablet_diagnostics_project_into_workspace_panel() {
    let mut runtime = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run its startup frame");
    {
        let native_frame = runtime
            .world_mut()
            .resource_mut::<NativeTabletFrameResource>()
            .expect("native tablet frame resource should exist");
        native_frame
            .backend_health
            .push(NativeTabletBackendHealth::active(
                NativeTabletBackendKind::WindowsPointer,
                "test active backend",
            ));
        native_frame.telemetry.sample_rate_hz = 180.0;
        native_frame.telemetry.max_segment_gap_px = 18.0;
        native_frame.telemetry.pressure_available = true;
        native_frame.telemetry.tilt_available = true;
    }

    runtime = runtime
        .run_for_frames(1)
        .expect("native tablet diagnostics frame should run");

    let host = runtime
        .world()
        .resource::<DrawingHostResource>()
        .expect("drawing host resource should exist");
    let panel = &host.app.workspace().tablet_panel;
    assert_eq!(panel.active_backend, "Windows Pointer/Ink");
    assert_eq!(panel.sample_rate_hz, 180.0);
    assert_eq!(panel.max_segment_gap_px, 18.0);
    assert!(panel.pressure_available);
    assert!(panel.tilt_available);
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
    assert!(app.ink_runtime().preview_products().is_empty());
    assert!(
        stroke_primitive_count(app.last_frame()) > 0,
        "released preview should remain visible through the immediate stroke primitive"
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
    assert!(app.ink_runtime().preview_products().is_empty());
    assert!(stroke_primitive_count(app.last_frame()) > 0);

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
    assert_eq!(stroke_primitive_count(app.last_frame()), 0);
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
    assert!(app.ink_runtime().preview_products().is_empty());
    assert!(
        stroke_primitive_count(app.last_frame()) > 0,
        "new released preview should overlay the last accepted committed ink immediately"
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
fn long_active_preview_updates_only_dirty_tail_tiles() {
    let mut app = RunenwerkDrawApp::new();
    let mut executor = RuntimeJobExecutorResource::with_config(RuntimeJobExecutorConfig::serial());
    let start = screen_point_for_canvas(&app, 64.0, 64.0);
    app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
        PointerEventKind::Down,
        start,
        UiVector::ZERO,
        Some(PointerButton::Primary),
        Modifiers::default(),
        1,
    )));
    process_drawing_preview_ink_jobs(&mut app, &mut executor);

    let mut max_dirty_tiles = app.ink_runtime().last_preview_dirty_tile_count();
    for step in 1..=20 {
        let canvas_position = 64.0 + step as f64 * 192.0;
        let screen_position = screen_point_for_canvas(&app, canvas_position, canvas_position);
        app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
            PointerEventKind::Move,
            screen_position,
            UiVector::ZERO,
            Some(PointerButton::Primary),
            Modifiers::default(),
            0,
        )));
        let report = process_drawing_preview_ink_jobs(&mut app, &mut executor);
        assert!(
            report.submitted_count <= 1,
            "preview scheduling should keep at most one catch-up job per update"
        );
        max_dirty_tiles = max_dirty_tiles.max(app.ink_runtime().last_preview_dirty_tile_count());
        assert!(
            app.ink_runtime().last_preview_dirty_tile_count() <= 8,
            "each small preview update should only reform the current stroke tail tiles"
        );
    }

    let preview = app
        .preview_stroke()
        .expect("long active stroke should still be previewing");
    assert_eq!(preview.samples.len(), 21);
    assert!(
        stroke_primitive_count(app.last_frame()) > 0,
        "immediate stroke feedback should remain present while tiles catch up"
    );
    assert!(
        app.ink_runtime().preview_products().len() > max_dirty_tiles,
        "the accumulated preview may span more tiles than any single dirty-tail update"
    );
    assert!(app.ink_runtime().diagnostics().iter().all(|diagnostic| {
        diagnostic.code != DrawingTileFormationDiagnosticCode::TooManyAffectedTiles
    }));
}

#[test]
fn runtime_uploads_preview_products_only_when_generation_changes() {
    let mut runtime = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run its startup frame");
    runtime.insert_resource(RuntimeJobExecutorResource::with_config(
        RuntimeJobExecutorConfig::serial(),
    ));

    {
        let host = runtime
            .world_mut()
            .resource_mut::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        let app = &mut host.app;
        let start = screen_point_for_canvas(app, 64.0, 64.0);
        app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
            PointerEventKind::Down,
            start,
            UiVector::ZERO,
            Some(PointerButton::Primary),
            Modifiers::default(),
            1,
        )));
    }

    runtime = runtime
        .run_for_frames(1)
        .expect("initial preview job frame should run");

    let mut first_dirty_tail_count = {
        let host = runtime
            .world()
            .resource::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        assert!(!host.app.ink_runtime().preview_products().is_empty());
        host.app.ink_runtime().last_preview_dirty_tile_count()
    };

    for step in 1..=8 {
        {
            let host = runtime
                .world_mut()
                .resource_mut::<DrawingHostResource>()
                .expect("drawing host resource should exist");
            let app = &mut host.app;
            let canvas_position = 64.0 + step as f64 * 96.0;
            let screen_position = screen_point_for_canvas(app, canvas_position, canvas_position);
            app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
                PointerEventKind::Move,
                screen_position,
                UiVector::ZERO,
                Some(PointerButton::Primary),
                Modifiers::default(),
                0,
            )));
        }

        runtime = runtime
            .run_for_frames(1)
            .expect("preview catch-up frame should run");

        let dirty_tail_count = {
            let host = runtime
                .world()
                .resource::<DrawingHostResource>()
                .expect("drawing host resource should exist");
            host.app.ink_runtime().last_preview_dirty_tile_count()
        };
        first_dirty_tail_count = first_dirty_tail_count.max(dirty_tail_count);
        assert!(dirty_tail_count <= 8);
    }

    let preview_count = {
        let host = runtime
            .world()
            .resource::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        let preview_count = host.app.ink_runtime().preview_products().len();
        assert!(
            preview_count > first_dirty_tail_count,
            "the active preview should accumulate more tiles than the latest dirty tail"
        );
        preview_count
    };

    runtime = runtime
        .run_for_frames(1)
        .expect("stable preview frame should run");
    assert_eq!(
        ink_upload_count(&runtime),
        0,
        "unchanged preview products must not be re-uploaded every frame"
    );

    let next_dirty_tail_count = {
        {
            let host = runtime
                .world_mut()
                .resource_mut::<DrawingHostResource>()
                .expect("drawing host resource should exist");
            let app = &mut host.app;
            let screen_position = screen_point_for_canvas(app, 1_000.0, 1_000.0);
            app.dispatch_input(&UiInputEvent::Pointer(PointerEvent::new(
                PointerEventKind::Move,
                screen_position,
                UiVector::ZERO,
                Some(PointerButton::Primary),
                Modifiers::default(),
                0,
            )));
        }

        runtime = runtime
            .run_for_frames(1)
            .expect("dirty preview tail upload frame should run");
        let host = runtime
            .world()
            .resource::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        host.app.ink_runtime().last_preview_dirty_tile_count()
    };

    let upload_count = ink_upload_count(&runtime);
    assert!(
        upload_count > 0,
        "moving the active stroke should upload the changed preview tail"
    );
    assert!(
        upload_count <= next_dirty_tail_count,
        "preview uploads should be bounded by the latest dirty tail"
    );
    assert!(
        upload_count < preview_count,
        "preview uploads should not scale with the whole long active stroke"
    );
    assert!(
        next_dirty_tail_count <= first_dirty_tail_count.max(8),
        "the follow-up move should stay within the bounded dirty-tail policy"
    );
}

#[test]
fn runtime_uploads_only_new_committed_products_after_more_strokes() {
    let mut runtime = build_headless_app()
        .run_for_frames(1)
        .expect("drawing app should run its startup frame");
    let mut publications = ProductPublicationRuntimeResource::default();
    let mut snapshots = QuerySnapshotRuntimeResource::default();

    let first_visible_count = {
        let host = runtime
            .world_mut()
            .resource_mut::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        let first_start = screen_point_for_canvas(&host.app, 512.0, 512.0);
        let first_end = screen_point_for_canvas(&host.app, 620.0, 560.0);
        draw_and_accept_stroke(
            &mut host.app,
            &mut publications,
            &mut snapshots,
            first_start,
            first_end,
        );
        host.app.ink_runtime().visible_product_count()
    };
    assert!(first_visible_count > 0);

    runtime = runtime
        .run_for_frames(1)
        .expect("first committed upload frame should run");
    assert_eq!(
        ink_upload_count(&runtime),
        first_visible_count,
        "the first visible generation should upload once"
    );
    runtime = runtime
        .run_for_frames(1)
        .expect("stable committed frame should run");
    assert_eq!(
        ink_upload_count(&runtime),
        0,
        "unchanged committed ink must not be re-uploaded every frame"
    );

    let total_visible_count = {
        let host = runtime
            .world_mut()
            .resource_mut::<DrawingHostResource>()
            .expect("drawing host resource should exist");
        let second_start = screen_point_for_canvas(&host.app, 3_100.0, 3_100.0);
        let second_end = screen_point_for_canvas(&host.app, 3_240.0, 3_180.0);
        draw_and_accept_stroke(
            &mut host.app,
            &mut publications,
            &mut snapshots,
            second_start,
            second_end,
        );
        host.app.ink_runtime().visible_product_count()
    };
    assert!(total_visible_count > first_visible_count);

    runtime = runtime
        .run_for_frames(1)
        .expect("second committed upload frame should run");
    let upload_count = ink_upload_count(&runtime);
    assert!(upload_count > 0);
    assert!(
        upload_count < total_visible_count,
        "adding another stroke should upload only new or changed products, not all existing ink"
    );
    runtime = runtime
        .run_for_frames(1)
        .expect("second stable committed frame should run");
    assert_eq!(ink_upload_count(&runtime), 0);
}

#[test]
fn committed_ink_cache_hit_stages_products_without_job_submission() {
    let mut app = RunenwerkDrawApp::new();
    let mut publications = ProductPublicationRuntimeResource::default();
    let mut snapshots = QuerySnapshotRuntimeResource::default();
    let mut executor = RuntimeJobExecutorResource::with_config(RuntimeJobExecutorConfig::serial());
    let mut cache = RuntimeProductCacheResource::default();
    let start = screen_point_for_canvas(&app, 512.0, 512.0);
    let end = screen_point_for_canvas(&app, 620.0, 560.0);

    draw_stroke(&mut app, start, end);
    let first_report = publish_drawing_ink_products_with_executor_and_cache(
        &mut app,
        &mut publications,
        &mut executor,
        &mut cache,
        &barrier(BarrierKind::ProductPublication),
    );
    assert!(first_report.published_count > 0);
    publish_drawing_ink_query_snapshots(
        &mut app,
        &mut snapshots,
        &barrier(BarrierKind::QuerySnapshotPublication),
    );
    assert!(app.ink_runtime().visible_product_count() > 0);
    let submitted_after_first = executor.diagnostics().submitted_count;
    let cached_entries = cache.snapshot().entry_count;
    assert_eq!(cached_entries, app.ink_runtime().visible_product_count());

    let cached_tiles = visible_tile_ids(&app);
    app.ink_runtime_mut().mark_dirty_tiles(cached_tiles.clone());
    let second_report = publish_drawing_ink_products_with_executor_and_cache(
        &mut app,
        &mut publications,
        &mut executor,
        &mut cache,
        &barrier(BarrierKind::ProductPublication),
    );

    assert!(second_report.published_count > 0);
    assert_eq!(
        executor.diagnostics().submitted_count,
        submitted_after_first,
        "cache hit should stage cached products without submitting another runtime job"
    );
    publish_drawing_ink_query_snapshots(
        &mut app,
        &mut snapshots,
        &barrier(BarrierKind::QuerySnapshotPublication),
    );
    assert!(cached_tiles.is_disjoint(app.ink_runtime().dirty_tiles()));
    assert!(
        cache
            .snapshot()
            .decisions
            .iter()
            .any(|decision| decision.kind == product::ProductCacheDecisionKind::Hit)
    );
    assert!(
        app.ink_runtime()
            .journal()
            .iter()
            .any(|entry| entry.summary.contains("committed ink cache hit"))
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
fn drawing_runtime_uses_worker_backed_jobs_by_default() {
    let app = build_headless_app();
    let executor = app
        .world()
        .resource::<RuntimeJobExecutorResource>()
        .expect("runtime job executor should exist");
    assert_eq!(
        executor.config(),
        &RuntimeJobExecutorConfig::worker_pool(2, 64),
        "Draw should use worker-backed jobs unless the app replaces the executor"
    );
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

fn stylus_pointer_event(
    kind: PointerEventKind,
    position: UiPoint,
    timestamp_micros: u64,
    pressure: f32,
) -> UiInputEvent {
    UiInputEvent::Pointer(
        PointerEvent::new(
            kind,
            position,
            UiVector::ZERO,
            Some(PointerButton::Primary),
            Modifiers::default(),
            if kind == PointerEventKind::Down { 1 } else { 0 },
        )
        .with_packet(
            PointerPacket::stylus(PointerDeviceId(901), PointerToolKind::Pen)
                .with_contact(PointerContactState::Contact)
                .with_timestamp_micros(timestamp_micros)
                .with_pressure(pressure)
                .with_latency_class(PointerLatencyClass::LowLatencyPreview),
        ),
    )
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

fn draw_and_accept_stroke(
    app: &mut RunenwerkDrawApp,
    publications: &mut ProductPublicationRuntimeResource,
    snapshots: &mut QuerySnapshotRuntimeResource,
    start: UiPoint,
    end: UiPoint,
) {
    draw_stroke(app, start, end);
    publish_visible_ink_until_clean(app, publications, snapshots);
}

fn publish_visible_ink_until_clean(
    app: &mut RunenwerkDrawApp,
    publications: &mut ProductPublicationRuntimeResource,
    snapshots: &mut QuerySnapshotRuntimeResource,
) {
    let mut batches = 0;
    while !app.ink_runtime().dirty_tiles().is_empty() {
        batches += 1;
        assert!(batches <= 8, "dirty tile batches should drain promptly");
        publish_drawing_ink_products(app, publications, &barrier(BarrierKind::ProductPublication));
        publish_drawing_ink_query_snapshots(
            app,
            snapshots,
            &barrier(BarrierKind::QuerySnapshotPublication),
        );
    }
}

fn ink_upload_count(app: &engine::App) -> usize {
    app.world()
        .resource::<RenderDynamicTextureUploadRegistryResource>()
        .expect("dynamic texture upload registry should exist")
        .snapshot()
        .len()
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

fn stroke_primitive_count(frame: &ui_render_data::UiFrame) -> usize {
    frame
        .surfaces
        .iter()
        .flat_map(|surface| surface.layers.iter())
        .flat_map(|layer| layer.primitives.iter())
        .filter(|primitive| matches!(primitive, UiPrimitive::Stroke(_)))
        .count()
}

fn visible_tile_ids(app: &RunenwerkDrawApp) -> BTreeSet<CanvasTileId> {
    app.ink_runtime()
        .visible_products()
        .map(|product| product.metadata.tile_id)
        .collect()
}
