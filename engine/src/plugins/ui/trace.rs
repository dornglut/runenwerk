use ui_hosts::HostKind;
use ui_program::RouteId;
use ui_surface::SurfaceInstanceId;

use super::{
    UiActionDispatchFailureReason, UiRuntimeDiagnosticCode, UiRuntimeDirtyCause,
    UiRuntimeSourceProgramFacts, UiTypedActionDescriptor, UiTypedActionId,
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
    RuntimeEvaluation,
    StateSnapshot,
    Invalidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRuntimeTraceEvent {
    kind: UiRuntimeTraceEventKind,
    action_id: Option<UiTypedActionId>,
    route: Option<RouteId>,
    host: Option<HostKind>,
    surface_instance_id: Option<SurfaceInstanceId>,
    failure_reason: Option<UiActionDispatchFailureReason>,
    diagnostic_code: Option<UiRuntimeDiagnosticCode>,
    runtime_id: Option<String>,
    source_id: Option<String>,
    program_id: Option<String>,
    dirty_cause: Option<UiRuntimeDirtyCause>,
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
            action_id: Some(descriptor.action_id().clone()),
            route: Some(route),
            host: Some(host),
            surface_instance_id,
            failure_reason: None,
            diagnostic_code: None,
            runtime_id: None,
            source_id: None,
            program_id: None,
            dirty_cause: None,
        }
    }

    pub fn runtime_evaluation(
        runtime_id: impl Into<String>,
        facts: &UiRuntimeSourceProgramFacts,
    ) -> Self {
        Self::runtime_event(
            UiRuntimeTraceEventKind::RuntimeEvaluation,
            runtime_id,
            facts,
            None,
        )
    }

    pub fn state_snapshot(
        runtime_id: impl Into<String>,
        facts: &UiRuntimeSourceProgramFacts,
    ) -> Self {
        Self::runtime_event(
            UiRuntimeTraceEventKind::StateSnapshot,
            runtime_id,
            facts,
            None,
        )
    }

    pub fn invalidation(
        runtime_id: impl Into<String>,
        facts: &UiRuntimeSourceProgramFacts,
        dirty_cause: UiRuntimeDirtyCause,
    ) -> Self {
        Self::runtime_event(
            UiRuntimeTraceEventKind::Invalidation,
            runtime_id,
            facts,
            Some(dirty_cause),
        )
    }

    fn runtime_event(
        kind: UiRuntimeTraceEventKind,
        runtime_id: impl Into<String>,
        facts: &UiRuntimeSourceProgramFacts,
        dirty_cause: Option<UiRuntimeDirtyCause>,
    ) -> Self {
        Self {
            kind,
            action_id: None,
            route: None,
            host: None,
            surface_instance_id: None,
            failure_reason: None,
            diagnostic_code: None,
            runtime_id: Some(runtime_id.into()),
            source_id: Some(facts.source_id().to_owned()),
            program_id: Some(facts.program_id().to_owned()),
            dirty_cause,
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

    pub fn action_id(&self) -> Option<&UiTypedActionId> {
        self.action_id.as_ref()
    }

    pub fn route(&self) -> Option<&RouteId> {
        self.route.as_ref()
    }

    pub fn host(&self) -> Option<HostKind> {
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

    pub fn runtime_id(&self) -> Option<&str> {
        self.runtime_id.as_deref()
    }

    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }

    pub fn program_id(&self) -> Option<&str> {
        self.program_id.as_deref()
    }

    pub fn dirty_cause(&self) -> Option<UiRuntimeDirtyCause> {
        self.dirty_cause
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
