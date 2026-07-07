use engine::plugins::render::backend::RenderSurfaceId;
use engine::plugins::render::{
    FeatureContributionStatus, PreparedUiFrameResource, RenderFrameProducerId, RenderPlugin,
    SurfaceFrameRoute, SurfaceFrameSubmission, SurfaceFrameSubmissionOrder,
    SurfaceFrameSubmissionRegistryResource,
};
use engine::plugins::{DebugMetricsPlugin, ScenePlugin};
use engine::prelude::{App, InputState, ResMut, Update};
use winit::event::ElementState;
use winit::keyboard::KeyCode;

const SCENE_OVERLAY_FRAME_PRODUCER_ID: RenderFrameProducerId = render_frame_producer_id(1);
const DEBUG_METRICS_FRAME_PRODUCER_ID: RenderFrameProducerId = render_frame_producer_id(2);

const fn render_frame_producer_id(raw: u64) -> RenderFrameProducerId {
    match RenderFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("render frame producer id constants must be non-zero"),
    }
}

#[test]
fn runtime_ui_producer_migration_scene_owner_publishes_scene_frame_submission() {
    let mut app = App::headless();
    app.add_plugin(RenderPlugin);
    app.add_scene("engine/tests/fixtures/scene_templates/main_menu.ron");
    app.add_plugin(ScenePlugin);

    let app = app
        .run_for_frames(1)
        .expect("scene producer migration frame should run");

    let registry = app
        .world()
        .resource::<SurfaceFrameSubmissionRegistryResource>()
        .expect("surface frame registry should exist");
    let submission = registry
        .get(&SCENE_OVERLAY_FRAME_PRODUCER_ID)
        .expect("scene owner should publish producer 1");
    assert_eq!(submission.producer_id, SCENE_OVERLAY_FRAME_PRODUCER_ID);
    assert_eq!(submission.render_surface_id, None);
    assert_eq!(submission.route, SurfaceFrameRoute::Screen);
    assert_eq!(submission.order, SurfaceFrameSubmissionOrder::new(0, 0));
    assert!(
        submission.primitive_count_hint() > 0,
        "scene overlay submission should carry the built overlay frame"
    );
    assert_eq!(submission.rect_shader_asset_id, None);

    let prepared = app
        .world()
        .resource::<PreparedUiFrameResource>()
        .expect("RenderPlugin should prepare UI frame resource");
    assert_eq!(prepared.status, FeatureContributionStatus::Ready);
    let prepared_submission = prepared
        .payload
        .submissions
        .iter()
        .find(|submission| submission.producer_id == SCENE_OVERLAY_FRAME_PRODUCER_ID)
        .expect("prepared payload should include scene producer");
    assert_eq!(prepared_submission.route, "screen");
    assert_eq!(prepared_submission.layer, 0);
    assert_eq!(prepared_submission.priority, 0);
    assert_eq!(
        prepared_submission.primitive_count_hint(),
        submission.primitive_count_hint()
    );
}

#[test]
fn runtime_ui_producer_migration_debug_owner_publishes_debug_frame_submission() {
    let mut app = App::headless();
    app.add_plugin(RenderPlugin);
    app.add_plugin(DebugMetricsPlugin);
    app.add_systems(Update, inject_f10);

    let app = app
        .run_for_frames(1)
        .expect("debug producer migration frame should run");

    let registry = app
        .world()
        .resource::<SurfaceFrameSubmissionRegistryResource>()
        .expect("surface frame registry should exist");
    let submission = registry
        .get(&DEBUG_METRICS_FRAME_PRODUCER_ID)
        .expect("debug metrics owner should publish producer 2");
    assert_eq!(submission.producer_id, DEBUG_METRICS_FRAME_PRODUCER_ID);
    assert_eq!(submission.render_surface_id, None);
    assert_eq!(submission.route, SurfaceFrameRoute::Screen);
    assert_eq!(submission.order, SurfaceFrameSubmissionOrder::new(100, 0));
    assert!(
        submission.primitive_count_hint() > 0,
        "debug metrics submission should carry the diagnostics frame"
    );

    let prepared = app
        .world()
        .resource::<PreparedUiFrameResource>()
        .expect("RenderPlugin should prepare UI frame resource");
    assert_eq!(prepared.status, FeatureContributionStatus::Ready);
    let prepared_submission = prepared
        .payload
        .submissions
        .iter()
        .find(|submission| submission.producer_id == DEBUG_METRICS_FRAME_PRODUCER_ID)
        .expect("prepared payload should include debug metrics producer");
    assert_eq!(prepared_submission.route, "screen");
    assert_eq!(prepared_submission.layer, 100);
    assert_eq!(prepared_submission.priority, 0);
    assert_eq!(
        prepared_submission.primitive_count_hint(),
        submission.primitive_count_hint()
    );
}

#[test]
fn runtime_ui_producer_migration_render_plugin_does_not_collect_scene_or_debug_producers() {
    let mut app = App::headless();
    app.add_plugin(RenderPlugin);
    {
        let registry = app
            .world_mut()
            .resource_mut::<SurfaceFrameSubmissionRegistryResource>()
            .expect("RenderPlugin should initialize the generic registry");
        registry.replace(
            SurfaceFrameSubmission::new(SCENE_OVERLAY_FRAME_PRODUCER_ID)
                .with_route(SurfaceFrameRoute::ViewportOverlay)
                .with_order(SurfaceFrameSubmissionOrder::new(77, 3)),
        );
        registry.replace(
            SurfaceFrameSubmission::new(DEBUG_METRICS_FRAME_PRODUCER_ID)
                .with_route(SurfaceFrameRoute::WorldProjected)
                .with_order(SurfaceFrameSubmissionOrder::new(88, 4)),
        );
    }

    let app = app
        .run_for_frames(1)
        .expect("RenderPlugin-only frame should run");
    let registry = app
        .world()
        .resource::<SurfaceFrameSubmissionRegistryResource>()
        .expect("surface frame registry should exist");
    let scene_submission = registry
        .get(&SCENE_OVERLAY_FRAME_PRODUCER_ID)
        .expect("generic scene producer submission should be untouched by RenderPlugin");
    assert_eq!(scene_submission.route, SurfaceFrameRoute::ViewportOverlay);
    assert_eq!(
        scene_submission.order,
        SurfaceFrameSubmissionOrder::new(77, 3)
    );
    let debug_submission = registry
        .get(&DEBUG_METRICS_FRAME_PRODUCER_ID)
        .expect("generic debug producer submission should be untouched by RenderPlugin");
    assert_eq!(debug_submission.route, SurfaceFrameRoute::WorldProjected);
    assert_eq!(
        debug_submission.order,
        SurfaceFrameSubmissionOrder::new(88, 4)
    );
}

fn inject_f10(mut input: ResMut<InputState>) {
    input.handle_keyboard_input(KeyCode::F10, ElementState::Pressed, None);
}

#[test]
fn runtime_ui_producer_migration_preserves_runtime_producer_order_for_primary_surface() {
    let mut app = App::headless();
    app.add_plugin(RenderPlugin);
    app.add_scene("engine/tests/fixtures/scene_templates/main_menu.ron");
    app.add_plugin(ScenePlugin);
    app.add_plugin(DebugMetricsPlugin);
    app.add_systems(Update, inject_f10);

    let app = app
        .run_for_frames(1)
        .expect("combined producer migration frame should run");
    let prepared = app
        .world()
        .resource::<PreparedUiFrameResource>()
        .expect("RenderPlugin should prepare UI frame resource");
    let producer_order = prepared
        .payload_for_surface(RenderSurfaceId::primary())
        .submissions
        .iter()
        .map(|submission| submission.producer_id)
        .collect::<Vec<_>>();

    assert_eq!(
        producer_order,
        vec![
            SCENE_OVERLAY_FRAME_PRODUCER_ID,
            DEBUG_METRICS_FRAME_PRODUCER_ID,
        ]
    );
    assert_eq!(
        prepared.status_for_surface(RenderSurfaceId::primary()),
        FeatureContributionStatus::Ready
    );
}
