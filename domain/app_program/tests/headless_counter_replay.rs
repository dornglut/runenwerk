use app_program::{
    AppActionId, AppActionPayload, AppActionPayloadValue, AppActionVersion, AppDiagnostic,
    AppModelRevision, AppModelValue, AppReducerInput, AppReplayScenario, COUNTER_CAPABILITY,
    COUNTER_INCREMENT_ACTION_ID, COUNTER_INCREMENT_ROUTE, COUNTER_RESET_ROUTE,
    COUNTER_ROUTE_SCHEMA_VERSION, COUNTER_WIN_THRESHOLD, CounterAction, CounterScreen,
    RouteActionMapping, RouteActionResolutionStatus, counter_action_for_test, counter_capability,
    counter_increment_event, counter_initial_snapshot, counter_positive_scenario,
    counter_program_id, counter_projection, counter_reducer, counter_route_action_map,
};
use app_program::{
    AppActionPayloadShape, NAMESPACE_ACTION_SCHEMA, NAMESPACE_EFFECT_PLAN,
    NAMESPACE_HOST_COMPATIBILITY, NAMESPACE_MODEL_SCHEMA, NAMESPACE_PROJECTION, NAMESPACE_REDUCER,
    NAMESPACE_REPLAY, NAMESPACE_REPORT_BUDGET, NAMESPACE_ROUTE_ACTION_RESOLVE,
    NAMESPACE_VERSION_COMPATIBILITY,
};
use ui_program::{
    RouteCapability, RouteId, RouteSchemaVersion, UiEventPacket, UiEventSourceControlId,
};
use ui_schema::{UiSchemaRef, UiSchemaValue};

#[test]
fn counter_replay_reaches_win_resets_and_reports_deterministically() {
    let route_map = counter_route_action_map();
    let scenario = counter_positive_scenario();
    let trace = scenario.run(&route_map, counter_reducer, counter_projection);
    let report = trace.to_report();
    let repeated_report = counter_positive_scenario()
        .run(
            &counter_route_action_map(),
            counter_reducer,
            counter_projection,
        )
        .to_report();

    assert!(trace.passed());
    assert_eq!(trace.steps.len(), (COUNTER_WIN_THRESHOLD as usize) + 1);
    assert_eq!(trace.initial_model.integer("counter.count"), Some(0));
    assert_eq!(
        trace.steps[0].route_resolution.status,
        RouteActionResolutionStatus::Accepted
    );
    assert_eq!(
        trace.steps[0]
            .route_resolution
            .action
            .as_ref()
            .map(|action| action.action_id.as_str()),
        Some(COUNTER_INCREMENT_ACTION_ID)
    );
    assert_eq!(
        trace.steps[0]
            .reducer_outcome
            .as_ref()
            .and_then(|outcome| outcome.before_model.integer("counter.count")),
        Some(0)
    );
    assert_eq!(
        trace.steps[0]
            .reducer_outcome
            .as_ref()
            .and_then(|outcome| outcome.after_model.integer("counter.count")),
        Some(1)
    );
    assert_eq!(trace.steps[4].model_after.integer("counter.count"), Some(5));
    assert_eq!(
        trace.steps[4]
            .projection_after
            .projection
            .as_ref()
            .map(|projection| projection.screen_id.as_str()),
        Some(CounterScreen::Win.screen_id())
    );
    let reset_step = trace.steps.last().expect("reset step must exist");
    assert_eq!(
        reset_step
            .route_resolution
            .action
            .as_ref()
            .map(|action| CounterAction::from_app_action(action).unwrap()),
        Some(CounterAction::Reset)
    );
    assert_eq!(reset_step.model_before.integer("counter.count"), Some(5));
    assert_eq!(reset_step.model_after.integer("counter.count"), Some(0));
    assert_eq!(
        reset_step
            .projection_after
            .projection
            .as_ref()
            .map(|projection| projection.screen_id.as_str()),
        Some(CounterScreen::Counter.screen_id())
    );
    assert_eq!(report, repeated_report);
    assert!(report.passed);
}

#[test]
fn ui_event_packet_can_feed_route_action_map_without_app_program_ui_dependency() {
    let packet = UiEventPacket::new(
        RouteId::new(COUNTER_INCREMENT_ROUTE),
        RouteSchemaVersion::new(COUNTER_ROUTE_SCHEMA_VERSION),
        UiSchemaRef::new("counter.route.payload", 1),
        UiSchemaValue::null(),
    )
    .with_capability(RouteCapability::new(COUNTER_CAPABILITY))
    .with_source_control(UiEventSourceControlId::new("control.counter.increment"));
    let request = app_program::RouteActionRequest::new(
        packet.route.as_str(),
        packet.schema_version.value(),
        AppActionPayload::Unit,
    )
    .with_capability(counter_capability())
    .with_source_control(
        packet
            .source_control
            .as_ref()
            .expect("packet has source control")
            .as_str(),
    );

    let resolution = counter_route_action_map().resolve(&request);

    assert!(resolution.is_accepted());
    assert_eq!(
        resolution
            .action
            .as_ref()
            .map(|action| action.action_id.as_str()),
        Some(COUNTER_INCREMENT_ACTION_ID)
    );
}

#[test]
fn unknown_route_fails_closed() {
    let event = app_program::RouteActionRequest::new(
        "counter.unknown",
        COUNTER_ROUTE_SCHEMA_VERSION,
        AppActionPayload::Unit,
    )
    .with_capability(counter_capability());

    let trace = scenario_with(event).run(
        &counter_route_action_map(),
        counter_reducer,
        counter_projection,
    );

    assert_failed_without_mutation(&trace, RouteActionResolutionStatus::MissingRoute);
    assert_namespace(&trace.to_report(), NAMESPACE_ROUTE_ACTION_RESOLVE);
}

#[test]
fn wrong_route_schema_version_fails_closed() {
    let event = app_program::RouteActionRequest::new(
        COUNTER_INCREMENT_ROUTE,
        COUNTER_ROUTE_SCHEMA_VERSION + 1,
        AppActionPayload::Unit,
    )
    .with_capability(counter_capability());

    let trace = scenario_with(event).run(
        &counter_route_action_map(),
        counter_reducer,
        counter_projection,
    );

    assert_failed_without_mutation(&trace, RouteActionResolutionStatus::WrongRouteSchemaVersion);
    assert_namespace(&trace.to_report(), NAMESPACE_VERSION_COMPATIBILITY);
}

#[test]
fn invalid_action_payload_fails_closed() {
    let event = app_program::RouteActionRequest::new(
        COUNTER_INCREMENT_ROUTE,
        COUNTER_ROUTE_SCHEMA_VERSION,
        AppActionPayload::object([(
            "unexpected",
            AppActionPayloadValue::String("private payload contents stay summarized".to_owned()),
        )]),
    )
    .with_capability(counter_capability());

    let trace = scenario_with(event).run(
        &counter_route_action_map(),
        counter_reducer,
        counter_projection,
    );

    assert_failed_without_mutation(&trace, RouteActionResolutionStatus::InvalidPayload);
    assert_namespace(&trace.to_report(), NAMESPACE_ACTION_SCHEMA);
}

#[test]
fn missing_capability_fails_closed() {
    let event = app_program::RouteActionRequest::new(
        COUNTER_INCREMENT_ROUTE,
        COUNTER_ROUTE_SCHEMA_VERSION,
        AppActionPayload::Unit,
    );

    let trace = scenario_with(event).run(
        &counter_route_action_map(),
        counter_reducer,
        counter_projection,
    );

    assert_failed_without_mutation(&trace, RouteActionResolutionStatus::MissingCapability);
    assert_namespace(&trace.to_report(), NAMESPACE_HOST_COMPATIBILITY);
}

#[test]
fn route_diagnostic_rejection_fails_closed() {
    let event = counter_increment_event().with_diagnostic(AppDiagnostic::new(
        NAMESPACE_ROUTE_ACTION_RESOLVE,
        "app.route_action.resolve.injected_route_diagnostic",
        "route event was already diagnostic-bearing",
    ));

    let trace = scenario_with(event).run(
        &counter_route_action_map(),
        counter_reducer,
        counter_projection,
    );

    assert_failed_without_mutation(
        &trace,
        RouteActionResolutionStatus::RejectedRouteDiagnostics,
    );
    assert_namespace(&trace.to_report(), NAMESPACE_ROUTE_ACTION_RESOLVE);
}

#[test]
fn reducer_diagnostic_rejection_fails_closed() {
    let route_map = counter_route_action_map().with_mapping(
        RouteActionMapping::new(
            "counter.reducer_reject",
            COUNTER_ROUTE_SCHEMA_VERSION,
            AppActionId::new("counter.action.unsupported"),
            AppActionVersion::new(1),
            AppActionPayloadShape::unit(),
        )
        .with_required_capability(counter_capability()),
    );
    let event = app_program::RouteActionRequest::new(
        "counter.reducer_reject",
        COUNTER_ROUTE_SCHEMA_VERSION,
        AppActionPayload::Unit,
    )
    .with_capability(counter_capability());

    let trace = scenario_with(event).run(&route_map, counter_reducer, counter_projection);

    assert_failed_without_mutation(&trace, RouteActionResolutionStatus::Accepted);
    assert_namespace(&trace.to_report(), NAMESPACE_REDUCER);
}

#[test]
fn projection_diagnostic_rejection_fails_closed() {
    let invalid_model = counter_initial_snapshot()
        .with_value("counter.count", AppModelValue::Integer(-1))
        .with_revision(AppModelRevision::new(9));
    let scenario = AppReplayScenario::new(
        "counter.scenario.projection_reject",
        counter_program_id(),
        invalid_model,
    )
    .with_event(counter_increment_event());

    let trace = scenario.run(
        &counter_route_action_map(),
        counter_reducer,
        counter_projection,
    );

    assert!(!trace.passed());
    assert_eq!(trace.final_model.integer("counter.count"), Some(-1));
    assert_eq!(trace.final_model.revision.value(), 9);
    assert_namespace(&trace.to_report(), NAMESPACE_MODEL_SCHEMA);
    assert_namespace(&trace.to_report(), NAMESPACE_PROJECTION);
}

#[test]
fn rejected_action_does_not_mutate_model_state() {
    let trace = scenario_with(app_program::RouteActionRequest::new(
        COUNTER_RESET_ROUTE,
        COUNTER_ROUTE_SCHEMA_VERSION + 3,
        AppActionPayload::Unit,
    ))
    .run(
        &counter_route_action_map(),
        counter_reducer,
        counter_projection,
    );

    let step = &trace.steps[0];
    assert_eq!(step.model_before, step.model_after);
    assert_eq!(step.model_after.revision.value(), 0);
    assert_eq!(step.model_after.integer("counter.count"), Some(0));
}

#[test]
fn report_namespaces_distinguish_failure_classes() {
    let mut reports = Vec::new();
    reports.push(
        scenario_with(app_program::RouteActionRequest::new(
            "counter.unknown",
            COUNTER_ROUTE_SCHEMA_VERSION,
            AppActionPayload::Unit,
        ))
        .run(
            &counter_route_action_map(),
            counter_reducer,
            counter_projection,
        )
        .to_report(),
    );
    reports.push(
        scenario_with(
            app_program::RouteActionRequest::new(
                COUNTER_INCREMENT_ROUTE,
                COUNTER_ROUTE_SCHEMA_VERSION,
                AppActionPayload::object([("unexpected", AppActionPayloadValue::Bool(true))]),
            )
            .with_capability(counter_capability()),
        )
        .run(
            &counter_route_action_map(),
            counter_reducer,
            counter_projection,
        )
        .to_report(),
    );
    reports.push(
        scenario_with(
            app_program::RouteActionRequest::new(
                "counter.reducer_reject",
                COUNTER_ROUTE_SCHEMA_VERSION,
                AppActionPayload::Unit,
            )
            .with_capability(counter_capability()),
        )
        .run(
            &counter_route_action_map().with_mapping(
                RouteActionMapping::new(
                    "counter.reducer_reject",
                    COUNTER_ROUTE_SCHEMA_VERSION,
                    AppActionId::new("counter.action.unsupported"),
                    AppActionVersion::new(1),
                    AppActionPayloadShape::unit(),
                )
                .with_required_capability(counter_capability()),
            ),
            counter_reducer,
            counter_projection,
        )
        .to_report(),
    );
    reports.push(
        AppReplayScenario::new(
            "counter.scenario.bad_projection",
            counter_program_id(),
            counter_initial_snapshot().with_value("counter.count", AppModelValue::Integer(-1)),
        )
        .with_event(counter_increment_event())
        .run(
            &counter_route_action_map(),
            counter_reducer,
            counter_projection,
        )
        .to_report(),
    );

    let namespaces = reports
        .iter()
        .flat_map(|report| report.diagnostic_namespaces())
        .collect::<std::collections::BTreeSet<_>>();

    assert!(namespaces.contains(NAMESPACE_ROUTE_ACTION_RESOLVE));
    assert!(namespaces.contains(NAMESPACE_ACTION_SCHEMA));
    assert!(namespaces.contains(NAMESPACE_REDUCER));
    assert!(namespaces.contains(NAMESPACE_PROJECTION));
    assert!(namespaces.contains(NAMESPACE_EFFECT_PLAN));
    assert!(namespaces.contains(NAMESPACE_REPLAY));
    assert!(namespaces.contains(NAMESPACE_REPORT_BUDGET));
}

#[test]
fn counter_reducer_is_pure_for_same_input() {
    let action = counter_action_for_test(COUNTER_INCREMENT_ACTION_ID);
    let input = AppReducerInput::new(counter_initial_snapshot(), action);

    let first = counter_reducer(input.clone());
    let second = counter_reducer(input);

    assert_eq!(first, second);
    assert!(first.accepted);
    assert!(first.effect_plan.is_no_effect());
}

fn scenario_with(event: app_program::RouteActionRequest) -> AppReplayScenario {
    AppReplayScenario::new(
        "counter.scenario.single_event",
        counter_program_id(),
        counter_initial_snapshot(),
    )
    .with_event(event)
}

fn assert_failed_without_mutation(
    trace: &app_program::AppReplayTrace,
    expected_status: RouteActionResolutionStatus,
) {
    assert!(!trace.passed());
    let step = &trace.steps[0];
    assert_eq!(step.route_resolution.status, expected_status);
    assert_eq!(step.model_before, step.model_after);
    assert_eq!(trace.final_model.integer("counter.count"), Some(0));
    assert_eq!(trace.final_model.revision.value(), 0);
}

fn assert_namespace(report: &app_program::AppProgramReport, namespace: &str) {
    assert!(
        report.diagnostic_namespaces().contains(namespace),
        "expected namespace {namespace} in {:#?}",
        report.diagnostic_namespaces()
    );
}
