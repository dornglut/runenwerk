use engine::plugins::ui::{
    UiAction, UiActionDispatchFailureReason, UiActionDispatchReportsResource, UiActionEvent,
    UiActionHandler, UiHostActionExecutor, UiHostMutationIntent, UiHostMutationReceipt,
    UiHostMutationRejection, UiRuntimeDiagnosticCode, UiRuntimeDiagnosticsResource,
    UiRuntimeTraceEventKind, UiRuntimeTraceResource, UiTypedActionDescriptor, UiTypedActionId,
    dispatch_ui_action,
};
use ui_hosts::{HeadlessHost, HostCommand, HostKind, HostRouteMapVersion, HostRouteMapping};
use ui_program::{RouteCapability, RouteId, RouteSchemaVersion, UiEventPacket};
use ui_schema::{UiSchemaRef, UiSchemaValue};
use ui_surface::SurfaceInstanceId;

const ROUTE_MAP_VERSION: HostRouteMapVersion = HostRouteMapVersion::new(1);

#[test]
fn ui_action_dispatch_known_action_mutates_only_through_host_owner_and_traces() {
    let action = CounterIncrementAction;
    let handler = CounterIncrementHandler;
    let host = mapped_host(&handler.host_intent(&action));
    let event =
        UiActionEvent::new(valid_packet()).with_surface_instance_id(SurfaceInstanceId::new(10));
    let mut executor = CounterHostExecutor::default();
    let mut reports = UiActionDispatchReportsResource::default();
    let mut trace = UiRuntimeTraceResource::default();
    let mut diagnostics = UiRuntimeDiagnosticsResource::default();

    let report = dispatch_ui_action(
        &action,
        &handler,
        &event,
        &host,
        &mut executor,
        &mut reports,
        &mut trace,
        &mut diagnostics,
    );

    assert!(report.is_accepted());
    assert_eq!(executor.count, 1);
    assert_eq!(reports.latest_report(), Some(&report));
    assert_eq!(report.action_id().as_str(), "counter.increment");
    assert_eq!(report.route().as_str(), "counter.increment");
    assert_eq!(report.host(), HostKind::Headless);
    assert_eq!(
        report
            .host_command()
            .expect("accepted report should record host command")
            .command_id,
        "counter.increment"
    );
    assert_eq!(
        report
            .domain_command()
            .expect("accepted report should record domain command")
            .command_id,
        "increment"
    );
    assert!(diagnostics.is_empty());
    assert_trace_contains(
        &trace,
        &[
            UiRuntimeTraceEventKind::Mounted,
            UiRuntimeTraceEventKind::Input,
            UiRuntimeTraceEventKind::Route,
            UiRuntimeTraceEventKind::Capability,
            UiRuntimeTraceEventKind::Dispatch,
            UiRuntimeTraceEventKind::Mutation,
        ],
    );
}

#[test]
fn ui_action_dispatch_rejections_do_not_mutate_and_record_failure_reasons() {
    assert_rejected(
        HeadlessHost::new(ROUTE_MAP_VERSION),
        valid_packet(),
        CounterHostExecutor::default(),
        UiActionDispatchFailureReason::UnknownRoute,
    );

    assert_rejected(
        mapped_host(&CounterIncrementHandler.host_intent(&CounterIncrementAction)),
        valid_packet_with_schema_version(RouteSchemaVersion::new(2)),
        CounterHostExecutor::default(),
        UiActionDispatchFailureReason::SchemaMismatch,
    );

    assert_rejected(
        mapped_host(&CounterIncrementHandler.host_intent(&CounterIncrementAction)),
        valid_packet_without_capability(),
        CounterHostExecutor::default(),
        UiActionDispatchFailureReason::CapabilityMismatch,
    );

    assert_rejected(
        mapped_host(&CounterIncrementHandler.host_intent(&CounterIncrementAction)),
        packet_with_payload_schema(UiSchemaRef::new("counter.increment.wrong_payload", 1)),
        CounterHostExecutor::default(),
        UiActionDispatchFailureReason::PayloadMismatch,
    );

    assert_rejected(
        mapped_host(&CounterIncrementHandler.host_intent(&CounterIncrementAction)),
        valid_packet(),
        CounterHostExecutor {
            failure: Some(UiHostMutationRejection::missing_host_data()),
            ..CounterHostExecutor::default()
        },
        UiActionDispatchFailureReason::MissingHostData,
    );
}

fn assert_rejected(
    host: HeadlessHost,
    packet: UiEventPacket,
    mut executor: CounterHostExecutor,
    expected: UiActionDispatchFailureReason,
) {
    let action = CounterIncrementAction;
    let handler = CounterIncrementHandler;
    let event = UiActionEvent::new(packet);
    let mut reports = UiActionDispatchReportsResource::default();
    let mut trace = UiRuntimeTraceResource::default();
    let mut diagnostics = UiRuntimeDiagnosticsResource::default();

    let report = dispatch_ui_action(
        &action,
        &handler,
        &event,
        &host,
        &mut executor,
        &mut reports,
        &mut trace,
        &mut diagnostics,
    );

    assert!(!report.is_accepted());
    assert_eq!(report.failure_reason(), Some(expected));
    assert_eq!(executor.count, 0);
    assert_eq!(reports.latest_report(), Some(&report));
    assert_eq!(diagnostics.len(), 1);

    let diagnostic = &diagnostics.entries()[0];
    assert_eq!(
        diagnostic.code,
        UiRuntimeDiagnosticCode::ActionDispatchRejected
    );
    let action_dispatch = diagnostic
        .action_dispatch
        .as_ref()
        .expect("rejected action should record action dispatch diagnostics");
    assert_eq!(action_dispatch.action_id, "counter.increment");
    assert_eq!(action_dispatch.host, HostKind::Headless);
    assert_eq!(action_dispatch.failure_reason, expected);

    assert_trace_contains(
        &trace,
        &[
            UiRuntimeTraceEventKind::Input,
            UiRuntimeTraceEventKind::Rejection,
            UiRuntimeTraceEventKind::Diagnostic,
        ],
    );
}

fn mapped_host(intent: &UiHostMutationIntent) -> HeadlessHost {
    HeadlessHost::new(ROUTE_MAP_VERSION)
        .with_mapping(intent.to_host_route_mapping(ROUTE_MAP_VERSION))
}

fn valid_packet() -> UiEventPacket {
    packet_with_payload_schema(payload_schema())
}

fn valid_packet_without_capability() -> UiEventPacket {
    UiEventPacket::new(
        RouteId::new("counter.increment"),
        RouteSchemaVersion::new(1),
        payload_schema(),
        payload(),
    )
}

fn valid_packet_with_schema_version(schema_version: RouteSchemaVersion) -> UiEventPacket {
    UiEventPacket::new(
        RouteId::new("counter.increment"),
        schema_version,
        payload_schema(),
        payload(),
    )
    .with_capability(RouteCapability::new("counter.action.increment"))
}

fn packet_with_payload_schema(payload_schema: UiSchemaRef) -> UiEventPacket {
    UiEventPacket::new(
        RouteId::new("counter.increment"),
        RouteSchemaVersion::new(1),
        payload_schema,
        payload(),
    )
    .with_capability(RouteCapability::new("counter.action.increment"))
}

fn payload_schema() -> UiSchemaRef {
    UiSchemaRef::new("counter.increment.payload", 1)
}

fn payload() -> UiSchemaValue {
    UiSchemaValue::object([("amount", UiSchemaValue::integer(1))])
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
struct CounterIncrementAction;

impl UiAction for CounterIncrementAction {
    fn action_descriptor(&self) -> UiTypedActionDescriptor {
        UiTypedActionDescriptor::new(
            UiTypedActionId::new("counter.increment"),
            RouteId::new("counter.increment"),
            RouteSchemaVersion::new(1),
            payload_schema(),
            RouteCapability::new("counter.action.increment"),
        )
    }
}

struct CounterIncrementHandler;

impl UiActionHandler<CounterIncrementAction> for CounterIncrementHandler {
    fn host_intent(&self, action: &CounterIncrementAction) -> UiHostMutationIntent {
        UiHostMutationIntent::new(
            action.action_descriptor(),
            HostCommand::new(HostKind::Headless, "counter.increment"),
        )
        .with_domain_command(ui_hosts::DomainCommand::new("counter", "increment"))
    }
}

#[derive(Default)]
struct CounterHostExecutor {
    count: u32,
    failure: Option<UiHostMutationRejection>,
}

impl UiHostActionExecutor for CounterHostExecutor {
    fn apply(
        &mut self,
        intent: &UiHostMutationIntent,
        _packet: &UiEventPacket,
        _mapping: &HostRouteMapping,
    ) -> Result<UiHostMutationReceipt, UiHostMutationRejection> {
        if let Some(failure) = self.failure.clone() {
            return Err(failure);
        }

        self.count += 1;
        Ok(UiHostMutationReceipt::from_intent(intent))
    }
}
