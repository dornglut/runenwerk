use engine::plugins::render::features::world::sdf_raymarch::{
    RenderSdfDistanceMipLevel, RenderSdfRaymarchAccelerationReport, RenderSdfRaymarchCandidate,
    RenderSdfRaymarchCandidateList,
};
use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuTimingCapability,
    RenderSdfProductionEvidenceRequest, RenderSdfProductionHardwareProfile,
    RenderSdfResidencyBudgetInspection, RenderSdfResidencyInspection,
    RenderSdfRuntimeVisualEvidence, inspect_render_sdf_production_evidence,
};

fn residency() -> RenderSdfResidencyInspection {
    RenderSdfResidencyInspection {
        addressable_product_count: 3,
        selected_product_count: 3,
        requested_product_count: 3,
        resident_product_count: 3,
        resident_page_count: 6,
        resident_brick_count: 6,
        clipmap_window_count: 3,
        invalidated_product_count: 0,
        rejected_product_count: 0,
        resident_bytes: 24_576,
        upload_bytes: 6144,
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

fn raymarch() -> RenderSdfRaymarchAccelerationReport {
    RenderSdfRaymarchAccelerationReport {
        resident_product_count: 3,
        resident_page_count: 6,
        resident_brick_count: 6,
        clipmap_window_count: 3,
        distance_mips: vec![
            RenderSdfDistanceMipLevel {
                level: 0,
                source_page_count: 2,
                source_brick_count: 2,
                conservative_min_distance: 0.0,
                max_safe_step: 1.0,
                unsafe_overstep_risk: false,
            },
            RenderSdfDistanceMipLevel {
                level: 1,
                source_page_count: 2,
                source_brick_count: 2,
                conservative_min_distance: 0.0,
                max_safe_step: 0.5,
                unsafe_overstep_risk: false,
            },
            RenderSdfDistanceMipLevel {
                level: 2,
                source_page_count: 2,
                source_brick_count: 2,
                conservative_min_distance: 0.0,
                max_safe_step: 0.33,
                unsafe_overstep_risk: false,
            },
        ],
        candidate_lists: vec![RenderSdfRaymarchCandidateList {
            tile_index: 0,
            depth_slice: 0,
            candidate_count: 3,
            rejected_candidate_count: 0,
            candidates: vec![
                RenderSdfRaymarchCandidate {
                    product_id: 1,
                    cache_generation: 1,
                    scale_band: "Near".to_string(),
                    page_count: 2,
                    brick_count: 2,
                    resident_bytes: 8192,
                },
                RenderSdfRaymarchCandidate {
                    product_id: 2,
                    cache_generation: 2,
                    scale_band: "Mid".to_string(),
                    page_count: 2,
                    brick_count: 2,
                    resident_bytes: 8192,
                },
                RenderSdfRaymarchCandidate {
                    product_id: 3,
                    cache_generation: 3,
                    scale_band: "Far".to_string(),
                    page_count: 2,
                    brick_count: 2,
                    resident_bytes: 8192,
                },
            ],
        }],
        total_candidate_count: 3,
        rejected_candidate_count: 0,
        max_candidates_per_list: 8,
        max_steps_per_ray: 128,
        fullscreen_entity_multiplier: 1,
        diagnostics: Vec::new(),
    }
}

fn timings() -> RenderDebugTimingsState {
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[
        PassTimingSample {
            flow_id: "sdf.runtime".to_string(),
            pass_id: "sdf.prepare".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.18,
            dispatch_workgroups: Some([1, 1, 1]),
        },
        PassTimingSample {
            flow_id: "sdf.runtime".to_string(),
            pass_id: "sdf.compose".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.41,
            dispatch_workgroups: None,
        },
    ]);
    timings.observe_gpu_timing_diagnostic(
        engine::plugins::render::inspect::RenderGpuTimingDiagnostic::unsupported(
            "portable test profile does not require timestamp queries",
        ),
    );
    timings
}

fn visual_evidence() -> Vec<RenderSdfRuntimeVisualEvidence> {
    ["near", "mid", "far", "summary"]
        .into_iter()
        .enumerate()
        .map(|(index, coverage_band)| RenderSdfRuntimeVisualEvidence {
            view_label: format!("sdf.{coverage_band}"),
            coverage_band: coverage_band.to_string(),
            artifact_path: format!(
                "engine/benchmark-artifacts/render-sdf-runtime-evidence/{coverage_band}.txt"
            ),
            step_count: 24 + u32::try_from(index).expect("test index fits in u32"),
            missed_surface_risk: false,
            overstep_risk: false,
        })
        .collect()
}

fn request() -> RenderSdfProductionEvidenceRequest {
    RenderSdfProductionEvidenceRequest {
        hardware_profile: RenderSdfProductionHardwareProfile {
            profile_key: "portable-sdf-runtime".to_string(),
            adapter_name: Some("portable test profile".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        residency: residency(),
        raymarch: raymarch(),
        timings: timings(),
        visual_evidence: visual_evidence(),
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-sdf-runtime-evidence/README.md".to_string(),
            "docs-site/src/content/docs/engine/benchmarks/render-sdf-runtime-evidence.md"
                .to_string(),
        ],
    }
}

#[test]
fn render_sdf_runtime_evidence_reports_ready_runtime_chain() {
    let report = inspect_render_sdf_production_evidence(request());

    assert!(report.is_runtime_ready());
    assert_eq!(report.counts.resident_product_count, 3);
    assert_eq!(report.counts.distance_mip_count, 3);
    assert_eq!(report.counts.visual_evidence_count, 4);
    assert_eq!(report.timings.cpu_pass_sample_count, 2);
    assert_eq!(report.timings.gpu_timing_diagnostic_count, 1);
}

#[test]
fn render_sdf_runtime_evidence_fails_closed_without_visual_and_benchmark_evidence() {
    let mut request = request();
    request.visual_evidence.clear();
    request.benchmark_commands.clear();

    let report = inspect_render_sdf_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "missing_visual_evidence" })
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "missing_benchmark_command" })
    );
}

#[test]
fn render_sdf_runtime_evidence_fails_closed_on_raymarch_count_drift() {
    let mut request = request();
    request.raymarch.resident_page_count += 1;

    let report = inspect_render_sdf_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "raymarch_page_mismatch" })
    );
}
