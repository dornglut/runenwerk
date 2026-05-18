//! Resource binding derivation for material compiler outputs.

use material_graph::MaterialIr;

use super::{CompiledMaterialResourceBinding, CompiledMaterialTextureDimension};

pub(super) fn compiled_resource_bindings(ir: &MaterialIr) -> Vec<CompiledMaterialResourceBinding> {
    ir.required_resources
        .iter()
        .enumerate()
        .map(|(index, resource)| {
            let texture_dimension = match resource.reference.kind.as_str() {
                "asset.catalog.texture3d"
                | "asset.catalog.texture_3d"
                | "texture3d"
                | "texture_3d" => CompiledMaterialTextureDimension::D3,
                _ => CompiledMaterialTextureDimension::D2,
            };
            CompiledMaterialResourceBinding {
                node_id: resource.node_id.raw(),
                binding_key: resource.binding_key.clone(),
                bind_group: 1,
                texture_binding: (index as u32).saturating_mul(2),
                sampler_binding: (index as u32).saturating_mul(2).saturating_add(1),
                texture_dimension,
            }
        })
        .collect()
}

pub(crate) fn material_resource_declarations(
    bindings: &[CompiledMaterialResourceBinding],
) -> String {
    if bindings.is_empty() {
        return String::new();
    }
    let mut lines = Vec::new();
    for binding in bindings {
        let texture_type = match binding.texture_dimension {
            CompiledMaterialTextureDimension::D2 => "texture_2d<f32>",
            CompiledMaterialTextureDimension::D3 => "texture_3d<f32>",
        };
        lines.push(format!(
            "@group({}) @binding({})\nvar {} : {};",
            binding.bind_group,
            binding.texture_binding,
            texture_binding_variable(binding),
            texture_type
        ));
        lines.push(format!(
            "@group({}) @binding({})\nvar {} : sampler;",
            binding.bind_group,
            binding.sampler_binding,
            sampler_binding_variable(binding)
        ));
    }
    let mut declarations = lines.join("\n");
    declarations.push('\n');
    declarations
}

pub(crate) fn texture_binding_variable(binding: &CompiledMaterialResourceBinding) -> String {
    format!("rw_material_texture_{}", binding.node_id)
}

pub(crate) fn sampler_binding_variable(binding: &CompiledMaterialResourceBinding) -> String {
    format!("rw_material_sampler_{}", binding.node_id)
}
