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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderTemporalResolutionPolicy {
    Native,
    Fixed,
    Dynamic { min_scale: f32, max_scale: f32 },
}

impl RenderTemporalResolutionPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Fixed => "fixed",
            Self::Dynamic { .. } => "dynamic",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalResolutionEvidence {
    pub internal_size: [u32; 2],
    pub output_size: [u32; 2],
    pub policy: RenderTemporalResolutionPolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTemporalResolutionInspection {
    pub internal_size: [u32; 2],
    pub output_size: [u32; 2],
    pub scale_x: f32,
    pub scale_y: f32,
    pub policy: RenderTemporalResolutionPolicy,
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
    pub native_fallback_reason: Option<String>,
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
    pub native_fallback_reason: Option<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct RenderFixedResolutionExecutionEvidence {
    pub resolution: RenderTemporalResolutionEvidence,
    pub native_fallback_active: bool,
    pub native_fallback_reason: Option<String>,
    pub target_key: Option<crate::plugins::render::RenderDynamicTextureTargetKey>,
    pub internal_view_id: Option<String>,
    pub scene_invocation_id: Option<crate::plugins::render::PreparedFlowInvocationId>,
    pub resolve_invocation_id: Option<crate::plugins::render::PreparedFlowInvocationId>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RenderFixedResolutionExecutionEvidenceError {
    #[error("fixed-resolution prepared frame belongs to a different render surface than the admission")]
    SurfaceIdentityMismatch,
    #[error("fixed-resolution prepared surface extent does not match admitted output extent")]
    OutputExtentMismatch,
    #[error("fixed-resolution prepared frame is missing the native main output view")]
    MissingOutputView,
    #[error(
        "fixed-resolution native main output view extent does not match admitted output extent"
    )]
    OutputViewExtentMismatch,
    #[error("fixed-resolution prepared frame is missing the admitted internal view")]
    MissingInternalView,
    #[error("fixed-resolution prepared frame is missing the admitted dynamic color target")]
    MissingDynamicTarget,
    #[error("fixed-resolution prepared dynamic color target differs from the admitted descriptor")]
    DynamicTargetDescriptorMismatch,
    #[error("fixed-resolution prepared frame is missing the admitted scene invocation")]
    MissingSceneInvocation,
    #[error(
        "fixed-resolution prepared frame still contains a native-main invocation for the selected scene flow"
    )]
    UnexpectedNativeSceneInvocation,
    #[error(
        "fixed-resolution prepared scene invocation does not retain the admitted target binding"
    )]
    SceneTargetBindingMismatch,
    #[error("fixed-resolution prepared frame is missing the admitted resolve invocation")]
    MissingResolveInvocation,
    #[error(
        "fixed-resolution prepared resolve invocation does not retain the admitted source binding"
    )]
    ResolveSourceBindingMismatch,
    #[error("native fallback frame still contains the fixed internal view")]
    NativeFallbackRetainsInternalView,
    #[error("native fallback frame still contains the fixed dynamic target")]
    NativeFallbackRetainsDynamicTarget,
    #[error("native fallback frame still contains the fixed scene invocation")]
    NativeFallbackRetainsFixedSceneInvocation,
    #[error("native fallback frame still contains the fixed resolve invocation")]
    NativeFallbackRetainsResolveInvocation,
    #[error(
        "native fallback frame is missing a native-main invocation for the selected scene flow"
    )]
    MissingNativeFallbackSceneInvocation,
    #[error("native fallback scene invocation does not retain the admitted native target binding")]
    NativeFallbackSceneBindingMismatch,
}

pub fn inspect_fixed_resolution_execution(
    admission: &crate::plugins::render::RenderFixedResolutionExecutionAdmission,
    frame: &crate::plugins::render::PreparedRenderFrame,
) -> Result<RenderFixedResolutionExecutionEvidence, RenderFixedResolutionExecutionEvidenceError> {
    let (expected_surface_id, expected_output_size) = match admission {
        crate::plugins::render::RenderFixedResolutionExecutionAdmission::Fixed(prepared) => {
            (prepared.render_surface_id, prepared.output_size)
        }
        crate::plugins::render::RenderFixedResolutionExecutionAdmission::NativeFallback(
            fallback,
        ) => (fallback.render_surface_id, fallback.output_size),
    };
    if frame.surface.render_surface_id != expected_surface_id {
        return Err(RenderFixedResolutionExecutionEvidenceError::SurfaceIdentityMismatch);
    }
    if frame.surface.target_size_px != expected_output_size {
        return Err(RenderFixedResolutionExecutionEvidenceError::OutputExtentMismatch);
    }
    let output_view = frame
        .views
        .iter()
        .find(|view| {
            view.view_id == "main"
                && view.kind == crate::plugins::render::PreparedViewKind::MainSurface
        })
        .ok_or(RenderFixedResolutionExecutionEvidenceError::MissingOutputView)?;
    if output_view.target_size_px != expected_output_size {
        return Err(RenderFixedResolutionExecutionEvidenceError::OutputViewExtentMismatch);
    }

    match admission {
        crate::plugins::render::RenderFixedResolutionExecutionAdmission::NativeFallback(
            fallback,
        ) => {
            if frame
                .views
                .iter()
                .any(|view| view.view_id == fallback.internal_view_id)
            {
                return Err(
                    RenderFixedResolutionExecutionEvidenceError::NativeFallbackRetainsInternalView,
                );
            }
            if frame
                .dynamic_texture_targets
                .iter()
                .any(|target| target.key == fallback.target_key)
            {
                return Err(
                    RenderFixedResolutionExecutionEvidenceError::NativeFallbackRetainsDynamicTarget,
                );
            }
            if frame
                .flow_invocations
                .iter()
                .any(|invocation| invocation.invocation_id == fallback.fixed_scene_invocation_id)
            {
                return Err(
                    RenderFixedResolutionExecutionEvidenceError::NativeFallbackRetainsFixedSceneInvocation,
                );
            }
            if frame
                .flow_invocations
                .iter()
                .any(|invocation| invocation.invocation_id == fallback.resolve_invocation_id)
            {
                return Err(
                    RenderFixedResolutionExecutionEvidenceError::NativeFallbackRetainsResolveInvocation,
                );
            }

            let native_scene = if let Some(expected) = &fallback.native_scene_invocation {
                let actual = frame
                    .flow_invocations
                    .iter()
                    .find(|invocation| {
                        invocation.invocation_id == expected.invocation_id
                            && invocation.flow_id == expected.flow_id
                            && invocation.view_id == "main"
                    })
                    .ok_or(
                        RenderFixedResolutionExecutionEvidenceError::MissingNativeFallbackSceneInvocation,
                    )?;
                if actual.target_alias_bindings != expected.target_alias_bindings
                    || actual.history_signature != expected.history_signature
                {
                    return Err(
                        RenderFixedResolutionExecutionEvidenceError::NativeFallbackSceneBindingMismatch,
                    );
                }
                actual
            } else {
                frame
                    .flow_invocations
                    .iter()
                    .find(|invocation| {
                        invocation.flow_id == fallback.scene_flow_id && invocation.view_id == "main"
                    })
                    .ok_or(
                        RenderFixedResolutionExecutionEvidenceError::MissingNativeFallbackSceneInvocation,
                    )?
            };

            Ok(RenderFixedResolutionExecutionEvidence {
                resolution: RenderTemporalResolutionEvidence {
                    internal_size: [fallback.output_size.0, fallback.output_size.1],
                    output_size: [fallback.output_size.0, fallback.output_size.1],
                    policy: RenderTemporalResolutionPolicy::Native,
                },
                native_fallback_active: true,
                native_fallback_reason: Some(fallback.reason.clone()),
                target_key: None,
                internal_view_id: None,
                scene_invocation_id: Some(native_scene.invocation_id.clone()),
                resolve_invocation_id: None,
            })
        }
        crate::plugins::render::RenderFixedResolutionExecutionAdmission::Fixed(prepared) => {
            if frame.flow_invocations.iter().any(|invocation| {
                invocation.flow_id == prepared.scene_invocation.flow_id
                    && invocation.view_id == "main"
            }) {
                return Err(
                    RenderFixedResolutionExecutionEvidenceError::UnexpectedNativeSceneInvocation,
                );
            }

            let internal_view = frame
                .views
                .iter()
                .find(|view| view.view_id == prepared.internal_view.view_id)
                .filter(|view| {
                    view.kind == prepared.internal_view.kind
                        && view.target_size_px == prepared.internal_view.target_size_px
                        && view.history_signature == prepared.internal_view.history_signature
                })
                .ok_or(RenderFixedResolutionExecutionEvidenceError::MissingInternalView)?;

            let target = frame
                .dynamic_texture_targets
                .iter()
                .find(|target| target.key == prepared.target_key)
                .ok_or(RenderFixedResolutionExecutionEvidenceError::MissingDynamicTarget)?;
            if target != &prepared.dynamic_target {
                return Err(
                    RenderFixedResolutionExecutionEvidenceError::DynamicTargetDescriptorMismatch,
                );
            }

            let scene = frame
                .flow_invocations
                .iter()
                .find(|invocation| {
                    invocation.invocation_id == prepared.scene_invocation.invocation_id
                })
                .filter(|invocation| {
                    invocation.flow_id == prepared.scene_invocation.flow_id
                        && invocation.view_id == internal_view.view_id
                })
                .ok_or(RenderFixedResolutionExecutionEvidenceError::MissingSceneInvocation)?;
            if scene.target_alias_bindings != prepared.scene_invocation.target_alias_bindings
                || scene.history_signature != prepared.scene_invocation.history_signature
            {
                return Err(
                    RenderFixedResolutionExecutionEvidenceError::SceneTargetBindingMismatch,
                );
            }

            let resolve = frame
                .flow_invocations
                .iter()
                .find(|invocation| {
                    invocation.invocation_id == prepared.resolve_invocation.invocation_id
                })
                .filter(|invocation| {
                    invocation.flow_id == prepared.resolve_invocation.flow_id
                        && invocation.view_id == "main"
                })
                .ok_or(RenderFixedResolutionExecutionEvidenceError::MissingResolveInvocation)?;
            if resolve.target_alias_bindings != prepared.resolve_invocation.target_alias_bindings
                || resolve.history_signature != prepared.resolve_invocation.history_signature
            {
                return Err(
                    RenderFixedResolutionExecutionEvidenceError::ResolveSourceBindingMismatch,
                );
            }

            Ok(RenderFixedResolutionExecutionEvidence {
                resolution: RenderTemporalResolutionEvidence {
                    internal_size: [prepared.internal_size.0, prepared.internal_size.1],
                    output_size: [prepared.output_size.0, prepared.output_size.1],
                    policy: RenderTemporalResolutionPolicy::Fixed,
                },
                native_fallback_active: false,
                native_fallback_reason: None,
                target_key: Some(target.key.clone()),
                internal_view_id: Some(internal_view.view_id.clone()),
                scene_invocation_id: Some(scene.invocation_id.clone()),
                resolve_invocation_id: Some(resolve.invocation_id.clone()),
            })
        }
    }
}

pub fn inspect_render_temporal_inputs(
    request: RenderTemporalInspectionRequest,
) -> RenderTemporalInspection {
    let resolution = inspect_resolution(&request.resolution);
    let counts = count_inputs(&request.inputs);
    let mut diagnostics = Vec::new();

    validate_resolution(&request.resolution, &resolution, &mut diagnostics);
    validate_native_fallback(&request, &mut diagnostics);
    validate_jitter(&request.jitter, &mut diagnostics);
    validate_history(&request, &mut diagnostics);
    validate_inputs(&request, &mut diagnostics);
    validate_reconstruction_mode(&request, &mut diagnostics);

    RenderTemporalInspection {
        frame_index: request.frame_index,
        reconstruction_mode: request.reconstruction_mode,
        native_fallback_active: request.native_fallback_active,
        native_fallback_reason: request.native_fallback_reason,
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
        policy: resolution.policy,
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

    match evidence.policy {
        RenderTemporalResolutionPolicy::Native => {
            if evidence.internal_size != evidence.output_size {
                diagnostics.push(RenderTemporalDiagnostic::error(
                    "native_resolution_mismatch",
                    "native temporal resolution requires equal internal and output extents",
                ));
            }
        }
        RenderTemporalResolutionPolicy::Fixed => {}
        RenderTemporalResolutionPolicy::Dynamic {
            min_scale,
            max_scale,
        } => {
            if !min_scale.is_finite()
                || !max_scale.is_finite()
                || min_scale <= 0.0
                || max_scale <= 0.0
                || min_scale > max_scale
            {
                diagnostics.push(RenderTemporalDiagnostic::error(
                    "invalid_dynamic_resolution_limits",
                    "temporal dynamic-resolution scale limits must be finite, positive, and ordered",
                ));
                return;
            }

            if inspection.scale_x < min_scale
                || inspection.scale_x > max_scale
                || inspection.scale_y < min_scale
                || inspection.scale_y > max_scale
            {
                diagnostics.push(RenderTemporalDiagnostic::error(
                    "dynamic_resolution_out_of_bounds",
                    format!(
                        "temporal internal/output scale ({:.3}, {:.3}) is outside configured range [{:.3}, {:.3}]",
                        inspection.scale_x, inspection.scale_y, min_scale, max_scale
                    ),
                ));
            }
        }
    }
}

fn validate_native_fallback(
    request: &RenderTemporalInspectionRequest,
    diagnostics: &mut Vec<RenderTemporalDiagnostic>,
) {
    let reason = request
        .native_fallback_reason
        .as_deref()
        .unwrap_or("")
        .trim();
    if request.native_fallback_active && reason.is_empty() {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "native_fallback_missing_reason",
            "active native temporal fallback requires an explicit reason",
        ));
    }
    if !request.native_fallback_active && !reason.is_empty() {
        diagnostics.push(RenderTemporalDiagnostic::error(
            "inactive_native_fallback_has_reason",
            "native temporal fallback reason is present while fallback is inactive",
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

    if request.reconstruction_mode == RenderTemporalReconstructionMode::Taau {
        match request.resolution.policy {
            RenderTemporalResolutionPolicy::Native => {
                diagnostics.push(RenderTemporalDiagnostic::error(
                    "taau_without_scaled_resolution",
                    "TAAU reconstruction requires fixed sub-native or dynamic internal-resolution evidence",
                ));
            }
            RenderTemporalResolutionPolicy::Fixed => {
                if request.resolution.internal_size == request.resolution.output_size
                    || request.resolution.internal_size[0] > request.resolution.output_size[0]
                    || request.resolution.internal_size[1] > request.resolution.output_size[1]
                {
                    diagnostics.push(RenderTemporalDiagnostic::error(
                        "taau_without_scaled_resolution",
                        "fixed TAAU reconstruction requires a sub-native internal extent",
                    ));
                }
            }
            RenderTemporalResolutionPolicy::Dynamic { .. } => {
                if request.resolution.internal_size[0] > request.resolution.output_size[0]
                    || request.resolution.internal_size[1] > request.resolution.output_size[1]
                {
                    diagnostics.push(RenderTemporalDiagnostic::error(
                        "taau_without_scaled_resolution",
                        "dynamic TAAU reconstruction cannot use a current internal extent larger than output",
                    ));
                }
            }
        }
    }
}
