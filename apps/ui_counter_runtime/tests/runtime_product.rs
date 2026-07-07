use engine::plugins::render::backend::RenderSurfaceId;
use engine::plugins::render::{FeatureContributionStatus, PreparedUiFrameResource};
use engine::plugins::ui::{
    UiActionDispatchReportsResource, UiMountRequestsResource, UiRuntimeDirtyCause,
    UiRuntimeEvaluationResource, UiRuntimeFramePublicationResource, UiRuntimePreparedFrameResource,
    UiRuntimeTraceEventKind, UiRuntimeTraceResource,
};
use engine::prelude::InputState;
use ui_counter_runtime::{
    COUNTER_SCREEN_ID, COUNTER_VALUE_STATE_KEY, Counter, CounterActionKind, CounterActionSource,
    CounterAgentScript, CounterRuntimeOptions, CounterRuntimeState, CounterVisibleUiResource,
    TRACE_STATUS_STATE_KEY, build_counter_app, trace_events_to_jsonl,
};
use ui_hosts::HostKind;
use ui_render_data::UiPrimitiveFamily;
use ui_schema::UiSchemaValue;
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

#[test]
fn counter_product_mounts_evaluates_and_publishes_ui_frame() {
    let app = build_counter_app(CounterRuntimeOptions::headless())
        .expect("counter app should build")
        .run_for_frames(1)
        .expect("headless frame should run");

    let mounts = app
        .world()
        .resource::<UiMountRequestsResource>()
        .expect("mount requests should be installed");
    assert_eq!(mounts.records()[0].screen_identity(), COUNTER_SCREEN_ID);
    assert_eq!(mounts.mounted_sessions().len(), 1);

    let runtime = app
        .world()
        .resource::<UiRuntimeEvaluationResource>()
        .expect("UI runtime evaluation resource should exist");
    let evaluation = runtime
        .latest_report()
        .expect("counter app should evaluate a UI source");
    assert_eq!(evaluation.source().screen_id(), COUNTER_SCREEN_ID);
    assert_eq!(
        evaluation.output().state_value(COUNTER_VALUE_STATE_KEY),
        Some(&UiSchemaValue::string("Count: 0"))
    );
    assert_eq!(
        evaluation.output().state_value(TRACE_STATUS_STATE_KEY),
        Some(&UiSchemaValue::string("Ready for counter input"))
    );
    assert!(evaluation.frame_payload().primitive_count() > 0);
    let dirty_causes = evaluation.dirty_causes().collect::<Vec<_>>();
    for required in [
        UiRuntimeDirtyCause::HostData,
        UiRuntimeDirtyCause::Surface,
        UiRuntimeDirtyCause::RenderPublication,
    ] {
        assert!(
            dirty_causes.contains(&required),
            "missing dirty cause {required:?}: {dirty_causes:?}"
        );
    }

    let publications = app
        .world()
        .resource::<UiRuntimeFramePublicationResource>()
        .expect("publication resource should exist");
    let publication = publications
        .latest_report()
        .expect("UI runtime publication should run");
    assert!(publication.is_published(), "{publication:?}");

    let visible_ui = app
        .world()
        .resource::<CounterVisibleUiResource>()
        .expect("counter visible UI proof should exist");
    assert_visible_labels(
        visible_ui,
        &[
            "Runenwerk Counter",
            "Runtime UI",
            "Count: 0",
            "Increment",
            "Decrement",
            "Reset",
            "Status: Ready",
            "Trace",
            "Trace empty",
        ],
    );
    for action in CounterActionKind::all() {
        let control = visible_ui
            .control_for_action(action)
            .expect("visible action control should be hit-testable");
        assert_eq!(control.route(), Some(action.route()));
        assert!(control.bounds().width > 1.0, "{control:?}");
        assert!(control.bounds().height > 1.0, "{control:?}");
    }
    let summary = visible_ui
        .frame_summary()
        .expect("visible proof should include a frame summary");
    assert!(summary.count_for_family(UiPrimitiveFamily::Rect) >= 3);
    assert!(summary.count_for_family(UiPrimitiveFamily::Border) >= 3);
    assert!(summary.count_for_family(UiPrimitiveFamily::GlyphRun) >= 3);
    assert_eq!(
        publication.primitive_count(),
        summary.primitive_count as usize
    );

    let prepared_runtime_frame = app
        .world()
        .resource::<UiRuntimePreparedFrameResource>()
        .expect("UiPlugin prepared frame resource should exist")
        .latest_record()
        .expect("counter app should prepare a runtime frame");
    assert_eq!(
        prepared_runtime_frame.frame_revision(),
        visible_ui
            .frame_revision()
            .expect("visible UI should track frame revision")
    );
    assert_eq!(
        prepared_runtime_frame.primitive_count(),
        publication.primitive_count()
    );
    assert!(
        prepared_runtime_frame
            .content_labels()
            .iter()
            .any(|label| label == "Count: 0")
    );
    assert!(
        prepared_runtime_frame
            .interactive_routes()
            .iter()
            .any(|route| route == "counter.increment")
    );

    let prepared = app
        .world()
        .resource::<PreparedUiFrameResource>()
        .expect("render plugin should prepare UI frame resource");
    assert_eq!(
        prepared.status_for_surface(RenderSurfaceId::primary()),
        FeatureContributionStatus::Ready
    );

    let trace = app
        .world()
        .resource::<UiRuntimeTraceResource>()
        .expect("trace resource should exist");
    assert_trace_contains(
        trace,
        &[
            UiRuntimeTraceEventKind::RuntimeEvaluation,
            UiRuntimeTraceEventKind::StateSnapshot,
            UiRuntimeTraceEventKind::Invalidation,
            UiRuntimeTraceEventKind::UiFramePublished,
        ],
    );
}

#[test]
fn scripted_agent_actions_dispatch_through_game_host_and_update_evaluation() {
    let script = CounterAgentScript::new([
        CounterActionKind::Increment,
        CounterActionKind::Increment,
        CounterActionKind::Decrement,
        CounterActionKind::Reset,
    ]);
    let app = build_counter_app(CounterRuntimeOptions::headless().with_agent_script(script))
        .expect("counter app should build")
        .run_for_frames(1)
        .expect("headless frame should run");

    let counter = app
        .world()
        .resource::<Counter>()
        .expect("counter resource should exist");
    assert_eq!(counter.value(), 0);

    let state = app
        .world()
        .resource::<CounterRuntimeState>()
        .expect("counter runtime state should exist");
    assert_eq!(state.action_history().len(), 4);
    assert!(
        state
            .action_history()
            .iter()
            .all(|record| record.source() == CounterActionSource::AgentScript)
    );
    assert_eq!(
        state.latest_action().map(|record| record.action()),
        Some(CounterActionKind::Reset)
    );

    let reports = app
        .world()
        .resource::<UiActionDispatchReportsResource>()
        .expect("dispatch reports should exist");
    assert_eq!(reports.len(), 4);
    assert!(reports.reports().iter().all(|report| report.is_accepted()));
    assert!(
        reports
            .reports()
            .iter()
            .all(|report| report.host() == HostKind::Game)
    );

    let runtime = app
        .world()
        .resource::<UiRuntimeEvaluationResource>()
        .expect("UI runtime evaluation resource should exist");
    let evaluation = runtime
        .latest_report()
        .expect("counter app should evaluate after scripted dispatch");
    assert_eq!(
        evaluation.output().state_value(COUNTER_VALUE_STATE_KEY),
        Some(&UiSchemaValue::string("Count: 0"))
    );
    assert!(
        evaluation
            .output()
            .state_value(TRACE_STATUS_STATE_KEY)
            .and_then(UiSchemaValue::as_str)
            .expect("status binding should be text")
            .contains("Reset")
    );

    let visible_ui = app
        .world()
        .resource::<CounterVisibleUiResource>()
        .expect("visible UI proof should exist");
    assert_visible_labels(
        visible_ui,
        &["Count: 0", "Status: Reset", "Trace", "Agent Reset 1>0"],
    );

    let trace = app
        .world()
        .resource::<UiRuntimeTraceResource>()
        .expect("trace resource should exist");
    assert_trace_contains(
        trace,
        &[
            UiRuntimeTraceEventKind::Input,
            UiRuntimeTraceEventKind::Route,
            UiRuntimeTraceEventKind::Capability,
            UiRuntimeTraceEventKind::Dispatch,
            UiRuntimeTraceEventKind::Mutation,
            UiRuntimeTraceEventKind::RuntimeEvaluation,
            UiRuntimeTraceEventKind::UiFramePublished,
        ],
    );
}

#[test]
fn keyboard_actions_use_same_dispatch_path_as_agent_actions() {
    let mut app =
        build_counter_app(CounterRuntimeOptions::headless()).expect("counter app should build");
    {
        let input = app
            .world_mut()
            .resource_mut::<InputState>()
            .expect("input state should be installed");
        input.handle_keyboard_input(KeyCode::ArrowUp, ElementState::Pressed, None);
    }

    let app = app
        .run_for_frames(1)
        .expect("headless frame should process input");
    let counter = app
        .world()
        .resource::<Counter>()
        .expect("counter resource should exist");
    assert_eq!(counter.value(), 1);

    let state = app
        .world()
        .resource::<CounterRuntimeState>()
        .expect("counter runtime state should exist");
    assert_eq!(state.action_history().len(), 1);
    assert_eq!(
        state.action_history()[0].source(),
        CounterActionSource::HumanKeyboard
    );
    assert_eq!(state.action_history()[0].route(), "counter.increment");

    let reports = app
        .world()
        .resource::<UiActionDispatchReportsResource>()
        .expect("dispatch reports should exist");
    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports
            .latest_report()
            .map(|report| report.route().as_str()),
        Some("counter.increment")
    );
    assert!(
        reports
            .latest_report()
            .is_some_and(|report| report.is_accepted())
    );
}

#[test]
fn visible_pointer_controls_dispatch_mutate_and_rerender_counter_ui() {
    let mut app = build_counter_app(CounterRuntimeOptions::headless())
        .expect("counter app should build")
        .run_for_frames(1)
        .expect("initial frame should publish visible controls");

    release_visible_action(&mut app, CounterActionKind::Increment);
    let mut app = app
        .run_for_frames(1)
        .expect("release-only frame should not dispatch");
    let counter = app
        .world()
        .resource::<Counter>()
        .expect("counter resource should exist");
    assert_eq!(counter.value(), 0);
    let reports = app
        .world()
        .resource::<UiActionDispatchReportsResource>()
        .expect("dispatch reports should exist");
    assert_eq!(reports.len(), 0);

    click_visible_action(&mut app, CounterActionKind::Increment);
    let mut app = app
        .run_for_frames(1)
        .expect("pointer frame should process increment");

    let counter = app
        .world()
        .resource::<Counter>()
        .expect("counter resource should exist");
    assert_eq!(counter.value(), 1);
    let state = app
        .world()
        .resource::<CounterRuntimeState>()
        .expect("counter runtime state should exist");
    assert_eq!(state.action_history().len(), 1);
    assert_eq!(
        state.action_history()[0].source(),
        CounterActionSource::HumanPointer
    );
    assert_eq!(state.action_history()[0].route(), "counter.increment");

    let reports = app
        .world()
        .resource::<UiActionDispatchReportsResource>()
        .expect("dispatch reports should exist");
    assert_eq!(reports.len(), 1);
    assert!(
        reports
            .latest_report()
            .is_some_and(|report| report.is_accepted() && report.host() == HostKind::Game)
    );

    let visible_ui = app
        .world()
        .resource::<CounterVisibleUiResource>()
        .expect("visible UI proof should exist");
    assert_visible_labels(
        visible_ui,
        &["Count: 1", "Status: +", "Trace", "Pointer + 0>1"],
    );
    let first_revision = visible_ui
        .frame_revision()
        .expect("visible UI should record first frame revision");

    click_visible_action(&mut app, CounterActionKind::Decrement);
    let mut app = app
        .run_for_frames(1)
        .expect("pointer frame should process decrement");
    let counter = app
        .world()
        .resource::<Counter>()
        .expect("counter resource should exist");
    assert_eq!(counter.value(), 0);
    assert_visible_labels(
        app.world()
            .resource::<CounterVisibleUiResource>()
            .expect("visible UI proof should exist"),
        &["Count: 0", "Status: -", "Pointer - 1>0"],
    );

    click_visible_action(&mut app, CounterActionKind::Reset);
    let app = app
        .run_for_frames(1)
        .expect("pointer frame should process reset");
    let counter = app
        .world()
        .resource::<Counter>()
        .expect("counter resource should exist");
    assert_eq!(counter.value(), 0);
    let visible_ui = app
        .world()
        .resource::<CounterVisibleUiResource>()
        .expect("visible UI proof should exist");
    assert_visible_labels(
        visible_ui,
        &["Count: 0", "Status: Reset", "Pointer Reset 0>0"],
    );
    assert!(
        visible_ui
            .frame_revision()
            .expect("visible UI revision should exist")
            > first_revision
    );

    let trace = app
        .world()
        .resource::<UiRuntimeTraceResource>()
        .expect("trace resource should exist");
    assert_trace_contains(
        trace,
        &[
            UiRuntimeTraceEventKind::Input,
            UiRuntimeTraceEventKind::Route,
            UiRuntimeTraceEventKind::Capability,
            UiRuntimeTraceEventKind::Dispatch,
            UiRuntimeTraceEventKind::Mutation,
            UiRuntimeTraceEventKind::RuntimeEvaluation,
            UiRuntimeTraceEventKind::UiFramePublished,
        ],
    );
}

#[test]
fn trace_jsonl_serializes_dispatch_mutation_evaluation_and_publication() {
    let script = CounterAgentScript::new([CounterActionKind::Increment, CounterActionKind::Reset]);
    let app = build_counter_app(CounterRuntimeOptions::headless().with_agent_script(script))
        .expect("counter app should build")
        .run_for_frames(1)
        .expect("headless frame should run");
    let trace = app
        .world()
        .resource::<UiRuntimeTraceResource>()
        .expect("trace resource should exist");

    let jsonl = trace_events_to_jsonl(trace).expect("trace should serialize as jsonl");
    assert!(jsonl.contains("\"kind\":\"dispatch\""), "{jsonl}");
    assert!(jsonl.contains("\"kind\":\"mutation\""), "{jsonl}");
    assert!(jsonl.contains("\"kind\":\"runtime_evaluation\""), "{jsonl}");
    assert!(jsonl.contains("\"kind\":\"ui_frame_published\""), "{jsonl}");

    for line in jsonl.lines() {
        serde_json::from_str::<serde_json::Value>(line).expect("trace line should be valid JSON");
    }
}

fn click_visible_action(app: &mut engine::prelude::App, action: CounterActionKind) {
    let (x, y) = {
        let visible_ui = app
            .world()
            .resource::<CounterVisibleUiResource>()
            .expect("visible UI proof should exist before pointer input");
        visible_ui
            .control_for_action(action)
            .expect("visible action control should exist")
            .center_point()
    };
    let input = app
        .world_mut()
        .resource_mut::<InputState>()
        .expect("input state should be installed");
    input.handle_cursor_moved(x, y);
    input.handle_mouse_input(ElementState::Pressed, MouseButton::Left);
    input.handle_mouse_input(ElementState::Released, MouseButton::Left);
}

fn release_visible_action(app: &mut engine::prelude::App, action: CounterActionKind) {
    let (x, y) = {
        let visible_ui = app
            .world()
            .resource::<CounterVisibleUiResource>()
            .expect("visible UI proof should exist before pointer input");
        visible_ui
            .control_for_action(action)
            .expect("visible action control should exist")
            .center_point()
    };
    let input = app
        .world_mut()
        .resource_mut::<InputState>()
        .expect("input state should be installed");
    input.handle_cursor_moved(x, y);
    input.handle_mouse_input(ElementState::Released, MouseButton::Left);
}

fn assert_visible_labels(visible_ui: &CounterVisibleUiResource, expected: &[&str]) {
    for label in expected {
        assert!(
            visible_ui.labels().iter().any(|actual| actual == label),
            "visible label {label:?} missing from {:?}",
            visible_ui.labels()
        );
    }
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
