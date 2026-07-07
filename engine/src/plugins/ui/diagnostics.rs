use super::{
    UiActionDispatchFailureReason, UiMountFailureReason, UiMountSource, UiTypedIdentityError,
};

use ui_hosts::HostKind;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiRuntimeDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiRuntimeDiagnosticCode {
    PluginInstall,
    ResourceInitialization,
    MountRequestRejected,
    TypedContractRejected,
    ActionDispatchRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMountDiagnostic {
    pub screen_identity: String,
    pub mount_source: UiMountSource,
    pub failure_reason: UiMountFailureReason,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiTypedContractKind {
    Screen,
    Source,
    Action,
    HostIntent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiTypedContractFailureReason {
    InvalidIdentity(UiTypedIdentityError),
    MissingSourceFact,
    MissingActionCapability,
    HostIntentRejected,
}

impl UiTypedContractFailureReason {
    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidIdentity(_) => "typed UI contract identity is invalid",
            Self::MissingSourceFact => "typed UI source did not produce required source facts",
            Self::MissingActionCapability => "typed UI action did not declare a route capability",
            Self::HostIntentRejected => "typed UI host intent was rejected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTypedContractDiagnostic {
    pub contract: UiTypedContractKind,
    pub identity: String,
    pub failure_reason: UiTypedContractFailureReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiActionDispatchDiagnostic {
    pub action_id: String,
    pub route: String,
    pub host: HostKind,
    pub failure_reason: UiActionDispatchFailureReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRuntimeDiagnostic {
    pub code: UiRuntimeDiagnosticCode,
    pub severity: UiRuntimeDiagnosticSeverity,
    pub message: &'static str,
    pub mount: Option<UiMountDiagnostic>,
    pub typed_contract: Option<UiTypedContractDiagnostic>,
    pub action_dispatch: Option<UiActionDispatchDiagnostic>,
}

impl UiRuntimeDiagnostic {
    pub fn new(
        code: UiRuntimeDiagnosticCode,
        severity: UiRuntimeDiagnosticSeverity,
        message: &'static str,
    ) -> Self {
        Self {
            code,
            severity,
            message,
            mount: None,
            typed_contract: None,
            action_dispatch: None,
        }
    }

    pub fn mount_rejected(
        screen_identity: impl Into<String>,
        mount_source: UiMountSource,
        failure_reason: UiMountFailureReason,
    ) -> Self {
        Self {
            code: UiRuntimeDiagnosticCode::MountRequestRejected,
            severity: UiRuntimeDiagnosticSeverity::Error,
            message: failure_reason.message(),
            mount: Some(UiMountDiagnostic {
                screen_identity: screen_identity.into(),
                mount_source,
                failure_reason,
            }),
            typed_contract: None,
            action_dispatch: None,
        }
    }

    pub fn typed_contract_rejected(
        contract: UiTypedContractKind,
        identity: impl Into<String>,
        failure_reason: UiTypedContractFailureReason,
    ) -> Self {
        Self {
            code: UiRuntimeDiagnosticCode::TypedContractRejected,
            severity: UiRuntimeDiagnosticSeverity::Error,
            message: failure_reason.message(),
            mount: None,
            typed_contract: Some(UiTypedContractDiagnostic {
                contract,
                identity: identity.into(),
                failure_reason,
            }),
            action_dispatch: None,
        }
    }

    pub fn action_dispatch_rejected(
        action_id: impl Into<String>,
        route: impl Into<String>,
        host: HostKind,
        failure_reason: UiActionDispatchFailureReason,
    ) -> Self {
        Self {
            code: UiRuntimeDiagnosticCode::ActionDispatchRejected,
            severity: UiRuntimeDiagnosticSeverity::Error,
            message: failure_reason.message(),
            mount: None,
            typed_contract: None,
            action_dispatch: Some(UiActionDispatchDiagnostic {
                action_id: action_id.into(),
                route: route.into(),
                host,
                failure_reason,
            }),
        }
    }
}

/// Diagnostics collected by the UI runtime foundation.
#[derive(Debug, Clone, PartialEq, Eq, Default, ecs::Resource)]
pub struct UiRuntimeDiagnosticsResource {
    entries: Vec<UiRuntimeDiagnostic>,
}

impl UiRuntimeDiagnosticsResource {
    pub fn entries(&self) -> &[UiRuntimeDiagnostic] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push(&mut self, diagnostic: UiRuntimeDiagnostic) {
        self.entries.push(diagnostic);
    }
}
