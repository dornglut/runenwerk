use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuResidencyBudgetInspection,
    RenderGpuResidencyInspection, RenderGpuTimingCapability, RenderPassTimingEvidence,
    RenderScaleProductionEvidenceRequest, RenderScaleProductionHardwareProfile,
    RenderScaleVisibilityCapabilityStatus, RenderScaleVisibilityInspection,
    inspect_render_scale_production_evidence,
};

fn residency(resident_count: usize) -> RenderGpuResidencyInspection {
    RenderGpuResidencyInspection {
        addressable_count: 1_000_000,
        selected_count: 128_000,
        requested_count: 128_000,
        accepted_count: 128_000,
        resident_count,
        allocated_count: resident_count,
        preserved_count: 0,
        invalidated_count: 0,
        evicted_count: 0,
        rejected_count: 0,
        resident_bytes: resident_count as u64 * 4096,
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

fn visibility(
    resident_count: usize,
    visible_count: usize,
    compacted_count: usize,
    submitted_draw_count: usize,
) -> RenderScaleVisibilityInspection {
    RenderScaleVisibilityInspection {
        resident_count,
        visible_count,
        culled_count: resident_count.saturating_sub(visible_count),
        compacted_count,
        submitted_draw_count,
        indirect_command_count: usize::from(submitted_draw_count > 0),
        storage_compaction_status: "supported".to_string(),
        indirect_submission_status: "supported".to_string(),
        diagnostics: Vec::new(),
        records: Vec::new(),
    }
}

fn timings(gpu_capability: RenderGpuTimingCapability) -> RenderDebugTimingsState {
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[PassTimingSample {
        flow_id: "test.scale".to_string(),
        pass_id: "scale.visibility".to_string(),
        pass_kind: "compute".to_string(),
        millis: 0.25,
        dispatch_workgroups: Some([4, 1, 1]),
    }]);
    match gpu_capability {
        RenderGpuTimingCapability::Supported => {
            timings.observe_gpu_pass_timing_evidence(&[RenderPassTimingEvidence::gpu_sample(
                Some(1),
                Some(1),
                "test.scale",
                "scale.visibility",
                "compute",
                0.2,
            )]);
        }
        RenderGpuTimingCapability::Unsupported => {
            timings.observe_gpu_pass_timing_evidence(&[RenderPassTimingEvidence::gpu_diagnostic(
                Some(1),
                Some(1),
                "test.scale",
                "scale.visibility",
                "compute",
                engine::plugins::render::inspect::RenderGpuTimingDiagnostic::unsupported(
                    "timestamp queries unsupported by adapter",
                ),
            )]);
        }
        RenderGpuTimingCapability::UnavailableThisFrame
        | RenderGpuTimingCapability::ReadbackPending => {}
    }
    timings
}

fn hardware_profile(
    gpu_capability: RenderGpuTimingCapability,
) -> RenderScaleProductionHardwareProfile {
    RenderScaleProductionHardwareProfile {
        profile_key: "test-scale-profile".to_string(),
        adapter_name: Some("test adapter".to_string()),
        backend: Some("wgpu-test".to_string()),
        timestamp_query: gpu_capability,
        storage_compaction: RenderScaleVisibilityCapabilityStatus::Supported,
        indirect_submission: RenderScaleVisibilityCapabilityStatus::Supported,
        readback: RenderScaleVisibilityCapabilityStatus::Supported,
    }
}

fn request(
    residency: RenderGpuResidencyInspection,
    visibility: RenderScaleVisibilityInspection,
    gpu_capability: RenderGpuTimingCapability,
) -> RenderScaleProductionEvidenceRequest {
    RenderScaleProductionEvidenceRequest {
        hardware_profile: hardware_profile(gpu_capability),
        residency,
        visibility,
        timings: timings(gpu_capability),
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-scale-evidence/test-profile.md".to_string(),
        ],
    }
}

#[test]
fn render_scale_production_evidence_reports_ready_runtime_chain() {
    let report = inspect_render_scale_production_evidence(request(
        residency(4),
        visibility(4, 3, 3, 3),
        RenderGpuTimingCapability::Supported,
    ));

    assert!(report.is_runtime_ready());
    assert_eq!(report.counts.addressable_count, 1_000_000);
    assert_eq!(report.counts.resident_count, 4);
    assert_eq!(report.counts.visible_count, 3);
    assert_eq!(report.counts.submitted_draw_count, 3);
    assert_eq!(
        report.hardware_profile.timestamp_query_status,
        RenderGpuTimingCapability::Supported.as_str()
    );
    assert_eq!(report.timings.timing_source, "gpu_timestamp_query");
}

#[test]
fn render_scale_production_evidence_fails_closed_on_count_invariant_drift() {
    let report = inspect_render_scale_production_evidence(request(
        residency(2),
        visibility(2, 3, 2, 3),
        RenderGpuTimingCapability::Supported,
    ));

    assert!(!report.is_runtime_ready());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "visible_exceeds_resident")
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "submitted_exceeds_compacted")
    );
}

#[test]
fn render_scale_production_evidence_keeps_unsupported_gpu_timing_explicit() {
    let report = inspect_render_scale_production_evidence(request(
        residency(2),
        visibility(2, 1, 1, 1),
        RenderGpuTimingCapability::Unsupported,
    ));

    assert!(report.is_runtime_ready());
    assert_eq!(report.warning_count(), 1);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "timestamp_queries_unsupported")
    );
    assert_eq!(report.timings.timing_source, "cpu_encode_submit");
}
