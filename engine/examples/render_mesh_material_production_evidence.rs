use engine::plugins::render::inspect::{
    PassTimingSample, RenderDebugTimingsState, RenderGpuTimingCapability,
    RenderGpuTimingDiagnostic, RenderMeshMaterialHandoffCounts,
    RenderMeshMaterialHandoffInspection, RenderMeshMaterialProductionEvidenceReport,
    RenderMeshMaterialProductionEvidenceRequest, RenderMeshMaterialProductionHardwareProfile,
    RenderMeshMaterialRuntimeVisualEvidence, RenderPipelineFallbackCounts,
    RenderPipelineFallbackInspection, inspect_render_mesh_material_production_evidence,
};

fn main() {
    let report = build_report();
    println!(
        "render mesh/material production evidence ready={} errors={} warnings={} profile={}",
        report.is_runtime_ready(),
        report.error_count(),
        report.warning_count(),
        report.hardware_profile.profile_key
    );
    println!(
        "materials={} slots={} material_passes={} pipeline_passes={} visual={} pixels={} cpu_ms={:.3} gpu_diagnostics={}",
        report.counts.material_instance_count,
        report.counts.material_binding_slot_count,
        report.counts.material_consuming_pass_count,
        report.counts.pipeline_backed_pass_count,
        report.counts.visual_evidence_count,
        report.counts.rendered_pixel_count,
        report.timings.cpu_total_pass_millis,
        report.timings.gpu_timing_diagnostic_count
    );
}

fn build_report() -> RenderMeshMaterialProductionEvidenceReport {
    inspect_render_mesh_material_production_evidence(RenderMeshMaterialProductionEvidenceRequest {
        hardware_profile: RenderMeshMaterialProductionHardwareProfile {
            profile_key: "standalone-mesh-material-runtime".to_string(),
            adapter_name: Some("standalone portable profile".to_string()),
            backend: Some("wgpu".to_string()),
            timestamp_query: RenderGpuTimingCapability::Unsupported,
        },
        material_handoff: material_handoff(),
        pipeline_fallback: pipeline_fallback(),
        timings: timings(),
        visual_evidence: vec![RenderMeshMaterialRuntimeVisualEvidence {
            view_label: "standalone.mesh.material.summary".to_string(),
            artifact_path:
                "engine/benchmark-artifacts/render-mesh-material-production-evidence/summary.txt"
                    .to_string(),
            material_table_identity: "scene.material.table:v1".to_string(),
            scene_shader_identity: "shader.identity.scene".to_string(),
            material_instance_count: 1,
            rendered_pixel_count: 4096,
            consumed_material_handoff: true,
            consumed_pipeline_fallback: true,
        }],
        benchmark_commands: vec!["cargo bench -p engine --bench render_flow_planning".to_string()],
        artifact_paths: vec![
            "engine/benchmark-artifacts/render-mesh-material-production-evidence/README.md"
                .to_string(),
            "docs-site/src/content/docs/reports/benchmarks/render/mesh-material-production-evidence.md"
                .to_string(),
        ],
    })
}

fn material_handoff() -> RenderMeshMaterialHandoffInspection {
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

fn pipeline_fallback() -> RenderPipelineFallbackInspection {
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
            flow_id: "mesh.material.production".to_string(),
            pass_id: "mesh.material.prepare".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.12,
            dispatch_workgroups: Some([1, 1, 1]),
        },
        PassTimingSample {
            flow_id: "mesh.material.production".to_string(),
            pass_id: "mesh.material.draw".to_string(),
            pass_kind: "graphics".to_string(),
            millis: 0.31,
            dispatch_workgroups: None,
        },
    ]);
    timings.observe_gpu_timing_diagnostic(RenderGpuTimingDiagnostic::unsupported(
        "standalone mesh/material evidence command records unsupported timestamp queries explicitly",
    ));
    timings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_builds_ready_mesh_material_production_report() {
        let report = build_report();

        assert!(report.is_runtime_ready());
        assert_eq!(report.counts.material_instance_count, 1);
        assert_eq!(report.counts.pipeline_backed_pass_count, 1);
        assert_eq!(report.counts.rendered_pixel_count, 4096);
        assert_eq!(report.timings.gpu_timing_diagnostic_count, 1);
    }
}
