use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuTimingCapability,
    RenderGpuTimingDiagnostic, RenderRayReconstructionInputEvidence,
    RenderRayReconstructionInputKind, RenderTemporalHistoryEvidence, RenderTemporalInputEvidence,
    RenderTemporalInputKind, RenderTemporalInspection, RenderTemporalInspectionRequest,
    RenderTemporalJitterEvidence, RenderTemporalProductionEvidenceReport,
    RenderTemporalProductionEvidenceRequest, RenderTemporalProductionHardwareProfile,
    RenderTemporalReconstructionMode, RenderTemporalResolutionEvidence,
    RenderTemporalRuntimeVisualEvidence, RenderTemporalUpscalingAdapterEvidence,
    RenderTemporalUpscalingAdapterKind, RenderTemporalUpscalingCapabilityState,
    RenderTemporalUpscalingInspection, RenderTemporalUpscalingInspectionRequest,
    inspect_render_temporal_inputs, inspect_render_temporal_production_evidence,
    inspect_render_temporal_upscaling,
};

fn main() {
    let report = build_report();
    println!(
        "render temporal production evidence ready={} errors={} warnings={} profile={}",
        report.is_runtime_ready(),
        report.error_count(),
        report.warning_count(),
        report.hardware_profile.profile_key
    );
    println!(
        "temporal_required={} ray_required={} visual={} fallback_visuals={} pixels={} cpu_ms={:.3} gpu_diagnostics={}",
        report.counts.temporal_required_input_count,
        report.counts.ray_required_input_count,
        report.counts.visual_evidence_count,
        report.counts.fallback_visual_count,
        report.counts.rendered_pixel_count,
        report.timings.cpu_total_pass_millis,
        report.timings.gpu_timing_diagnostic_count
    );
}

fn build_report() -> RenderTemporalProductionEvidenceReport {
    let temporal = temporal();
    let upscaling = upscaling(temporal.clone());
    inspect_render_temporal_production_evidence(RenderTemporalProductionEvidenceRequest {
        hardware_profile: RenderTemporalProductionHardwareProfile {
            profile_key: "standalone-temporal-production".to_string(),
            adapter_name: Some("standalone portable profile".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        temporal,
        upscaling,
        timings: timings(),
        visual_evidence: visual_evidence(),
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-temporal-production-evidence/README.md".to_string(),
            "docs-site/src/content/docs/reports/benchmarks/render/temporal-production-evidence.md"
                .to_string(),
        ],
    })
}

fn temporal() -> RenderTemporalInspection {
    inspect_render_temporal_inputs(RenderTemporalInspectionRequest {
        frame_index: 23,
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
            phase_index: 6,
            phase_count: 8,
            offset: [-0.25, 0.125],
        },
        history: RenderTemporalHistoryEvidence {
            resource_id: "history.main.color".to_string(),
            current_signature: "temporal.signature.current".to_string(),
            previous_signature: Some("temporal.signature.current".to_string()),
            age_frames: 7,
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
        "standalone temporal evidence command records unsupported timestamp queries explicitly",
    ));
    timings
}

fn visual_evidence() -> Vec<RenderTemporalRuntimeVisualEvidence> {
    vec![
        RenderTemporalRuntimeVisualEvidence {
            view_label: "standalone.temporal.taau".to_string(),
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
            view_label: "standalone.temporal.native_fallback".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_builds_ready_temporal_production_report() {
        let report = build_report();

        assert!(report.is_runtime_ready());
        assert_eq!(report.counts.temporal_required_input_count, 3);
        assert_eq!(report.counts.ray_required_input_count, 5);
        assert_eq!(report.counts.rendered_pixel_count, 12_288);
        assert_eq!(report.timings.gpu_timing_diagnostic_count, 1);
    }
}
