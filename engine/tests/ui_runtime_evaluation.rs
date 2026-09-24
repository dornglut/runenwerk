use std::collections::BTreeMap;

use engine::plugins::ui::{
    IntoUi, UiMountRequestsResource, UiPlugin, UiRuntimeDirtyCause, UiRuntimeEvaluationInput,
    UiRuntimeEvaluationResource, UiRuntimeTraceEventKind, UiRuntimeTraceResource, UiScreen,
    UiTypedScreenId, UiTypedSource,
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
fn ui_runtime_evaluation_uses_source_program_evaluator_view_and_frame_payload() {
    let input = counter_evaluation_input();
    let mounted_session = mounted_counter_session();
    let mut runtime = UiRuntimeEvaluationResource::default();
    let mut trace = UiRuntimeTraceResource::default();
    let mut diagnostics = Default::default();

    let report = runtime.evaluate(
        &input,
        Some(&mounted_session),
        counter_context("Clicked 1 / 5", 1),
        &mut trace,
        &mut diagnostics,
    );

    assert_eq!(report.source().screen_id(), "counter.screen");
    assert_eq!(report.source().source_id(), "counter.screen.source");
    assert_eq!(report.source().program_id(), "counter.screen.program");
    assert!(report.source().source_count() > 0);
    assert!(report.source().source_map_count() > 0);
    assert!(report.source().control_count() > 0);
    assert_eq!(report.source().binding_count(), 1);
    assert_eq!(report.source().state_requirement_count(), 1);

    assert!(report.runtime_view().passed());
    assert_eq!(
        report.output().state_value(COUNTER_TEXT_KEY),
        Some(&UiSchemaValue::string("Clicked 1 / 5"))
    );
    assert_eq!(report.output().dirty_binding_count(), 1);
    assert_eq!(
        report.frame_payload().text_layout_request_count(),
        report.output().text_layout_request_count()
    );
    assert_eq!(
        report.frame_payload().visual_operator_count(),
        report.output().visual_operator_count()
    );
    assert!(report.frame_payload().primitive_count() > 0);
    assert!(diagnostics.is_empty());

    assert_trace_contains(
        &trace,
        &[
            UiRuntimeTraceEventKind::RuntimeEvaluation,
            UiRuntimeTraceEventKind::StateSnapshot,
            UiRuntimeTraceEventKind::Invalidation,
        ],
    );
}

#[test]
fn ui_runtime_evaluation_snapshot_replay_dirty_causes_and_host_text_change_are_stable() {
    let input = counter_evaluation_input();
    let mounted_session = mounted_counter_session();
    let mut runtime = UiRuntimeEvaluationResource::default();
    let mut trace = UiRuntimeTraceResource::default();
    let mut diagnostics = Default::default();

    let first = runtime.evaluate(
        &input,
        Some(&mounted_session),
        counter_context("Clicked 0 / 5", 1),
        &mut trace,
        &mut diagnostics,
    );
    let second = runtime.evaluate(
        &input,
        Some(&mounted_session),
        counter_context("Clicked 1 / 5", 2),
        &mut trace,
        &mut diagnostics,
    );

    assert_eq!(
        first.output().state_value(COUNTER_TEXT_KEY),
        Some(&UiSchemaValue::string("Clicked 0 / 5"))
    );
    assert_eq!(
        second.output().state_value(COUNTER_TEXT_KEY),
        Some(&UiSchemaValue::string("Clicked 1 / 5"))
    );
    assert_ne!(
        first.output().state_value(COUNTER_TEXT_KEY),
        second.output().state_value(COUNTER_TEXT_KEY)
    );

    assert_eq!(
        runtime.replay_snapshot(second.runtime_id(), second.source().source_id()),
        Some(second.snapshot())
    );
    assert_eq!(second.snapshot().source_id(), "counter.screen.source");
    assert_eq!(second.snapshot().program_id(), "counter.screen.program");
    assert!(second.snapshot().surface_instance_id().is_some());
    assert!(second.snapshot().session_scope_id().is_some());

    let causes = second.dirty_causes().collect::<Vec<_>>();
    for required in [
        UiRuntimeDirtyCause::Source,
        UiRuntimeDirtyCause::HostData,
        UiRuntimeDirtyCause::Session,
        UiRuntimeDirtyCause::Layout,
        UiRuntimeDirtyCause::Text,
        UiRuntimeDirtyCause::Theme,
        UiRuntimeDirtyCause::Primitive,
        UiRuntimeDirtyCause::Surface,
        UiRuntimeDirtyCause::RenderPublication,
    ] {
        assert!(
            causes.contains(&required),
            "missing dirty cause {required:?}: {causes:?}"
        );
    }
    assert!(diagnostics.is_empty());
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

fn assert_trace_contains(trace: &UiRuntimeTraceResource, kinds: &[UiRuntimeTraceEventKind]) {
    for kind in kinds {
        assert!(
            trace.events().iter().any(|event| event.kind() == *kind),
            "trace missing {kind:?}: {:?}",
            trace.events()
        );
    }
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
