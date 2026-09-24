use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuResidencyBudgetInspection,
    RenderGpuResidencyInspection, RenderGpuTimingCapability, RenderPassTimingEvidence,
    RenderScaleProductionEvidenceReport, RenderScaleProductionEvidenceRequest,
    RenderScaleProductionHardwareProfile, RenderScaleVisibilityCandidate,
    RenderScaleVisibilityCapabilities, RenderScaleVisibilityCapabilityStatus,
    inspect_render_scale_production_evidence, inspect_render_scale_visibility,
};

fn build_residency() -> RenderGpuResidencyInspection {
    RenderGpuResidencyInspection {
        addressable_count: 1_000_000,
        selected_count: 128_000,
        requested_count: 128_000,
        accepted_count: 128_000,
        resident_count: 4,
        allocated_count: 4,
        preserved_count: 0,
        invalidated_count: 0,
        evicted_count: 0,
        rejected_count: 0,
        resident_bytes: 16_384,
        upload_bytes: 4096,
        budget: RenderGpuResidencyBudgetInspection {
            max_resident_entries: 128_000,
            max_resident_bytes: 512 * 1024 * 1024,
            max_upload_bytes_per_frame: 16 * 1024 * 1024,
            resident_entry_status: "within_budget".to_string(),
            resident_byte_status: "within_budget".to_string(),
            upload_byte_status: "within_budget".to_string(),
            hard_pinned_over_entry_budget: false,
        },
        diagnostic_count: 0,
        entries: Vec::new(),
        journal: Vec::new(),
    }
}

fn build_report() -> RenderScaleProductionEvidenceReport {
    let visibility = inspect_render_scale_visibility(
        &[
            RenderScaleVisibilityCandidate {
                product_id: 1,
                cache_id: "scale.chunk.1".to_string(),
                center: [0.0, 0.0, 0.0],
                radius: 0.2,
                screen_size_px: 128.0,
                resident_bytes: 4096,
            },
            RenderScaleVisibilityCandidate {
                product_id: 2,
                cache_id: "scale.chunk.2".to_string(),
                center: [0.35, 0.0, 0.0],
                radius: 0.15,
                screen_size_px: 48.0,
                resident_bytes: 4096,
            },
            RenderScaleVisibilityCandidate {
                product_id: 3,
                cache_id: "scale.chunk.3".to_string(),
                center: [4.0, 0.0, 0.0],
                radius: 0.2,
                screen_size_px: 64.0,
                resident_bytes: 4096,
            },
            RenderScaleVisibilityCandidate {
                product_id: 4,
                cache_id: "scale.chunk.4".to_string(),
                center: [0.0, 0.0, 0.0],
                radius: 0.1,
                screen_size_px: 0.25,
                resident_bytes: 4096,
            },
        ],
        Default::default(),
        RenderScaleVisibilityCapabilities::supported(),
    );

    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[
        PassTimingSample {
            flow_id: "example.scale".to_string(),
            pass_id: "scale.visibility".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.18,
            dispatch_workgroups: Some([8, 1, 1]),
        },
        PassTimingSample {
            flow_id: "example.scale".to_string(),
            pass_id: "scale.indirect_draw".to_string(),
            pass_kind: "graphics".to_string(),
            millis: 0.27,
            dispatch_workgroups: None,
        },
    ]);
    timings.observe_gpu_pass_timing_evidence(&[RenderPassTimingEvidence::gpu_sample(
        Some(1),
        Some(1),
        "example.scale",
        "scale.indirect_draw",
        "graphics",
        0.24,
    )]);

    inspect_render_scale_production_evidence(RenderScaleProductionEvidenceRequest {
        hardware_profile: RenderScaleProductionHardwareProfile {
            profile_key: "example-wgpu-portable-scale".to_string(),
            adapter_name: Some("example adapter".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Supported,
            storage_compaction: RenderScaleVisibilityCapabilityStatus::Supported,
            indirect_submission: RenderScaleVisibilityCapabilityStatus::Supported,
            readback: RenderScaleVisibilityCapabilityStatus::Supported,
        },
        residency: build_residency(),
        visibility,
        timings,
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-scale-evidence/example-profile.md".to_string(),
            "docs-site/src/content/docs/engine/benchmarks/render-scale-evidence.md".to_string(),
        ],
    })
}

fn main() {
    let report = build_report();
    println!(
        "render scale evidence ready={} errors={} warnings={} profile={}",
        report.is_runtime_ready(),
        report.error_count(),
        report.warning_count(),
        report.hardware_profile.profile_key
    );
    println!(
        "addressable={} resident={} visible={} compacted={} submitted={} indirect_commands={} cpu_ms={:.3} gpu_ms={:.3}",
        report.counts.addressable_count,
        report.counts.resident_count,
        report.counts.visible_count,
        report.counts.compacted_count,
        report.counts.submitted_draw_count,
        report.counts.indirect_command_count,
        report.timings.cpu_total_pass_millis,
        report.timings.gpu_total_pass_millis
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_builds_ready_scale_evidence_report() {
        let report = build_report();

        assert!(report.is_runtime_ready());
        assert_eq!(report.counts.addressable_count, 1_000_000);
        assert_eq!(report.counts.resident_count, 4);
        assert_eq!(report.counts.visible_count, 2);
        assert_eq!(report.counts.submitted_draw_count, 2);
        assert_eq!(report.timings.gpu_pass_sample_count, 1);
    }
}
