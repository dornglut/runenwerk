use super::{
    RenderDebugTimingsState, RenderGpuTimingCapability, RenderTemporalDiagnosticSeverity,
    RenderTemporalInspection, RenderTemporalReconstructionMode, RenderTemporalUpscalingInspection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTemporalProductionEvidenceSeverity {
    Info,
    Warning,
    Error,
}

impl RenderTemporalProductionEvidenceSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTemporalProductionEvidenceDiagnostic {
    pub severity: RenderTemporalProductionEvidenceSeverity,
    pub code: String,
    pub message: String,
}

impl RenderTemporalProductionEvidenceDiagnostic {
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderTemporalProductionEvidenceSeverity::Warning,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderTemporalProductionEvidenceSeverity::Error,
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTemporalProductionHardwareProfile {
    pub profile_key: String,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub timestamp_query: RenderGpuTimingCapability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTemporalProductionHardwareProfileInspection {
    pub profile_key: String,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub timestamp_query_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTemporalRuntimeVisualEvidence {
    pub view_label: String,
    pub artifact_path: String,
    pub reconstruction_mode: RenderTemporalReconstructionMode,
    pub internal_size: [u32; 2],
    pub output_size: [u32; 2],
    pub rendered_pixel_count: u64,
    pub history_valid: bool,
    pub native_fallback_visible: bool,
    pub consumed_temporal_inputs: bool,
    pub consumed_temporal_upscaling: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTemporalProductionEvidenceCounts {
    pub temporal_required_input_count: usize,
    pub temporal_available_required_input_count: usize,
    pub temporal_missing_required_input_count: usize,
    pub ray_required_input_count: usize,
    pub ray_available_required_input_count: usize,
    pub ray_missing_required_input_count: usize,
    pub visual_evidence_count: usize,
    pub fallback_visual_count: usize,
    pub rendered_pixel_count: u64,
    pub adapter_invocation_allowed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalProductionTimingEvidence {
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
pub struct RenderTemporalProductionEvidenceReport {
    pub hardware_profile: RenderTemporalProductionHardwareProfileInspection,
    pub counts: RenderTemporalProductionEvidenceCounts,
    pub timings: RenderTemporalProductionTimingEvidence,
    pub visual_evidence: Vec<RenderTemporalRuntimeVisualEvidence>,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
    pub diagnostics: Vec<RenderTemporalProductionEvidenceDiagnostic>,
}

impl RenderTemporalProductionEvidenceReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.severity == RenderTemporalProductionEvidenceSeverity::Error
            })
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.severity == RenderTemporalProductionEvidenceSeverity::Warning
            })
            .count()
    }

    pub fn is_runtime_ready(&self) -> bool {
        self.error_count() == 0
    }
}

#[derive(Debug, Clone)]
pub struct RenderTemporalProductionEvidenceRequest {
    pub hardware_profile: RenderTemporalProductionHardwareProfile,
    pub temporal: RenderTemporalInspection,
    pub upscaling: RenderTemporalUpscalingInspection,
    pub timings: RenderDebugTimingsState,
    pub visual_evidence: Vec<RenderTemporalRuntimeVisualEvidence>,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
}

pub fn inspect_render_temporal_production_evidence(
    request: RenderTemporalProductionEvidenceRequest,
) -> RenderTemporalProductionEvidenceReport {
    let hardware_profile = RenderTemporalProductionHardwareProfileInspection {
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
        &request.temporal,
        &request.upscaling,
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
    validate_count_invariants(&request, &counts, &mut diagnostics);
    collect_source_diagnostics(&request, &mut diagnostics);

    RenderTemporalProductionEvidenceReport {
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
    temporal: &RenderTemporalInspection,
    upscaling: &RenderTemporalUpscalingInspection,
    visual_evidence: &[RenderTemporalRuntimeVisualEvidence],
) -> RenderTemporalProductionEvidenceCounts {
    RenderTemporalProductionEvidenceCounts {
        temporal_required_input_count: temporal.counts.required_input_count,
        temporal_available_required_input_count: temporal.counts.available_required_input_count,
        temporal_missing_required_input_count: temporal.counts.missing_required_input_count,
        ray_required_input_count: upscaling.ray_input_counts.required_input_count,
        ray_available_required_input_count: upscaling
            .ray_input_counts
            .available_required_input_count,
        ray_missing_required_input_count: upscaling.ray_input_counts.missing_required_input_count,
        visual_evidence_count: visual_evidence.len(),
        fallback_visual_count: visual_evidence
            .iter()
            .filter(|evidence| evidence.native_fallback_visible)
            .count(),
        rendered_pixel_count: visual_evidence
            .iter()
            .map(|evidence| evidence.rendered_pixel_count)
            .sum(),
        adapter_invocation_allowed: upscaling.adapter_invocation_allowed,
    }
}

fn timings_from_state(timings: &RenderDebugTimingsState) -> RenderTemporalProductionTimingEvidence {
    let timing_source = if timings.gpu_pass_sample_count > 0 {
        "gpu_timestamp_query"
    } else {
        "cpu_encode_submit"
    };
    RenderTemporalProductionTimingEvidence {
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
    profile: &RenderTemporalProductionHardwareProfile,
    timings: &RenderDebugTimingsState,
    diagnostics: &mut Vec<RenderTemporalProductionEvidenceDiagnostic>,
) {
    if profile.profile_key.trim().is_empty() {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_hardware_profile",
            "temporal production evidence requires a hardware or capability profile key",
        ));
    }

    if profile.timestamp_query != timings.gpu_timing_capability {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::warning(
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
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_gpu_timing_diagnostic",
            "unsupported or unavailable temporal GPU timing evidence must include a typed diagnostic",
        ));
    }
}

fn validate_evidence_inputs(
    request: &RenderTemporalProductionEvidenceRequest,
    diagnostics: &mut Vec<RenderTemporalProductionEvidenceDiagnostic>,
) {
    if request.benchmark_commands.is_empty() {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_benchmark_command",
            "temporal production evidence requires at least one benchmark command",
        ));
    }
    if request.artifact_paths.is_empty() {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_artifact_path",
            "temporal production evidence requires at least one raw artifact or human report path",
        ));
    }
    if request.timings.pass_sample_count == 0 && request.timings.gpu_pass_sample_count == 0 {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_timing_evidence",
            "temporal production evidence requires CPU or GPU pass timing evidence",
        ));
    }
    if request.visual_evidence.is_empty() {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_visual_evidence",
            "temporal production evidence requires runtime visual evidence references",
        ));
    }

    for evidence in &request.visual_evidence {
        if evidence.artifact_path.trim().is_empty() {
            diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
                "missing_visual_artifact",
                format!(
                    "temporal visual evidence {} is missing an artifact path",
                    evidence.view_label
                ),
            ));
        }
        if evidence.rendered_pixel_count == 0 {
            diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
                "missing_rendered_pixels",
                format!(
                    "temporal visual evidence {} has no rendered pixel evidence",
                    evidence.view_label
                ),
            ));
        }
        if !evidence.history_valid {
            diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
                "visual_history_invalid",
                format!(
                    "temporal visual evidence {} reports invalid history",
                    evidence.view_label
                ),
            ));
        }
        if !evidence.consumed_temporal_inputs {
            diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
                "visual_without_temporal_inputs",
                format!(
                    "temporal visual evidence {} did not consume temporal input inspection",
                    evidence.view_label
                ),
            ));
        }
        if !evidence.consumed_temporal_upscaling {
            diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
                "visual_without_temporal_upscaling",
                format!(
                    "temporal visual evidence {} did not consume upscaling inspection",
                    evidence.view_label
                ),
            ));
        }
        if evidence.internal_size[0] == 0
            || evidence.internal_size[1] == 0
            || evidence.output_size[0] == 0
            || evidence.output_size[1] == 0
        {
            diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
                "visual_invalid_resolution",
                format!(
                    "temporal visual evidence {} has invalid internal/output resolution",
                    evidence.view_label
                ),
            ));
        }
    }
}

fn validate_count_invariants(
    request: &RenderTemporalProductionEvidenceRequest,
    counts: &RenderTemporalProductionEvidenceCounts,
    diagnostics: &mut Vec<RenderTemporalProductionEvidenceDiagnostic>,
) {
    if !request.temporal.is_ready() {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "temporal_inputs_not_ready",
            "temporal production evidence requires ready temporal input inspection",
        ));
    }
    if !request.upscaling.is_ready() {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "temporal_upscaling_not_ready",
            "temporal production evidence requires ready upscaling/ray input inspection",
        ));
    }
    if request.upscaling.temporal.frame_index != request.temporal.frame_index {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "temporal_frame_mismatch",
            "temporal production evidence requires temporal and upscaling inspections from the same frame",
        ));
    }
    if counts.temporal_required_input_count == 0 {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_temporal_required_inputs",
            "temporal production evidence requires required temporal input evidence",
        ));
    }
    if counts.temporal_missing_required_input_count > 0 {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_temporal_required_input",
            "temporal production evidence cannot have missing required temporal inputs",
        ));
    }
    if counts.ray_required_input_count == 0 {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_ray_required_inputs",
            "temporal production evidence requires required ray reconstruction input evidence",
        ));
    }
    if counts.ray_missing_required_input_count > 0 {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_ray_required_input",
            "temporal production evidence cannot have missing required ray reconstruction inputs",
        ));
    }
    if counts.rendered_pixel_count == 0 {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "missing_rendered_pixel_count",
            "temporal production evidence requires nonzero rendered pixel evidence",
        ));
    }
    if counts.visual_evidence_count > 0
        && counts.fallback_visual_count == counts.visual_evidence_count
    {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
            "fallback_only_temporal_claim",
            "temporal production evidence cannot be proven only by native fallback visuals",
        ));
    }
}

fn collect_source_diagnostics(
    request: &RenderTemporalProductionEvidenceRequest,
    diagnostics: &mut Vec<RenderTemporalProductionEvidenceDiagnostic>,
) {
    for diagnostic in &request.temporal.diagnostics {
        match diagnostic.severity {
            RenderTemporalDiagnosticSeverity::Error => {
                diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
                    format!("temporal_{}", diagnostic.code),
                    diagnostic.message.clone(),
                ));
            }
            RenderTemporalDiagnosticSeverity::Warning => {
                diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::warning(
                    format!("temporal_{}", diagnostic.code),
                    diagnostic.message.clone(),
                ));
            }
        }
    }
    for diagnostic in &request.upscaling.diagnostics {
        match diagnostic.severity {
            RenderTemporalDiagnosticSeverity::Error => {
                diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::error(
                    format!("temporal_upscaling_{}", diagnostic.code),
                    diagnostic.message.clone(),
                ));
            }
            RenderTemporalDiagnosticSeverity::Warning => {
                diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::warning(
                    format!("temporal_upscaling_{}", diagnostic.code),
                    diagnostic.message.clone(),
                ));
            }
        }
    }
    for diagnostic in &request.timings.gpu_timing_diagnostics {
        diagnostics.push(RenderTemporalProductionEvidenceDiagnostic::warning(
            diagnostic.kind.as_str(),
            diagnostic.message.clone(),
        ));
    }
}
