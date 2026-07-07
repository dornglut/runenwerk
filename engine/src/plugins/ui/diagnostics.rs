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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRuntimeDiagnostic {
    pub code: UiRuntimeDiagnosticCode,
    pub severity: UiRuntimeDiagnosticSeverity,
    pub message: &'static str,
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
