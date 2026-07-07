use ui_hosts::HostKind;
use ui_program::RouteId;
use ui_surface::SurfaceInstanceId;

use super::{
    UiActionDispatchFailureReason, UiRuntimeDiagnosticCode, UiTypedActionDescriptor,
    UiTypedActionId,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiRuntimeTraceEventKind {
    Mounted,
    Input,
    Route,
    Capability,
    Dispatch,
    Mutation,
    Rejection,
    Diagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRuntimeTraceEvent {
    kind: UiRuntimeTraceEventKind,
    action_id: UiTypedActionId,
    route: RouteId,
    host: HostKind,
    surface_instance_id: Option<SurfaceInstanceId>,
    failure_reason: Option<UiActionDispatchFailureReason>,
    diagnostic_code: Option<UiRuntimeDiagnosticCode>,
}

impl UiRuntimeTraceEvent {
    pub fn mounted(
        descriptor: &UiTypedActionDescriptor,
        route: RouteId,
        host: HostKind,
        surface_instance_id: SurfaceInstanceId,
    ) -> Self {
        Self::new(
            UiRuntimeTraceEventKind::Mounted,
            descriptor,
            route,
            host,
            Some(surface_instance_id),
        )
    }

    pub fn input(descriptor: &UiTypedActionDescriptor, route: RouteId, host: HostKind) -> Self {
        Self::new(
            UiRuntimeTraceEventKind::Input,
            descriptor,
            route,
            host,
            None,
        )
    }

    pub fn route_event(
        descriptor: &UiTypedActionDescriptor,
        route: RouteId,
        host: HostKind,
    ) -> Self {
        Self::new(
            UiRuntimeTraceEventKind::Route,
            descriptor,
            route,
            host,
            None,
        )
    }

    pub fn capability(
        descriptor: &UiTypedActionDescriptor,
        route: RouteId,
        host: HostKind,
    ) -> Self {
        Self::new(
            UiRuntimeTraceEventKind::Capability,
            descriptor,
            route,
            host,
            None,
        )
    }

    pub fn dispatch(descriptor: &UiTypedActionDescriptor, route: RouteId, host: HostKind) -> Self {
        Self::new(
            UiRuntimeTraceEventKind::Dispatch,
            descriptor,
            route,
            host,
            None,
        )
    }

    pub fn mutation(descriptor: &UiTypedActionDescriptor, route: RouteId, host: HostKind) -> Self {
        Self::new(
            UiRuntimeTraceEventKind::Mutation,
            descriptor,
            route,
            host,
            None,
        )
    }

    pub fn rejection(
        descriptor: &UiTypedActionDescriptor,
        route: RouteId,
        host: HostKind,
        failure_reason: UiActionDispatchFailureReason,
    ) -> Self {
        Self::new(
            UiRuntimeTraceEventKind::Rejection,
            descriptor,
            route,
            host,
            None,
        )
        .with_failure_reason(failure_reason)
    }

    pub fn diagnostic(
        descriptor: &UiTypedActionDescriptor,
        route: RouteId,
        host: HostKind,
        failure_reason: UiActionDispatchFailureReason,
        diagnostic_code: UiRuntimeDiagnosticCode,
    ) -> Self {
        Self::new(
            UiRuntimeTraceEventKind::Diagnostic,
            descriptor,
            route,
            host,
            None,
        )
        .with_failure_reason(failure_reason)
        .with_diagnostic_code(diagnostic_code)
    }

    fn new(
        kind: UiRuntimeTraceEventKind,
        descriptor: &UiTypedActionDescriptor,
        route: RouteId,
        host: HostKind,
        surface_instance_id: Option<SurfaceInstanceId>,
    ) -> Self {
        Self {
            kind,
            action_id: descriptor.action_id().clone(),
            route,
            host,
            surface_instance_id,
            failure_reason: None,
            diagnostic_code: None,
        }
    }

    fn with_failure_reason(mut self, failure_reason: UiActionDispatchFailureReason) -> Self {
        self.failure_reason = Some(failure_reason);
        self
    }

    fn with_diagnostic_code(mut self, diagnostic_code: UiRuntimeDiagnosticCode) -> Self {
        self.diagnostic_code = Some(diagnostic_code);
        self
    }

    pub fn kind(&self) -> UiRuntimeTraceEventKind {
        self.kind
    }

    pub fn action_id(&self) -> &UiTypedActionId {
        &self.action_id
    }

    pub fn route(&self) -> &RouteId {
        &self.route
    }

    pub fn host(&self) -> HostKind {
        self.host
    }

    pub fn surface_instance_id(&self) -> Option<SurfaceInstanceId> {
        self.surface_instance_id
    }

    pub fn failure_reason(&self) -> Option<UiActionDispatchFailureReason> {
        self.failure_reason
    }

    pub fn diagnostic_code(&self) -> Option<UiRuntimeDiagnosticCode> {
        self.diagnostic_code
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ecs::Resource)]
pub struct UiRuntimeTraceResource {
    events: Vec<UiRuntimeTraceEvent>,
}

impl UiRuntimeTraceResource {
    pub fn events(&self) -> &[UiRuntimeTraceEvent] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn record(&mut self, event: UiRuntimeTraceEvent) {
        self.events.push(event);
    }
}
