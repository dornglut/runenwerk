use super::sdf_residency::RenderSdfResidencyInspection;
use super::timings::{RenderDebugTimingsState, RenderGpuTimingCapability};
use crate::plugins::render::features::world::sdf_raymarch::{
    RenderSdfRaymarchAccelerationReport, RenderSdfRaymarchDiagnosticSeverity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSdfProductionEvidenceSeverity {
    Info,
    Warning,
    Error,
}

impl RenderSdfProductionEvidenceSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfProductionEvidenceDiagnostic {
    pub severity: RenderSdfProductionEvidenceSeverity,
    pub code: String,
    pub message: String,
}

impl RenderSdfProductionEvidenceDiagnostic {
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderSdfProductionEvidenceSeverity::Warning,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderSdfProductionEvidenceSeverity::Error,
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfProductionHardwareProfile {
    pub profile_key: String,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub timestamp_query: RenderGpuTimingCapability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfProductionHardwareProfileInspection {
    pub profile_key: String,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub timestamp_query_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfRuntimeVisualEvidence {
    pub view_label: String,
    pub coverage_band: String,
    pub artifact_path: String,
    pub step_count: u32,
    pub missed_surface_risk: bool,
    pub overstep_risk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSdfProductionEvidenceCounts {
    pub selected_product_count: usize,
    pub resident_product_count: usize,
    pub resident_page_count: usize,
    pub resident_brick_count: usize,
    pub clipmap_window_count: usize,
    pub resident_bytes: u64,
    pub upload_bytes: u64,
    pub distance_mip_count: usize,
    pub candidate_list_count: usize,
    pub total_candidate_count: usize,
    pub rejected_candidate_count: usize,
    pub max_steps_per_ray: u32,
    pub visual_evidence_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderSdfProductionTimingEvidence {
    pub timing_source: String,
    pub gpu_timing_capability: String,
    pub cpu_pass_sample_count: usize,
    pub cpu_total_pass_millis: f32,
    pub cpu_slowest_pass_id: Option<String>,
    pub cpu_slowest_pass_millis: f32,
    pub gpu_pass_sample_count: usize,
    pub gpu_total_pass_millis: f32,
    pub gpu_slowest_pass_id: Option<String>,
    pub gpu_slowest_pass_millis: f32,
    pub gpu_timing_diagnostic_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderSdfProductionEvidenceReport {
    pub hardware_profile: RenderSdfProductionHardwareProfileInspection,
    pub counts: RenderSdfProductionEvidenceCounts,
    pub timings: RenderSdfProductionTimingEvidence,
    pub visual_evidence: Vec<RenderSdfRuntimeVisualEvidence>,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
    pub diagnostics: Vec<RenderSdfProductionEvidenceDiagnostic>,
}

impl RenderSdfProductionEvidenceReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == RenderSdfProductionEvidenceSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.severity == RenderSdfProductionEvidenceSeverity::Warning
            })
            .count()
    }

    pub fn is_runtime_ready(&self) -> bool {
        self.error_count() == 0
    }
}

#[derive(Debug, Clone)]
pub struct RenderSdfProductionEvidenceRequest {
    pub hardware_profile: RenderSdfProductionHardwareProfile,
    pub residency: RenderSdfResidencyInspection,
    pub raymarch: RenderSdfRaymarchAccelerationReport,
    pub timings: RenderDebugTimingsState,
    pub visual_evidence: Vec<RenderSdfRuntimeVisualEvidence>,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
}

pub fn inspect_render_sdf_production_evidence(
    request: RenderSdfProductionEvidenceRequest,
) -> RenderSdfProductionEvidenceReport {
    let hardware_profile = RenderSdfProductionHardwareProfileInspection {
        profile_key: request.hardware_profile.profile_key.clone(),
        adapter_name: request.hardware_profile.adapter_name.clone(),
        backend: request.hardware_profile.backend.clone(),
        timestamp_query_status: request
            .hardware_profile
            .timestamp_query
            .as_str()
            .to_string(),
    };
    let counts = counts_from_request(
        &request.residency,
        &request.raymarch,
        &request.visual_evidence,
    );
    let timings = timings_from_state(&request.timings);
    let mut diagnostics = Vec::new();

    validate_profile(
        &request.hardware_profile,
        &request.timings,
        &mut diagnostics,
    );
    validate_evidence_inputs(&request, &mut diagnostics);
    validate_count_invariants(&request.residency, &request.raymarch, &mut diagnostics);
    collect_source_diagnostics(&request, &mut diagnostics);

    RenderSdfProductionEvidenceReport {
        hardware_profile,
        counts,
        timings,
        visual_evidence: request.visual_evidence,
        benchmark_commands: request.benchmark_commands,
        artifact_paths: request.artifact_paths,
        diagnostics,
    }
}

fn counts_from_request(
    residency: &RenderSdfResidencyInspection,
    raymarch: &RenderSdfRaymarchAccelerationReport,
    visual_evidence: &[RenderSdfRuntimeVisualEvidence],
) -> RenderSdfProductionEvidenceCounts {
    RenderSdfProductionEvidenceCounts {
        selected_product_count: residency.selected_product_count,
        resident_product_count: residency.resident_product_count,
        resident_page_count: residency.resident_page_count,
        resident_brick_count: residency.resident_brick_count,
        clipmap_window_count: residency.clipmap_window_count,
        resident_bytes: residency.resident_bytes,
        upload_bytes: residency.upload_bytes,
        distance_mip_count: raymarch.distance_mips.len(),
        candidate_list_count: raymarch.candidate_lists.len(),
        total_candidate_count: raymarch.total_candidate_count,
        rejected_candidate_count: raymarch.rejected_candidate_count,
        max_steps_per_ray: raymarch.max_steps_per_ray,
        visual_evidence_count: visual_evidence.len(),
    }
}

fn timings_from_state(timings: &RenderDebugTimingsState) -> RenderSdfProductionTimingEvidence {
    let timing_source = if timings.gpu_pass_sample_count > 0 {
        "gpu_timestamp_query"
    } else {
        "cpu_encode_submit"
    };
    RenderSdfProductionTimingEvidence {
        timing_source: timing_source.to_string(),
        gpu_timing_capability: timings.gpu_timing_capability.as_str().to_string(),
        cpu_pass_sample_count: timings.pass_sample_count,
        cpu_total_pass_millis: timings.total_pass_millis,
        cpu_slowest_pass_id: timings.slowest_pass_id.clone(),
        cpu_slowest_pass_millis: timings.slowest_pass_millis,
        gpu_pass_sample_count: timings.gpu_pass_sample_count,
        gpu_total_pass_millis: timings.gpu_total_pass_millis,
        gpu_slowest_pass_id: timings.gpu_slowest_pass_id.clone(),
        gpu_slowest_pass_millis: timings.gpu_slowest_pass_millis,
        gpu_timing_diagnostic_count: timings.gpu_timing_diagnostics.len(),
    }
}

fn validate_profile(
    profile: &RenderSdfProductionHardwareProfile,
    timings: &RenderDebugTimingsState,
    diagnostics: &mut Vec<RenderSdfProductionEvidenceDiagnostic>,
) {
    if profile.profile_key.trim().is_empty() {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "missing_hardware_profile",
            "SDF runtime evidence requires a hardware or capability profile key",
        ));
    }

    if profile.timestamp_query != timings.gpu_timing_capability {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::warning(
            "timestamp_profile_mismatch",
            format!(
                "hardware profile reports timestamp status {}, but timing evidence reports {}",
                profile.timestamp_query.as_str(),
                timings.gpu_timing_capability.as_str()
            ),
        ));
    }

    if profile.timestamp_query != RenderGpuTimingCapability::Supported
        && timings.gpu_timing_diagnostics.is_empty()
    {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "missing_gpu_timing_diagnostic",
            "unsupported or unavailable SDF GPU timing evidence must include a typed diagnostic",
        ));
    }
}

fn validate_evidence_inputs(
    request: &RenderSdfProductionEvidenceRequest,
    diagnostics: &mut Vec<RenderSdfProductionEvidenceDiagnostic>,
) {
    if request.benchmark_commands.is_empty() {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "missing_benchmark_command",
            "SDF runtime evidence requires at least one benchmark command",
        ));
    }
    if request.artifact_paths.is_empty() {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "missing_artifact_path",
            "SDF runtime evidence requires at least one raw artifact or human report path",
        ));
    }
    if request.timings.pass_sample_count == 0 && request.timings.gpu_pass_sample_count == 0 {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "missing_timing_evidence",
            "SDF runtime evidence requires CPU or GPU pass timing evidence",
        ));
    }
    if request.visual_evidence.is_empty() {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "missing_visual_evidence",
            "SDF runtime evidence requires near, mid, far, and summary visual evidence",
        ));
    }
    for required_band in ["near", "mid", "far", "summary"] {
        if !request
            .visual_evidence
            .iter()
            .any(|evidence| evidence.coverage_band == required_band)
        {
            diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
                "missing_visual_band",
                format!("SDF runtime visual evidence is missing {required_band} coverage"),
            ));
        }
    }
    for evidence in &request.visual_evidence {
        if evidence.artifact_path.trim().is_empty() {
            diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
                "missing_visual_artifact",
                format!(
                    "SDF runtime visual evidence {} is missing an artifact path",
                    evidence.view_label
                ),
            ));
        }
        if evidence.step_count == 0 {
            diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
                "missing_step_count",
                format!(
                    "SDF runtime visual evidence {} must report a nonzero step count",
                    evidence.view_label
                ),
            ));
        }
        if evidence.missed_surface_risk {
            diagnostics.push(RenderSdfProductionEvidenceDiagnostic::warning(
                "missed_surface_risk",
                format!(
                    "SDF runtime visual evidence {} reports missed-surface risk",
                    evidence.view_label
                ),
            ));
        }
        if evidence.overstep_risk {
            diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
                "visual_overstep_risk",
                format!(
                    "SDF runtime visual evidence {} reports overstep risk",
                    evidence.view_label
                ),
            ));
        }
    }
}

fn validate_count_invariants(
    residency: &RenderSdfResidencyInspection,
    raymarch: &RenderSdfRaymarchAccelerationReport,
    diagnostics: &mut Vec<RenderSdfProductionEvidenceDiagnostic>,
) {
    if residency.resident_product_count == 0 {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "missing_resident_sdf_products",
            "SDF runtime evidence requires resident SDF products",
        ));
    }
    if !raymarch.is_acceleration_ready() {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "raymarch_not_ready",
            "SDF runtime evidence requires ready raymarch acceleration evidence",
        ));
    }
    if raymarch.resident_product_count != residency.resident_product_count {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "raymarch_residency_mismatch",
            "SDF raymarch resident product count must match residency evidence",
        ));
    }
    if raymarch.resident_page_count != residency.resident_page_count {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "raymarch_page_mismatch",
            "SDF raymarch page count must match residency evidence",
        ));
    }
    if raymarch.resident_brick_count != residency.resident_brick_count {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "raymarch_brick_mismatch",
            "SDF raymarch brick count must match residency evidence",
        ));
    }
    if raymarch.distance_mips.is_empty() {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "missing_distance_mips",
            "SDF runtime evidence requires distance mip evidence",
        ));
    }
    if raymarch.candidate_lists.is_empty() {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
            "missing_candidate_lists",
            "SDF runtime evidence requires tile/depth candidate-list evidence",
        ));
    }
}

fn collect_source_diagnostics(
    request: &RenderSdfProductionEvidenceRequest,
    diagnostics: &mut Vec<RenderSdfProductionEvidenceDiagnostic>,
) {
    if request.residency.diagnostic_count > 0 {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::warning(
            "residency_diagnostics_present",
            format!(
                "SDF residency inspection reported {} diagnostics",
                request.residency.diagnostic_count
            ),
        ));
    }
    for diagnostic in &request.raymarch.diagnostics {
        match diagnostic.severity {
            RenderSdfRaymarchDiagnosticSeverity::Error => {
                diagnostics.push(RenderSdfProductionEvidenceDiagnostic::error(
                    diagnostic.kind.as_str(),
                    diagnostic.message.clone(),
                ));
            }
            RenderSdfRaymarchDiagnosticSeverity::Warning => {
                diagnostics.push(RenderSdfProductionEvidenceDiagnostic::warning(
                    diagnostic.kind.as_str(),
                    diagnostic.message.clone(),
                ));
            }
        }
    }
    for diagnostic in &request.timings.gpu_timing_diagnostics {
        diagnostics.push(RenderSdfProductionEvidenceDiagnostic::warning(
            diagnostic.kind.as_str(),
            diagnostic.message.clone(),
        ));
    }
    for (status, code) in [
        (
            &request.residency.budget.page_status,
            "sdf_page_budget_pressure",
        ),
        (
            &request.residency.budget.brick_status,
            "sdf_brick_budget_pressure",
        ),
        (
            &request.residency.budget.resident_byte_status,
            "sdf_resident_byte_budget_pressure",
        ),
        (
            &request.residency.budget.upload_byte_status,
            "sdf_upload_byte_budget_pressure",
        ),
        (
            &request.residency.budget.clipmap_page_status,
            "sdf_clipmap_page_budget_pressure",
        ),
    ] {
        if status != "within_budget" {
            diagnostics.push(RenderSdfProductionEvidenceDiagnostic::warning(
                code,
                format!("SDF runtime evidence reports {status}"),
            ));
        }
    }
}
