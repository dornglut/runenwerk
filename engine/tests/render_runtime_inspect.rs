use engine::plugins::render::inspect::{
    CaptureStage, CaptureTextureClass, PassTimingSample, RenderCaptureIdentity,
    RenderCapturePointIdentity, RenderDebugTimingsState, RenderPassProvenanceRecord,
    RenderPassProvenanceState, deterministic_capture_filename, inspect_prepared_render_frame,
    inspect_resources, inspect_texture_resources, resource_kind_name, summarize_pass_timings,
};
use engine::plugins::render::pipelines::{FlowPassKind, FlowPrimitiveTopologyClass};
use engine::plugins::render::{
    PreparedFlowInputs, PreparedFlowInvocation, PreparedFlowInvocationId, PreparedFrameContext,
    PreparedFrameContributions, PreparedRenderFrame, PreparedShaderSnapshot, PreparedSurfaceInfo,
    PreparedTargetBinding, PreparedViewFrame, RenderDynamicTextureRetention,
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey, RenderFlow,
    RenderResourceDescriptor, RenderResourceId, RenderTextureSampleMode, RenderTextureTargetFormat,
    RenderTextureTargetUsage,
};
use product::{
    ProductAuthorityClass, ProductFreshness, ProductIdentity, ProductQueryPolicy, ProductResidency,
    ProductScaleBand, RenderProductSelection, RenderResidencyRequest, RenderSelectedProduct,
    RenderTargetDescriptor,
};
use std::collections::BTreeMap;
use ui_render_data::ViewportSurfaceBindingRegistry;
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
    let descriptor = RenderResourceDescriptor::storage_buffer::<InspectStorage>(
        RenderResourceId::try_from_raw(1).unwrap(),
    );
    assert_eq!(resource_kind_name(&descriptor), "storage_buffer");
}

#[test]
fn resource_inspection_exposes_target_alias_metadata() {
    let flow = RenderFlow::new("inspect.alias")
        .with_color_target_alias("viewport.scene_color")
        .with_depth_target_alias("viewport.depth");

    let resources = inspect_resources(&flow);
    let color_alias = resources
        .iter()
        .find(|entry| entry.target_alias_label.as_deref() == Some("viewport.scene_color"))
        .expect("color target alias should be inspectable");

    assert_eq!(color_alias.kind, "target_alias(color)");
    assert_eq!(color_alias.target_alias_kind.as_deref(), Some("color"));

    let textures = inspect_texture_resources(&flow);
    assert!(textures.iter().any(|entry| {
        entry.category == "target_alias(depth)"
            && entry.target_alias_label.as_deref() == Some("viewport.depth")
            && entry.target_alias_kind.as_deref() == Some("depth")
    }));
}

#[test]
fn prepared_frame_inspection_exposes_render_product_selection_targets_views_invocations_and_history()
 {
    let target_key = RenderDynamicTextureTargetKey::new("editor.viewport.1", "scene_color");
    let flow_id = engine::plugins::render::RenderFlowId::try_from_raw(3).unwrap();
    let resource_id = RenderResourceId::try_from_raw(9).unwrap();
    let mut target_alias_bindings = BTreeMap::new();
    target_alias_bindings.insert(
        "viewport.scene_color".to_string(),
        PreparedTargetBinding::DynamicTexture(target_key.clone()),
    );
    target_alias_bindings.insert(
        "viewport.fallback".to_string(),
        PreparedTargetBinding::FlowOwned(resource_id),
    );

    let frame = PreparedRenderFrame {
        context: PreparedFrameContext {
            frame_index: 42,
            flow_registry_revision: 1,
            shader_registry_revision: 2,
            prepare_epoch: 5,
        },
        surface: PreparedSurfaceInfo {
            target_size_px: (1920, 1080),
        },
        views: vec![
            PreparedViewFrame::main((1920, 1080)),
            PreparedViewFrame {
                view_id: "viewport.1".to_string(),
                kind: engine::plugins::render::PreparedViewKind::OffscreenProduct,
                target_size_px: (640, 360),
                history_signature: Some("camera:v1".to_string()),
            },
        ],
        flows: BTreeMap::new(),
        flow_invocations: vec![PreparedFlowInvocation {
            invocation_id: PreparedFlowInvocationId::new("viewport.1.scene"),
            flow_id,
            view_id: "viewport.1".to_string(),
            inputs: PreparedFlowInputs::default(),
            target_alias_bindings,
            history_signature: Some("camera:v1".to_string()),
        }],
        dynamic_texture_targets: vec![RenderDynamicTextureTargetDescriptor::new(
            target_key.clone(),
            640,
            360,
            RenderTextureTargetFormat::Rgba8Unorm,
            RenderTextureTargetUsage::color_sampled(),
            RenderTextureSampleMode::FilterableFloat,
            RenderDynamicTextureRetention::RetainForFrames(2),
        )],
        product_selections: vec![
            RenderProductSelection::new("viewport.1")
                .with_selected_product(RenderSelectedProduct {
                    product_id: ProductIdentity::new(77),
                    scale_band: ProductScaleBand::Preview,
                    generation: 12,
                    freshness: ProductFreshness::Current,
                    residency: ProductResidency::Resident,
                    authority_class: ProductAuthorityClass::DeterministicDerived,
                    query_policy: ProductQueryPolicy::StrictCurrentOnly,
                })
                .with_required_target(RenderTargetDescriptor::new(
                    "editor.viewport.1:scene_color",
                    640,
                    360,
                    "rgba8_unorm",
                ))
                .with_residency_request(RenderResidencyRequest::new(
                    ProductIdentity::new(77),
                    ProductResidency::Resident,
                    100,
                    true,
                )),
        ],
        viewport_surface_bindings: ViewportSurfaceBindingRegistry::default(),
        contributions: PreparedFrameContributions::default(),
        shader: PreparedShaderSnapshot {
            registry_revision: 2,
        },
    };

    let inspection = inspect_prepared_render_frame(&frame);
    let history_signatures = frame
        .dynamic_target_history_signatures()
        .expect("prepared frame history signatures should be coherent");

    assert_eq!(inspection.frame_index, 42);
    assert_eq!(
        history_signatures.get(&target_key).map(String::as_str),
        Some("camera:v1")
    );
    assert_eq!(inspection.views.len(), 2);
    assert_eq!(inspection.views[1].kind, "offscreen_product");
    assert_eq!(
        inspection.views[1].history_signature.as_deref(),
        Some("camera:v1")
    );
    assert_eq!(
        inspection.dynamic_texture_targets[0].key,
        "editor.viewport.1:scene_color"
    );
    assert_eq!(
        inspection.dynamic_texture_targets[0].retention,
        "retain_for_frames(2)"
    );
    assert!(inspection.dynamic_texture_targets[0].displayable);
    assert!(inspection.dynamic_texture_targets[0].sampleable);
    assert_eq!(
        inspection.flow_invocations[0].invocation_id,
        "viewport.1.scene"
    );
    assert_eq!(
        inspection.flow_invocations[0].history_signature.as_deref(),
        Some("camera:v1")
    );
    assert!(
        inspection.flow_invocations[0]
            .target_alias_bindings
            .iter()
            .any(|binding| {
                binding.alias == "viewport.scene_color"
                    && binding.binding == "dynamic_texture(editor.viewport.1:scene_color)"
            })
    );
    assert_eq!(inspection.product_selections.len(), 1);
    assert_eq!(inspection.product_selections[0].view_id, "viewport.1");
    assert_eq!(
        inspection.product_selections[0].selected_products[0].product_id,
        77
    );
    assert_eq!(
        inspection.product_selections[0].residency_requests[0].priority,
        100
    );
}

#[test]
fn prepared_frame_rejects_conflicting_dynamic_target_history_signatures() {
    let target_key = RenderDynamicTextureTargetKey::new("editor.viewport.1", "scene_color");
    let flow_id = engine::plugins::render::RenderFlowId::try_from_raw(3).unwrap();
    let binding = PreparedTargetBinding::DynamicTexture(target_key);
    let mut first_bindings = BTreeMap::new();
    first_bindings.insert("viewport.scene_color".to_string(), binding.clone());
    let mut second_bindings = BTreeMap::new();
    second_bindings.insert("viewport.scene_color".to_string(), binding);

    let frame = PreparedRenderFrame {
        context: PreparedFrameContext {
            frame_index: 42,
            flow_registry_revision: 1,
            shader_registry_revision: 2,
            prepare_epoch: 5,
        },
        surface: PreparedSurfaceInfo {
            target_size_px: (1920, 1080),
        },
        views: vec![PreparedViewFrame::offscreen_product(
            "viewport.1",
            (640, 360),
        )],
        flows: BTreeMap::new(),
        flow_invocations: vec![
            PreparedFlowInvocation {
                invocation_id: PreparedFlowInvocationId::new("viewport.1.scene.a"),
                flow_id,
                view_id: "viewport.1".to_string(),
                inputs: PreparedFlowInputs::default(),
                target_alias_bindings: first_bindings,
                history_signature: Some("camera:a".to_string()),
            },
            PreparedFlowInvocation {
                invocation_id: PreparedFlowInvocationId::new("viewport.1.scene.b"),
                flow_id,
                view_id: "viewport.1".to_string(),
                inputs: PreparedFlowInputs::default(),
                target_alias_bindings: second_bindings,
                history_signature: Some("camera:b".to_string()),
            },
        ],
        dynamic_texture_targets: Vec::new(),
        product_selections: Vec::new(),
        viewport_surface_bindings: ViewportSurfaceBindingRegistry::default(),
        contributions: PreparedFrameContributions::default(),
        shader: PreparedShaderSnapshot {
            registry_revision: 2,
        },
    };

    let err = frame
        .dynamic_target_history_signatures()
        .expect_err("one target cannot carry conflicting history signatures");
    assert!(
        err.to_string().contains("incompatible history signatures"),
        "unexpected error: {err}"
    );
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
