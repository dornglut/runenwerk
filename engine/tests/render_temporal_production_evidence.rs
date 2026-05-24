use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuTimingCapability,
    RenderGpuTimingDiagnostic, RenderRayReconstructionInputEvidence,
    RenderRayReconstructionInputKind, RenderTemporalHistoryEvidence, RenderTemporalInputEvidence,
    RenderTemporalInputKind, RenderTemporalInspection, RenderTemporalInspectionRequest,
    RenderTemporalJitterEvidence, RenderTemporalProductionEvidenceReport,
    RenderTemporalProductionEvidenceRequest, RenderTemporalProductionEvidenceSeverity,
    RenderTemporalProductionHardwareProfile, RenderTemporalReconstructionMode,
    RenderTemporalResolutionEvidence, RenderTemporalRuntimeVisualEvidence,
    RenderTemporalUpscalingAdapterEvidence, RenderTemporalUpscalingAdapterKind,
    RenderTemporalUpscalingCapabilityState, RenderTemporalUpscalingInspection,
    RenderTemporalUpscalingInspectionRequest, inspect_render_temporal_inputs,
    inspect_render_temporal_production_evidence, inspect_render_temporal_upscaling,
};

#[test]
fn render_temporal_production_evidence_reports_runtime_ready_temporal_chain() {
    let report = inspect_render_temporal_production_evidence(request());

    assert!(report.is_runtime_ready());
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.counts.temporal_required_input_count, 3);
    assert_eq!(report.counts.ray_required_input_count, 5);
    assert_eq!(report.counts.visual_evidence_count, 2);
    assert_eq!(report.counts.fallback_visual_count, 1);
    assert_eq!(report.timings.gpu_timing_diagnostic_count, 1);
}

#[test]
fn render_temporal_production_evidence_fails_without_visual_evidence() {
    let mut request = request();
    request.visual_evidence.clear();

    let report = inspect_render_temporal_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(has_error(&report, "missing_visual_evidence"));
    assert!(has_error(&report, "missing_rendered_pixel_count"));
}

#[test]
fn render_temporal_production_evidence_fails_without_benchmark_or_artifact_paths() {
    let mut request = request();
    request.benchmark_commands.clear();
    request.artifact_paths.clear();

    let report = inspect_render_temporal_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(has_error(&report, "missing_benchmark_command"));
    assert!(has_error(&report, "missing_artifact_path"));
}

#[test]
fn render_temporal_production_evidence_fails_when_visual_skips_source_inspections() {
    let mut request = request();
    request.visual_evidence[0].consumed_temporal_inputs = false;
    request.visual_evidence[0].consumed_temporal_upscaling = false;

    let report = inspect_render_temporal_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(has_error(&report, "visual_without_temporal_inputs"));
    assert!(has_error(&report, "visual_without_temporal_upscaling"));
}

#[test]
fn render_temporal_production_evidence_rejects_fallback_only_runtime_claims() {
    let mut request = request();
    for evidence in &mut request.visual_evidence {
        evidence.native_fallback_visible = true;
        evidence.reconstruction_mode = RenderTemporalReconstructionMode::Native;
    }

    let report = inspect_render_temporal_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(has_error(&report, "fallback_only_temporal_claim"));
}

#[test]
fn render_temporal_production_evidence_fails_when_upscaling_report_is_not_ready() {
    let mut request = request();
    request.upscaling.ray_inputs[5].available = false;
    request.upscaling.ray_inputs[5].product_id = None;
    request.upscaling.ray_inputs[5].generation = None;
    request.upscaling =
        inspect_render_temporal_upscaling(RenderTemporalUpscalingInspectionRequest {
            temporal: request.temporal.clone(),
            adapter: request.upscaling.adapter,
            ray_inputs: request.upscaling.ray_inputs,
            native_fallback_visible: false,
            adapter_required_for_correctness: false,
        });

    let report = inspect_render_temporal_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(has_error(&report, "temporal_upscaling_not_ready"));
    assert!(has_error(
        &report,
        "temporal_upscaling_missing_required_ray_reconstruction_input"
    ));
}

fn request() -> RenderTemporalProductionEvidenceRequest {
    let temporal = temporal();
    let upscaling = upscaling(temporal.clone());
    RenderTemporalProductionEvidenceRequest {
        hardware_profile: RenderTemporalProductionHardwareProfile {
            profile_key: "test-temporal-production-profile".to_string(),
            adapter_name: Some("test portable profile".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        temporal,
        upscaling,
        timings: timings(),
        visual_evidence: visual_evidence(),
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-temporal-production-evidence/summary.txt"
                .to_string(),
            "docs-site/src/content/docs/reports/benchmarks/render/temporal-production-evidence.md"
                .to_string(),
        ],
    }
}

fn temporal() -> RenderTemporalInspection {
    inspect_render_temporal_inputs(RenderTemporalInspectionRequest {
        frame_index: 17,
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
            phase_index: 5,
            phase_count: 8,
            offset: [0.125, -0.25],
        },
        history: RenderTemporalHistoryEvidence {
            resource_id: "history.main.color".to_string(),
            current_signature: "temporal.signature.current".to_string(),
            previous_signature: Some("temporal.signature.current".to_string()),
            age_frames: 6,
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

fn upscaling(temporal: RenderTemporalInspection) -> RenderTemporalUpscalingInspection {
    inspect_render_temporal_upscaling(RenderTemporalUpscalingInspectionRequest {
        temporal,
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
        native_fallback_visible: false,
        adapter_required_for_correctness: false,
    })
}

fn timings() -> RenderDebugTimingsState {
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[
        PassTimingSample {
            flow_id: "temporal.production".to_string(),
            pass_id: "temporal.reconstruct".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.24,
            dispatch_workgroups: None,
        },
        PassTimingSample {
            flow_id: "temporal.production".to_string(),
            pass_id: "temporal.resolve".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.18,
            dispatch_workgroups: None,
        },
    ]);
    timings.observe_gpu_timing_diagnostic(RenderGpuTimingDiagnostic::unsupported(
        "test temporal production evidence records unsupported timestamp queries explicitly",
    ));
    timings
}

fn visual_evidence() -> Vec<RenderTemporalRuntimeVisualEvidence> {
    vec![
        RenderTemporalRuntimeVisualEvidence {
            view_label: "test.temporal.taau".to_string(),
            artifact_path:
                "engine/benchmark-artifacts/render-temporal-production-evidence/taau.txt"
                    .to_string(),
            reconstruction_mode: RenderTemporalReconstructionMode::Taau,
            internal_size: [1280, 720],
            output_size: [1920, 1080],
            rendered_pixel_count: 8192,
            history_valid: true,
            native_fallback_visible: false,
            consumed_temporal_inputs: true,
            consumed_temporal_upscaling: true,
        },
        RenderTemporalRuntimeVisualEvidence {
            view_label: "test.temporal.native_fallback".to_string(),
            artifact_path:
                "engine/benchmark-artifacts/render-temporal-production-evidence/fallback.txt"
                    .to_string(),
            reconstruction_mode: RenderTemporalReconstructionMode::Native,
            internal_size: [1920, 1080],
            output_size: [1920, 1080],
            rendered_pixel_count: 4096,
            history_valid: true,
            native_fallback_visible: true,
            consumed_temporal_inputs: true,
            consumed_temporal_upscaling: true,
        },
    ]
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
        generation: available.then_some(101),
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
        generation: available.then_some(202),
    }
}

fn has_error(report: &RenderTemporalProductionEvidenceReport, code: &str) -> bool {
    report.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == RenderTemporalProductionEvidenceSeverity::Error
            && diagnostic.code == code
    })
}
