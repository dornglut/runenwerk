use super::{RenderTemporalDiagnostic, RenderTemporalDiagnosticSeverity, RenderTemporalInspection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTemporalUpscalingAdapterKind {
    Native,
    FsrStyle,
    VendorSpecific,
}

impl RenderTemporalUpscalingAdapterKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::FsrStyle => "fsr_style",
            Self::VendorSpecific => "vendor_specific",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTemporalUpscalingCapabilityState {
    Supported,
    Unsupported,
    Disabled,
}

impl RenderTemporalUpscalingCapabilityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderRayReconstructionInputKind {
    MotionVectors,
    Depth,
    Exposure,
    ReactiveMask,
    DisocclusionMask,
    RaymarchDistance,
    RayQueryHitDistance,
}

impl RenderRayReconstructionInputKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MotionVectors => "motion_vectors",
            Self::Depth => "depth",
            Self::Exposure => "exposure",
            Self::ReactiveMask => "reactive_mask",
            Self::DisocclusionMask => "disocclusion_mask",
            Self::RaymarchDistance => "raymarch_distance",
            Self::RayQueryHitDistance => "ray_query_hit_distance",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTemporalUpscalingAdapterEvidence {
    pub kind: RenderTemporalUpscalingAdapterKind,
    pub capability_state: RenderTemporalUpscalingCapabilityState,
    pub required_capabilities: Vec<String>,
    pub unsupported_reason: Option<String>,
    pub invocation_requested: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRayReconstructionInputEvidence {
    pub kind: RenderRayReconstructionInputKind,
    pub required: bool,
    pub available: bool,
    pub product_id: Option<String>,
    pub generation: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderRayReconstructionInputCounts {
    pub input_count: usize,
    pub required_input_count: usize,
    pub available_required_input_count: usize,
    pub missing_required_input_count: usize,
    pub available_optional_input_count: usize,
    pub missing_optional_input_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalUpscalingInspectionRequest {
    pub temporal: RenderTemporalInspection,
    pub adapter: RenderTemporalUpscalingAdapterEvidence,
    pub ray_inputs: Vec<RenderRayReconstructionInputEvidence>,
    pub native_fallback_visible: bool,
    pub adapter_required_for_correctness: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalUpscalingInspection {
    pub temporal: RenderTemporalInspection,
    pub adapter: RenderTemporalUpscalingAdapterEvidence,
    pub ray_input_counts: RenderRayReconstructionInputCounts,
    pub ray_inputs: Vec<RenderRayReconstructionInputEvidence>,
    pub native_fallback_visible: bool,
    pub adapter_required_for_correctness: bool,
    pub adapter_invocation_allowed: bool,
    pub diagnostics: Vec<RenderTemporalDiagnostic>,
}

impl RenderTemporalUpscalingInspection {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == RenderTemporalDiagnosticSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == RenderTemporalDiagnosticSeverity::Warning)
            .count()
    }

    pub fn is_ready(&self) -> bool {
        self.error_count() == 0
    }
}

pub fn inspect_render_temporal_upscaling(
    request: RenderTemporalUpscalingInspectionRequest,
) -> RenderTemporalUpscalingInspection {
    let ray_input_counts = count_ray_inputs(&request.ray_inputs);
    let mut diagnostics = Vec::new();

    validate_temporal_inputs(&request.temporal, &mut diagnostics);
    validate_adapter(&request, &mut diagnostics);
    validate_ray_inputs(&request, &mut diagnostics);

    let adapter_invocation_allowed =
        adapter_invocation_allowed(&request, &ray_input_counts, &diagnostics);

    RenderTemporalUpscalingInspection {
        temporal: request.temporal,
        adapter: request.adapter,
        ray_input_counts,
        ray_inputs: request.ray_inputs,
        native_fallback_visible: request.native_fallback_visible,
        adapter_required_for_correctness: request.adapter_required_for_correctness,
        adapter_invocation_allowed,
        diagnostics,
    }
}

fn count_ray_inputs(
    inputs: &[RenderRayReconstructionInputEvidence],
) -> RenderRayReconstructionInputCounts {
    RenderRayReconstructionInputCounts {
        input_count: inputs.len(),
        required_input_count: inputs.iter().filter(|input| input.required).count(),
        available_required_input_count: inputs
            .iter()
            .filter(|input| input.required && input.available)
            .count(),
        missing_required_input_count: inputs
            .iter()
            .filter(|input| input.required && !input.available)
            .count(),
        available_optional_input_count: inputs
            .iter()
            .filter(|input| !input.required && input.available)
            .count(),
        missing_optional_input_count: inputs
            .iter()
            .filter(|input| !input.required && !input.available)
            .count(),
    }
}

fn validate_temporal_inputs(
    temporal: &RenderTemporalInspection,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    for diagnostic in &temporal.diagnostics {
        let code = match diagnostic.severity {
            RenderTemporalDiagnosticSeverity::Error => "temporal_input_error",
            RenderTemporalDiagnosticSeverity::Warning => "temporal_input_warning",
        };
        diagnostics.push(RenderTemporalDiagnostic {
            severity: diagnostic.severity,
            code,
            message: format!(
                "temporal input inspection reported '{}': {}",
                diagnostic.code, diagnostic.message
            ),
        });
    }
}

fn validate_adapter(
    request: &RenderTemporalUpscalingInspectionRequest,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    let adapter = &request.adapter;

    if request.adapter_required_for_correctness {
        diagnostics.push(error(
            "adapter_required_for_correctness",
            "optional temporal upscaling adapters cannot be required for baseline renderer correctness",
        ));
    }

    if request.temporal.native_fallback_active && !request.native_fallback_visible {
        diagnostics.push(error(
            "native_fallback_not_visible",
            "native fallback is active but not reported as visible to renderer diagnostics",
        ));
    }

    if adapter.kind == RenderTemporalUpscalingAdapterKind::Native && adapter.invocation_requested {
        diagnostics.push(warning(
            "native_adapter_invocation_requested",
            "native rendering is the fallback path, not an optional upscaling adapter invocation",
        ));
    }

    match adapter.capability_state {
        RenderTemporalUpscalingCapabilityState::Supported => {
            if adapter.unsupported_reason.is_some() {
                diagnostics.push(warning(
                    "supported_adapter_has_unsupported_reason",
                    "supported temporal upscaling adapter still reports an unsupported reason",
                ));
            }
        }
        RenderTemporalUpscalingCapabilityState::Unsupported => {
            if adapter
                .unsupported_reason
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
            {
                diagnostics.push(error(
                    "unsupported_adapter_missing_reason",
                    "unsupported temporal upscaling adapter requires a typed unsupported reason",
                ));
            }
            validate_unavailable_adapter(request, diagnostics);
        }
        RenderTemporalUpscalingCapabilityState::Disabled => {
            validate_unavailable_adapter(request, diagnostics);
        }
    }
}

fn validate_unavailable_adapter(
    request: &RenderTemporalUpscalingInspectionRequest,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    if !request.adapter.invocation_requested {
        return;
    }

    if request.native_fallback_visible || request.temporal.native_fallback_active {
        diagnostics.push(warning(
            "adapter_unavailable_native_fallback",
            format!(
                "temporal upscaling adapter '{}' is {}; native fallback is visible",
                request.adapter.kind.as_str(),
                request.adapter.capability_state.as_str()
            ),
        ));
    } else {
        diagnostics.push(error(
            "adapter_unavailable_without_visible_fallback",
            format!(
                "temporal upscaling adapter '{}' is {} but no visible native fallback is reported",
                request.adapter.kind.as_str(),
                request.adapter.capability_state.as_str()
            ),
        ));
    }
}

fn validate_ray_inputs(
    request: &RenderTemporalUpscalingInspectionRequest,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    if request.ray_inputs.is_empty() {
        diagnostics.push(error(
            "missing_ray_reconstruction_inputs",
            "temporal upscaling inspection requires explicit ray reconstruction input evidence",
        ));
    }

    for input in &request.ray_inputs {
        if input.available {
            if input.product_id.as_deref().unwrap_or("").trim().is_empty() {
                diagnostics.push(error(
                    "available_ray_input_missing_product",
                    format!(
                        "available ray reconstruction input '{}' has no product identity",
                        input.kind.as_str()
                    ),
                ));
            }
            if input.generation.is_none() {
                diagnostics.push(error(
                    "available_ray_input_missing_generation",
                    format!(
                        "available ray reconstruction input '{}' has no producer generation",
                        input.kind.as_str()
                    ),
                ));
            }
        } else if input.required {
            if request.native_fallback_visible || request.temporal.native_fallback_active {
                diagnostics.push(warning(
                    "required_ray_input_missing_native_fallback",
                    format!(
                        "required ray reconstruction input '{}' is missing; native fallback is visible",
                        input.kind.as_str()
                    ),
                ));
            } else {
                diagnostics.push(error(
                    "missing_required_ray_reconstruction_input",
                    format!(
                        "required ray reconstruction input '{}' is missing",
                        input.kind.as_str()
                    ),
                ));
            }
        } else {
            diagnostics.push(warning(
                "optional_ray_input_missing",
                format!(
                    "optional ray reconstruction input '{}' is missing",
                    input.kind.as_str()
                ),
            ));
        }
    }
}

fn adapter_invocation_allowed(
    request: &RenderTemporalUpscalingInspectionRequest,
    counts: &RenderRayReconstructionInputCounts,
    diagnostics: &[RenderTemporalDiagnostic],
) -> bool {
    request.adapter.invocation_requested
        && request.adapter.kind != RenderTemporalUpscalingAdapterKind::Native
        && request.adapter.capability_state == RenderTemporalUpscalingCapabilityState::Supported
        && request.temporal.is_ready()
        && counts.missing_required_input_count == 0
        && !request.adapter_required_for_correctness
        && !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == RenderTemporalDiagnosticSeverity::Error)
}

fn error(code: &'static str, message: impl Into<String>) -> RenderTemporalDiagnostic {
    RenderTemporalDiagnostic {
        severity: RenderTemporalDiagnosticSeverity::Error,
        code,
        message: message.into(),
    }
}

fn warning(code: &'static str, message: impl Into<String>) -> RenderTemporalDiagnostic {
    RenderTemporalDiagnostic {
        severity: RenderTemporalDiagnosticSeverity::Warning,
        code,
        message: message.into(),
    }
}
