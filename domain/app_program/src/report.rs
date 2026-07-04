//! Report and diagnostic structures for app-program proofs.

use std::collections::BTreeSet;

pub const NAMESPACE_PROGRAM_ID: &str = "app.program.id";
pub const NAMESPACE_MODEL_SCHEMA: &str = "app.model.schema";
pub const NAMESPACE_ACTION_SCHEMA: &str = "app.action.schema";
pub const NAMESPACE_ROUTE_ACTION_RESOLVE: &str = "app.route_action.resolve";
pub const NAMESPACE_REDUCER: &str = "app.reducer";
pub const NAMESPACE_EFFECT_PLAN: &str = "app.effect.plan";
pub const NAMESPACE_PROJECTION: &str = "app.projection";
pub const NAMESPACE_REPLAY: &str = "app.replay";
pub const NAMESPACE_HOST_COMPATIBILITY: &str = "app.host.compatibility";
pub const NAMESPACE_VERSION_COMPATIBILITY: &str = "app.version.compatibility";
pub const NAMESPACE_REPORT_BUDGET: &str = "app.report.budget";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AppDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppDiagnostic {
    pub namespace: String,
    pub code: String,
    pub summary: String,
    pub severity: AppDiagnosticSeverity,
    pub source: Option<String>,
}

impl AppDiagnostic {
    pub fn new(
        namespace: impl Into<String>,
        code: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            code: code.into(),
            summary: summary.into(),
            severity: AppDiagnosticSeverity::Error,
            source: None,
        }
    }

    pub fn info(
        namespace: impl Into<String>,
        code: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            code: code.into(),
            summary: summary.into(),
            severity: AppDiagnosticSeverity::Info,
            source: None,
        }
    }

    pub fn warning(
        namespace: impl Into<String>,
        code: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            code: code.into(),
            summary: summary.into(),
            severity: AppDiagnosticSeverity::Warning,
            source: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == AppDiagnosticSeverity::Error
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteActionResolutionReport {
    pub route_id: String,
    pub route_schema_version: u32,
    pub status: String,
    pub action_id: Option<String>,
    pub action_version: Option<u32>,
    pub payload_summary: String,
    pub payload_summary_truncated: bool,
    pub diagnostics: Vec<AppDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CounterReducerTraceReport {
    pub step_index: usize,
    pub accepted: bool,
    pub action_id: Option<String>,
    pub before_revision: u64,
    pub after_revision: u64,
    pub count_before: Option<i64>,
    pub count_after: Option<i64>,
    pub diagnostics: Vec<AppDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CounterViewProjectionReport {
    pub step_index: usize,
    pub model_revision: u64,
    pub screen_id: Option<String>,
    pub route_ids: Vec<String>,
    pub diagnostics: Vec<AppDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CounterEffectPlanReport {
    pub step_index: usize,
    pub plan_kind: String,
    pub effect_count: usize,
    pub diagnostics: Vec<AppDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppProgramReport {
    pub scenario_id: String,
    pub program_id: String,
    pub passed: bool,
    pub initial_revision: u64,
    pub final_revision: u64,
    pub route_reports: Vec<RouteActionResolutionReport>,
    pub reducer_reports: Vec<CounterReducerTraceReport>,
    pub projection_reports: Vec<CounterViewProjectionReport>,
    pub effect_reports: Vec<CounterEffectPlanReport>,
    pub diagnostics: Vec<AppDiagnostic>,
}

impl AppProgramReport {
    pub fn diagnostic_namespaces(&self) -> BTreeSet<String> {
        let mut namespaces = BTreeSet::new();
        for diagnostic in self.all_diagnostics() {
            namespaces.insert(diagnostic.namespace.clone());
        }
        namespaces
    }

    pub fn all_diagnostics(&self) -> Vec<&AppDiagnostic> {
        let mut diagnostics = Vec::new();
        diagnostics.extend(self.diagnostics.iter());
        for report in &self.route_reports {
            diagnostics.extend(report.diagnostics.iter());
        }
        for report in &self.reducer_reports {
            diagnostics.extend(report.diagnostics.iter());
        }
        for report in &self.projection_reports {
            diagnostics.extend(report.diagnostics.iter());
        }
        for report in &self.effect_reports {
            diagnostics.extend(report.diagnostics.iter());
        }
        diagnostics
    }
}
