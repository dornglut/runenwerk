use super::screen::{UiTypedIdentityError, validate_typed_contract_id};
use super::{
    UiActionDispatchFailureReason, UiActionDispatchReport, UiActionDispatchReportsResource,
    UiActionEvent, UiActionHandler, UiHostActionExecutor, UiHostMutationFailureReason,
    UiRuntimeDiagnostic, UiRuntimeDiagnosticCode, UiRuntimeDiagnosticsResource,
    UiRuntimeTraceEvent, UiRuntimeTraceResource,
};

use ui_hosts::{HostKind, HostRouteMapping, HostRouteResolutionStatus, UiHost};
use ui_program::{RouteCapability, RouteId, RouteSchemaVersion};
use ui_schema::UiSchemaRef;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiTypedActionId(String);

impl UiTypedActionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("typed UI action IDs must be stable namespaced IDs")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, UiTypedIdentityError> {
        Ok(Self(validate_typed_contract_id("action", value.into())?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiTypedActionDescriptor {
    action_id: UiTypedActionId,
    route: RouteId,
    schema_version: RouteSchemaVersion,
    payload_schema: UiSchemaRef,
    capability: RouteCapability,
}

impl UiTypedActionDescriptor {
    pub fn new(
        action_id: UiTypedActionId,
        route: RouteId,
        schema_version: RouteSchemaVersion,
        payload_schema: UiSchemaRef,
        capability: RouteCapability,
    ) -> Self {
        Self {
            action_id,
            route,
            schema_version,
            payload_schema,
            capability,
        }
    }

    pub fn action_id(&self) -> &UiTypedActionId {
        &self.action_id
    }

    pub fn route(&self) -> &RouteId {
        &self.route
    }

    pub fn schema_version(&self) -> RouteSchemaVersion {
        self.schema_version
    }

    pub fn payload_schema(&self) -> &UiSchemaRef {
        &self.payload_schema
    }

    pub fn capability(&self) -> &RouteCapability {
        &self.capability
    }
}

pub trait UiAction {
    fn action_descriptor(&self) -> UiTypedActionDescriptor;
}

pub fn dispatch_ui_action<A, Handler, Host, Executor>(
    action: &A,
    handler: &Handler,
    event: &UiActionEvent,
    host: &Host,
    executor: &mut Executor,
    reports: &mut UiActionDispatchReportsResource,
    trace: &mut UiRuntimeTraceResource,
    diagnostics: &mut UiRuntimeDiagnosticsResource,
) -> UiActionDispatchReport
where
    A: UiAction,
    Handler: UiActionHandler<A>,
    Host: UiHost,
    Executor: UiHostActionExecutor,
{
    let descriptor = action.action_descriptor();
    let route = event.packet().route.clone();
    let host_kind = host.kind();

    if let Some(surface_instance_id) = event.surface_instance_id() {
        trace.record(UiRuntimeTraceEvent::mounted(
            &descriptor,
            route.clone(),
            host_kind,
            surface_instance_id,
        ));
    }
    trace.record(UiRuntimeTraceEvent::input(
        &descriptor,
        route.clone(),
        host_kind,
    ));

    if event.packet().payload_schema() != descriptor.payload_schema()
        || !event.packet().payload.diagnostics.is_empty()
    {
        return reject_action_dispatch(
            &descriptor,
            route,
            host_kind,
            UiActionDispatchFailureReason::PayloadMismatch,
            reports,
            trace,
            diagnostics,
        );
    }

    let resolution = host.resolve_event(event.packet());
    match resolution.status {
        HostRouteResolutionStatus::Mapped => {
            trace.record(UiRuntimeTraceEvent::route_event(
                &descriptor,
                route.clone(),
                host_kind,
            ));
            trace.record(UiRuntimeTraceEvent::capability(
                &descriptor,
                route.clone(),
                host_kind,
            ));
        }
        HostRouteResolutionStatus::MissingRoute => {
            trace.record(UiRuntimeTraceEvent::route_event(
                &descriptor,
                route.clone(),
                host_kind,
            ));
            return reject_action_dispatch(
                &descriptor,
                route,
                host_kind,
                UiActionDispatchFailureReason::UnknownRoute,
                reports,
                trace,
                diagnostics,
            );
        }
        HostRouteResolutionStatus::UnsupportedSchemaVersion => {
            trace.record(UiRuntimeTraceEvent::route_event(
                &descriptor,
                route.clone(),
                host_kind,
            ));
            return reject_action_dispatch(
                &descriptor,
                route,
                host_kind,
                UiActionDispatchFailureReason::SchemaMismatch,
                reports,
                trace,
                diagnostics,
            );
        }
        HostRouteResolutionStatus::MissingCapability => {
            trace.record(UiRuntimeTraceEvent::route_event(
                &descriptor,
                route.clone(),
                host_kind,
            ));
            trace.record(UiRuntimeTraceEvent::capability(
                &descriptor,
                route.clone(),
                host_kind,
            ));
            return reject_action_dispatch(
                &descriptor,
                route,
                host_kind,
                UiActionDispatchFailureReason::CapabilityMismatch,
                reports,
                trace,
                diagnostics,
            );
        }
    }

    let mapping = resolution
        .mapping
        .as_ref()
        .expect("mapped host route resolution should include a host route mapping");
    dispatch_mapped_action(
        action,
        handler,
        event,
        &descriptor,
        route,
        host_kind,
        mapping,
        executor,
        reports,
        trace,
        diagnostics,
    )
}

fn dispatch_mapped_action<A, Handler, Executor>(
    action: &A,
    handler: &Handler,
    event: &UiActionEvent,
    descriptor: &UiTypedActionDescriptor,
    route: RouteId,
    host: HostKind,
    mapping: &HostRouteMapping,
    executor: &mut Executor,
    reports: &mut UiActionDispatchReportsResource,
    trace: &mut UiRuntimeTraceResource,
    diagnostics: &mut UiRuntimeDiagnosticsResource,
) -> UiActionDispatchReport
where
    A: UiAction,
    Handler: UiActionHandler<A>,
    Executor: UiHostActionExecutor,
{
    let intent = handler.host_intent(action);
    if intent.action() != descriptor
        || mapping.host_command != *intent.host_command()
        || mapping.domain_command.as_ref() != intent.domain_command()
    {
        return reject_action_dispatch(
            descriptor,
            route,
            host,
            UiActionDispatchFailureReason::HostRejected,
            reports,
            trace,
            diagnostics,
        );
    }

    trace.record(UiRuntimeTraceEvent::dispatch(
        descriptor,
        route.clone(),
        host,
    ));

    match executor.apply(&intent, event.packet(), mapping) {
        Ok(receipt) => {
            trace.record(UiRuntimeTraceEvent::mutation(
                descriptor,
                route.clone(),
                host,
            ));
            let report = UiActionDispatchReport::accepted(
                descriptor.action_id().clone(),
                route,
                host,
                receipt,
            );
            reports.record(report.clone());
            report
        }
        Err(rejection) => reject_action_dispatch(
            descriptor,
            route,
            host,
            match rejection.failure_reason() {
                UiHostMutationFailureReason::MissingHostData => {
                    UiActionDispatchFailureReason::MissingHostData
                }
                UiHostMutationFailureReason::RejectedByHost => {
                    UiActionDispatchFailureReason::HostRejected
                }
            },
            reports,
            trace,
            diagnostics,
        ),
    }
}

fn reject_action_dispatch(
    descriptor: &UiTypedActionDescriptor,
    route: RouteId,
    host: HostKind,
    failure_reason: UiActionDispatchFailureReason,
    reports: &mut UiActionDispatchReportsResource,
    trace: &mut UiRuntimeTraceResource,
    diagnostics: &mut UiRuntimeDiagnosticsResource,
) -> UiActionDispatchReport {
    let diagnostic = UiRuntimeDiagnostic::action_dispatch_rejected(
        descriptor.action_id().as_str(),
        route.as_str(),
        host,
        failure_reason,
    );
    diagnostics.push(diagnostic);
    trace.record(UiRuntimeTraceEvent::rejection(
        descriptor,
        route.clone(),
        host,
        failure_reason,
    ));
    trace.record(UiRuntimeTraceEvent::diagnostic(
        descriptor,
        route.clone(),
        host,
        failure_reason,
        UiRuntimeDiagnosticCode::ActionDispatchRejected,
    ));
    let report = UiActionDispatchReport::rejected(
        descriptor.action_id().clone(),
        route,
        host,
        failure_reason,
    );
    reports.record(report.clone());
    report
}
