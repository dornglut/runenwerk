use std::collections::BTreeMap;

use engine::plugins::render::backend::RenderSurfaceId;
use engine::plugins::render::{
    FeatureContributionStatus, PreparedUiFrameResource, RenderPlugin,
    SurfaceFrameSubmissionRegistryResource,
};
use engine::plugins::ui::{
    IntoUi, UI_RUNTIME_FRAME_PRODUCER_ID, UiMountRequestsResource, UiPlugin,
    UiRuntimeDiagnosticCode, UiRuntimeDiagnosticsResource, UiRuntimeEvaluationInput,
    UiRuntimeEvaluationResource, UiRuntimeFramePublicationFailureReason,
    UiRuntimeFramePublicationResource, UiRuntimeFramePublicationStatus,
    UiRuntimeFramePublicationTarget, UiRuntimeTraceEventKind, UiRuntimeTraceResource, UiScreen,
    UiTypedScreenId, UiTypedSource, publish_latest_ui_runtime_frame,
};
use engine::prelude::{App, AppUiExt};
use ui_binding::HostDataSnapshot;
use ui_controls::{BUTTON_CONTROL_KIND_ID, ControlPackageRegistry, runenwerk_control_package};
use ui_definition::{
    AuthoredBindingRef, AuthoredControlAccessibilityDefinition, AuthoredControlKindId,
    AuthoredControlValue, AuthoredId, AuthoredRouteId, UiNodeDefinition, UiValueBinding,
};
use ui_evaluator::UiEvaluationContext;
use ui_program::UiProgramSourceId;
use ui_schema::UiSchemaValue;

const COUNTER_TEXT_KEY: &str = "state.counter.output.selected";

#[test]
fn ui_render_publication_writes_surface_frame_submission_and_trace() {
    let runtime = evaluated_counter_runtime("Clicked 2 / 5", 2);
    let mut submissions = SurfaceFrameSubmissionRegistryResource::default();
    let mut publications = UiRuntimeFramePublicationResource::default();
    let mut trace = UiRuntimeTraceResource::default();
    let mut diagnostics = UiRuntimeDiagnosticsResource::default();
    let target = UiRuntimeFramePublicationTarget::default();
    let expected_payload = runtime
        .latest_report()
        .expect("evaluation should produce report")
        .frame_payload()
        .clone();

    let report = publish_latest_ui_runtime_frame(
        &runtime,
        &target,
        &mut submissions,
        &mut publications,
        &mut trace,
        &mut diagnostics,
    );

    assert!(report.is_published());
    assert_eq!(report.status(), UiRuntimeFramePublicationStatus::Published);
    assert_eq!(report.producer_id(), UI_RUNTIME_FRAME_PRODUCER_ID);
    assert_eq!(report.render_surface_id(), RenderSurfaceId::primary());
    assert_eq!(
        report.frame_revision(),
        Some(expected_payload.frame_revision())
    );
    assert_eq!(report.primitive_count(), expected_payload.primitive_count());
    assert!(diagnostics.is_empty());
    assert_eq!(publications.latest_report(), Some(&report));

    let submission = submissions
        .get_for_surface(&UI_RUNTIME_FRAME_PRODUCER_ID, RenderSurfaceId::primary())
        .expect("UiPlugin publication should write a surface-scoped submission");
    assert_eq!(submission.producer_id, UI_RUNTIME_FRAME_PRODUCER_ID);
    assert_eq!(
        submission.render_surface_id,
        Some(RenderSurfaceId::primary())
    );
    assert_eq!(
        submission.primitive_count_hint(),
        expected_payload.primitive_count()
    );

    assert_frame_trace_contains(&trace, UiRuntimeTraceEventKind::UiFramePublished, &report);
    assert_frame_trace_contains(&trace, UiRuntimeTraceEventKind::UiFramePresented, &report);

    let output = runtime
        .latest_report()
        .expect("evaluation should produce report")
        .output();
    assert_eq!(
        output.state_value(COUNTER_TEXT_KEY),
        Some(&UiSchemaValue::string("Clicked 2 / 5"))
    );
}

#[test]
fn ui_render_publication_missing_evaluation_records_report_and_diagnostic() {
    let previous_runtime = evaluated_counter_runtime("Clicked 1 / 5", 1);
    let runtime = UiRuntimeEvaluationResource::default();
    let target = UiRuntimeFramePublicationTarget::default();
    let mut submissions = SurfaceFrameSubmissionRegistryResource::default();
    let mut publications = UiRuntimeFramePublicationResource::default();
    let mut trace = UiRuntimeTraceResource::default();
    let mut diagnostics = UiRuntimeDiagnosticsResource::default();
    let previous_report = publish_latest_ui_runtime_frame(
        &previous_runtime,
        &target,
        &mut submissions,
        &mut publications,
        &mut trace,
        &mut diagnostics,
    );
    assert!(previous_report.is_published());
    assert!(
        submissions
            .get_for_surface(&UI_RUNTIME_FRAME_PRODUCER_ID, RenderSurfaceId::primary())
            .is_some()
    );

    let report = publish_latest_ui_runtime_frame(
        &runtime,
        &target,
        &mut submissions,
        &mut publications,
        &mut trace,
        &mut diagnostics,
    );

    assert!(!report.is_published());
    assert_eq!(
        report.status(),
        UiRuntimeFramePublicationStatus::MissingRuntimeEvaluation
    );
    assert!(submissions.is_empty());
    assert!(
        submissions
            .get_for_surface(&UI_RUNTIME_FRAME_PRODUCER_ID, RenderSurfaceId::primary())
            .is_none()
    );
    assert_eq!(publications.latest_report(), Some(&report));
    assert_eq!(diagnostics.len(), 1);

    let diagnostic = &diagnostics.entries()[0];
    assert_eq!(
        diagnostic.code,
        UiRuntimeDiagnosticCode::FramePublicationRejected
    );
    let frame_publication = diagnostic
        .frame_publication
        .as_ref()
        .expect("missing frame should record publication diagnostic facts");
    assert_eq!(frame_publication.producer_id, UI_RUNTIME_FRAME_PRODUCER_ID);
    assert_eq!(
        frame_publication.render_surface_id,
        RenderSurfaceId::primary()
    );
    assert_eq!(
        frame_publication.failure_reason,
        UiRuntimeFramePublicationFailureReason::MissingRuntimeEvaluation
    );
    assert_frame_trace_contains(&trace, UiRuntimeTraceEventKind::UiFramePublished, &report);
}

#[test]
fn ui_render_publication_prepares_payload_when_plugins_run_render_prepare() {
    let mut app = App::headless();
    app.add_plugin(RenderPlugin);
    app.add_plugin(UiPlugin);
    app.insert_resource(evaluated_counter_runtime("Clicked 3 / 5", 3));

    let app = app
        .run_for_frames(1)
        .expect("headless frame should run render prepare systems");

    let publications = app
        .world()
        .resource::<UiRuntimeFramePublicationResource>()
        .expect("publication resource should exist");
    let report = publications
        .latest_report()
        .expect("publication system should record report");
    assert!(report.is_published(), "{report:?}");

    let prepared = app
        .world()
        .resource::<PreparedUiFrameResource>()
        .expect("RenderPlugin should prepare UI frame resource");
    assert_eq!(
        prepared.status_for_surface(RenderSurfaceId::primary()),
        FeatureContributionStatus::Ready
    );
    let prepared_submission = prepared
        .payload_for_surface(RenderSurfaceId::primary())
        .submissions
        .iter()
        .find(|submission| submission.producer_id == UI_RUNTIME_FRAME_PRODUCER_ID)
        .expect("prepared UI payload should include UiPlugin publication producer");
    assert_eq!(
        prepared_submission.producer_id,
        UI_RUNTIME_FRAME_PRODUCER_ID
    );
    assert_eq!(
        prepared_submission.primitive_count_hint(),
        report.primitive_count()
    );
}

#[test]
fn ui_render_publication_can_feed_prepare_resource_directly() {
    let runtime = evaluated_counter_runtime("Clicked 4 / 5", 4);
    let mut submissions = SurfaceFrameSubmissionRegistryResource::default();
    let mut publications = UiRuntimeFramePublicationResource::default();
    let mut trace = UiRuntimeTraceResource::default();
    let mut diagnostics = UiRuntimeDiagnosticsResource::default();
    let target = UiRuntimeFramePublicationTarget::default();

    let report = publish_latest_ui_runtime_frame(
        &runtime,
        &target,
        &mut submissions,
        &mut publications,
        &mut trace,
        &mut diagnostics,
    );
    assert!(report.is_published());

    let mut app = App::headless();
    app.add_plugin(RenderPlugin);
    app.insert_resource(submissions);
    let app = app
        .run_for_frames(1)
        .expect("direct prepared resource path should run");
    let prepared = app
        .world()
        .resource::<PreparedUiFrameResource>()
        .expect("prepared UI frame resource should exist");
    let prepared_submission = prepared
        .payload_for_surface(RenderSurfaceId::primary())
        .submissions
        .iter()
        .find(|submission| submission.producer_id == UI_RUNTIME_FRAME_PRODUCER_ID)
        .expect("prepare should consume UiPlugin surface-frame submission");
    assert_eq!(
        prepared_submission.primitive_count_hint(),
        report.primitive_count()
    );
}

fn evaluated_counter_runtime(text: &str, revision: u64) -> UiRuntimeEvaluationResource {
    let input = counter_evaluation_input();
    let mounted_session = mounted_counter_session();
    let mut runtime = UiRuntimeEvaluationResource::default();
    let mut trace = UiRuntimeTraceResource::default();
    let mut diagnostics = UiRuntimeDiagnosticsResource::default();

    let report = runtime.evaluate(
        &input,
        Some(&mounted_session),
        counter_context(text, revision),
        &mut trace,
        &mut diagnostics,
    );
    assert!(report.frame_payload().primitive_count() > 0);
    assert!(diagnostics.is_empty(), "{:?}", diagnostics.entries());
    runtime
}

fn counter_evaluation_input() -> UiRuntimeEvaluationInput {
    let registry = ControlPackageRegistry::new()
        .with_package(runenwerk_control_package())
        .expect("runenwerk controls package should register");
    let source = CounterScreen.into_ui_source();
    let lowering = source.lower_with_registry_snapshot(&registry.snapshot());

    assert!(lowering.passed(), "{:?}", lowering.formation().diagnostics);
    UiRuntimeEvaluationInput::from_lowering_report(&lowering)
}

fn mounted_counter_session() -> engine::plugins::ui::UiMountedSessionRecord {
    let mut app = App::headless();
    app.add_plugin(UiPlugin);
    app.ui().mount("CounterScreen");

    app.world()
        .resource::<UiMountRequestsResource>()
        .expect("UI mount requests should exist")
        .mounted_sessions()[0]
        .clone()
}

fn counter_context(text: &str, revision: u64) -> UiEvaluationContext {
    UiEvaluationContext::default().with_host_data(HostDataSnapshot::new(
        "counter.output.text",
        UiSchemaValue::string(text),
        revision,
    ))
}

fn assert_frame_trace_contains(
    trace: &UiRuntimeTraceResource,
    kind: UiRuntimeTraceEventKind,
    report: &engine::plugins::ui::UiRuntimeFramePublicationReport,
) {
    assert!(
        trace.events().iter().any(|event| {
            event.kind() == kind
                && event.render_producer_id() == Some(report.producer_id())
                && event.render_surface_id() == Some(report.render_surface_id())
                && event.frame_revision() == report.frame_revision()
                && event.frame_publication_status() == Some(report.status())
        }),
        "trace missing {kind:?} for report {report:?}: {:?}",
        trace.events()
    );
}

#[derive(Debug, Copy, Clone)]
struct CounterScreen;

impl UiScreen for CounterScreen {
    fn screen_id(&self) -> UiTypedScreenId {
        UiTypedScreenId::new("counter.screen")
    }

    fn build_source(&self) -> UiTypedSource {
        UiTypedSource::new(
            self.screen_id(),
            UiProgramSourceId::new("counter.screen.source"),
            UiNodeDefinition::Column {
                id: AuthoredId::new("counter.root"),
                children: vec![
                    UiNodeDefinition::Label {
                        id: AuthoredId::new("counter.title"),
                        label: UiValueBinding::static_text("Counter"),
                        availability: None,
                    },
                    counter_output_control(),
                ],
            },
        )
    }
}

fn counter_output_control() -> UiNodeDefinition {
    let mut properties = BTreeMap::new();
    properties.insert(
        "label".to_owned(),
        AuthoredControlValue::String("Counter output".to_owned()),
    );

    let mut bindings = BTreeMap::new();
    bindings.insert(
        "selected".to_owned(),
        AuthoredBindingRef::new("counter.output.text"),
    );

    UiNodeDefinition::Control {
        id: AuthoredId::new("counter.output"),
        kind: AuthoredControlKindId::new(BUTTON_CONTROL_KIND_ID),
        properties,
        bindings,
        route: Some(AuthoredRouteId::new("counter.increment")),
        accessibility: Some(AuthoredControlAccessibilityDefinition {
            role: "button".to_owned(),
            label: Some("Counter output".to_owned()),
        }),
        children: Vec::new(),
    }
}
