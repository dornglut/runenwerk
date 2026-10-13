use super::*;
use graph::{
    CyclePolicy, GraphDefinition, GraphId, GraphMetadataEntry, GraphValue, NodeDefinition, NodeId,
    PortDefinition, PortDirection, PortId, PortTypeId,
};
use material_graph::{
    MaterialGraphDocument, MaterialGraphDocumentId, MaterialIr, MaterialNodeCatalog,
    MaterialIrInputSource, MaterialNodeOp, MaterialOutputTarget, MaterialValueType,
    lower_material_graph,
};
use resource_ref::ResourceRef;

fn ir() -> MaterialIr {
    let document = MaterialGraphDocument::new(
        MaterialGraphDocumentId::new(7),
        "mat",
        GraphDefinition::new(
            GraphId::new(1),
            "mat",
            CyclePolicy::RejectDirectedCycles,
            [NodeDefinition::new(
                NodeId::new(1),
                "pbr.output",
                [PortDefinition::new(
                    PortId::new(1),
                    "base_color",
                    PortDirection::Input,
                    PortTypeId::new(1),
                )],
            )],
            [],
        ),
        MaterialOutputTarget::RenderMaterial,
    );
    let lowering = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());
    assert!(
        !lowering.report.has_blocking_issues(),
        "{:?}",
        lowering.report.issues()
    );
    lowering.product.expect("formed").executable_ir.expect("ir")
}

fn material_channel_color_ir() -> MaterialIr {
    let color = PortTypeId::new(1);
    let scalar = PortTypeId::new(2);
    let document = MaterialGraphDocument::new(
        MaterialGraphDocumentId::new(8),
        "slot_color",
        GraphDefinition::new(
            GraphId::new(1),
            "slot_color",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(1),
                    "sdf.material_channel",
                    [PortDefinition::new(
                        PortId::new(1),
                        "value",
                        PortDirection::Output,
                        scalar,
                    )],
                ),
                NodeDefinition::new(
                    NodeId::new(2),
                    "proc.ramp",
                    [
                        PortDefinition::new(PortId::new(2), "value", PortDirection::Input, scalar),
                        PortDefinition::new(PortId::new(3), "color", PortDirection::Output, color),
                    ],
                ),
                NodeDefinition::new(
                    NodeId::new(3),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(4),
                        "base_color",
                        PortDirection::Input,
                        color,
                    )],
                ),
            ],
            [
                graph::EdgeDefinition::new(graph::EdgeId::new(1), PortId::new(1), PortId::new(2)),
                graph::EdgeDefinition::new(graph::EdgeId::new(2), PortId::new(3), PortId::new(4)),
            ],
        ),
        MaterialOutputTarget::RenderMaterial,
    );
    let lowering = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());
    assert!(
        !lowering.report.has_blocking_issues(),
        "{:?}",
        lowering.report.issues()
    );
    lowering.product.expect("formed").executable_ir.expect("ir")
}

fn texture_base_color_ir(document_id: u64, stable_id: &str) -> MaterialIr {
    let color = PortTypeId::new(1);
    let vec2 = PortTypeId::new(3);
    let texture_ref = ResourceRef::new("asset.catalog.texture2d", stable_id).expect("resource ref");
    let document = MaterialGraphDocument::new(
        MaterialGraphDocumentId::new(document_id),
        "texture_slot",
        GraphDefinition::new(
            GraphId::new(1),
            "texture_slot",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(1),
                    "texture.sample_2d",
                    [
                        PortDefinition::new(PortId::new(1), "uv", PortDirection::Input, vec2),
                        PortDefinition::new(PortId::new(2), "color", PortDirection::Output, color),
                    ],
                )
                .with_values([GraphMetadataEntry::new(
                    material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                    GraphValue::resource(texture_ref),
                )]),
                NodeDefinition::new(
                    NodeId::new(2),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(3),
                        "base_color",
                        PortDirection::Input,
                        color,
                    )],
                ),
            ],
            [graph::EdgeDefinition::new(
                graph::EdgeId::new(1),
                PortId::new(2),
                PortId::new(3),
            )],
        ),
        MaterialOutputTarget::RenderMaterial,
    );
    let lowering = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());
    assert!(
        !lowering.report.has_blocking_issues(),
        "{:?}",
        lowering.report.issues()
    );
    lowering.product.expect("formed").executable_ir.expect("ir")
}

fn all_first_slice_ops_ir() -> MaterialIr {
    let catalog = MaterialNodeCatalog::first_slice();
    let nodes = catalog
        .descriptors()
        .enumerate()
        .map(|(index, descriptor)| {
            let mut node =
                NodeDefinition::new(NodeId::new((index + 1) as u64), descriptor.key.clone(), []);
            if descriptor.key == "texture.sample_2d" {
                node = node.with_values([GraphMetadataEntry::new(
                    material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                    GraphValue::resource(
                        ResourceRef::new("asset.catalog.texture2d", "albedo")
                            .expect("resource ref"),
                    ),
                )]);
            }
            if descriptor.key == "texture.sample_3d" {
                node = node.with_values([GraphMetadataEntry::new(
                    material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                    GraphValue::resource(
                        ResourceRef::new("asset.catalog.texture3d", "volume")
                            .expect("resource ref"),
                    ),
                )]);
            }
            node
        })
        .collect::<Vec<_>>();
    let document = MaterialGraphDocument::new(
        MaterialGraphDocumentId::new(9),
        "all_ops",
        GraphDefinition::new(
            GraphId::new(1),
            "all_ops",
            CyclePolicy::RejectDirectedCycles,
            nodes,
            [],
        ),
        MaterialOutputTarget::RenderMaterial,
    );
    let lowering = lower_material_graph(&document, &catalog);
    assert!(
        !lowering.report.has_blocking_issues(),
        "{:?}",
        lowering.report.issues()
    );
    lowering.product.expect("formed").executable_ir.expect("ir")
}

#[test]
fn compiler_emits_valid_deterministic_wgsl() {
    let ir = ir();
    let first = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("compile");
    let second = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("compile");

    assert_eq!(first.wgsl, second.wgsl);
    assert_eq!(first.scene_wgsl, second.scene_wgsl);
    assert_eq!(first.identity, second.identity);
    assert_eq!(first.scene_identity, second.scene_identity);
    assert!(first.wgsl.contains("fn evaluate_material"));
    assert!(first.scene_wgsl.contains("fn evaluate_material"));
    assert!(
        first
            .scene_wgsl
            .contains("EditorViewportSceneProductUniform")
    );
    assert!(first.wgsl.contains("fn fs_main"));
}

#[test]
fn fixture_changes_shader_identity() {
    let ir = ir();
    let sphere = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("compile");
    let plane = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Plane,
    })
    .expect("compile");

    assert_ne!(sphere.identity, plane.identity);
}

#[test]
fn compiler_generates_semantics_for_every_first_slice_node() {
    let ir = all_first_slice_ops_ir();
    let shader = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::FieldMaterial,
    })
    .expect("all first-slice ops should compile");

    assert!(shader.wgsl.contains("rw_noise3"));
    assert!(shader.wgsl.contains("rw_fbm3"));
    assert!(shader.wgsl.contains("texture_2d<f32>"));
    assert!(shader.wgsl.contains("texture_3d<f32>"));
    assert!(shader.wgsl.contains("textureSample"));
    assert!(shader.wgsl.contains("rw_triplanar_weights"));
    assert!(shader.scene_wgsl.contains("texture_2d<f32>"));
    assert!(shader.scene_wgsl.contains("textureSample"));
    assert!(shader.scene_wgsl.contains("march_scene"));
}

#[test]
fn generated_scene_wgsl_reads_hit_material_slot() {
    let ir = ir();
    let shader = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("compile");

    assert!(shader.scene_wgsl.contains("struct SceneDistanceSample"));
    assert!(shader.scene_wgsl.contains("sdf_primitive_index : u32"));
    assert!(shader.scene_wgsl.contains("material_slot_index : u32"));
    assert!(
        shader
            .scene_wgsl
            .contains("let material_slot_index = slot.w;")
    );
    assert!(
        shader
            .scene_wgsl
            .contains("evaluate_scene_material(march.material_slot_index")
    );
    assert!(
        !shader
            .scene_wgsl
            .contains("let material = evaluate_material(MaterialEvalContext("),
        "generated scene WGSL must not feed a global material slot into material evaluation"
    );
}

#[test]
fn generated_scene_wgsl_uses_typed_u32_material_slot_lane() {
    let ir = ir();
    let shader = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("compile");

    assert!(
        shader
            .scene_wgsl
            .contains("primitive_slot_flags : array<vec4<u32>, 64>"),
        "SDF material slot identity must travel through a typed u32 lane"
    );
    assert!(
        shader
            .scene_wgsl
            .contains("let material_slot_index = slot.w;")
    );
    assert!(
        !shader.scene_wgsl.contains("u32(max(params_b.z, 0.0))"),
        "SDF material slot identity must not be decoded from an untyped f32 params lane"
    );
}

#[test]
fn generated_scene_wgsl_renders_model_mesh_regions_from_material_selection_lane() {
    let ir = ir();
    let shader = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("compile");

    assert!(
        shader
            .scene_wgsl
            .contains("model_mesh_region_flags : array<vec4<u32>, 16>"),
        "model/mesh material-region slots must travel through a typed u32 lane"
    );
    assert!(
        shader
            .scene_wgsl
            .contains("fn model_mesh_region_at(viewport_local: vec2<f32>)")
    );
    assert!(
        shader
            .scene_wgsl
            .contains("return ModelMeshRegionHit(true, flags.x")
    );
    assert!(
        shader
            .scene_wgsl
            .contains("evaluate_scene_material(hit.material_slot_index")
    );
}

#[test]
fn generated_scene_wgsl_selects_material_from_hit_sdf_slot() {
    let ir = material_channel_color_ir();
    let shader = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("compile");

    assert!(
        shader
            .scene_wgsl
            .contains("fn evaluate_scene_material(material_slot_index: u32")
    );
    assert!(
        shader
            .scene_wgsl
            .contains("slot_ctx.material_channel = f32(material_slot_index);")
    );
    assert!(
        shader
            .scene_wgsl
            .contains("let n1_value: f32 = ctx.material_channel;")
    );
    assert!(
        shader
            .scene_wgsl
            .contains("mix(vec4<f32>(0.0, 0.0, 0.0, 1.0), vec4<f32>(1.0, 1.0, 1.0, 1.0), clamp(n1_value, 0.0, 1.0))")
    );
    assert!(
        shader
            .scene_wgsl
            .contains("let material = evaluate_scene_material(march.material_slot_index")
    );
    assert!(shader.scene_wgsl.contains("material_error_output"));
}

#[test]
fn scene_material_table_wgsl_dispatches_to_source_backed_slot_evaluators() {
    let default_ir = ir();
    let assigned_ir = material_channel_color_ir();
    let shader = compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
        slots: vec![
            SceneMaterialTableSlot {
                slot_index: 0,
                material_instance_id: "material.asset.10".to_string(),
                ir: &default_ir,
            },
            SceneMaterialTableSlot {
                slot_index: 1,
                material_instance_id: "material.asset.11".to_string(),
                ir: &assigned_ir,
            },
        ],
    })
    .expect("scene material table should compile");

    assert_eq!(shader.slot_count, 2);
    assert!(shader.wgsl.contains("fn evaluate_material_slot_0"));
    assert!(shader.wgsl.contains("fn evaluate_material_slot_1"));
    assert!(shader.wgsl.contains("switch material_slot_index"));
    assert!(
        shader
            .wgsl
            .contains("case 0u: { return evaluate_material_slot_0(ctx); }")
    );
    assert!(
        shader
            .wgsl
            .contains("case 1u: { return evaluate_material_slot_1(ctx); }")
    );
    assert!(
        !shader
            .wgsl
            .contains("slot_ctx.material_channel = f32(material_slot_index);"),
        "scene material tables must dispatch by slot instead of rewriting material_channel"
    );
    assert!(
        shader
            .wgsl
            .contains("fn model_mesh_region_at(viewport_local: vec2<f32>)")
            && shader
                .wgsl
                .contains("evaluate_scene_material(hit.material_slot_index"),
        "scene material table shaders must retain the model/mesh material-region render path"
    );
}

#[test]
fn scene_material_table_compiles_texture_bearing_slots_to_valid_wgsl() {
    let default_ir = texture_base_color_ir(10, "texture.albedo");
    let assigned_ir = texture_base_color_ir(11, "texture.moss");

    let shader = compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
        slots: vec![
            SceneMaterialTableSlot {
                slot_index: 0,
                material_instance_id: "material.asset.10".to_string(),
                ir: &default_ir,
            },
            SceneMaterialTableSlot {
                slot_index: 1,
                material_instance_id: "material.asset.11".to_string(),
                ir: &assigned_ir,
            },
        ],
    })
    .expect("texture-bearing scene material table should compile");

    assert_eq!(shader.slot_count, 2);
    assert!(shader.wgsl.contains("texture_2d<f32>"));
    assert!(shader.wgsl.contains("textureSample"));
    assert!(shader.wgsl.contains("fn evaluate_material_slot_0"));
    assert!(shader.wgsl.contains("fn evaluate_material_slot_1"));
    assert!(
        shader.wgsl.contains("@group(1) @binding(0)")
            && shader.wgsl.contains("@group(1) @binding(2)"),
        "different texture resources must receive different table-wide bindings"
    );
}

#[test]
fn scene_material_table_slot_evaluators_use_table_wide_resource_bindings() {
    let default_ir = texture_base_color_ir(12, "texture.albedo");
    let assigned_ir = texture_base_color_ir(13, "texture.moss");

    let shader = compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
        slots: vec![
            SceneMaterialTableSlot {
                slot_index: 0,
                material_instance_id: "material.asset.10".to_string(),
                ir: &default_ir,
            },
            SceneMaterialTableSlot {
                slot_index: 1,
                material_instance_id: "material.asset.11".to_string(),
                ir: &assigned_ir,
            },
        ],
    })
    .expect("texture-bearing scene material table should compile");

    assert_eq!(shader.resource_bindings.len(), 2);
    assert_eq!(shader.resource_bindings[0].resource_slot_index, 0);
    assert_eq!(shader.resource_bindings[1].resource_slot_index, 1);
    assert_eq!(shader.resource_bindings[0].material_table_slot, Some(0));
    assert_eq!(shader.resource_bindings[1].material_table_slot, Some(1));
    assert_eq!(shader.resource_bindings[0].texture_binding, 0);
    assert_eq!(shader.resource_bindings[1].texture_binding, 2);
    assert!(
        shader
            .wgsl
            .contains("textureSample(rw_material_texture_0, rw_material_sampler_1")
    );
    assert!(
        shader
            .wgsl
            .contains("textureSample(rw_material_texture_2, rw_material_sampler_3")
    );
    assert_eq!(shader.wgsl.matches("@group(1) @binding(0)").count(), 1);
    assert_eq!(shader.wgsl.matches("@group(1) @binding(2)").count(), 1);
}

#[test]
fn scene_material_table_deduplicates_identical_resource_refs() {
    let default_ir = texture_base_color_ir(14, "texture.shared");
    let assigned_ir = texture_base_color_ir(15, "texture.shared");

    let shader = compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
        slots: vec![
            SceneMaterialTableSlot {
                slot_index: 0,
                material_instance_id: "material.asset.10".to_string(),
                ir: &default_ir,
            },
            SceneMaterialTableSlot {
                slot_index: 1,
                material_instance_id: "material.asset.11".to_string(),
                ir: &assigned_ir,
            },
        ],
    })
    .expect("texture-bearing scene material table should compile");

    assert_eq!(shader.resource_bindings.len(), 2);
    assert!(
        shader
            .resource_bindings
            .iter()
            .all(|binding| binding.texture_binding == 0)
    );
    assert!(
        shader
            .resource_bindings
            .iter()
            .all(|binding| binding.resource_slot_index == 0)
    );
    assert_eq!(
        shader
            .resource_bindings
            .iter()
            .map(|binding| binding.material_table_slot)
            .collect::<Vec<_>>(),
        vec![Some(0), Some(1)]
    );
    assert_eq!(shader.wgsl.matches("@group(1) @binding(0)").count(), 1);
}

#[test]
fn compiler_publishes_single_material_records_matching_generated_wgsl() {
    let ir = texture_base_color_ir(26, "texture.albedo");
    let shader = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect("texture-bearing material should compile");

    assert_eq!(shader.resource_bindings.len(), 1);
    let binding = &shader.resource_bindings[0];
    assert_eq!(binding.resource_slot_index, 0);
    assert_eq!(binding.material_table_slot, None);
    assert_eq!(binding.bind_group, 1);
    assert_eq!(binding.texture_binding, 0);
    assert_eq!(binding.sampler_binding, 1);
    for source in [&shader.wgsl, &shader.scene_wgsl] {
        assert!(source.contains("@group(1) @binding(0)"));
        assert!(source.contains("@group(1) @binding(1)"));
        assert!(source.contains("rw_material_texture_0"));
        assert!(source.contains("rw_material_sampler_1"));
    }
}

#[test]
fn scene_material_table_identity_changes_for_resource_layout_and_mapping() {
    let albedo_ir = texture_base_color_ir(16, "texture.albedo");
    let moss_ir = texture_base_color_ir(17, "texture.moss");
    let sand_ir = texture_base_color_ir(18, "texture.sand");

    let baseline = compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
        slots: vec![
            SceneMaterialTableSlot {
                slot_index: 0,
                material_instance_id: "material.asset.10".to_string(),
                ir: &albedo_ir,
            },
            SceneMaterialTableSlot {
                slot_index: 1,
                material_instance_id: "material.asset.11".to_string(),
                ir: &moss_ir,
            },
        ],
    })
    .expect("baseline table should compile");
    let changed_resource = compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
        slots: vec![
            SceneMaterialTableSlot {
                slot_index: 0,
                material_instance_id: "material.asset.10".to_string(),
                ir: &albedo_ir,
            },
            SceneMaterialTableSlot {
                slot_index: 1,
                material_instance_id: "material.asset.11".to_string(),
                ir: &sand_ir,
            },
        ],
    })
    .expect("changed resource table should compile");
    let changed_mapping = compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
        slots: vec![
            SceneMaterialTableSlot {
                slot_index: 0,
                material_instance_id: "material.asset.10".to_string(),
                ir: &moss_ir,
            },
            SceneMaterialTableSlot {
                slot_index: 1,
                material_instance_id: "material.asset.11".to_string(),
                ir: &albedo_ir,
            },
        ],
    })
    .expect("changed mapping table should compile");
    let changed_slot_order =
        compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
            slots: vec![
                SceneMaterialTableSlot {
                    slot_index: 1,
                    material_instance_id: "material.asset.10".to_string(),
                    ir: &albedo_ir,
                },
                SceneMaterialTableSlot {
                    slot_index: 0,
                    material_instance_id: "material.asset.11".to_string(),
                    ir: &moss_ir,
                },
            ],
        })
        .expect("changed slot order table should compile");

    assert_ne!(baseline.identity, changed_resource.identity);
    assert_ne!(
        baseline.resource_layout_identity,
        changed_resource.resource_layout_identity
    );
    assert_ne!(baseline.identity, changed_mapping.identity);
    assert_ne!(
        baseline.resource_layout_identity,
        changed_mapping.resource_layout_identity
    );
    assert_ne!(baseline.identity, changed_slot_order.identity);
    assert_ne!(
        baseline.resource_layout_identity,
        changed_slot_order.resource_layout_identity
    );
}

#[test]
fn compiler_rejects_missing_output_node() {
    let mut ir = ir();
    ir.nodes.retain(|node| node.op != MaterialNodeOp::PbrOutput);

    let error = compile_material_shader(MaterialShaderCompileRequest {
        ir: &ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .expect_err("missing output should fail");

    assert_eq!(error, MaterialShaderCompileError::MissingOutputNode);
}


fn output_ir(target: MaterialOutputTarget) -> MaterialIr {
    let mut ir = ir();
    ir.output_target = target;
    ir
}

fn lowered_authored_output_ir(
    target: MaterialOutputTarget,
    input: &str,
    value: &str,
) -> MaterialIr {
    let document = MaterialGraphDocument::new(
        MaterialGraphDocumentId::new(18),
        "authored_output",
        GraphDefinition::new(
            GraphId::new(1),
            "authored_output",
            CyclePolicy::RejectDirectedCycles,
            [NodeDefinition::new(NodeId::new(1), "pbr.output", []).with_values([
                GraphMetadataEntry::new(input, GraphValue::Text(value.to_string())),
            ])],
            [],
        ),
        target,
    );
    let lowering = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());
    assert!(
        !lowering.report.has_blocking_issues(),
        "{:?}",
        lowering.report.issues()
    );
    lowering.product.expect("formed").executable_ir.expect("ir")
}

fn lowered_connected_output_ir(target: MaterialOutputTarget, input: &str) -> MaterialIr {
    let float = MaterialValueType::Float.port_type_id();
    let document = MaterialGraphDocument::new(
        MaterialGraphDocumentId::new(19),
        "connected_output",
        GraphDefinition::new(
            GraphId::new(1),
            "connected_output",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(10),
                    "pbr.roughness",
                    [PortDefinition::new(
                        PortId::new(1),
                        "value",
                        PortDirection::Output,
                        float,
                    )],
                ),
                NodeDefinition::new(
                    NodeId::new(11),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(2),
                        input,
                        PortDirection::Input,
                        float,
                    )],
                ),
            ],
            [graph::EdgeDefinition::new(
                graph::EdgeId::new(1),
                PortId::new(1),
                PortId::new(2),
            )],
        ),
        target,
    );
    let lowering = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());
    assert!(
        !lowering.report.has_blocking_issues(),
        "{:?}",
        lowering.report.issues()
    );
    lowering.product.expect("formed").executable_ir.expect("ir")
}

fn compile_ir(ir: &MaterialIr) -> Result<CompiledMaterialShader, MaterialShaderCompileError> {
    compile_material_shader(MaterialShaderCompileRequest {
        ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
}

#[test]
fn renderer_capabilities_admit_default_output_inputs_for_both_shader_targets() {
    for target in [MaterialOutputTarget::PbrPreview, MaterialOutputTarget::RenderMaterial] {
        let ir = output_ir(target);
        let output = ir
            .nodes
            .iter()
            .find(|node| node.op == MaterialNodeOp::PbrOutput)
            .expect("output");
        assert!(output
            .inputs
            .iter()
            .all(|input| matches!(input.source, MaterialIrInputSource::Constant(_))));
        assert_eq!(compile_ir(&ir).expect("default material"), compile_ir(&ir).unwrap());
    }
}

#[test]
fn renderer_capabilities_admit_only_realized_explicit_pbr_values() {
    for (target, input, value) in [
        (MaterialOutputTarget::PbrPreview, "base_color", "0.2 0.4 0.6 1.0"),
        (MaterialOutputTarget::PbrPreview, "roughness", "0.25"),
        (MaterialOutputTarget::PbrPreview, "metallic", "0.7"),
        (MaterialOutputTarget::PbrPreview, "emissive", "0.1 0.2 0.3"),
        (MaterialOutputTarget::PbrPreview, "opacity", "0.6"),
        (MaterialOutputTarget::RenderMaterial, "base_color", "0.2 0.4 0.6 1.0"),
        (MaterialOutputTarget::RenderMaterial, "emissive", "0.1 0.2 0.3"),
        (MaterialOutputTarget::RenderMaterial, "opacity", "0.6"),
    ] {
        let ir = lowered_authored_output_ir(target, input, value);
        let output = ir.nodes.iter().find(|node| node.op == MaterialNodeOp::PbrOutput).unwrap();
        assert!(matches!(
            output.inputs.iter().find(|source| source.name == input).unwrap().source,
            MaterialIrInputSource::NodeValue { .. }
        ));
        compile_ir(&ir).unwrap_or_else(|error| {
            panic!("{target:?} authored {input} should compile: {error}")
        });
    }
}

#[test]
fn renderer_capabilities_reject_ignored_explicit_pbr_values() {
    for (target, input) in [
        (MaterialOutputTarget::PbrPreview, "normal_strength"),
        (MaterialOutputTarget::PbrPreview, "material_channel"),
        (MaterialOutputTarget::RenderMaterial, "roughness"),
        (MaterialOutputTarget::RenderMaterial, "metallic"),
        (MaterialOutputTarget::RenderMaterial, "normal_strength"),
        (MaterialOutputTarget::RenderMaterial, "material_channel"),
    ] {
        let ir = lowered_authored_output_ir(target, input, "0.4");
        assert_eq!(
            compile_ir(&ir).expect_err("unsupported authored input must reject"),
            MaterialShaderCompileError::UnsupportedAuthoredOutputInput {
                target,
                node_id: 1,
                input: input.to_string(),
            }
        );
    }
}

#[test]
fn renderer_capabilities_reject_ignored_connections_but_keep_supported_connections() {
    let allowed = lowered_connected_output_ir(MaterialOutputTarget::PbrPreview, "roughness");
    let output = allowed.nodes.iter().find(|node| node.op == MaterialNodeOp::PbrOutput).unwrap();
    assert!(matches!(
        output.inputs.iter().find(|input| input.name == "roughness").unwrap().source,
        MaterialIrInputSource::Connected { .. }
    ));
    compile_ir(&allowed).expect("preview roughness connection is realized");

    for (target, input) in [
        (MaterialOutputTarget::PbrPreview, "normal_strength"),
        (MaterialOutputTarget::PbrPreview, "material_channel"),
        (MaterialOutputTarget::RenderMaterial, "roughness"),
        (MaterialOutputTarget::RenderMaterial, "metallic"),
        (MaterialOutputTarget::RenderMaterial, "normal_strength"),
        (MaterialOutputTarget::RenderMaterial, "material_channel"),
    ] {
        let ir = lowered_connected_output_ir(target, input);
        assert_eq!(
            compile_ir(&ir).expect_err("unsupported connected input must reject"),
            MaterialShaderCompileError::UnsupportedAuthoredOutputInput {
                target,
                node_id: 11,
                input: input.to_string(),
            }
        );
    }
}

#[test]
fn renderer_capabilities_reject_field_product_target_even_with_default_inputs() {
    let ir = output_ir(MaterialOutputTarget::FieldMaterialChannel);
    assert_eq!(
        compile_ir(&ir).expect_err("field product not realized"),
        MaterialShaderCompileError::UnsupportedOutputTarget {
            target: MaterialOutputTarget::FieldMaterialChannel,
        }
    );

    assert_eq!(
        compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
            slots: vec![SceneMaterialTableSlot {
                slot_index: 0,
                material_instance_id: "field.material".to_string(),
                ir: &ir,
            }],
        })
        .expect_err("field product must not compile in table"),
        MaterialShaderCompileError::UnsupportedOutputTarget {
            target: MaterialOutputTarget::FieldMaterialChannel,
        }
    );
}

#[test]
fn renderer_capabilities_reject_mixed_authored_semantics_without_publishing_wgsl() {
    let mut ir = lowered_authored_output_ir(MaterialOutputTarget::RenderMaterial, "base_color", "0.1 0.2 0.3 1.0");
    let output = ir.nodes.iter_mut().find(|node| node.op == MaterialNodeOp::PbrOutput).unwrap();
    output.inputs.iter_mut().find(|input| input.name == "roughness").unwrap().source =
        MaterialIrInputSource::NodeValue {
            key: "roughness".to_string(),
            canonical_value: GraphValue::Text("0.4".to_string()).canonical_component(),
        };
    assert_eq!(
        compile_ir(&ir).expect_err("mixed material must fail as a unit"),
        MaterialShaderCompileError::UnsupportedAuthoredOutputInput {
            target: MaterialOutputTarget::RenderMaterial,
            node_id: 1,
            input: "roughness".to_string(),
        }
    );
}

#[test]
fn scene_material_table_rejects_ignored_output_inputs_in_any_slot() {
    let valid = ir();
    let invalid = lowered_authored_output_ir(MaterialOutputTarget::RenderMaterial, "metallic", "0.8");
    assert_eq!(
        compile_scene_material_table_shader(SceneMaterialTableCompileRequest {
            slots: vec![
                SceneMaterialTableSlot {
                    slot_index: 0,
                    material_instance_id: "valid".to_string(),
                    ir: &valid,
                },
                SceneMaterialTableSlot {
                    slot_index: 1,
                    material_instance_id: "invalid".to_string(),
                    ir: &invalid,
                },
            ],
        })
        .expect_err("unsupported table slot must reject"),
        MaterialShaderCompileError::UnsupportedAuthoredOutputInput {
            target: MaterialOutputTarget::RenderMaterial,
            node_id: 1,
            input: "metallic".to_string(),
        }
    );
}

#[test]
fn unsupported_output_admission_does_not_suppress_structural_validation() {
    let mut ir = lowered_connected_output_ir(MaterialOutputTarget::RenderMaterial, "roughness");
    ir.nodes.retain(|node| node.node_id != NodeId::new(10));
    assert!(matches!(
        compile_ir(&ir).expect_err("dangling connection"),
        MaterialShaderCompileError::MissingConnectedOutput {
            input,
            source_node_id: 10,
            ..
        } if input == "roughness"
    ));
}
