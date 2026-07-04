use app_program::{
    AppActionCapability, AppActionPayload, AppActionPayloadValue, AppModelSnapshot,
    AppReducerInput, AppReplayScenario, AppViewProjection, COUNTER_CAPABILITY,
    COUNTER_INCREMENT_ROUTE, COUNTER_RESET_ROUTE, COUNTER_ROUTE_SCHEMA_VERSION,
    counter_initial_snapshot, counter_projection, counter_reducer, counter_route_action_map,
};
use ui_compiler::UiCompiler;
use ui_evaluator::{UiEvaluationContext, UiEvaluator};
use ui_hosts::{
    HeadlessHost, HostCommand, HostKind, HostRouteMapVersion, HostRouteMapping, HostSurfaceFacts,
    UiHost,
};
use ui_program::{
    ControlGraphNode, ControlKindRef, ControlNodeId, ControlPackageRef, ControlPropertySnapshot,
    ControlPropertySnapshotId, InteractionHandler, InteractionHandlerId, InteractionTrigger,
    RouteCapability, RouteId, RouteSchemaVersion, UiEventPacket, UiEventPhase,
    UiEventSourceControlId, UiProgram, UiProgramId, UiProgramSource, UiProgramSourceId,
    UiProgramSourceMapEntry, UiProgramTargetId, UiProgramVersion,
};
use ui_schema::{UiSchemaRef, UiSchemaValue};
use ui_state::UiStateModel;

fn main() {
    let route_map = counter_route_action_map();
    let host = counter_headless_host();
    let mut probe_model = counter_initial_snapshot();
    let mut scenario = AppReplayScenario::new(
        "counter.example.ui_generated_events",
        app_program::counter_program_id(),
        counter_initial_snapshot(),
    );
    let mut ui_summaries = Vec::new();

    for route in [
        COUNTER_INCREMENT_ROUTE,
        COUNTER_INCREMENT_ROUTE,
        COUNTER_INCREMENT_ROUTE,
        COUNTER_INCREMENT_ROUTE,
        COUNTER_INCREMENT_ROUTE,
        COUNTER_RESET_ROUTE,
    ] {
        let projection = counter_projection(&probe_model)
            .projection
            .expect("counter projection must succeed for the example");
        let (packet, summary) = run_headless_ui_step(&projection, &host, route);
        let request = route_action_request_from_ui_packet(&packet);
        let resolution = route_map.resolve(&request);
        let action = resolution
            .action
            .clone()
            .expect("UI-generated counter routes must resolve");
        let outcome = counter_reducer(AppReducerInput::new(probe_model.clone(), action));
        probe_model = outcome.after_model;
        scenario = scenario.with_event(request);
        ui_summaries.push(summary);
    }

    let trace = scenario.run(&route_map, counter_reducer, counter_projection);
    let report = trace.to_report();
    println!(
        "headless_counter_ui passed={} final_count={} final_revision={} ui_steps={}",
        report.passed,
        report
            .reducer_reports
            .last()
            .and_then(|reducer| reducer.count_after)
            .unwrap_or_default(),
        report.final_revision,
        ui_summaries.len()
    );
    for summary in ui_summaries {
        println!("{summary}");
    }
}

fn run_headless_ui_step(
    projection: &AppViewProjection,
    host: &HeadlessHost,
    route: &str,
) -> (UiEventPacket, String) {
    let program = ui_program_from_projection(projection);
    let compiler_report = UiCompiler.compile_report(&program);
    assert!(compiler_report.passed());
    let artifact = compiler_report.artifact;
    let mut state = UiStateModel::default();
    let output =
        UiEvaluator.evaluate_with_context(&artifact, &mut state, UiEvaluationContext::default());
    let receipt = host.consume_output(&output, HostSurfaceFacts::headless("surface.counter"));
    let packet = ui_packet_for_route(route);
    let host_resolution = host.resolve_event(&packet);
    assert!(host_resolution.is_mapped());
    assert!(output.diagnostics.is_empty());
    (
        packet,
        format!(
            "ui_step screen={} controls={} interactions={} diagnostics={} source_rows={}",
            projection.screen_id,
            output.controls.rows.len(),
            output.interaction.rows.len(),
            receipt.diagnostic_count,
            receipt.source_mapped_rows
        ),
    )
}

fn ui_program_from_projection(projection: &AppViewProjection) -> UiProgram {
    let route = projection
        .route_ids
        .first()
        .expect("counter projection exposes exactly one active route");
    let control_id = if route == COUNTER_RESET_ROUTE {
        ControlNodeId::new("control.counter.reset")
    } else {
        ControlNodeId::new("control.counter.increment")
    };
    let source_entry = UiProgramSourceMapEntry::new(
        UiProgramSourceId::new(format!("definition.{}", projection.screen_id)),
        UiProgramTargetId::new(format!("program.{}", control_id.as_str())),
    );
    let capability = RouteCapability::new(COUNTER_CAPABILITY);
    let mut program = UiProgram::new(
        UiProgramId::new("program.counter.headless"),
        UiProgramVersion::new(1),
    )
    .with_source(UiProgramSource::authored(
        UiProgramSourceId::new("definition.counter.headless"),
        "headless counter projection",
    ))
    .with_source_map_entry(source_entry.clone());
    program.graphs.control.add_node(
        ControlGraphNode::new(
            control_id.clone(),
            ControlPackageRef::new("runenwerk.counter.ui"),
            ControlKindRef::new("runenwerk.counter.ui.button"),
        )
        .with_capability(capability.clone()),
    );
    program
        .graphs
        .properties
        .add_snapshot(ControlPropertySnapshot::new(
            ControlPropertySnapshotId::new(format!("properties.{}", control_id.as_str())),
            control_id.clone(),
            UiSchemaRef::new("counter.ui.button.properties", 1),
            UiSchemaValue::object([(
                "screen",
                UiSchemaValue::stable_id_ref(projection.screen_id.clone()),
            )]),
        ));
    program.graphs.interaction.handlers.push(
        InteractionHandler::new(
            InteractionHandlerId::new(format!("interaction.{}", route)),
            control_id,
            InteractionTrigger::Press,
            RouteId::new(route.clone()),
            UiSchemaRef::new("counter.route.payload", 1),
        )
        .with_capability(capability),
    );
    program
}

fn counter_headless_host() -> HeadlessHost {
    let version = HostRouteMapVersion::new(1);
    HeadlessHost::new(version)
        .with_mapping(
            HostRouteMapping::new(
                RouteId::new(COUNTER_INCREMENT_ROUTE),
                RouteSchemaVersion::new(COUNTER_ROUTE_SCHEMA_VERSION),
                version,
                HostCommand::new(HostKind::Headless, "headless.counter.increment"),
            )
            .with_capability(RouteCapability::new(COUNTER_CAPABILITY)),
        )
        .with_mapping(
            HostRouteMapping::new(
                RouteId::new(COUNTER_RESET_ROUTE),
                RouteSchemaVersion::new(COUNTER_ROUTE_SCHEMA_VERSION),
                version,
                HostCommand::new(HostKind::Headless, "headless.counter.reset"),
            )
            .with_capability(RouteCapability::new(COUNTER_CAPABILITY)),
        )
}

fn ui_packet_for_route(route: &str) -> UiEventPacket {
    UiEventPacket::new(
        RouteId::new(route),
        RouteSchemaVersion::new(COUNTER_ROUTE_SCHEMA_VERSION),
        UiSchemaRef::new("counter.route.payload", 1),
        UiSchemaValue::null(),
    )
    .with_capability(RouteCapability::new(COUNTER_CAPABILITY))
    .with_phase(UiEventPhase::Commit)
    .with_source_control(UiEventSourceControlId::new(format!("control.{route}")))
}

fn route_action_request_from_ui_packet(packet: &UiEventPacket) -> app_program::RouteActionRequest {
    let payload = match &packet.payload.value {
        UiSchemaValue::Null => AppActionPayload::Unit,
        UiSchemaValue::Object(values) => {
            AppActionPayload::object(values.iter().filter_map(|(key, value)| match value {
                UiSchemaValue::Integer(integer) => {
                    Some((key.clone(), AppActionPayloadValue::Integer(*integer)))
                }
                UiSchemaValue::Bool(value) => {
                    Some((key.clone(), AppActionPayloadValue::Bool(*value)))
                }
                UiSchemaValue::String(value) => {
                    Some((key.clone(), AppActionPayloadValue::String(value.clone())))
                }
                _ => None,
            }))
        }
        _ => AppActionPayload::Unit,
    };
    let mut request = app_program::RouteActionRequest::new(
        packet.route.as_str(),
        packet.schema_version.value(),
        payload,
    );
    for capability in &packet.capabilities {
        request = request.with_capability(AppActionCapability::new(capability.as_str()));
    }
    if let Some(source_control) = &packet.source_control {
        request = request.with_source_control(source_control.as_str());
    }
    for source_map in &packet.source_map {
        request = request.with_source_map(format!(
            "{}->{}",
            source_map.source_id.as_str(),
            source_map.target_id.as_str()
        ));
    }
    request
}

#[allow(dead_code)]
fn _assert_no_ui_dependency_in_production(_: &AppModelSnapshot) {}
