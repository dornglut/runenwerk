use super::UiRuntimeInstallState;
use super::{UiHostMutationReceipt, UiTypedActionId};

use ui_hosts::{DomainCommand, HostCommand, HostKind};
use ui_program::RouteId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UiRuntimeReport {
    pub install_state: UiRuntimeInstallState,
    pub diagnostic_count: usize,
}

impl Default for UiRuntimeReport {
    fn default() -> Self {
        Self {
            install_state: UiRuntimeInstallState::Uninstalled,
            diagnostic_count: 0,
        }
    }
}

/// Latest lightweight UI runtime status report.
#[derive(Debug, Clone, PartialEq, Eq, Default, ecs::Resource)]
pub struct UiRuntimeReportResource {
    latest: UiRuntimeReport,
}

impl UiRuntimeReportResource {
    pub fn latest(&self) -> UiRuntimeReport {
        self.latest
    }

    pub(crate) fn record_plugin_installed(&mut self, diagnostic_count: usize) {
        self.latest = UiRuntimeReport {
            install_state: UiRuntimeInstallState::Installed,
            diagnostic_count,
        };
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiActionDispatchFailureReason {
    UnknownRoute,
    SchemaMismatch,
    CapabilityMismatch,
    PayloadMismatch,
    MissingHostData,
    HostRejected,
}

impl UiActionDispatchFailureReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnknownRoute => "unknown_route",
            Self::SchemaMismatch => "schema_mismatch",
            Self::CapabilityMismatch => "capability_mismatch",
            Self::PayloadMismatch => "payload_mismatch",
            Self::MissingHostData => "missing_host_data",
            Self::HostRejected => "host_rejected",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::UnknownRoute => "UI action route is not mapped by the host",
            Self::SchemaMismatch => "UI action schema version is not accepted by the host",
            Self::CapabilityMismatch => "UI action is missing a required route capability",
            Self::PayloadMismatch => "UI action payload does not match the typed action contract",
            Self::MissingHostData => "UI action host data required for mutation is missing",
            Self::HostRejected => "UI action host mutation was rejected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiActionDispatchReport {
    action_id: UiTypedActionId,
    route: RouteId,
    host: HostKind,
    accepted: bool,
    failure_reason: Option<UiActionDispatchFailureReason>,
    host_command: Option<HostCommand>,
    domain_command: Option<DomainCommand>,
}

impl UiActionDispatchReport {
    pub(crate) fn accepted(
        action_id: UiTypedActionId,
        route: RouteId,
        host: HostKind,
        receipt: UiHostMutationReceipt,
    ) -> Self {
        Self {
            action_id,
            route,
            host,
            accepted: true,
            failure_reason: None,
            host_command: Some(receipt.host_command().clone()),
            domain_command: receipt.domain_command().cloned(),
        }
    }

    pub(crate) fn rejected(
        action_id: UiTypedActionId,
        route: RouteId,
        host: HostKind,
        failure_reason: UiActionDispatchFailureReason,
    ) -> Self {
        Self {
            action_id,
            route,
            host,
            accepted: false,
            failure_reason: Some(failure_reason),
            host_command: None,
            domain_command: None,
        }
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

    pub fn is_accepted(&self) -> bool {
        self.accepted
    }

    pub fn failure_reason(&self) -> Option<UiActionDispatchFailureReason> {
        self.failure_reason
    }

    pub fn host_command(&self) -> Option<&HostCommand> {
        self.host_command.as_ref()
    }

    pub fn domain_command(&self) -> Option<&DomainCommand> {
        self.domain_command.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ecs::Resource)]
pub struct UiActionDispatchReportsResource {
    reports: Vec<UiActionDispatchReport>,
}

impl UiActionDispatchReportsResource {
    pub fn reports(&self) -> &[UiActionDispatchReport] {
        &self.reports
    }

    pub fn latest_report(&self) -> Option<&UiActionDispatchReport> {
        self.reports.last()
    }

    pub fn len(&self) -> usize {
        self.reports.len()
    }

    pub fn is_empty(&self) -> bool {
        self.reports.is_empty()
    }

    pub(crate) fn record(&mut self, report: UiActionDispatchReport) {
        self.reports.push(report);
    }
}
