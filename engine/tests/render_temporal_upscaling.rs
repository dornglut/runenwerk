use engine::plugins::render::inspect::{
    RenderRayReconstructionInputEvidence, RenderRayReconstructionInputKind,
    RenderTemporalDiagnostic, RenderTemporalDiagnosticSeverity, RenderTemporalHistoryEvidence,
    RenderTemporalInputEvidence, RenderTemporalInputKind, RenderTemporalInspection,
    RenderTemporalInspectionRequest, RenderTemporalJitterEvidence,
    RenderTemporalReconstructionMode, RenderTemporalResolutionEvidence,
    RenderTemporalUpscalingAdapterEvidence, RenderTemporalUpscalingAdapterKind,
    RenderTemporalUpscalingCapabilityState, RenderTemporalUpscalingInspection,
    RenderTemporalUpscalingInspectionRequest, inspect_render_temporal_inputs,
    inspect_render_temporal_upscaling,
};

#[test]
fn render_temporal_upscaling_reports_ready_supported_adapter_and_ray_inputs() {
    let report = inspect_render_temporal_upscaling(request(false));

    assert!(report.is_ready());
    assert!(report.adapter_invocation_allowed);
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.ray_input_counts.required_input_count, 5);
    assert_eq!(report.ray_input_counts.available_required_input_count, 5);
    assert_eq!(
        report.adapter.kind,
        RenderTemporalUpscalingAdapterKind::FsrStyle
    );
}

#[test]
fn render_temporal_upscaling_allows_unsupported_adapter_with_visible_native_fallback() {
    let mut request = request(true);
    request.adapter.capability_state = RenderTemporalUpscalingCapabilityState::Unsupported;
    request.adapter.unsupported_reason =
        Some("backend capability profile lacks adapter hook".to_string());

    let report = inspect_render_temporal_upscaling(request);

    assert!(report.is_ready());
    assert!(!report.adapter_invocation_allowed);
    assert!(has_warning(&report, "adapter_unavailable_native_fallback"));
}

#[test]
fn render_temporal_upscaling_fails_when_unsupported_adapter_hides_fallback() {
    let mut request = request(false);
    request.adapter.capability_state = RenderTemporalUpscalingCapabilityState::Unsupported;
    request.adapter.unsupported_reason = Some("adapter feature missing".to_string());

    let report = inspect_render_temporal_upscaling(request);

    assert!(!report.is_ready());
    assert!(has_error(
        &report,
        "adapter_unavailable_without_visible_fallback"
    ));
}

#[test]
fn render_temporal_upscaling_fails_closed_without_required_ray_input_or_fallback() {
    let mut request = request(false);
    request.ray_inputs[5].available = false;
    request.ray_inputs[5].product_id = None;
    request.ray_inputs[5].generation = None;

    let report = inspect_render_temporal_upscaling(request);

    assert!(!report.is_ready());
    assert!(!report.adapter_invocation_allowed);
    assert!(has_error(
        &report,
        "missing_required_ray_reconstruction_input"
    ));
}

#[test]
fn render_temporal_upscaling_reports_missing_ray_input_as_fallback_warning() {
    let mut request = request(true);
    request.ray_inputs[5].available = false;
    request.ray_inputs[5].product_id = None;
    request.ray_inputs[5].generation = None;

    let report = inspect_render_temporal_upscaling(request);

    assert!(report.is_ready());
    assert!(!report.adapter_invocation_allowed);
    assert!(has_warning(
        &report,
        "required_ray_input_missing_native_fallback"
    ));
}

#[test]
fn render_temporal_upscaling_fails_when_adapter_is_required_for_correctness() {
    let mut request = request(false);
    request.adapter_required_for_correctness = true;

    let report = inspect_render_temporal_upscaling(request);

    assert!(!report.is_ready());
    assert!(!report.adapter_invocation_allowed);
    assert!(has_error(&report, "adapter_required_for_correctness"));
}

#[test]
fn render_temporal_upscaling_propagates_temporal_input_errors() {
    let mut request = request(false);
    request.temporal.history.previous_signature = Some("temporal.signature.previous".to_string());
    request.temporal.diagnostics.push(RenderTemporalDiagnostic {
        severity: RenderTemporalDiagnosticSeverity::Error,
        code: "valid_history_signature_mismatch",
        message: "temporal history is marked valid but previous and current signatures differ"
            .to_string(),
    });

    let report = inspect_render_temporal_upscaling(request);

    assert!(!report.is_ready());
    assert!(!report.adapter_invocation_allowed);
    assert!(has_error(&report, "temporal_input_error"));
}

fn request(native_fallback_visible: bool) -> RenderTemporalUpscalingInspectionRequest {
    RenderTemporalUpscalingInspectionRequest {
        temporal: temporal(native_fallback_visible),
        adapter: RenderTemporalUpscalingAdapterEvidence {
            kind: RenderTemporalUpscalingAdapterKind::FsrStyle,
            capability_state: RenderTemporalUpscalingCapabilityState::Supported,
            required_capabilities: vec![
                "temporal.history.valid".to_string(),
                "dynamic_resolution".to_string(),
                "ray_reconstruction.inputs".to_string(),
            ],
            unsupported_reason: None,
            invocation_requested: true,
        },
        ray_inputs: vec![
            ray_input(RenderRayReconstructionInputKind::MotionVectors, true, true),
            ray_input(RenderRayReconstructionInputKind::Depth, true, true),
            ray_input(RenderRayReconstructionInputKind::Exposure, true, true),
            ray_input(RenderRayReconstructionInputKind::ReactiveMask, false, true),
            ray_input(
                RenderRayReconstructionInputKind::DisocclusionMask,
                false,
                false,
            ),
            ray_input(
                RenderRayReconstructionInputKind::RaymarchDistance,
                true,
                true,
            ),
            ray_input(
                RenderRayReconstructionInputKind::RayQueryHitDistance,
                true,
                true,
            ),
        ],
        native_fallback_visible,
        adapter_required_for_correctness: false,
    }
}

fn temporal(native_fallback_active: bool) -> RenderTemporalInspection {
    inspect_render_temporal_inputs(RenderTemporalInspectionRequest {
        frame_index: 11,
        reconstruction_mode: RenderTemporalReconstructionMode::Taau,
        native_fallback_active,
        resolution: RenderTemporalResolutionEvidence {
            internal_size: [1280, 720],
            output_size: [1920, 1080],
            min_scale: 0.5,
            max_scale: 1.0,
            dynamic_resolution_enabled: true,
        },
        jitter: RenderTemporalJitterEvidence {
            sequence_id: "halton-2-3:v1".to_string(),
            phase_index: 4,
            phase_count: 8,
            offset: [-0.125, 0.25],
        },
        history: RenderTemporalHistoryEvidence {
            resource_id: "history.main.color".to_string(),
            current_signature: "temporal.signature.current".to_string(),
            previous_signature: Some("temporal.signature.current".to_string()),
            age_frames: 5,
            valid: true,
            invalidation_reason: None,
        },
        inputs: vec![
            temporal_input(RenderTemporalInputKind::MotionVectors, true, true),
            temporal_input(RenderTemporalInputKind::Depth, true, true),
            temporal_input(RenderTemporalInputKind::Exposure, true, true),
            temporal_input(RenderTemporalInputKind::ReactiveMask, false, true),
        ],
    })
}

fn temporal_input(
    kind: RenderTemporalInputKind,
    required: bool,
    available: bool,
) -> RenderTemporalInputEvidence {
    RenderTemporalInputEvidence {
        kind,
        required,
        available,
        product_id: available.then(|| format!("temporal.product.{}", kind.as_str())),
        generation: available.then_some(51),
    }
}

fn ray_input(
    kind: RenderRayReconstructionInputKind,
    required: bool,
    available: bool,
) -> RenderRayReconstructionInputEvidence {
    RenderRayReconstructionInputEvidence {
        kind,
        required,
        available,
        product_id: available.then(|| format!("ray.product.{}", kind.as_str())),
        generation: available.then_some(77),
    }
}

fn has_error(report: &RenderTemporalUpscalingInspection, code: &str) -> bool {
    has_diagnostic(report, code, RenderTemporalDiagnosticSeverity::Error)
}

fn has_warning(report: &RenderTemporalUpscalingInspection, code: &str) -> bool {
    has_diagnostic(report, code, RenderTemporalDiagnosticSeverity::Warning)
}

fn has_diagnostic(
    report: &RenderTemporalUpscalingInspection,
    code: &str,
    severity: RenderTemporalDiagnosticSeverity,
) -> bool {
    report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == severity && diagnostic.code == code)
}
