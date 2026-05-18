//! Canonical material shader identity encoding.

use material_graph::{MaterialIr, MaterialIrInputSource, MaterialOutputTarget};

use super::{MATERIAL_WGSL_COMPILER_CONTRACT_VERSION, MaterialPreviewFixture};

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
        for value in &node.values {
            encoder.field("value_key", &value.key);
            encoder.field("value_value", &value.canonical_value);
        }
    }
    encoder.finish_hex()
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
