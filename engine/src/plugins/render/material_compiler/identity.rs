//! Canonical material shader identity encoding.

use material_graph::{MaterialIr, MaterialIrInputSource, MaterialOutputTarget};

use super::bindings::SceneMaterialTableResourceBindingPlan;
use super::{
    MATERIAL_WGSL_COMPILER_CONTRACT_VERSION, MaterialPreviewFixture, SceneMaterialTableSlot,
};

pub(super) fn material_shader_identity(ir: &MaterialIr, fixture: MaterialPreviewFixture) -> String {
    let mut encoder = CanonicalShaderIdentityEncoder::default();
    encoder.field(
        "compiler_contract",
        &MATERIAL_WGSL_COMPILER_CONTRACT_VERSION.to_string(),
    );
    encoder.field("ir_contract", &ir.contract_version.to_string());
    encoder.field("document", &ir.document_id.raw().to_string());
    encoder.field("target", output_target_label(ir.output_target));
    encoder.field("fixture", fixture.label());
    encoder.number("node_count", ir.nodes.len() as u64);
    for node in &ir.nodes {
        encoder.field("node_id", &node.node_id.raw().to_string());
        encoder.field("node_op", node.op.label());
        for input in &node.inputs {
            encoder.field("input_name", &input.name);
            encoder.field("input_type", input.value_type.label());
            match &input.source {
                MaterialIrInputSource::Connected {
                    node_id,
                    output_name,
                } => {
                    encoder.field("input_source", "connected");
                    encoder.field("input_source_node", &node_id.raw().to_string());
                    encoder.field("input_source_output", output_name);
                }
                MaterialIrInputSource::Constant(literal) => {
                    encoder.field("input_source", "constant");
                    encoder.field("input_literal", &literal.canonical_component());
                }
                MaterialIrInputSource::NodeValue {
                    key,
                    canonical_value,
                } => {
                    encoder.field("input_source", "node_value");
                    encoder.field("input_value_key", key);
                    encoder.field("input_value", canonical_value);
                }
            }
        }
        for output in &node.outputs {
            encoder.field("output_name", &output.name);
            encoder.field("output_type", output.value_type.label());
        }
        for value in &node.values {
            encoder.field("value_key", &value.key);
            encoder.field("value_value", &value.canonical_value);
        }
    }
    encoder.number("resource_count", ir.required_resources.len() as u64);
    for resource in &ir.required_resources {
        encoder.field("resource_node", &resource.node_id.raw().to_string());
        encoder.field("resource_key", &resource.binding_key);
        encoder.field("resource_ref", &resource.reference.canonical_component());
    }
    encoder.finish_hex()
}

pub(super) fn material_scene_shader_identity(ir: &MaterialIr) -> String {
    let mut encoder = CanonicalShaderIdentityEncoder::default();
    encoder.field(
        "compiler_contract",
        &MATERIAL_WGSL_COMPILER_CONTRACT_VERSION.to_string(),
    );
    encoder.field("entrypoint", "scene");
    encoder.field("ir_contract", &ir.contract_version.to_string());
    encoder.field("document", &ir.document_id.raw().to_string());
    encoder.field("target", output_target_label(ir.output_target));
    for node in &ir.nodes {
        encoder.field("node_id", &node.node_id.raw().to_string());
        encoder.field("node_op", node.op.label());
        for input in &node.inputs {
            encoder.field("input_name", &input.name);
            encoder.field("input_type", input.value_type.label());
            match &input.source {
                MaterialIrInputSource::Connected {
                    node_id,
                    output_name,
                } => {
                    encoder.field("input_source", "connected");
                    encoder.field("input_source_node", &node_id.raw().to_string());
                    encoder.field("input_source_output", output_name);
                }
                MaterialIrInputSource::Constant(literal) => {
                    encoder.field("input_source", "constant");
                    encoder.field("input_literal", &literal.canonical_component());
                }
                MaterialIrInputSource::NodeValue {
                    key,
                    canonical_value,
                } => {
                    encoder.field("input_source", "node_value");
                    encoder.field("input_value_key", key);
                    encoder.field("input_value", canonical_value);
                }
            }
        }
        for output in &node.outputs {
            encoder.field("output_name", &output.name);
            encoder.field("output_type", output.value_type.label());
        }
        for value in &node.values {
            encoder.field("value_key", &value.key);
            encoder.field("value_value", &value.canonical_value);
        }
    }
    encoder.number("resource_count", ir.required_resources.len() as u64);
    for resource in &ir.required_resources {
        encoder.field("resource_node", &resource.node_id.raw().to_string());
        encoder.field("resource_key", &resource.binding_key);
        encoder.field("resource_ref", &resource.reference.canonical_component());
    }
    encoder.finish_hex()
}

pub(super) fn material_scene_table_shader_identity(
    slots: &[SceneMaterialTableSlot<'_>],
    resource_plan: &SceneMaterialTableResourceBindingPlan,
) -> String {
    let mut encoder = CanonicalShaderIdentityEncoder::default();
    encoder.field(
        "compiler_contract",
        &MATERIAL_WGSL_COMPILER_CONTRACT_VERSION.to_string(),
    );
    encoder.field("entrypoint", "scene_material_table");
    encoder.number("slot_count", slots.len() as u64);
    for slot in slots {
        encoder.number("slot_index", slot.slot_index as u64);
        encoder.field("material_instance", &slot.material_instance_id);
        encoder.field("ir_contract", &slot.ir.contract_version.to_string());
        encoder.field("document", &slot.ir.document_id.raw().to_string());
        encoder.field("target", output_target_label(slot.ir.output_target));
        for node in &slot.ir.nodes {
            encoder.field("node_id", &node.node_id.raw().to_string());
            encoder.field("node_op", node.op.label());
            for value in &node.values {
                encoder.field("value_key", &value.key);
                encoder.field("value_value", &value.canonical_value);
            }
        }
    }
    encode_scene_table_resource_layout(&mut encoder, resource_plan);
    encoder.finish_hex()
}

pub(super) fn material_scene_table_resource_layout_identity(
    resource_plan: &SceneMaterialTableResourceBindingPlan,
) -> String {
    let mut encoder = CanonicalShaderIdentityEncoder::default();
    encoder.field(
        "compiler_contract",
        &MATERIAL_WGSL_COMPILER_CONTRACT_VERSION.to_string(),
    );
    encoder.field("entrypoint", "scene_material_table_resource_layout");
    encode_scene_table_resource_layout(&mut encoder, resource_plan);
    encoder.finish_hex()
}

fn encode_scene_table_resource_layout(
    encoder: &mut CanonicalShaderIdentityEncoder,
    resource_plan: &SceneMaterialTableResourceBindingPlan,
) {
    encoder.number(
        "scene_table_resource_layout_count",
        resource_plan.layout_entries.len() as u64,
    );
    for entry in &resource_plan.layout_entries {
        encoder.number("resource_slot_index", entry.resource_slot_index as u64);
        encoder.field("resource_identity", &entry.resource_identity);
        encoder.number("bind_group", entry.bind_group as u64);
        encoder.number("texture_binding", entry.texture_binding as u64);
        encoder.number("sampler_binding", entry.sampler_binding as u64);
        encoder.field(
            "texture_dimension",
            compiled_texture_dimension_label(entry.texture_dimension),
        );
    }
    encoder.number(
        "scene_table_slot_resource_mapping_count",
        resource_plan.slot_mappings.len() as u64,
    );
    for mapping in &resource_plan.slot_mappings {
        encoder.number("mapping_slot_index", mapping.slot_index as u64);
        encoder.field("mapping_material_instance", &mapping.material_instance_id);
        encoder.number("mapping_node_id", mapping.node_id);
        encoder.field("mapping_binding_key", &mapping.binding_key);
        encoder.field("mapping_resource_identity", &mapping.resource_identity);
        encoder.number(
            "mapping_resource_slot_index",
            mapping.resource_slot_index as u64,
        );
        encoder.number("mapping_texture_binding", mapping.texture_binding as u64);
        encoder.number("mapping_sampler_binding", mapping.sampler_binding as u64);
        encoder.field(
            "mapping_texture_dimension",
            compiled_texture_dimension_label(mapping.texture_dimension),
        );
    }
}

fn compiled_texture_dimension_label(
    dimension: super::CompiledMaterialTextureDimension,
) -> &'static str {
    match dimension {
        super::CompiledMaterialTextureDimension::D2 => "2d",
        super::CompiledMaterialTextureDimension::D3 => "3d",
    }
}

#[derive(Default)]
struct CanonicalShaderIdentityEncoder {
    bytes: Vec<u8>,
}

impl CanonicalShaderIdentityEncoder {
    fn number(&mut self, label: &str, value: u64) {
        self.field(label, &value.to_string());
    }

    fn field(&mut self, label: &str, value: &str) {
        self.bytes.extend_from_slice(label.as_bytes());
        self.bytes.push(b'=');
        self.bytes
            .extend_from_slice(value.len().to_string().as_bytes());
        self.bytes.push(b':');
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(b'\n');
    }

    fn finish_hex(self) -> String {
        self.bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

pub(crate) fn output_target_label(output_target: MaterialOutputTarget) -> &'static str {
    match output_target {
        MaterialOutputTarget::PbrPreview => "pbr_preview",
        MaterialOutputTarget::FieldMaterialChannel => "field_material_channel",
        MaterialOutputTarget::RenderMaterial => "render_material",
    }
}

pub(crate) fn fixture_id(fixture: MaterialPreviewFixture) -> u32 {
    match fixture {
        MaterialPreviewFixture::Sphere => 0,
        MaterialPreviewFixture::Box => 1,
        MaterialPreviewFixture::Plane => 2,
        MaterialPreviewFixture::SdfPrimitive => 3,
        MaterialPreviewFixture::FieldMaterial => 4,
    }
}
