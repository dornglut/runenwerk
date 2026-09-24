use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuTimingCapability,
    RenderGpuTimingDiagnostic, RenderMeshMaterialHandoffCounts,
    RenderMeshMaterialHandoffDiagnostic, RenderMeshMaterialHandoffDiagnosticSeverity,
    RenderMeshMaterialHandoffInspection, RenderMeshMaterialProductionEvidenceRequest,
    RenderMeshMaterialProductionHardwareProfile, RenderMeshMaterialRuntimeVisualEvidence,
    RenderPipelineFallbackCounts, RenderPipelineFallbackDiagnostic,
    RenderPipelineFallbackDiagnosticSeverity, RenderPipelineFallbackInspection,
    inspect_render_mesh_material_production_evidence,
};

#[test]
fn render_mesh_material_production_evidence_reports_ready_runtime_chain() {
    let report = inspect_render_mesh_material_production_evidence(request());

    assert!(report.is_runtime_ready(), "{:?}", report.diagnostics);
    assert_eq!(report.counts.material_instance_count, 1);
    assert_eq!(report.counts.material_consuming_pass_count, 1);
    assert_eq!(report.counts.pipeline_backed_pass_count, 1);
    assert_eq!(report.counts.visual_evidence_count, 1);
    assert_eq!(report.counts.rendered_pixel_count, 4096);
    assert_eq!(report.timings.cpu_pass_sample_count, 2);
    assert_eq!(report.timings.gpu_timing_diagnostic_count, 1);
}

#[test]
fn render_mesh_material_production_evidence_fails_closed_without_visual_or_benchmark_evidence() {
    let mut request = request();
    request.visual_evidence.clear();
    request.benchmark_commands.clear();

    let report = inspect_render_mesh_material_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(has_error(&report.diagnostics, "missing_visual_evidence"));
    assert!(has_error(&report.diagnostics, "missing_benchmark_command"));
    assert!(has_error(
        &report.diagnostics,
        "missing_rendered_pixel_count"
    ));
}

#[test]
fn render_mesh_material_production_evidence_fails_closed_on_unconsumed_visual_proof() {
    let mut request = request();
    request.visual_evidence[0].consumed_material_handoff = false;
    request.visual_evidence[0].consumed_pipeline_fallback = false;

    let report = inspect_render_mesh_material_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(has_error(
        &report.diagnostics,
        "visual_without_material_handoff"
    ));
    assert!(has_error(
        &report.diagnostics,
        "visual_without_pipeline_fallback"
    ));
}

#[test]
fn render_mesh_material_production_evidence_fails_closed_on_source_inspection_errors() {
    let mut request = request();
    request
        .material_handoff
        .diagnostics
        .push(RenderMeshMaterialHandoffDiagnostic {
            severity: RenderMeshMaterialHandoffDiagnosticSeverity::Error,
            code: "missing_scene_material_bundle",
            message: "missing scene bundle".to_string(),
        });
    request
        .pipeline_fallback
        .diagnostics
        .push(RenderPipelineFallbackDiagnostic {
            severity: RenderPipelineFallbackDiagnosticSeverity::Error,
            code: "material_shader_fallback_forbidden",
            message: "fallback shader used".to_string(),
        });

    let report = inspect_render_mesh_material_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(has_error(
        &report.diagnostics,
        "material_handoff_missing_scene_material_bundle"
    ));
    assert!(has_error(
        &report.diagnostics,
        "pipeline_fallback_material_shader_fallback_forbidden"
    ));
}

#[test]
fn render_mesh_material_production_evidence_fails_closed_on_material_fallback_pass() {
    let mut request = request();
    request
        .pipeline_fallback
        .counts
        .material_fallback_pass_count = 1;

    let report = inspect_render_mesh_material_production_evidence(request);

    assert!(!report.is_runtime_ready());
    assert!(has_error(&report.diagnostics, "material_fallback_present"));
}

fn request() -> RenderMeshMaterialProductionEvidenceRequest {
    RenderMeshMaterialProductionEvidenceRequest {
        hardware_profile: RenderMeshMaterialProductionHardwareProfile {
            profile_key: "portable-mesh-material-runtime".to_string(),
            adapter_name: Some("portable test profile".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        material_handoff: ready_handoff(),
        pipeline_fallback: ready_pipeline_fallback(),
        timings: timings(),
        visual_evidence: vec![visual_evidence()],
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-mesh-material-production-evidence/README.md"
                .to_string(),
            "docs-site/src/content/docs/reports/benchmarks/render/mesh-material-production-evidence.md"
                .to_string(),
        ],
    }
}

fn ready_handoff() -> RenderMeshMaterialHandoffInspection {
    RenderMeshMaterialHandoffInspection {
        counts: RenderMeshMaterialHandoffCounts {
            material_instance_count: 1,
            texture_binding_count: 1,
            material_binding_slot_count: 1,
            model_mesh_selection_count: 1,
            material_consuming_pass_count: 1,
            pass_exposed_model_mesh_selection_count: 1,
        },
        scene_shader_identity: Some("shader.identity.scene".to_string()),
        scene_shader_path: Some("generated/scene_material.wgsl".to_string()),
        shader_artifact_id: Some("shader.artifact.scene".to_string()),
        shader_cache_key: Some("shader.cache.scene".to_string()),
        material_table_identity: Some("scene.material.table:v1".to_string()),
        resource_layout_identity: Some("resource.layout:v1".to_string()),
        diagnostics: Vec::new(),
    }
}

fn ready_pipeline_fallback() -> RenderPipelineFallbackInspection {
    RenderPipelineFallbackInspection {
        counts: RenderPipelineFallbackCounts {
            pass_count: 1,
            pipeline_backed_pass_count: 1,
            material_pass_count: 1,
            fallback_pass_count: 0,
            material_fallback_pass_count: 0,
            shader_failure_event_count: 1,
            prior_valid_shader_failure_count: 1,
            pipeline_cache_hit_count: 2,
            pipeline_cache_miss_count: 1,
            pipeline_cache_failure_count: 0,
        },
        shader_reload_status: None,
        passes: Vec::new(),
        shader_failures: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn timings() -> RenderDebugTimingsState {
    let mut timings = RenderDebugTimingsState::default();
    timings.observe_pass_timings(&[
        PassTimingSample {
            flow_id: "mesh.material.runtime".to_string(),
            pass_id: "mesh.material.prepare".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.12,
            dispatch_workgroups: Some([1, 1, 1]),
        },
        PassTimingSample {
            flow_id: "mesh.material.runtime".to_string(),
            pass_id: "mesh.material.draw".to_string(),
            pass_kind: "graphics".to_string(),
            millis: 0.31,
            dispatch_workgroups: None,
        },
    ]);
    timings.observe_gpu_timing_diagnostic(RenderGpuTimingDiagnostic::unsupported(
        "portable test profile does not require timestamp queries",
    ));
    timings
}

fn visual_evidence() -> RenderMeshMaterialRuntimeVisualEvidence {
    RenderMeshMaterialRuntimeVisualEvidence {
        view_label: "mesh.material.summary".to_string(),
        artifact_path:
            "engine/benchmark-artifacts/render-mesh-material-production-evidence/summary.txt"
                .to_string(),
        material_table_identity: "scene.material.table:v1".to_string(),
        scene_shader_identity: "shader.identity.scene".to_string(),
        material_instance_count: 1,
        rendered_pixel_count: 4096,
        consumed_material_handoff: true,
        consumed_pipeline_fallback: true,
    }
}

fn has_error(
    diagnostics: &[engine::plugins::render::inspect::RenderMeshMaterialProductionEvidenceDiagnostic],
    code: &str,
) -> bool {
    diagnostics.iter().any(|diagnostic| {
        diagnostic.severity
            == engine::plugins::render::inspect::RenderMeshMaterialProductionEvidenceSeverity::Error
            && diagnostic.code == code
    })
}
