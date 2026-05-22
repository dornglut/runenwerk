use super::{
    PreparedRenderFrameInspection, RenderCapturePointIdentity, RenderCaptureTerminalCode,
    RenderDebugFrameReport, RenderDebugTimingsState, RenderExecutionGraphPreflightInspection,
    RenderFragmentMergeInspection, RenderGpuTimingCapability,
    RenderProductSurfaceManifestInspection, RenderReadinessBudgetReport,
    RenderReadinessBudgetStatus,
};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderReadinessDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl RenderReadinessDiagnosticSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderReadinessDiagnosticKind {
    MissingPreparedFrameInspection,
    MissingSurfaceIdentity,
    PreflightErrors,
    ProductSurfaceDiagnostics,
    FragmentErrors,
    CaptureFailures,
    CaptureInvariantViolation,
    CaptureReplayManifestInvalid,
    BudgetOverrun,
    BudgetNotMeasured,
    GpuTimingDiagnostics,
}

impl RenderReadinessDiagnosticKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingPreparedFrameInspection => "missing_prepared_frame_inspection",
            Self::MissingSurfaceIdentity => "missing_surface_identity",
            Self::PreflightErrors => "preflight_errors",
            Self::ProductSurfaceDiagnostics => "product_surface_diagnostics",
            Self::FragmentErrors => "fragment_errors",
            Self::CaptureFailures => "capture_failures",
            Self::CaptureInvariantViolation => "capture_invariant_violation",
            Self::CaptureReplayManifestInvalid => "capture_replay_manifest_invalid",
            Self::BudgetOverrun => "budget_overrun",
            Self::BudgetNotMeasured => "budget_not_measured",
            Self::GpuTimingDiagnostics => "gpu_timing_diagnostics",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReadinessDiagnostic {
    pub severity: RenderReadinessDiagnosticSeverity,
    pub kind: RenderReadinessDiagnosticKind,
    pub source_report: String,
    pub scope: Option<String>,
    pub frame_index: Option<u64>,
    pub render_surface_id: Option<u64>,
    pub native_window_id: Option<u64>,
    pub producer_id: Option<u64>,
    pub flow_id: Option<String>,
    pub pass_id: Option<String>,
    pub view_id: Option<String>,
    pub invocation_id: Option<String>,
    pub fragment_package_id: Option<String>,
    pub capture_point: Option<RenderCapturePointIdentity>,
    pub budget_kind: Option<String>,
    pub message: String,
}

impl RenderReadinessDiagnostic {
    pub fn new(
        severity: RenderReadinessDiagnosticSeverity,
        kind: RenderReadinessDiagnosticKind,
        source_report: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            kind,
            source_report: source_report.into(),
            scope: None,
            frame_index: None,
            render_surface_id: None,
            native_window_id: None,
            producer_id: None,
            flow_id: None,
            pass_id: None,
            view_id: None,
            invocation_id: None,
            fragment_package_id: None,
            capture_point: None,
            budget_kind: None,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderReadinessSourceReportSummary {
    pub prepared_frame_present: bool,
    pub product_surface_manifest_count: usize,
    pub product_surface_diagnostic_count: usize,
    pub preflight_diagnostic_count: usize,
    pub preflight_error_count: usize,
    pub fragment_merge_count: usize,
    pub fragment_error_count: usize,
    pub capture_result_count: usize,
    pub capture_failure_count: usize,
    pub budget_result_count: usize,
    pub budget_overrun_count: usize,
    pub gpu_pass_sample_count: usize,
    pub gpu_timing_diagnostic_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct RenderReadinessReportRequest {
    pub prepared_frame: Option<PreparedRenderFrameInspection>,
    pub product_surfaces: Vec<RenderProductSurfaceManifestInspection>,
    pub preflight: Option<RenderExecutionGraphPreflightInspection>,
    pub fragment_merges: Vec<RenderFragmentMergeInspection>,
    pub capture_report: Option<RenderDebugFrameReport>,
    pub timings: Option<RenderDebugTimingsState>,
    pub budget_report: RenderReadinessBudgetReport,
    pub replay_validation: Option<RenderReplayManifestValidation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderReadinessReport {
    pub frame_index: Option<u64>,
    pub render_surface_id: Option<u64>,
    pub native_window_id: Option<u64>,
    pub source_reports: RenderReadinessSourceReportSummary,
    pub diagnostics: Vec<RenderReadinessDiagnostic>,
    pub budget_report: RenderReadinessBudgetReport,
    pub replay_validation: Option<RenderReplayManifestValidation>,
}

impl RenderReadinessReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == RenderReadinessDiagnosticSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == RenderReadinessDiagnosticSeverity::Warning)
            .count()
    }

    pub fn is_ready(&self) -> bool {
        self.error_count() == 0
    }
}

pub fn inspect_render_readiness(request: RenderReadinessReportRequest) -> RenderReadinessReport {
    let frame_index = request
        .prepared_frame
        .as_ref()
        .map(|frame| frame.frame_index);
    let render_surface_id = request
        .prepared_frame
        .as_ref()
        .map(|frame| frame.render_surface_id);
    let native_window_id = request
        .prepared_frame
        .as_ref()
        .and_then(|frame| frame.native_window_id);

    let mut diagnostics = Vec::<RenderReadinessDiagnostic>::new();
    if request.prepared_frame.is_none() {
        diagnostics.push(RenderReadinessDiagnostic::new(
            RenderReadinessDiagnosticSeverity::Error,
            RenderReadinessDiagnosticKind::MissingPreparedFrameInspection,
            "prepared_frame",
            "readiness inspection requires a prepared-frame inspection source report",
        ));
    } else if native_window_id.is_none() {
        diagnostics.push(RenderReadinessDiagnostic {
            render_surface_id,
            frame_index,
            ..RenderReadinessDiagnostic::new(
                RenderReadinessDiagnosticSeverity::Warning,
                RenderReadinessDiagnosticKind::MissingSurfaceIdentity,
                "prepared_frame",
                "prepared frame has no native window identity",
            )
        });
    }

    if let Some(preflight) = &request.preflight
        && preflight.error_count > 0
    {
        for diagnostic in &preflight.diagnostics {
            diagnostics.push(RenderReadinessDiagnostic {
                flow_id: diagnostic.flow_id.clone(),
                pass_id: diagnostic.pass_id.clone(),
                view_id: diagnostic.view_id.clone(),
                invocation_id: diagnostic.invocation_id.clone(),
                frame_index,
                render_surface_id,
                native_window_id,
                ..RenderReadinessDiagnostic::new(
                    RenderReadinessDiagnosticSeverity::Error,
                    RenderReadinessDiagnosticKind::PreflightErrors,
                    "render_execution_graph_preflight",
                    diagnostic.message.clone(),
                )
            });
        }
    }

    for surface in &request.product_surfaces {
        for diagnostic in &surface.diagnostics {
            diagnostics.push(RenderReadinessDiagnostic {
                producer_id: Some(diagnostic.producer_id),
                view_id: diagnostic.view_id.clone(),
                invocation_id: diagnostic.invocation_id.clone(),
                frame_index,
                render_surface_id,
                native_window_id,
                ..RenderReadinessDiagnostic::new(
                    readiness_severity_from_product_surface(diagnostic.severity.as_str()),
                    RenderReadinessDiagnosticKind::ProductSurfaceDiagnostics,
                    "product_surface_manifest",
                    diagnostic.message.clone(),
                )
            });
        }
    }

    for merge in &request.fragment_merges {
        if merge.error_count > 0 {
            diagnostics.push(RenderReadinessDiagnostic {
                fragment_package_id: merge.package_id.clone(),
                frame_index,
                render_surface_id,
                native_window_id,
                ..RenderReadinessDiagnostic::new(
                    RenderReadinessDiagnosticSeverity::Error,
                    RenderReadinessDiagnosticKind::FragmentErrors,
                    "fragment_merge",
                    format!(
                        "fragment package {} has {} merge diagnostics",
                        merge.package_id.as_deref().unwrap_or("<unknown>"),
                        merge.error_count
                    ),
                )
            });
        }
    }

    if let Some(capture_report) = &request.capture_report {
        for violation in capture_report.validate_invariants() {
            diagnostics.push(RenderReadinessDiagnostic {
                frame_index: Some(capture_report.frame_index),
                render_surface_id,
                native_window_id,
                ..RenderReadinessDiagnostic::new(
                    RenderReadinessDiagnosticSeverity::Error,
                    RenderReadinessDiagnosticKind::CaptureInvariantViolation,
                    "capture_report",
                    violation,
                )
            });
        }
        for result in &capture_report.capture_results {
            if result.terminal.code.is_failure() {
                diagnostics.push(RenderReadinessDiagnostic {
                    frame_index: Some(capture_report.frame_index),
                    render_surface_id,
                    native_window_id,
                    flow_id: Some(result.capture_point.flow_id.clone()),
                    pass_id: Some(result.capture_point.pass_id.clone()),
                    capture_point: Some(result.capture_point.clone()),
                    ..RenderReadinessDiagnostic::new(
                        capture_failure_severity(result.terminal.code),
                        RenderReadinessDiagnosticKind::CaptureFailures,
                        "capture_report",
                        result
                            .terminal
                            .reason
                            .as_ref()
                            .map(|reason| reason.detail.clone())
                            .unwrap_or_else(|| {
                                format!("capture terminal {:?}", result.terminal.code)
                            }),
                    )
                });
            }
        }
    }

    for result in &request.budget_report.results {
        match result.status {
            RenderReadinessBudgetStatus::OverBudget => {
                diagnostics.push(RenderReadinessDiagnostic {
                    budget_kind: Some(result.kind.as_str().to_string()),
                    frame_index,
                    render_surface_id,
                    native_window_id,
                    ..RenderReadinessDiagnostic::new(
                        RenderReadinessDiagnosticSeverity::Warning,
                        RenderReadinessDiagnosticKind::BudgetOverrun,
                        "budget_report",
                        result.message.clone(),
                    )
                });
            }
            RenderReadinessBudgetStatus::NotMeasured => {
                diagnostics.push(RenderReadinessDiagnostic {
                    budget_kind: Some(result.kind.as_str().to_string()),
                    frame_index,
                    render_surface_id,
                    native_window_id,
                    ..RenderReadinessDiagnostic::new(
                        RenderReadinessDiagnosticSeverity::Warning,
                        RenderReadinessDiagnosticKind::BudgetNotMeasured,
                        "budget_report",
                        result.message.clone(),
                    )
                });
            }
            RenderReadinessBudgetStatus::WithinBudget => {}
        }
    }

    if let Some(timings) = &request.timings {
        for diagnostic in &timings.gpu_timing_diagnostics {
            diagnostics.push(RenderReadinessDiagnostic {
                frame_index: diagnostic.frame_index.or(frame_index),
                render_surface_id: diagnostic.render_surface_id.or(render_surface_id),
                native_window_id,
                flow_id: diagnostic.flow_id.clone(),
                pass_id: diagnostic.pass_id.clone(),
                ..RenderReadinessDiagnostic::new(
                    gpu_timing_readiness_severity(diagnostic.capability),
                    RenderReadinessDiagnosticKind::GpuTimingDiagnostics,
                    "render_gpu_timing",
                    diagnostic.message.clone(),
                )
            });
        }
    }

    if let Some(validation) = &request.replay_validation {
        diagnostics.extend(validation.diagnostics.iter().cloned());
    }

    RenderReadinessReport {
        frame_index,
        render_surface_id,
        native_window_id,
        source_reports: summarize_sources(&request),
        diagnostics,
        budget_report: request.budget_report,
        replay_validation: request.replay_validation,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReplayArtifactReference {
    pub capture_point: RenderCapturePointIdentity,
    pub artifact_path: Option<PathBuf>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReplayManifest {
    pub manifest_id: String,
    pub frame_index: u64,
    pub capability_profile_key: Option<String>,
    pub prepared_frame_digest: Option<String>,
    pub artifacts: Vec<RenderReplayArtifactReference>,
}

impl RenderReplayManifest {
    pub fn new(manifest_id: impl Into<String>, frame_index: u64) -> Self {
        Self {
            manifest_id: manifest_id.into(),
            frame_index,
            capability_profile_key: None,
            prepared_frame_digest: None,
            artifacts: Vec::new(),
        }
    }

    pub fn with_capability_profile(mut self, profile_key: impl Into<String>) -> Self {
        self.capability_profile_key = Some(profile_key.into());
        self
    }

    pub fn with_prepared_frame_digest(mut self, digest: impl Into<String>) -> Self {
        self.prepared_frame_digest = Some(digest.into());
        self
    }

    pub fn with_artifact(mut self, artifact: RenderReplayArtifactReference) -> Self {
        self.artifacts.push(artifact);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderReplayManifestStatus {
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReplayManifestValidation {
    pub manifest_id: String,
    pub status: RenderReplayManifestStatus,
    pub diagnostics: Vec<RenderReadinessDiagnostic>,
}

pub fn validate_render_replay_manifest(
    manifest: &RenderReplayManifest,
) -> RenderReplayManifestValidation {
    let mut diagnostics = Vec::<RenderReadinessDiagnostic>::new();
    if manifest
        .capability_profile_key
        .as_deref()
        .unwrap_or("")
        .is_empty()
    {
        diagnostics.push(replay_manifest_error(
            manifest,
            None,
            "replay manifest is missing a backend capability profile key",
        ));
    }
    if manifest
        .prepared_frame_digest
        .as_deref()
        .unwrap_or("")
        .is_empty()
    {
        diagnostics.push(replay_manifest_error(
            manifest,
            None,
            "replay manifest is missing a prepared-frame digest",
        ));
    }
    if manifest.artifacts.is_empty() {
        diagnostics.push(replay_manifest_error(
            manifest,
            None,
            "replay manifest has no renderer capture artifacts",
        ));
    }
    for artifact in &manifest.artifacts {
        if artifact.artifact_path.is_none() {
            diagnostics.push(replay_manifest_error(
                manifest,
                Some(artifact.capture_point.clone()),
                "replay artifact is missing an artifact path",
            ));
        }
        if artifact.format.as_deref().unwrap_or("").is_empty() {
            diagnostics.push(replay_manifest_error(
                manifest,
                Some(artifact.capture_point.clone()),
                "replay artifact is missing a texture format",
            ));
        }
    }

    RenderReplayManifestValidation {
        manifest_id: manifest.manifest_id.clone(),
        status: if diagnostics.is_empty() {
            RenderReplayManifestStatus::Valid
        } else {
            RenderReplayManifestStatus::Invalid
        },
        diagnostics,
    }
}

fn summarize_sources(request: &RenderReadinessReportRequest) -> RenderReadinessSourceReportSummary {
    RenderReadinessSourceReportSummary {
        prepared_frame_present: request.prepared_frame.is_some(),
        product_surface_manifest_count: request.product_surfaces.len(),
        product_surface_diagnostic_count: request
            .product_surfaces
            .iter()
            .map(|surface| surface.diagnostics.len())
            .sum(),
        preflight_diagnostic_count: request
            .preflight
            .as_ref()
            .map(|preflight| preflight.diagnostic_count)
            .unwrap_or(0),
        preflight_error_count: request
            .preflight
            .as_ref()
            .map(|preflight| preflight.error_count)
            .unwrap_or(0),
        fragment_merge_count: request.fragment_merges.len(),
        fragment_error_count: request
            .fragment_merges
            .iter()
            .map(|merge| merge.error_count)
            .sum(),
        capture_result_count: request
            .capture_report
            .as_ref()
            .map(|report| report.capture_results.len())
            .unwrap_or(0),
        capture_failure_count: request
            .capture_report
            .as_ref()
            .map(|report| {
                report
                    .capture_results
                    .iter()
                    .filter(|result| result.terminal.code.is_failure())
                    .count()
            })
            .unwrap_or(0),
        budget_result_count: request.budget_report.results.len(),
        budget_overrun_count: request.budget_report.over_budget_count(),
        gpu_pass_sample_count: request
            .timings
            .as_ref()
            .map(|timings| timings.gpu_pass_sample_count)
            .unwrap_or(0),
        gpu_timing_diagnostic_count: request
            .timings
            .as_ref()
            .map(|timings| timings.gpu_timing_diagnostics.len())
            .unwrap_or(0),
    }
}

fn gpu_timing_readiness_severity(
    capability: RenderGpuTimingCapability,
) -> RenderReadinessDiagnosticSeverity {
    match capability {
        RenderGpuTimingCapability::Supported => RenderReadinessDiagnosticSeverity::Info,
        RenderGpuTimingCapability::ReadbackPending
        | RenderGpuTimingCapability::UnavailableThisFrame
        | RenderGpuTimingCapability::Unsupported => RenderReadinessDiagnosticSeverity::Warning,
    }
}

fn readiness_severity_from_product_surface(value: &str) -> RenderReadinessDiagnosticSeverity {
    if value == "error" {
        RenderReadinessDiagnosticSeverity::Error
    } else {
        RenderReadinessDiagnosticSeverity::Warning
    }
}

fn capture_failure_severity(code: RenderCaptureTerminalCode) -> RenderReadinessDiagnosticSeverity {
    match code {
        RenderCaptureTerminalCode::Disabled | RenderCaptureTerminalCode::Skipped => {
            RenderReadinessDiagnosticSeverity::Warning
        }
        RenderCaptureTerminalCode::Completed => RenderReadinessDiagnosticSeverity::Info,
        RenderCaptureTerminalCode::Unmatched
        | RenderCaptureTerminalCode::Unsupported
        | RenderCaptureTerminalCode::ReadbackFailed
        | RenderCaptureTerminalCode::ExportFailed => RenderReadinessDiagnosticSeverity::Error,
    }
}

fn replay_manifest_error(
    manifest: &RenderReplayManifest,
    capture_point: Option<RenderCapturePointIdentity>,
    message: impl Into<String>,
) -> RenderReadinessDiagnostic {
    RenderReadinessDiagnostic {
        frame_index: Some(manifest.frame_index),
        capture_point,
        scope: Some(manifest.manifest_id.clone()),
        ..RenderReadinessDiagnostic::new(
            RenderReadinessDiagnosticSeverity::Error,
            RenderReadinessDiagnosticKind::CaptureReplayManifestInvalid,
            "capture_replay_manifest",
            message,
        )
    }
}
