use super::{
    PreparedRenderFrameInspection, RenderDebugFrameReport, RenderDebugTimingsState,
    RenderExecutionGraphPreflightInspection, RenderFragmentMergeInspection,
    RenderProductSurfaceManifestInspection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderReadinessBudgetKind {
    FrameTotalMillis,
    PreflightMillis,
    PassTotalMillis,
    DynamicTextureTargetCount,
    DynamicTextureUploadBytes,
    CaptureCount,
    CaptureFailureCount,
    PreflightErrorCount,
    FragmentErrorCount,
    ProductSurfaceDiagnosticCount,
}

impl RenderReadinessBudgetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FrameTotalMillis => "frame_total_ms",
            Self::PreflightMillis => "preflight_ms",
            Self::PassTotalMillis => "pass_total_ms",
            Self::DynamicTextureTargetCount => "dynamic_texture_target_count",
            Self::DynamicTextureUploadBytes => "dynamic_texture_upload_bytes",
            Self::CaptureCount => "capture_count",
            Self::CaptureFailureCount => "capture_failure_count",
            Self::PreflightErrorCount => "preflight_error_count",
            Self::FragmentErrorCount => "fragment_error_count",
            Self::ProductSurfaceDiagnosticCount => "product_surface_diagnostic_count",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderReadinessBudgetThreshold {
    pub kind: RenderReadinessBudgetKind,
    pub limit: f64,
}

impl RenderReadinessBudgetThreshold {
    pub fn max(kind: RenderReadinessBudgetKind, limit: f64) -> Self {
        Self { kind, limit }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderReadinessBudgetStatus {
    WithinBudget,
    OverBudget,
    NotMeasured,
}

impl RenderReadinessBudgetStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WithinBudget => "within_budget",
            Self::OverBudget => "over_budget",
            Self::NotMeasured => "not_measured",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderReadinessBudgetResult {
    pub kind: RenderReadinessBudgetKind,
    pub observed: Option<f64>,
    pub limit: f64,
    pub status: RenderReadinessBudgetStatus,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderReadinessBudgetReport {
    pub results: Vec<RenderReadinessBudgetResult>,
}

impl RenderReadinessBudgetReport {
    pub fn over_budget_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == RenderReadinessBudgetStatus::OverBudget)
            .count()
    }

    pub fn not_measured_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == RenderReadinessBudgetStatus::NotMeasured)
            .count()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderReadinessBudgetMeasurements {
    pub frame_total_ms: Option<f64>,
    pub preflight_ms: Option<f64>,
    pub pass_total_ms: Option<f64>,
    pub dynamic_texture_target_count: Option<f64>,
    pub dynamic_texture_upload_bytes: Option<f64>,
    pub capture_count: Option<f64>,
    pub capture_failure_count: Option<f64>,
    pub preflight_error_count: Option<f64>,
    pub fragment_error_count: Option<f64>,
    pub product_surface_diagnostic_count: Option<f64>,
}

impl RenderReadinessBudgetMeasurements {
    pub fn from_reports(
        prepared_frame: Option<&PreparedRenderFrameInspection>,
        product_surfaces: &[RenderProductSurfaceManifestInspection],
        preflight: Option<&RenderExecutionGraphPreflightInspection>,
        fragment_merges: &[RenderFragmentMergeInspection],
        capture_report: Option<&RenderDebugFrameReport>,
        timings: Option<&RenderDebugTimingsState>,
    ) -> Self {
        let dynamic_texture_target_count: Option<usize> = prepared_frame
            .map(|frame| frame.dynamic_texture_targets.len())
            .or_else(|| {
                if product_surfaces.is_empty() {
                    None
                } else {
                    Some(
                        product_surfaces
                            .iter()
                            .map(|surface| surface.dynamic_texture_targets.len())
                            .sum(),
                    )
                }
            });
        let dynamic_texture_upload_bytes: Option<usize> = if product_surfaces.is_empty() {
            None
        } else {
            Some(
                product_surfaces
                    .iter()
                    .flat_map(|surface| surface.dynamic_texture_uploads.iter())
                    .map(|upload| upload.byte_len)
                    .sum(),
            )
        };
        Self {
            frame_total_ms: timings.map(|value| f64::from(value.total_ms)),
            preflight_ms: timings.map(|value| f64::from(value.preflight_ms)),
            pass_total_ms: timings.map(|value| f64::from(value.total_pass_millis)),
            dynamic_texture_target_count: dynamic_texture_target_count.map(|value| value as f64),
            dynamic_texture_upload_bytes: dynamic_texture_upload_bytes.map(|value| value as f64),
            capture_count: capture_report.map(|report| report.capture_results.len() as f64),
            capture_failure_count: capture_report.map(|report| {
                report
                    .capture_results
                    .iter()
                    .filter(|result| result.terminal.code.is_failure())
                    .count() as f64
            }),
            preflight_error_count: preflight.map(|report| report.error_count as f64),
            fragment_error_count: Some(
                fragment_merges
                    .iter()
                    .map(|report| report.error_count)
                    .sum::<usize>() as f64,
            ),
            product_surface_diagnostic_count: if product_surfaces.is_empty() {
                None
            } else {
                Some(
                    product_surfaces
                        .iter()
                        .map(|surface| surface.diagnostics.len())
                        .sum::<usize>() as f64,
                )
            },
        }
    }

    pub fn measure(&self, kind: RenderReadinessBudgetKind) -> Option<f64> {
        match kind {
            RenderReadinessBudgetKind::FrameTotalMillis => self.frame_total_ms,
            RenderReadinessBudgetKind::PreflightMillis => self.preflight_ms,
            RenderReadinessBudgetKind::PassTotalMillis => self.pass_total_ms,
            RenderReadinessBudgetKind::DynamicTextureTargetCount => {
                self.dynamic_texture_target_count
            }
            RenderReadinessBudgetKind::DynamicTextureUploadBytes => {
                self.dynamic_texture_upload_bytes
            }
            RenderReadinessBudgetKind::CaptureCount => self.capture_count,
            RenderReadinessBudgetKind::CaptureFailureCount => self.capture_failure_count,
            RenderReadinessBudgetKind::PreflightErrorCount => self.preflight_error_count,
            RenderReadinessBudgetKind::FragmentErrorCount => self.fragment_error_count,
            RenderReadinessBudgetKind::ProductSurfaceDiagnosticCount => {
                self.product_surface_diagnostic_count
            }
        }
    }
}

pub fn evaluate_render_readiness_budgets(
    measurements: &RenderReadinessBudgetMeasurements,
    thresholds: &[RenderReadinessBudgetThreshold],
) -> RenderReadinessBudgetReport {
    RenderReadinessBudgetReport {
        results: thresholds
            .iter()
            .map(|threshold| evaluate_threshold(measurements, *threshold))
            .collect(),
    }
}

fn evaluate_threshold(
    measurements: &RenderReadinessBudgetMeasurements,
    threshold: RenderReadinessBudgetThreshold,
) -> RenderReadinessBudgetResult {
    match measurements.measure(threshold.kind) {
        Some(observed) if observed > threshold.limit => RenderReadinessBudgetResult {
            kind: threshold.kind,
            observed: Some(observed),
            limit: threshold.limit,
            status: RenderReadinessBudgetStatus::OverBudget,
            message: format!(
                "{} observed {} exceeds limit {}",
                threshold.kind.as_str(),
                observed,
                threshold.limit
            ),
        },
        Some(observed) => RenderReadinessBudgetResult {
            kind: threshold.kind,
            observed: Some(observed),
            limit: threshold.limit,
            status: RenderReadinessBudgetStatus::WithinBudget,
            message: format!(
                "{} observed {} within limit {}",
                threshold.kind.as_str(),
                observed,
                threshold.limit
            ),
        },
        None => RenderReadinessBudgetResult {
            kind: threshold.kind,
            observed: None,
            limit: threshold.limit,
            status: RenderReadinessBudgetStatus::NotMeasured,
            message: format!("{} was not measured", threshold.kind.as_str()),
        },
    }
}
