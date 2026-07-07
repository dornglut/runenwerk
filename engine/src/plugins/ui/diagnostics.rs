use super::{UiMountFailureReason, UiMountSource};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMountDiagnostic {
    pub screen_identity: String,
    pub mount_source: UiMountSource,
    pub failure_reason: UiMountFailureReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRuntimeDiagnostic {
    pub code: UiRuntimeDiagnosticCode,
    pub severity: UiRuntimeDiagnosticSeverity,
    pub message: &'static str,
    pub mount: Option<UiMountDiagnostic>,
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
