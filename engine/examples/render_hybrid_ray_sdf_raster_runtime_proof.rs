use engine::plugins::render::features::world::sdf_raymarch::{
    RenderSdfDistanceMipLevel, RenderSdfRaymarchAccelerationReport, RenderSdfRaymarchCandidate,
    RenderSdfRaymarchCandidateList,
};
use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuTimingCapability,
    RenderGpuTimingDiagnostic, RenderRayQueryAccelerationResourceEvidence,
    RenderRayQueryAccelerationResourceKind, RenderRayQueryAccelerationResourceStatus,
    RenderRayQueryAccelerationSourceLineage, RenderRayQueryCapabilityProfile,
    RenderRayQueryCapabilityState, RenderRayQueryInspection, RenderRayQueryInspectionRequest,
    RenderRayReconstructionInputEvidence, RenderRayReconstructionInputKind,
    RenderSdfProductionEvidenceReport, RenderSdfProductionEvidenceRequest,
    RenderSdfProductionHardwareProfile, RenderSdfResidencyBudgetInspection,
    RenderSdfResidencyInspection, RenderSdfRuntimeVisualEvidence, RenderTemporalHistoryEvidence,
    RenderTemporalInputEvidence, RenderTemporalInputKind, RenderTemporalInspection,
    RenderTemporalInspectionRequest, RenderTemporalJitterEvidence,
    RenderTemporalProductionEvidenceReport, RenderTemporalProductionEvidenceRequest,
    RenderTemporalProductionHardwareProfile, RenderTemporalReconstructionMode,
    RenderTemporalResolutionEvidence, RenderTemporalRuntimeVisualEvidence,
    RenderTemporalUpscalingAdapterEvidence, RenderTemporalUpscalingAdapterKind,
    RenderTemporalUpscalingCapabilityState, RenderTemporalUpscalingInspection,
    RenderTemporalUpscalingInspectionRequest, inspect_render_ray_query_capability,
    inspect_render_sdf_production_evidence, inspect_render_temporal_inputs,
    inspect_render_temporal_production_evidence, inspect_render_temporal_upscaling,
};

fn main() {
    let report = build_report();
    println!(
        "render hybrid ray/SDF/raster proof ready={} errors={} warnings={}",
        report.is_ready(),
        report.error_count(),
        report.warning_count()
    );
    println!(
        "raster_passes={} sdf_ready={} temporal_ready={} ray_query_supported={} fallback_visible={} timing_passes={} timing_labels={}",
        report.raster_passes.len(),
        report.sdf.is_runtime_ready(),
        report.temporal.is_runtime_ready(),
        report.ray_query_supported.ray_query_invocation_allowed,
        report.fallback.visible,
        report.timings.pass_sample_count,
        report.timing_pass_labels.join(",")
    );
}

#[derive(Debug, Clone)]
struct HybridRuntimeProofReport {
    raster_passes: Vec<HybridRasterPassEvidence>,
    sdf: RenderSdfProductionEvidenceReport,
    temporal: RenderTemporalProductionEvidenceReport,
    ray_query_supported: RenderRayQueryInspection,
    ray_query_fallback: RenderRayQueryInspection,
    fallback: HybridFallbackEvidence,
    timings: RenderDebugTimingsState,
    timing_pass_labels: Vec<String>,
}

impl HybridRuntimeProofReport {
    fn is_ready(&self) -> bool {
        self.error_count() == 0
            && !self.raster_passes.is_empty()
            && self.sdf.is_runtime_ready()
            && self.temporal.is_runtime_ready()
            && self.ray_query_supported.ray_query_invocation_allowed
            && !self.ray_query_fallback.ray_query_invocation_allowed
            && self
                .ray_query_fallback
                .capability_profile
                .native_fallback_visible
            && self.fallback.visible
            && self.has_required_timing_labels()
    }

    fn error_count(&self) -> usize {
        self.sdf.error_count()
            + self.temporal.error_count()
            + self.ray_query_supported.error_count()
            + self.ray_query_fallback.error_count()
            + usize::from(self.raster_passes.is_empty())
            + usize::from(!self.fallback.visible)
            + usize::from(!self.has_required_timing_labels())
    }

    fn warning_count(&self) -> usize {
        self.sdf.warning_count()
            + self.temporal.warning_count()
            + self.ray_query_supported.warning_count()
            + self.ray_query_fallback.warning_count()
    }

    fn has_required_timing_labels(&self) -> bool {
        ["raster", "sdf", "temporal", "ray_query", "fallback"]
            .iter()
            .all(|required| {
                self.timing_pass_labels
                    .iter()
                    .any(|label| label == required)
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HybridRasterPassEvidence {
    pass_id: String,
    material_binding: String,
    output_view: String,
    rendered_pixel_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HybridFallbackEvidence {
    view_label: String,
    artifact_path: String,
    visible: bool,
    reason: String,
}

fn build_report() -> HybridRuntimeProofReport {
    let timing_samples = timing_samples();
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&timing_samples);
    timings.observe_gpu_timing_diagnostic(RenderGpuTimingDiagnostic::unsupported(
        "standalone hybrid proof records unavailable timestamp-query evidence explicitly",
    ));

    HybridRuntimeProofReport {
        raster_passes: raster_passes(),
        sdf: sdf_report(),
        temporal: temporal_report(),
        ray_query_supported: ray_query_supported(),
        ray_query_fallback: ray_query_fallback(),
        fallback: fallback_evidence(),
        timings,
        timing_pass_labels: timing_samples
            .iter()
            .map(|sample| sample.pass_id.clone())
            .collect(),
    }
}

fn raster_passes() -> Vec<HybridRasterPassEvidence> {
    vec![HybridRasterPassEvidence {
        pass_id: "raster".to_string(),
        material_binding: "scene.material.bindings.public".to_string(),
        output_view: "hybrid.main.color".to_string(),
        rendered_pixel_count: 12_288,
    }]
}

fn sdf_report() -> RenderSdfProductionEvidenceReport {
    inspect_render_sdf_production_evidence(RenderSdfProductionEvidenceRequest {
        hardware_profile: RenderSdfProductionHardwareProfile {
            profile_key: "hybrid-proof-sdf".to_string(),
            adapter_name: Some("standalone portable profile".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        residency: sdf_residency(),
        raymarch: sdf_raymarch(),
        timings: sdf_timings(),
        visual_evidence: sdf_visual_evidence(),
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-sdf-runtime-evidence/README.md".to_string(),
            "docs-site/src/content/docs/engine/benchmarks/render-sdf-runtime-evidence.md"
                .to_string(),
        ],
    })
}

fn sdf_residency() -> RenderSdfResidencyInspection {
    RenderSdfResidencyInspection {
        addressable_product_count: 3,
        selected_product_count: 3,
        requested_product_count: 3,
        resident_product_count: 3,
        resident_page_count: 8,
        resident_brick_count: 8,
        clipmap_window_count: 3,
        invalidated_product_count: 0,
        rejected_product_count: 0,
        resident_bytes: 32_768,
        upload_bytes: 4096,
        budget: RenderSdfResidencyBudgetInspection {
            page_status: "within_budget".to_string(),
            brick_status: "within_budget".to_string(),
            resident_byte_status: "within_budget".to_string(),
            upload_byte_status: "within_budget".to_string(),
            clipmap_page_status: "within_budget".to_string(),
        },
        diagnostic_count: 0,
        diagnostics: Vec::new(),
        entries: Vec::new(),
        clipmap_windows: Vec::new(),
    }
}

fn sdf_raymarch() -> RenderSdfRaymarchAccelerationReport {
    let candidates = (1..=3)
        .map(|product_id| RenderSdfRaymarchCandidate {
            product_id,
            cache_generation: product_id + 10,
            scale_band: match product_id {
                1 => "Near",
                2 => "Mid",
                _ => "Far",
            }
            .to_string(),
            page_count: if product_id == 3 { 4 } else { 2 },
            brick_count: if product_id == 3 { 4 } else { 2 },
            resident_bytes: if product_id == 3 { 16_384 } else { 8192 },
        })
        .collect::<Vec<_>>();

    RenderSdfRaymarchAccelerationReport {
        resident_product_count: 3,
        resident_page_count: 8,
        resident_brick_count: 8,
        clipmap_window_count: 3,
        distance_mips: (0_u8..3)
            .map(|level| RenderSdfDistanceMipLevel {
                level,
                source_page_count: if level == 2 { 4 } else { 2 },
                source_brick_count: if level == 2 { 4 } else { 2 },
                conservative_min_distance: 0.0,
                max_safe_step: 1.0 / (f32::from(level) + 1.0),
                unsafe_overstep_risk: false,
            })
            .collect(),
        candidate_lists: vec![RenderSdfRaymarchCandidateList {
            tile_index: 0,
            depth_slice: 0,
            candidate_count: candidates.len(),
            rejected_candidate_count: 0,
            candidates,
        }],
        total_candidate_count: 3,
        rejected_candidate_count: 0,
        max_candidates_per_list: 8,
        max_steps_per_ray: 160,
        fullscreen_entity_multiplier: 1,
        diagnostics: Vec::new(),
    }
}

fn sdf_timings() -> RenderDebugTimingsState {
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[PassTimingSample {
        flow_id: "hybrid.proof".to_string(),
        pass_id: "sdf".to_string(),
        pass_kind: "compute".to_string(),
        millis: 0.21,
        dispatch_workgroups: Some([1, 1, 1]),
    }]);
    timings.observe_gpu_timing_diagnostic(RenderGpuTimingDiagnostic::unsupported(
        "standalone hybrid SDF evidence records unsupported timestamp queries explicitly",
    ));
    timings
}

fn sdf_visual_evidence() -> Vec<RenderSdfRuntimeVisualEvidence> {
    ["near", "mid", "far", "summary"]
        .into_iter()
        .enumerate()
        .map(|(index, coverage_band)| RenderSdfRuntimeVisualEvidence {
            view_label: format!("hybrid.sdf.{coverage_band}"),
            coverage_band: coverage_band.to_string(),
            artifact_path: format!(
                "engine/benchmark-artifacts/render-sdf-runtime-evidence/{coverage_band}.txt"
            ),
            step_count: 32 + u32::try_from(index).expect("example index fits in u32") * 8,
            missed_surface_risk: false,
            overstep_risk: false,
        })
        .collect()
}

fn temporal_report() -> RenderTemporalProductionEvidenceReport {
    let temporal = temporal_inputs();
    let upscaling = temporal_upscaling(temporal.clone());
    inspect_render_temporal_production_evidence(RenderTemporalProductionEvidenceRequest {
        hardware_profile: RenderTemporalProductionHardwareProfile {
            profile_key: "hybrid-proof-temporal".to_string(),
            adapter_name: Some("standalone portable profile".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        temporal,
        upscaling,
        timings: temporal_timings(),
        visual_evidence: temporal_visual_evidence(),
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-temporal-production-evidence/README.md".to_string(),
            "docs-site/src/content/docs/reports/benchmarks/render/temporal-production-evidence.md"
                .to_string(),
        ],
    })
}

fn temporal_inputs() -> RenderTemporalInspection {
    inspect_render_temporal_inputs(RenderTemporalInspectionRequest {
        frame_index: 74,
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
            phase_index: 2,
            phase_count: 8,
            offset: [0.125, -0.25],
        },
        history: RenderTemporalHistoryEvidence {
            resource_id: "hybrid.history.color".to_string(),
            current_signature: "hybrid.temporal.signature".to_string(),
            previous_signature: Some("hybrid.temporal.signature".to_string()),
            age_frames: 4,
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

fn temporal_upscaling(temporal: RenderTemporalInspection) -> RenderTemporalUpscalingInspection {
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

fn temporal_timings() -> RenderDebugTimingsState {
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[PassTimingSample {
        flow_id: "hybrid.proof".to_string(),
        pass_id: "temporal".to_string(),
        pass_kind: "fullscreen".to_string(),
        millis: 0.28,
        dispatch_workgroups: None,
    }]);
    timings.observe_gpu_timing_diagnostic(RenderGpuTimingDiagnostic::unsupported(
        "standalone hybrid temporal evidence records unsupported timestamp queries explicitly",
    ));
    timings
}

fn temporal_visual_evidence() -> Vec<RenderTemporalRuntimeVisualEvidence> {
    vec![
        RenderTemporalRuntimeVisualEvidence {
            view_label: "hybrid.temporal.taau".to_string(),
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
            view_label: "hybrid.temporal.native_fallback".to_string(),
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
        product_id: available.then(|| format!("hybrid.temporal.{}", kind.as_str())),
        generation: available.then_some(303),
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
        product_id: available.then(|| format!("hybrid.ray.{}", kind.as_str())),
        generation: available.then_some(404),
    }
}

fn ray_query_supported() -> RenderRayQueryInspection {
    inspect_render_ray_query_capability(RenderRayQueryInspectionRequest {
        capability_profile: RenderRayQueryCapabilityProfile {
            backend: Some("wgpu-test-profile".to_string()),
            ray_query: RenderRayQueryCapabilityState::Supported,
            raytracing_pipeline: RenderRayQueryCapabilityState::Supported,
            acceleration_structure_build: RenderRayQueryCapabilityState::Supported,
            shader_table: RenderRayQueryCapabilityState::Supported,
            timestamp_query: RenderRayQueryCapabilityState::Supported,
            readback: RenderRayQueryCapabilityState::Supported,
            required_capabilities: vec![
                "ray_query".to_string(),
                "acceleration_structure".to_string(),
            ],
            unsupported_reason: None,
            native_fallback_visible: false,
        },
        acceleration_resources: vec![
            acceleration_resource(
                RenderRayQueryAccelerationResourceKind::BottomLevel,
                "hybrid.blas.scene_mesh",
                acceleration_lineage("mesh", "scene_mesh.lod0", 4100, 7, "mesh-cache:4100:7"),
                16_384,
            ),
            acceleration_resource(
                RenderRayQueryAccelerationResourceKind::TopLevel,
                "hybrid.tlas.main_scene",
                acceleration_lineage("scene", "scene.hybrid.main", 4200, 7, "scene-cache:4200:7"),
                8192,
            ),
        ],
        max_acceleration_resource_bytes: Some(65_536),
    })
}

fn ray_query_fallback() -> RenderRayQueryInspection {
    inspect_render_ray_query_capability(RenderRayQueryInspectionRequest {
        capability_profile: RenderRayQueryCapabilityProfile::portable_unsupported(
            "portable example profile has no hardware ray-query support",
        ),
        acceleration_resources: Vec::new(),
        max_acceleration_resource_bytes: Some(0),
    })
}

fn acceleration_resource(
    kind: RenderRayQueryAccelerationResourceKind,
    debug_label: &str,
    lineage: RenderRayQueryAccelerationSourceLineage,
    memory_bytes: u64,
) -> RenderRayQueryAccelerationResourceEvidence {
    let build_version = lineage.generation;
    RenderRayQueryAccelerationResourceEvidence {
        kind,
        debug_label: debug_label.to_string(),
        status: RenderRayQueryAccelerationResourceStatus::Ready,
        source_lineage: vec![lineage],
        memory_bytes,
        build_version,
        invalidation_reason: None,
        exposes_backend_handle: false,
    }
}

fn acceleration_lineage(
    source_kind: &str,
    source_id: &str,
    product_id: u64,
    generation: u64,
    cache_id: &str,
) -> RenderRayQueryAccelerationSourceLineage {
    RenderRayQueryAccelerationSourceLineage {
        source_kind: source_kind.to_string(),
        source_id: source_id.to_string(),
        product_id: Some(product_id),
        generation: Some(generation),
        cache_id: Some(cache_id.to_string()),
    }
}

fn fallback_evidence() -> HybridFallbackEvidence {
    HybridFallbackEvidence {
        view_label: "hybrid.native.non_rt_fallback".to_string(),
        artifact_path:
            "engine/benchmark-artifacts/render-hybrid-ray-sdf-raster-runtime-proof/fallback.txt"
                .to_string(),
        visible: true,
        reason: "ray-query unsupported path remains raster plus SDF raymarch plus temporal"
            .to_string(),
    }
}

fn timing_samples() -> Vec<PassTimingSample> {
    vec![
        PassTimingSample {
            flow_id: "hybrid.proof".to_string(),
            pass_id: "raster".to_string(),
            pass_kind: "render".to_string(),
            millis: 0.34,
            dispatch_workgroups: None,
        },
        PassTimingSample {
            flow_id: "hybrid.proof".to_string(),
            pass_id: "sdf".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.21,
            dispatch_workgroups: Some([1, 1, 1]),
        },
        PassTimingSample {
            flow_id: "hybrid.proof".to_string(),
            pass_id: "temporal".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.28,
            dispatch_workgroups: None,
        },
        PassTimingSample {
            flow_id: "hybrid.proof".to_string(),
            pass_id: "ray_query".to_string(),
            pass_kind: "trace".to_string(),
            millis: 0.18,
            dispatch_workgroups: None,
        },
        PassTimingSample {
            flow_id: "hybrid.proof".to_string(),
            pass_id: "fallback".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.12,
            dispatch_workgroups: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_builds_ready_hybrid_runtime_proof() {
        let report = build_report();

        assert!(report.is_ready());
        assert_eq!(report.error_count(), 0);
        assert_eq!(report.raster_passes.len(), 1);
        assert!(report.sdf.is_runtime_ready());
        assert!(report.temporal.is_runtime_ready());
        assert!(report.ray_query_supported.ray_query_invocation_allowed);
        assert!(!report.ray_query_fallback.ray_query_invocation_allowed);
        assert!(
            report
                .ray_query_fallback
                .capability_profile
                .native_fallback_visible
        );
        assert!(report.fallback.visible);
        assert_eq!(report.timings.pass_sample_count, 5);
        assert!(report.has_required_timing_labels());
    }
}
