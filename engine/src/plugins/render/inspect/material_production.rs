use super::material_handoff::{
    RenderMeshMaterialHandoffDiagnosticSeverity, RenderMeshMaterialHandoffInspection,
};
use super::pipeline_fallback::{
    RenderPipelineFallbackDiagnosticSeverity, RenderPipelineFallbackInspection,
};
use super::timings::{RenderDebugTimingsState, RenderGpuTimingCapability};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMeshMaterialProductionEvidenceSeverity {
    Info,
    Warning,
    Error,
}

impl RenderMeshMaterialProductionEvidenceSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMeshMaterialProductionEvidenceDiagnostic {
    pub severity: RenderMeshMaterialProductionEvidenceSeverity,
    pub code: String,
    pub message: String,
}

impl RenderMeshMaterialProductionEvidenceDiagnostic {
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderMeshMaterialProductionEvidenceSeverity::Warning,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: RenderMeshMaterialProductionEvidenceSeverity::Error,
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMeshMaterialProductionHardwareProfile {
    pub profile_key: String,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub timestamp_query: RenderGpuTimingCapability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMeshMaterialProductionHardwareProfileInspection {
    pub profile_key: String,
    pub adapter_name: Option<String>,
    pub backend: Option<String>,
    pub timestamp_query_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMeshMaterialRuntimeVisualEvidence {
    pub view_label: String,
    pub artifact_path: String,
    pub material_table_identity: String,
    pub scene_shader_identity: String,
    pub material_instance_count: usize,
    pub rendered_pixel_count: u64,
    pub consumed_material_handoff: bool,
    pub consumed_pipeline_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderMeshMaterialProductionEvidenceCounts {
    pub material_instance_count: usize,
    pub material_binding_slot_count: usize,
    pub model_mesh_selection_count: usize,
    pub material_consuming_pass_count: usize,
    pub pipeline_backed_pass_count: usize,
    pub fallback_pass_count: usize,
    pub material_fallback_pass_count: usize,
    pub shader_failure_event_count: usize,
    pub prior_valid_shader_failure_count: usize,
    pub visual_evidence_count: usize,
    pub rendered_pixel_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderMeshMaterialProductionTimingEvidence {
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
pub struct RenderMeshMaterialProductionEvidenceReport {
    pub hardware_profile: RenderMeshMaterialProductionHardwareProfileInspection,
    pub counts: RenderMeshMaterialProductionEvidenceCounts,
    pub timings: RenderMeshMaterialProductionTimingEvidence,
    pub visual_evidence: Vec<RenderMeshMaterialRuntimeVisualEvidence>,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
    pub diagnostics: Vec<RenderMeshMaterialProductionEvidenceDiagnostic>,
}

impl RenderMeshMaterialProductionEvidenceReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.severity == RenderMeshMaterialProductionEvidenceSeverity::Error
            })
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.severity == RenderMeshMaterialProductionEvidenceSeverity::Warning
            })
            .count()
    }

    pub fn is_runtime_ready(&self) -> bool {
        self.error_count() == 0
    }
}

#[derive(Debug, Clone)]
pub struct RenderMeshMaterialProductionEvidenceRequest {
    pub hardware_profile: RenderMeshMaterialProductionHardwareProfile,
    pub material_handoff: RenderMeshMaterialHandoffInspection,
    pub pipeline_fallback: RenderPipelineFallbackInspection,
    pub timings: RenderDebugTimingsState,
    pub visual_evidence: Vec<RenderMeshMaterialRuntimeVisualEvidence>,
    pub benchmark_commands: Vec<String>,
    pub artifact_paths: Vec<String>,
}

pub fn inspect_render_mesh_material_production_evidence(
    request: RenderMeshMaterialProductionEvidenceRequest,
) -> RenderMeshMaterialProductionEvidenceReport {
    let hardware_profile = RenderMeshMaterialProductionHardwareProfileInspection {
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
        &request.material_handoff,
        &request.pipeline_fallback,
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
    validate_count_invariants(&counts, &mut diagnostics);
    collect_source_diagnostics(&request, &mut diagnostics);

    RenderMeshMaterialProductionEvidenceReport {
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
    material_handoff: &RenderMeshMaterialHandoffInspection,
    pipeline_fallback: &RenderPipelineFallbackInspection,
    visual_evidence: &[RenderMeshMaterialRuntimeVisualEvidence],
) -> RenderMeshMaterialProductionEvidenceCounts {
    RenderMeshMaterialProductionEvidenceCounts {
        material_instance_count: material_handoff.counts.material_instance_count,
        material_binding_slot_count: material_handoff.counts.material_binding_slot_count,
        model_mesh_selection_count: material_handoff.counts.model_mesh_selection_count,
        material_consuming_pass_count: material_handoff.counts.material_consuming_pass_count,
        pipeline_backed_pass_count: pipeline_fallback.counts.pipeline_backed_pass_count,
        fallback_pass_count: pipeline_fallback.counts.fallback_pass_count,
        material_fallback_pass_count: pipeline_fallback.counts.material_fallback_pass_count,
        shader_failure_event_count: pipeline_fallback.counts.shader_failure_event_count,
        prior_valid_shader_failure_count: pipeline_fallback.counts.prior_valid_shader_failure_count,
        visual_evidence_count: visual_evidence.len(),
        rendered_pixel_count: visual_evidence
            .iter()
            .map(|evidence| evidence.rendered_pixel_count)
            .sum(),
    }
}

fn timings_from_state(
    timings: &RenderDebugTimingsState,
) -> RenderMeshMaterialProductionTimingEvidence {
    let timing_source = if timings.gpu_pass_sample_count > 0 {
        "gpu_timestamp_query"
    } else {
        "cpu_encode_submit"
    };
    RenderMeshMaterialProductionTimingEvidence {
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
    profile: &RenderMeshMaterialProductionHardwareProfile,
    timings: &RenderDebugTimingsState,
    diagnostics: &mut Vec<RenderMeshMaterialProductionEvidenceDiagnostic>,
) {
    if profile.profile_key.trim().is_empty() {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_hardware_profile",
            "mesh/material production evidence requires a hardware or capability profile key",
        ));
    }

    if profile.timestamp_query != timings.gpu_timing_capability {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::warning(
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
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_gpu_timing_diagnostic",
            "unsupported or unavailable mesh/material GPU timing evidence must include a typed diagnostic",
        ));
    }
}

fn validate_evidence_inputs(
    request: &RenderMeshMaterialProductionEvidenceRequest,
    diagnostics: &mut Vec<RenderMeshMaterialProductionEvidenceDiagnostic>,
) {
    if request.benchmark_commands.is_empty() {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_benchmark_command",
            "mesh/material production evidence requires at least one benchmark command",
        ));
    }
    if request.artifact_paths.is_empty() {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_artifact_path",
            "mesh/material production evidence requires at least one raw artifact or human report path",
        ));
    }
    if request.timings.pass_sample_count == 0 && request.timings.gpu_pass_sample_count == 0 {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_timing_evidence",
            "mesh/material production evidence requires CPU or GPU pass timing evidence",
        ));
    }
    if request.visual_evidence.is_empty() {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_visual_evidence",
            "mesh/material production evidence requires at least one visual artifact reference",
        ));
    }

    for evidence in &request.visual_evidence {
        if evidence.artifact_path.trim().is_empty() {
            diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
                "missing_visual_artifact",
                format!(
                    "mesh/material visual evidence {} is missing an artifact path",
                    evidence.view_label
                ),
            ));
        }
        if evidence.rendered_pixel_count == 0 {
            diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
                "missing_rendered_pixels",
                format!(
                    "mesh/material visual evidence {} has no rendered pixel evidence",
                    evidence.view_label
                ),
            ));
        }
        if evidence.material_instance_count == 0 {
            diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
                "missing_visual_material_instance",
                format!(
                    "mesh/material visual evidence {} has no material instance evidence",
                    evidence.view_label
                ),
            ));
        }
        if evidence.material_table_identity.trim().is_empty() {
            diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
                "missing_visual_material_table_identity",
                format!(
                    "mesh/material visual evidence {} has no material table identity",
                    evidence.view_label
                ),
            ));
        }
        if evidence.scene_shader_identity.trim().is_empty() {
            diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
                "missing_visual_scene_shader_identity",
                format!(
                    "mesh/material visual evidence {} has no scene shader identity",
                    evidence.view_label
                ),
            ));
        }
        if !evidence.consumed_material_handoff {
            diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
                "visual_without_material_handoff",
                format!(
                    "mesh/material visual evidence {} did not consume material handoff inspection",
                    evidence.view_label
                ),
            ));
        }
        if !evidence.consumed_pipeline_fallback {
            diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
                "visual_without_pipeline_fallback",
                format!(
                    "mesh/material visual evidence {} did not consume pipeline/fallback inspection",
                    evidence.view_label
                ),
            ));
        }
    }
}

fn validate_count_invariants(
    counts: &RenderMeshMaterialProductionEvidenceCounts,
    diagnostics: &mut Vec<RenderMeshMaterialProductionEvidenceDiagnostic>,
) {
    if counts.material_instance_count == 0 {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_material_instances",
            "mesh/material production evidence requires prepared material instances",
        ));
    }
    if counts.material_binding_slot_count == 0 {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_material_binding_slots",
            "mesh/material production evidence requires material binding slots",
        ));
    }
    if counts.material_consuming_pass_count == 0 {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_material_consuming_pass",
            "mesh/material production evidence requires material-consuming pass evidence",
        ));
    }
    if counts.pipeline_backed_pass_count == 0 {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_pipeline_backed_pass",
            "mesh/material production evidence requires pipeline-backed pass evidence",
        ));
    }
    if counts.material_fallback_pass_count > 0 {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "material_fallback_present",
            "mesh/material production evidence cannot be fallback-only for material passes",
        ));
    }
    if counts.rendered_pixel_count == 0 {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
            "missing_rendered_pixel_count",
            "mesh/material production evidence requires nonzero rendered pixel evidence",
        ));
    }
}

fn collect_source_diagnostics(
    request: &RenderMeshMaterialProductionEvidenceRequest,
    diagnostics: &mut Vec<RenderMeshMaterialProductionEvidenceDiagnostic>,
) {
    for diagnostic in &request.material_handoff.diagnostics {
        match diagnostic.severity {
            RenderMeshMaterialHandoffDiagnosticSeverity::Error => {
                diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
                    format!("material_handoff_{}", diagnostic.code),
                    diagnostic.message.clone(),
                ));
            }
            RenderMeshMaterialHandoffDiagnosticSeverity::Warning => {
                diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::warning(
                    format!("material_handoff_{}", diagnostic.code),
                    diagnostic.message.clone(),
                ));
            }
        }
    }
    for diagnostic in &request.pipeline_fallback.diagnostics {
        match diagnostic.severity {
            RenderPipelineFallbackDiagnosticSeverity::Error => {
                diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::error(
                    format!("pipeline_fallback_{}", diagnostic.code),
                    diagnostic.message.clone(),
                ));
            }
            RenderPipelineFallbackDiagnosticSeverity::Warning => {
                diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::warning(
                    format!("pipeline_fallback_{}", diagnostic.code),
                    diagnostic.message.clone(),
                ));
            }
        }
    }
    for diagnostic in &request.timings.gpu_timing_diagnostics {
        diagnostics.push(RenderMeshMaterialProductionEvidenceDiagnostic::warning(
            diagnostic.kind.as_str(),
            diagnostic.message.clone(),
        ));
    }
}
