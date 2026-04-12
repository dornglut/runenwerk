use engine::plugins::render::RenderResourceDescriptor;
use engine::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, PassTimingSample, RenderCaptureIdentity,
    RenderCapturePointIdentity, RenderDebugTimingsState, RenderPassProvenanceRecord,
    RenderPassProvenanceState, deterministic_capture_filename, resource_kind_name,
    summarize_pass_timings,
};
use engine::plugins::render::pipelines::{FlowPassKind, FlowPrimitiveTopologyClass};
use wgpu::TextureFormat;

#[derive(Debug, Clone, Copy, engine::plugins::render::GpuStorage)]
struct InspectStorage {
    value: u32,
}

#[test]
fn runtime_timing_snapshot_preserves_flow_pass_kind_and_dispatch_metadata() {
    let samples = vec![
        PassTimingSample {
            flow_id: "flow.a".to_string(),
            pass_id: "a.compute".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.6,
            dispatch_workgroups: Some([20, 12, 1]),
        },
        PassTimingSample {
            flow_id: "flow.a".to_string(),
            pass_id: "a.compose".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 1.1,
            dispatch_workgroups: None,
        },
    ];
    let snapshot = summarize_pass_timings(&samples);

    assert_eq!(snapshot.per_pass.len(), 2);
    assert_eq!(snapshot.per_pass[0].flow_id, "flow.a");
    assert_eq!(snapshot.per_pass[0].pass_kind, "compute");
    assert_eq!(snapshot.per_pass[0].dispatch_workgroups, Some([20, 12, 1]));
    assert_eq!(snapshot.slowest_pass_id.as_deref(), Some("a.compose"));
}

#[test]
fn debug_timing_state_extracts_compute_dispatch_samples() {
    let mut state = RenderDebugTimingsState::default();
    state.observe_pass_timings(&[
        PassTimingSample {
            flow_id: "flow.a".to_string(),
            pass_id: "a.compute".to_string(),
            pass_kind: "compute".to_string(),
            millis: 0.6,
            dispatch_workgroups: Some([10, 4, 1]),
        },
        PassTimingSample {
            flow_id: "flow.a".to_string(),
            pass_id: "a.compose".to_string(),
            pass_kind: "fullscreen".to_string(),
            millis: 0.2,
            dispatch_workgroups: None,
        },
    ]);

    assert_eq!(state.pass_sample_count, 2);
    assert_eq!(state.compute_dispatches.len(), 1);
    assert_eq!(state.compute_dispatches[0].flow_id, "flow.a");
    assert_eq!(state.compute_dispatches[0].pass_id, "a.compute");
    assert_eq!(state.compute_dispatches[0].workgroups, [10, 4, 1]);
}

#[test]
fn resource_kind_label_matches_descriptor_kind() {
    let descriptor = RenderResourceDescriptor::storage_buffer::<InspectStorage>("inspect.cells");
    assert_eq!(resource_kind_name(&descriptor), "storage_buffer");
}

#[test]
fn pass_provenance_state_preserves_required_human_fields() {
    let mut state = RenderPassProvenanceState::default();
    state.observe_frame(
        12,
        &[RenderPassProvenanceRecord {
            frame_index: 12,
            flow_id: "runenwerk.editor.main".to_string(),
            pass_id: "runenwerk.editor.viewport.sdf".to_string(),
            pass_label: "runenwerk.editor.viewport.sdf".to_string(),
            pass_kind: FlowPassKind::Fullscreen,
            order_index: 3,
            feature_id: Some("feature.ui".to_string()),
            shader_id: "editor_viewport_sdf".to_string(),
            shader_revision: 7,
            fallback_used: false,
            pipeline_stats_key: "flow:runenwerk.editor.main".to_string(),
            bind_group_layout_signature_hash: 10,
            material_specialization_fragment_hash: 11,
            view_signature_hash: 12,
            feature_runtime_version: 13,
            color_formats: vec![TextureFormat::Rgba8Unorm],
            depth_format: None,
            sample_count: 1,
            primitive_topology_class: FlowPrimitiveTopologyClass::TriangleList,
            render_targets: vec!["surface.color".to_string()],
            sampled_textures: vec!["surface.color".to_string()],
            storage_textures: Vec::new(),
            depth_targets: Vec::new(),
            capture_points_available: vec![RenderCapturePointIdentity {
                flow_id: "runenwerk.editor.main".to_string(),
                pass_id: "runenwerk.editor.viewport.sdf".to_string(),
                stage: CaptureStage::After,
                resource_id: "surface.color".to_string(),
                texture_class: CaptureTextureClass::ImportedTexture,
            }],
        }],
    );

    assert_eq!(state.frame_index, 12);
    assert_eq!(state.records.len(), 1);
    let record = &state.records[0];
    assert_eq!(record.flow_id, "runenwerk.editor.main");
    assert_eq!(record.pass_id, "runenwerk.editor.viewport.sdf");
    assert_eq!(record.pass_label, "runenwerk.editor.viewport.sdf");
    assert_eq!(record.shader_id, "editor_viewport_sdf");
    assert_eq!(record.shader_revision, 7);
    assert!(!record.fallback_used);
}

#[test]
fn deterministic_capture_filename_uses_required_identity_tuple() {
    let identity = RenderCaptureIdentity {
        frame_index: 9,
        pass_label: "runenwerk.editor.main.ui".to_string(),
        capture_point: RenderCapturePointIdentity {
            flow_id: "runenwerk.editor.main".to_string(),
            pass_id: "runenwerk.editor.main.ui".to_string(),
            stage: CaptureStage::Before,
            resource_id: "surface.color".to_string(),
            texture_class: CaptureTextureClass::ImportedTexture,
        },
    };

    let name = deterministic_capture_filename(&identity, "png");
    assert_eq!(
        name,
        "frame_9__flow_runenwerk_editor_main__pass_runenwerk_editor_main_ui__stage_before__resource_surface_color.png"
    );
}
