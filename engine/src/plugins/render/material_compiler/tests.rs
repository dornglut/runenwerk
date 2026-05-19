use super::*;
use graph::{
    CyclePolicy, GraphDefinition, GraphId, GraphMetadataEntry, GraphValue, NodeDefinition, NodeId,
    PortDefinition, PortDirection, PortId, PortTypeId,
};
use material_graph::{
    MaterialGraphDocument, MaterialGraphDocumentId, MaterialIr, MaterialNodeCatalog,
    MaterialNodeOp, MaterialOutputTarget, lower_material_graph,
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
    assert!(shader.wgsl.contains("case 0u: { return evaluate_material_slot_0(ctx); }"));
    assert!(shader.wgsl.contains("case 1u: { return evaluate_material_slot_1(ctx); }"));
    assert!(
        !shader
            .wgsl
            .contains("slot_ctx.material_channel = f32(material_slot_index);"),
        "scene material tables must dispatch by slot instead of rewriting material_channel"
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
