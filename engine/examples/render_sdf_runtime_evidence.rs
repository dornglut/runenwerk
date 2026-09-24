use engine::plugins::render::features::world::sdf_raymarch::{
    RenderSdfDistanceMipLevel, RenderSdfRaymarchAccelerationReport, RenderSdfRaymarchCandidate,
    RenderSdfRaymarchCandidateList,
};
use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuTimingCapability,
    RenderGpuTimingDiagnostic, RenderSdfProductionEvidenceReport,
    RenderSdfProductionEvidenceRequest, RenderSdfProductionHardwareProfile,
    RenderSdfResidencyBudgetInspection, RenderSdfResidencyInspection,
    RenderSdfRuntimeVisualEvidence, inspect_render_sdf_production_evidence,
};

fn main() {
    let report = build_report();
    println!(
        "render sdf runtime evidence ready={} errors={} warnings={} profile={}",
        report.is_runtime_ready(),
        report.error_count(),
        report.warning_count(),
        report.hardware_profile.profile_key
    );
    println!(
        "resident={} pages={} bricks={} clipmaps={} distance_mips={} candidate_lists={} visual={} cpu_ms={:.3} gpu_diagnostics={}",
        report.counts.resident_product_count,
        report.counts.resident_page_count,
        report.counts.resident_brick_count,
        report.counts.clipmap_window_count,
        report.counts.distance_mip_count,
        report.counts.candidate_list_count,
        report.counts.visual_evidence_count,
        report.timings.cpu_total_pass_millis,
        report.timings.gpu_timing_diagnostic_count
    );
}

fn build_report() -> RenderSdfProductionEvidenceReport {
    inspect_render_sdf_production_evidence(RenderSdfProductionEvidenceRequest {
        hardware_profile: RenderSdfProductionHardwareProfile {
            profile_key: "standalone-sdf-runtime".to_string(),
            adapter_name: Some("standalone portable profile".to_string()),
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
    })
}

fn residency() -> RenderSdfResidencyInspection {
    RenderSdfResidencyInspection {
        addressable_product_count: 4,
        selected_product_count: 4,
        requested_product_count: 4,
        resident_product_count: 4,
        resident_page_count: 10,
        resident_brick_count: 10,
        clipmap_window_count: 4,
        invalidated_product_count: 0,
        rejected_product_count: 0,
        resident_bytes: 40_960,
        upload_bytes: 8192,
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
    let candidates = (1..=4)
        .map(|product_id| RenderSdfRaymarchCandidate {
            product_id,
            cache_generation: product_id,
            scale_band: match product_id {
                1 => "Near",
                2 => "Mid",
                3 => "Far",
                _ => "Summary",
            }
            .to_string(),
            page_count: if product_id == 4 { 4 } else { 2 },
            brick_count: if product_id == 4 { 4 } else { 2 },
            resident_bytes: if product_id == 4 { 16_384 } else { 8192 },
        })
        .collect::<Vec<_>>();
    RenderSdfRaymarchAccelerationReport {
        resident_product_count: 4,
        resident_page_count: 10,
        resident_brick_count: 10,
        clipmap_window_count: 4,
        distance_mips: (0_u8..4)
            .map(|level| RenderSdfDistanceMipLevel {
                level,
                source_page_count: if level == 3 { 4 } else { 2 },
                source_brick_count: if level == 3 { 4 } else { 2 },
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
        total_candidate_count: 4,
        rejected_candidate_count: 0,
        max_candidates_per_list: 8,
        max_steps_per_ray: 160,
        fullscreen_entity_multiplier: 1,
        diagnostics: Vec::new(),
    }
}

fn timings() -> RenderDebugTimingsState {
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[
        PassTimingSample {
            flow_id: "sdf.runtime.evidence".to_string(),
            pass_id: "sdf.prepare".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.16,
            dispatch_workgroups: Some([1, 1, 1]),
        },
        PassTimingSample {
            flow_id: "sdf.runtime.evidence".to_string(),
            pass_id: "sdf.compose".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.39,
            dispatch_workgroups: None,
        },
    ]);
    timings.observe_gpu_timing_diagnostic(RenderGpuTimingDiagnostic::unsupported(
        "standalone SDF evidence command records unsupported timestamp queries explicitly",
    ));
    timings
}

fn visual_evidence() -> Vec<RenderSdfRuntimeVisualEvidence> {
    ["near", "mid", "far", "summary"]
        .into_iter()
        .enumerate()
        .map(|(index, coverage_band)| RenderSdfRuntimeVisualEvidence {
            view_label: format!("standalone.{coverage_band}"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_builds_ready_sdf_runtime_evidence_report() {
        let report = build_report();

        assert!(report.is_runtime_ready());
        assert_eq!(report.counts.resident_product_count, 4);
        assert_eq!(report.counts.visual_evidence_count, 4);
        assert_eq!(report.timings.gpu_timing_diagnostic_count, 1);
    }
}
