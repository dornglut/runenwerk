#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTemporalDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTemporalDiagnostic {
    pub severity: RenderTemporalDiagnosticSeverity,
    pub code: &'static str,
    pub message: String,
}

impl RenderTemporalDiagnostic {
    fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: RenderTemporalDiagnosticSeverity::Error,
            code,
            message: message.into(),
        }
    }

    fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: RenderTemporalDiagnosticSeverity::Warning,
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTemporalInputKind {
    MotionVectors,
    Depth,
    Exposure,
    Luminance,
    ReactiveMask,
}

impl RenderTemporalInputKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MotionVectors => "motion_vectors",
            Self::Depth => "depth",
            Self::Exposure => "exposure",
            Self::Luminance => "luminance",
            Self::ReactiveMask => "reactive_mask",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTemporalReconstructionMode {
    Native,
    Taa,
    Taau,
}

impl RenderTemporalReconstructionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Taa => "taa",
            Self::Taau => "taau",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalResolutionEvidence {
    pub internal_size: [u32; 2],
    pub output_size: [u32; 2],
    pub min_scale: f32,
    pub max_scale: f32,
    pub dynamic_resolution_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalResolutionInspection {
    pub internal_size: [u32; 2],
    pub output_size: [u32; 2],
    pub scale_x: f32,
    pub scale_y: f32,
    pub min_scale: f32,
    pub max_scale: f32,
    pub dynamic_resolution_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalJitterEvidence {
    pub sequence_id: String,
    pub phase_index: u64,
    pub phase_count: u64,
    pub offset: [f32; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalHistoryEvidence {
    pub resource_id: String,
    pub current_signature: String,
    pub previous_signature: Option<String>,
    pub age_frames: u32,
    pub valid: bool,
    pub invalidation_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTemporalInputEvidence {
    pub kind: RenderTemporalInputKind,
    pub required: bool,
    pub available: bool,
    pub product_id: Option<String>,
    pub generation: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderTemporalInputCounts {
    pub input_count: usize,
    pub required_input_count: usize,
    pub available_required_input_count: usize,
    pub missing_required_input_count: usize,
    pub available_optional_input_count: usize,
    pub missing_optional_input_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalInspectionRequest {
    pub frame_index: u64,
    pub reconstruction_mode: RenderTemporalReconstructionMode,
    pub native_fallback_active: bool,
    pub resolution: RenderTemporalResolutionEvidence,
    pub jitter: RenderTemporalJitterEvidence,
    pub history: RenderTemporalHistoryEvidence,
    pub inputs: Vec<RenderTemporalInputEvidence>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalInspection {
    pub frame_index: u64,
    pub reconstruction_mode: RenderTemporalReconstructionMode,
    pub native_fallback_active: bool,
    pub resolution: RenderTemporalResolutionInspection,
    pub jitter: RenderTemporalJitterEvidence,
    pub history: RenderTemporalHistoryEvidence,
    pub counts: RenderTemporalInputCounts,
    pub inputs: Vec<RenderTemporalInputEvidence>,
    pub diagnostics: Vec<RenderTemporalDiagnostic>,
}

impl RenderTemporalInspection {
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

pub fn inspect_render_temporal_inputs(
    request: RenderTemporalInspectionRequest,
) -> RenderTemporalInspection {
    let resolution = inspect_resolution(&request.resolution);
    let counts = count_inputs(&request.inputs);
    let mut diagnostics = Vec::new();

    validate_resolution(&request.resolution, &resolution, &mut diagnostics);
    validate_jitter(&request.jitter, &mut diagnostics);
    validate_history(&request, &mut diagnostics);
    validate_inputs(&request, &mut diagnostics);
    validate_reconstruction_mode(&request, &mut diagnostics);

    RenderTemporalInspection {
        frame_index: request.frame_index,
        reconstruction_mode: request.reconstruction_mode,
        native_fallback_active: request.native_fallback_active,
        resolution,
        jitter: request.jitter,
        history: request.history,
        counts,
        inputs: request.inputs,
        diagnostics,
    }
}

fn inspect_resolution(
    resolution: &RenderTemporalResolutionEvidence,
) -> RenderTemporalResolutionInspection {
    let scale_x = if resolution.output_size[0] == 0 {
        0.0
    } else {
        resolution.internal_size[0] as f32 / resolution.output_size[0] as f32
    };
    let scale_y = if resolution.output_size[1] == 0 {
        0.0
    } else {
        resolution.internal_size[1] as f32 / resolution.output_size[1] as f32
    };

    RenderTemporalResolutionInspection {
        internal_size: resolution.internal_size,
        output_size: resolution.output_size,
        scale_x,
        scale_y,
        min_scale: resolution.min_scale,
        max_scale: resolution.max_scale,
        dynamic_resolution_enabled: resolution.dynamic_resolution_enabled,
    }
}

fn count_inputs(inputs: &[RenderTemporalInputEvidence]) -> RenderTemporalInputCounts {
    RenderTemporalInputCounts {
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

fn validate_resolution(
    evidence: &RenderTemporalResolutionEvidence,
    inspection: &RenderTemporalResolutionInspection,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    if evidence.internal_size[0] == 0
        || evidence.internal_size[1] == 0
        || evidence.output_size[0] == 0
        || evidence.output_size[1] == 0
    {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "invalid_resolution_extent",
            "temporal inspection requires nonzero internal and output resolution",
        ));
    }

    if !evidence.min_scale.is_finite()
        || !evidence.max_scale.is_finite()
        || evidence.min_scale <= 0.0
        || evidence.max_scale <= 0.0
        || evidence.min_scale > evidence.max_scale
    {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "invalid_dynamic_resolution_limits",
            "temporal dynamic-resolution scale limits must be finite, positive, and ordered",
        ));
        return;
    }

    if !evidence.dynamic_resolution_enabled && evidence.internal_size != evidence.output_size {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "hidden_dynamic_resolution",
            "internal and output resolution differ while dynamic resolution is not reported as enabled",
        ));
    }

    if evidence.dynamic_resolution_enabled
        && (inspection.scale_x < evidence.min_scale
            || inspection.scale_x > evidence.max_scale
            || inspection.scale_y < evidence.min_scale
            || inspection.scale_y > evidence.max_scale)
    {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "dynamic_resolution_out_of_bounds",
            format!(
                "temporal internal/output scale ({:.3}, {:.3}) is outside configured range [{:.3}, {:.3}]",
                inspection.scale_x, inspection.scale_y, evidence.min_scale, evidence.max_scale
            ),
        ));
    }
}

fn validate_jitter(
    jitter: &RenderTemporalJitterEvidence,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    if jitter.sequence_id.trim().is_empty() {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "missing_jitter_sequence",
            "temporal jitter evidence requires a sequence identity",
        ));
    }
    if jitter.phase_count == 0 {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "missing_jitter_phase_count",
            "temporal jitter evidence requires a nonzero phase count",
        ));
    } else if jitter.phase_index >= jitter.phase_count {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "jitter_phase_out_of_range",
            format!(
                "temporal jitter phase {} is outside phase count {}",
                jitter.phase_index, jitter.phase_count
            ),
        ));
    }
    if !jitter.offset[0].is_finite() || !jitter.offset[1].is_finite() {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "invalid_jitter_offset",
            "temporal jitter offset must be finite",
        ));
    }
}

fn validate_history(
    request: &RenderTemporalInspectionRequest,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    let history = &request.history;
    if history.resource_id.trim().is_empty() {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "missing_history_resource",
            "temporal history evidence requires a resource identity",
        ));
    }
    if history.current_signature.trim().is_empty() {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "missing_history_signature",
            "temporal history evidence requires a current signature",
        ));
    }

    if history.valid {
        match history.previous_signature.as_deref() {
            Some(previous) if previous == history.current_signature => {}
            Some(_) => diagnostics.push(RenderTemporalDiagnostic::error(
                "valid_history_signature_mismatch",
                "temporal history is marked valid but previous and current signatures differ",
            )),
            None => diagnostics.push(RenderTemporalDiagnostic::error(
                "valid_history_missing_previous_signature",
                "temporal history is marked valid without previous signature evidence",
            )),
        }
        if history.invalidation_reason.is_some() {
            diagnostics.push(RenderTemporalDiagnostic::warning(
                "valid_history_has_invalidation_reason",
                "temporal history is valid but still reports an invalidation reason",
            ));
        }
    } else if history
        .invalidation_reason
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "missing_history_invalidation_reason",
            "invalid temporal history requires a typed invalidation reason",
        ));
    }

    if !history.valid
        && request.reconstruction_mode != RenderTemporalReconstructionMode::Native
        && !request.native_fallback_active
    {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "temporal_history_invalid_without_fallback",
            "temporal reconstruction cannot use invalid history without native fallback",
        ));
    }
}

fn validate_inputs(
    request: &RenderTemporalInspectionRequest,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    if request.inputs.is_empty() {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "missing_temporal_inputs",
            "temporal inspection requires explicit input availability evidence",
        ));
    }

    for input in &request.inputs {
        if input.available {
            if input.product_id.as_deref().unwrap_or("").trim().is_empty() {
                diagnostics.push(RenderTemporalDiagnostic::error(
                    "available_input_missing_product",
                    format!(
                        "available temporal input '{}' has no product identity",
                        input.kind.as_str()
                    ),
                ));
            }
            if input.generation.is_none() {
                diagnostics.push(RenderTemporalDiagnostic::error(
                    "available_input_missing_generation",
                    format!(
                        "available temporal input '{}' has no producer generation",
                        input.kind.as_str()
                    ),
                ));
            }
        } else if input.required {
            if request.native_fallback_active {
                diagnostics.push(RenderTemporalDiagnostic::warning(
                    "required_input_missing_native_fallback",
                    format!(
                        "required temporal input '{}' is missing; native fallback is active",
                        input.kind.as_str()
                    ),
                ));
            } else {
                diagnostics.push(RenderTemporalDiagnostic::error(
                    "missing_required_input",
                    format!(
                        "required temporal input '{}' is missing",
                        input.kind.as_str()
                    ),
                ));
            }
        } else {
            diagnostics.push(RenderTemporalDiagnostic::warning(
                "optional_input_missing",
                format!(
                    "optional temporal input '{}' is missing",
                    input.kind.as_str()
                ),
            ));
        }
    }
}

fn validate_reconstruction_mode(
    request: &RenderTemporalInspectionRequest,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    if request.reconstruction_mode == RenderTemporalReconstructionMode::Native
        && request.native_fallback_active
    {
        diagnostics.push(RenderTemporalDiagnostic::warning(
            "native_mode_with_native_fallback",
            "native fallback is active while reconstruction mode is already native",
        ));
    }

    if request.reconstruction_mode == RenderTemporalReconstructionMode::Taau
        && !request.resolution.dynamic_resolution_enabled
    {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "taau_without_dynamic_resolution",
            "TAAU reconstruction requires explicit dynamic internal resolution evidence",
        ));
    }
}
