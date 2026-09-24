use super::gpu_residency::RenderGpuResidencyInspection;
use super::scale_visibility::{
    RenderScaleVisibilityCapabilityStatus, RenderScaleVisibilityInspection,
};
use super::timings::{RenderDebugTimingsState, RenderGpuTimingCapability};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderScaleProductionEvidenceSeverity {
    Info,
    Warning,
    Error,
}

impl RenderScaleProductionEvidenceSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderScaleProductionEvidenceDiagnostic {
    pub severity: RenderScaleProductionEvidenceSeverity,
    pub code: String,
    pub message: String,
}

impl RenderScaleProductionEvidenceDiagnostic {
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderScaleProductionEvidenceSeverity::Info,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderScaleProductionEvidenceSeverity::Warning,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderScaleProductionEvidenceSeverity::Error,
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderScaleProductionHardwareProfile {
    pub profile_key: String,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub timestamp_query: RenderGpuTimingCapability,
    pub storage_compaction: RenderScaleVisibilityCapabilityStatus,
    pub indirect_submission: RenderScaleVisibilityCapabilityStatus,
    pub readback: RenderScaleVisibilityCapabilityStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderScaleProductionHardwareProfileInspection {
    pub profile_key: String,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub timestamp_query_status: String,
    pub storage_compaction_status: String,
    pub indirect_submission_status: String,
    pub readback_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderScaleProductionEvidenceCounts {
    pub addressable_count: usize,
    pub selected_count: usize,
    pub requested_count: usize,
    pub accepted_count: usize,
    pub resident_count: usize,
    pub resident_bytes: u64,
    pub upload_bytes: u64,
    pub visible_count: usize,
    pub culled_count: usize,
    pub compacted_count: usize,
    pub submitted_draw_count: usize,
    pub indirect_command_count: usize,
    pub resident_entry_budget_status: String,
    pub resident_byte_budget_status: String,
    pub upload_byte_budget_status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderScaleProductionTimingEvidence {
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
pub struct RenderScaleProductionEvidenceReport {
    pub hardware_profile: RenderScaleProductionHardwareProfileInspection,
    pub counts: RenderScaleProductionEvidenceCounts,
    pub timings: RenderScaleProductionTimingEvidence,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
    pub diagnostics: Vec<RenderScaleProductionEvidenceDiagnostic>,
}

impl RenderScaleProductionEvidenceReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.severity == RenderScaleProductionEvidenceSeverity::Error
            })
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.severity == RenderScaleProductionEvidenceSeverity::Warning
            })
            .count()
    }

    pub fn is_runtime_ready(&self) -> bool {
        self.error_count() == 0
    }
}

#[derive(Debug, Clone)]
pub struct RenderScaleProductionEvidenceRequest {
    pub hardware_profile: RenderScaleProductionHardwareProfile,
    pub residency: RenderGpuResidencyInspection,
    pub visibility: RenderScaleVisibilityInspection,
    pub timings: RenderDebugTimingsState,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
}

pub fn inspect_render_scale_production_evidence(
    request: RenderScaleProductionEvidenceRequest,
) -> RenderScaleProductionEvidenceReport {
    let hardware_profile = inspect_hardware_profile(&request.hardware_profile);
    let counts = counts_from_request(&request.residency, &request.visibility);
    let timings = timings_from_state(&request.timings);
    let mut diagnostics = Vec::new();

    validate_profile(
        &request.hardware_profile,
        &request.timings,
        &mut diagnostics,
    );
    validate_evidence_inputs(&request, &mut diagnostics);
    validate_count_invariants(&counts, &mut diagnostics);
    collect_source_diagnostics(&request, &mut diagnostics);

    RenderScaleProductionEvidenceReport {
        hardware_profile,
        counts,
        timings,
        benchmark_commands: request.benchmark_commands,
        artifact_paths: request.artifact_paths,
        diagnostics,
    }
}

fn inspect_hardware_profile(
    profile: &RenderScaleProductionHardwareProfile,
) -> RenderScaleProductionHardwareProfileInspection {
    RenderScaleProductionHardwareProfileInspection {
        profile_key: profile.profile_key.clone(),
        adapter_name: profile.adapter_name.clone(),
        backend: profile.backend.clone(),
        timestamp_query_status: profile.timestamp_query.as_str().to_string(),
        storage_compaction_status: profile.storage_compaction.as_str().to_string(),
        indirect_submission_status: profile.indirect_submission.as_str().to_string(),
        readback_status: profile.readback.as_str().to_string(),
    }
}

fn counts_from_request(
    residency: &RenderGpuResidencyInspection,
    visibility: &RenderScaleVisibilityInspection,
) -> RenderScaleProductionEvidenceCounts {
    RenderScaleProductionEvidenceCounts {
        addressable_count: residency.addressable_count,
        selected_count: residency.selected_count,
        requested_count: residency.requested_count,
        accepted_count: residency.accepted_count,
        resident_count: residency.resident_count,
        resident_bytes: residency.resident_bytes,
        upload_bytes: residency.upload_bytes,
        visible_count: visibility.visible_count,
        culled_count: visibility.culled_count,
        compacted_count: visibility.compacted_count,
        submitted_draw_count: visibility.submitted_draw_count,
        indirect_command_count: visibility.indirect_command_count,
        resident_entry_budget_status: residency.budget.resident_entry_status.clone(),
        resident_byte_budget_status: residency.budget.resident_byte_status.clone(),
        upload_byte_budget_status: residency.budget.upload_byte_status.clone(),
    }
}

fn timings_from_state(timings: &RenderDebugTimingsState) -> RenderScaleProductionTimingEvidence {
    let timing_source = if timings.gpu_pass_sample_count > 0 {
        "gpu_timestamp_query"
    } else {
        "cpu_encode_submit"
    };
    RenderScaleProductionTimingEvidence {
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
    profile: &RenderScaleProductionHardwareProfile,
    timings: &RenderDebugTimingsState,
    diagnostics: &mut Vec<RenderScaleProductionEvidenceDiagnostic>,
) {
    if profile.profile_key.trim().is_empty() {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "missing_hardware_profile",
            "renderer scale evidence requires a hardware or capability profile key",
        ));
    }

    if profile.timestamp_query != timings.gpu_timing_capability {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::warning(
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
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "missing_gpu_timing_diagnostic",
            "unsupported or unavailable GPU timing evidence must include a typed diagnostic",
        ));
    }

    if profile.readback == RenderScaleVisibilityCapabilityStatus::Unsupported {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::warning(
            "readback_unsupported",
            "hardware profile reports readback unsupported; runtime evidence must remain degraded and explicit",
        ));
    }
}

fn validate_evidence_inputs(
    request: &RenderScaleProductionEvidenceRequest,
    diagnostics: &mut Vec<RenderScaleProductionEvidenceDiagnostic>,
) {
    if request.benchmark_commands.is_empty() {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "missing_benchmark_command",
            "renderer scale production evidence requires at least one benchmark command",
        ));
    }
    if request.artifact_paths.is_empty() {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "missing_artifact_path",
            "renderer scale production evidence requires at least one raw artifact or human report path",
        ));
    }
    if request.timings.pass_sample_count == 0 && request.timings.gpu_pass_sample_count == 0 {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "missing_timing_evidence",
            "renderer scale production evidence requires CPU or GPU pass timing evidence",
        ));
    }
    if request.visibility.resident_count != request.residency.resident_count {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "visibility_resident_mismatch",
            "visibility evidence resident count must match residency evidence resident count",
        ));
    }
}

fn validate_count_invariants(
    counts: &RenderScaleProductionEvidenceCounts,
    diagnostics: &mut Vec<RenderScaleProductionEvidenceDiagnostic>,
) {
    if counts.selected_count > counts.addressable_count {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "selected_exceeds_addressable",
            "selected renderer records exceed addressable product records",
        ));
    }
    if counts.accepted_count > counts.requested_count {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "accepted_exceeds_requested",
            "accepted residency requests exceed requested residency records",
        ));
    }
    if counts.resident_count > counts.selected_count {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "resident_exceeds_selected",
            "resident renderer records exceed selected product records",
        ));
    }
    if counts.visible_count > counts.resident_count {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "visible_exceeds_resident",
            "visible renderer candidates exceed resident renderer records",
        ));
    }
    if counts.compacted_count > counts.visible_count {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "compacted_exceeds_visible",
            "compacted renderer candidates exceed visible candidates",
        ));
    }
    if counts.submitted_draw_count > counts.compacted_count {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "submitted_exceeds_compacted",
            "submitted renderer work exceeds compacted visible work",
        ));
    }
    if counts.submitted_draw_count > 0 && counts.indirect_command_count == 0 {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::error(
            "submitted_without_indirect_command",
            "submitted renderer work has no indirect command evidence",
        ));
    }
}

fn collect_source_diagnostics(
    request: &RenderScaleProductionEvidenceRequest,
    diagnostics: &mut Vec<RenderScaleProductionEvidenceDiagnostic>,
) {
    if request.residency.diagnostic_count > 0 {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::warning(
            "residency_diagnostics_present",
            format!(
                "residency inspection reported {} diagnostics",
                request.residency.diagnostic_count
            ),
        ));
    }
    for diagnostic in &request.visibility.diagnostics {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::warning(
            format!("visibility_{}", diagnostic.code),
            diagnostic.message.clone(),
        ));
    }
    for diagnostic in &request.timings.gpu_timing_diagnostics {
        diagnostics.push(RenderScaleProductionEvidenceDiagnostic::warning(
            diagnostic.kind.as_str(),
            diagnostic.message.clone(),
        ));
    }
}
