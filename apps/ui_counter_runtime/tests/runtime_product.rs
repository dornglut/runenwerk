use engine::plugins::ui::{
    UiActionDispatchReportsResource, UiRuntimeHitTargetResource, UiRuntimePreparedFrameResource,
    UiRuntimeTraceEventKind, UiRuntimeTraceResource,
};
use ui_counter_runtime::{
    Counter, CounterActionKind, CounterAgentScript, CounterRuntimeOptions, CounterRuntimeStatus,
    WINDOW_TITLE, build_counter_app, trace_events_to_jsonl,
};

#[test]
fn counter_app_prepares_visible_runtime_frame() {
    let app = build_counter_app(CounterRuntimeOptions::headless())
        .unwrap()
        .run_for_frames(1)
        .unwrap();

    let prepared = app
        .world()
        .resource::<UiRuntimePreparedFrameResource>()
        .unwrap()
        .latest_record()
        .unwrap();

    for expected in [
        WINDOW_TITLE,
        "Count: 0",
        "Increment",
        "Decrement",
        "Reset",
        "Ready for counter input",
        "Trace",
    ] {
        assert!(
            prepared
                .content_labels()
                .iter()
                .any(|label| label == expected),
            "missing visible label {expected:?}: {:?}",
            prepared.content_labels()
        );
    }

    for route in ["counter.increment", "counter.decrement", "counter.reset"] {
        assert!(
            prepared
                .interactive_routes()
                .iter()
                .any(|candidate| candidate == route),
            "missing interactive route {route:?}: {:?}",
            prepared.interactive_routes()
        );
    }

    let targets = app
        .world()
        .resource::<UiRuntimeHitTargetResource>()
        .unwrap();
    assert_eq!(targets.targets().len(), 3);
    for action in CounterActionKind::all() {
        let target = targets
            .targets()
            .iter()
            .find(|target| target.route() == Some(action.route()))
            .expect("counter action should have stable hit target");
        assert_eq!(target.label(), action.label());
        assert!(target.enabled());
        assert!(target.bounds().width > 1.0);
        assert!(target.bounds().height > 1.0);
    }
}

#[test]
fn scripted_agent_actions_use_generic_dispatch_trace_and_rerender() {
    let script = CounterAgentScript::new([
        CounterActionKind::Increment,
        CounterActionKind::Increment,
        CounterActionKind::Decrement,
        CounterActionKind::Reset,
    ]);
    let app = build_counter_app(CounterRuntimeOptions::headless().with_agent_script(script))
        .unwrap()
        .run_for_frames(1)
        .unwrap();

    assert_eq!(app.world().resource::<Counter>().unwrap().value(), 0);
    assert!(
        app.world()
            .resource::<CounterRuntimeStatus>()
            .unwrap()
            .line()
            .contains("Reset")
    );

    let reports = app
        .world()
        .resource::<UiActionDispatchReportsResource>()
        .unwrap();
    assert_eq!(reports.len(), 4);
    assert!(reports.reports().iter().all(|report| report.is_accepted()));

    let trace = app.world().resource::<UiRuntimeTraceResource>().unwrap();
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

    let prepared = app
        .world()
        .resource::<UiRuntimePreparedFrameResource>()
        .unwrap()
        .latest_record()
        .unwrap();
    assert!(
        prepared
            .content_labels()
            .iter()
            .any(|label| label == "Count: 0")
    );
}

#[test]
fn trace_jsonl_serializes_generic_runtime_trace_families() {
    let script = CounterAgentScript::new([CounterActionKind::Increment, CounterActionKind::Reset]);
    let app = build_counter_app(CounterRuntimeOptions::headless().with_agent_script(script))
        .unwrap()
        .run_for_frames(1)
        .unwrap();

    let jsonl = trace_events_to_jsonl(app.world().resource::<UiRuntimeTraceResource>().unwrap())
        .expect("trace should serialize");
    assert!(jsonl.contains("\"kind\":\"dispatch\""), "{jsonl}");
    assert!(jsonl.contains("\"kind\":\"mutation\""), "{jsonl}");
    assert!(jsonl.contains("\"kind\":\"runtime_evaluation\""), "{jsonl}");
    assert!(jsonl.contains("\"kind\":\"ui_frame_published\""), "{jsonl}");
    for line in jsonl.lines() {
        serde_json::from_str::<serde_json::Value>(line).unwrap();
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
