use engine::plugins::render::inspect::{
    RenderTemporalDiagnostic, RenderTemporalDiagnosticSeverity, RenderTemporalHistoryEvidence,
    RenderTemporalInputEvidence, RenderTemporalInputKind, RenderTemporalInspection,
    RenderTemporalInspectionRequest, RenderTemporalJitterEvidence,
    RenderTemporalReconstructionMode, RenderTemporalResolutionEvidence,
    inspect_render_temporal_inputs,
};

#[test]
fn render_temporal_inputs_report_ready_dynamic_resolution_chain() {
    let report = inspect_render_temporal_inputs(request());

    assert!(report.is_ready());
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.counts.required_input_count, 3);
    assert_eq!(report.counts.available_required_input_count, 3);
    assert_eq!(report.counts.missing_required_input_count, 0);
    assert_eq!(report.resolution.internal_size, [1280, 720]);
    assert_eq!(report.resolution.output_size, [1920, 1080]);
    assert_eq!(
        report.reconstruction_mode,
        RenderTemporalReconstructionMode::Taau
    );
}

#[test]
fn render_temporal_inputs_fail_closed_without_required_input_or_fallback() {
    let mut request = request();
    request.native_fallback_active = false;
    request.inputs[1].available = false;
    request.inputs[1].product_id = None;
    request.inputs[1].generation = None;

    let report = inspect_render_temporal_inputs(request);

    assert!(!report.is_ready());
    assert!(has_error(&report, "missing_required_input"));
}

#[test]
fn render_temporal_inputs_allow_missing_required_input_with_visible_native_fallback() {
    let mut request = request();
    request.native_fallback_active = true;
    request.inputs[0].available = false;
    request.inputs[0].product_id = None;
    request.inputs[0].generation = None;

    let report = inspect_render_temporal_inputs(request);

    assert!(report.is_ready());
    assert!(has_warning(
        &report,
        "required_input_missing_native_fallback"
    ));
}

#[test]
fn render_temporal_inputs_fail_closed_on_valid_history_signature_mismatch() {
    let mut request = request();
    request.history.previous_signature = Some("temporal.signature.previous".to_string());

    let report = inspect_render_temporal_inputs(request);

    assert!(!report.is_ready());
    assert!(has_error(&report, "valid_history_signature_mismatch"));
}

#[test]
fn render_temporal_inputs_fail_closed_on_hidden_dynamic_resolution() {
    let mut request = request();
    request.reconstruction_mode = RenderTemporalReconstructionMode::Taa;
    request.resolution.dynamic_resolution_enabled = false;

    let report = inspect_render_temporal_inputs(request);

    assert!(!report.is_ready());
    assert!(has_error(&report, "hidden_dynamic_resolution"));
}

#[test]
fn render_temporal_inputs_fail_closed_when_taau_lacks_dynamic_resolution() {
    let mut request = request();
    request.resolution.internal_size = [1920, 1080];
    request.resolution.dynamic_resolution_enabled = false;

    let report = inspect_render_temporal_inputs(request);

    assert!(!report.is_ready());
    assert!(has_error(&report, "taau_without_dynamic_resolution"));
}

fn request() -> RenderTemporalInspectionRequest {
    RenderTemporalInspectionRequest {
        frame_index: 7,
        reconstruction_mode: RenderTemporalReconstructionMode::Taau,
        native_fallback_active: false,
        resolution: RenderTemporalResolutionEvidence {
            internal_size: [1280, 720],
            output_size: [1920, 1080],
            min_scale: 0.5,
            max_scale: 1.0,
            dynamic_resolution_enabled: true,
        },
        jitter: RenderTemporalJitterEvidence {
            sequence_id: "halton-2-3:v1".to_string(),
            phase_index: 3,
            phase_count: 8,
            offset: [0.25, -0.125],
        },
        history: RenderTemporalHistoryEvidence {
            resource_id: "history.main.color".to_string(),
            current_signature: "temporal.signature.current".to_string(),
            previous_signature: Some("temporal.signature.current".to_string()),
            age_frames: 4,
            valid: true,
            invalidation_reason: None,
        },
        inputs: vec![
            input(RenderTemporalInputKind::MotionVectors, true, true),
            input(RenderTemporalInputKind::Depth, true, true),
            input(RenderTemporalInputKind::Exposure, true, true),
            input(RenderTemporalInputKind::ReactiveMask, false, false),
        ],
    }
}

fn input(
    kind: RenderTemporalInputKind,
    required: bool,
    available: bool,
) -> RenderTemporalInputEvidence {
    RenderTemporalInputEvidence {
        kind,
        required,
        available,
        product_id: available.then(|| format!("product.{}", kind.as_str())),
        generation: available.then_some(42),
    }
}

fn has_error(report: &RenderTemporalInspection, code: &str) -> bool {
    has_diagnostic(report, code, RenderTemporalDiagnosticSeverity::Error)
}

fn has_warning(report: &RenderTemporalInspection, code: &str) -> bool {
    has_diagnostic(report, code, RenderTemporalDiagnosticSeverity::Warning)
}

fn has_diagnostic(
    report: &RenderTemporalInspection,
    code: &str,
    severity: RenderTemporalDiagnosticSeverity,
) -> bool {
    report
        .diagnostics
        .iter()
        .any(|diagnostic: &RenderTemporalDiagnostic| {
            diagnostic.severity == severity && diagnostic.code == code
        })
}
