//! Resource binding derivation for material compiler outputs.

use std::collections::{BTreeMap, BTreeSet};

use material_graph::MaterialIr;

use super::{
    CompiledMaterialResourceBinding, CompiledMaterialTextureDimension, SceneMaterialTableSlot,
};

pub(super) fn compiled_resource_bindings(ir: &MaterialIr) -> Vec<CompiledMaterialResourceBinding> {
    ir.required_resources
        .iter()
        .enumerate()
        .map(|(index, resource)| compiled_resource_binding_for_resource(resource, index as u32))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SceneMaterialTableResourceBindingPlan {
    pub(super) layout_entries: Vec<SceneMaterialTableResourceLayoutEntry>,
    pub(super) slot_mappings: Vec<SceneMaterialTableResourceSlotMapping>,
    bindings_by_slot_index: BTreeMap<u32, Vec<CompiledMaterialResourceBinding>>,
}

impl SceneMaterialTableResourceBindingPlan {
    pub(super) fn bindings_for_slot(&self, slot_index: u32) -> &[CompiledMaterialResourceBinding] {
        self.bindings_by_slot_index
            .get(&slot_index)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(super) fn all_slot_bindings(&self) -> Vec<CompiledMaterialResourceBinding> {
        self.slot_mappings
            .iter()
            .map(SceneMaterialTableResourceSlotMapping::binding)
            .collect()
    }

    pub(super) fn declaration_bindings(&self) -> Vec<CompiledMaterialResourceBinding> {
        self.layout_entries
            .iter()
            .map(SceneMaterialTableResourceLayoutEntry::binding)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SceneMaterialTableResourceLayoutEntry {
    pub(super) resource_slot_index: u32,
    pub(super) resource_identity: String,
    pub(super) bind_group: u32,
    pub(super) texture_binding: u32,
    pub(super) sampler_binding: u32,
    pub(super) texture_dimension: CompiledMaterialTextureDimension,
}

impl SceneMaterialTableResourceLayoutEntry {
    fn binding(&self) -> CompiledMaterialResourceBinding {
        CompiledMaterialResourceBinding {
            node_id: u64::from(self.resource_slot_index),
            binding_key: format!("scene_table_resource_{}", self.resource_slot_index),
            bind_group: self.bind_group,
            texture_binding: self.texture_binding,
            sampler_binding: self.sampler_binding,
            texture_dimension: self.texture_dimension,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SceneMaterialTableResourceSlotMapping {
    pub(super) slot_index: u32,
    pub(super) material_instance_id: String,
    pub(super) node_id: u64,
    pub(super) binding_key: String,
    pub(super) resource_identity: String,
    pub(super) resource_slot_index: u32,
    pub(super) texture_binding: u32,
    pub(super) sampler_binding: u32,
    pub(super) texture_dimension: CompiledMaterialTextureDimension,
}

impl SceneMaterialTableResourceSlotMapping {
    fn binding(&self) -> CompiledMaterialResourceBinding {
        CompiledMaterialResourceBinding {
            node_id: self.node_id,
            binding_key: self.binding_key.clone(),
            bind_group: 1,
            texture_binding: self.texture_binding,
            sampler_binding: self.sampler_binding,
            texture_dimension: self.texture_dimension,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SceneMaterialTableResourceRequirement {
    slot_index: u32,
    material_instance_id: String,
    node_id: u64,
    binding_key: String,
    resource_identity: String,
    texture_dimension: CompiledMaterialTextureDimension,
}

pub(super) fn scene_material_table_resource_binding_plan(
    slots: &[SceneMaterialTableSlot<'_>],
) -> SceneMaterialTableResourceBindingPlan {
    let mut requirements = Vec::<SceneMaterialTableResourceRequirement>::new();
    for slot in slots {
        for resource in &slot.ir.required_resources {
            requirements.push(SceneMaterialTableResourceRequirement {
                slot_index: slot.slot_index,
                material_instance_id: slot.material_instance_id.clone(),
                node_id: resource.node_id.raw(),
                binding_key: resource.binding_key.clone(),
                resource_identity: compiler_resource_identity(resource),
                texture_dimension: texture_dimension_for_resource(resource),
            });
        }
    }
    requirements.sort_by(|left, right| {
        left.slot_index
            .cmp(&right.slot_index)
            .then(left.material_instance_id.cmp(&right.material_instance_id))
            .then(left.node_id.cmp(&right.node_id))
            .then(left.binding_key.cmp(&right.binding_key))
            .then(left.resource_identity.cmp(&right.resource_identity))
    });

    let mut resource_slots = BTreeMap::<String, u32>::new();
    let mut layout_entries = Vec::<SceneMaterialTableResourceLayoutEntry>::new();
    let mut slot_mappings = Vec::<SceneMaterialTableResourceSlotMapping>::new();
    let mut seen_slot_bindings = BTreeSet::<(u32, u64, String)>::new();
    for requirement in requirements {
        if !seen_slot_bindings.insert((
            requirement.slot_index,
            requirement.node_id,
            requirement.binding_key.clone(),
        )) {
            continue;
        }
        let resource_slot_index = match resource_slots.get(&requirement.resource_identity) {
            Some(index) => *index,
            None => {
                let index = resource_slots.len() as u32;
                resource_slots.insert(requirement.resource_identity.clone(), index);
                layout_entries.push(SceneMaterialTableResourceLayoutEntry {
                    resource_slot_index: index,
                    resource_identity: requirement.resource_identity.clone(),
                    bind_group: 1,
                    texture_binding: index.saturating_mul(2),
                    sampler_binding: index.saturating_mul(2).saturating_add(1),
                    texture_dimension: requirement.texture_dimension,
                });
                index
            }
        };
        let texture_binding = resource_slot_index.saturating_mul(2);
        let sampler_binding = texture_binding.saturating_add(1);
        slot_mappings.push(SceneMaterialTableResourceSlotMapping {
            slot_index: requirement.slot_index,
            material_instance_id: requirement.material_instance_id,
            node_id: requirement.node_id,
            binding_key: requirement.binding_key,
            resource_identity: requirement.resource_identity,
            resource_slot_index,
            texture_binding,
            sampler_binding,
            texture_dimension: requirement.texture_dimension,
        });
    }
    let mut bindings_by_slot_index = BTreeMap::<u32, Vec<CompiledMaterialResourceBinding>>::new();
    for mapping in &slot_mappings {
        bindings_by_slot_index
            .entry(mapping.slot_index)
            .or_default()
            .push(mapping.binding());
    }
    SceneMaterialTableResourceBindingPlan {
        layout_entries,
        slot_mappings,
        bindings_by_slot_index,
    }
}

fn compiled_resource_binding_for_resource(
    resource: &material_graph::MaterialResourceBinding,
    resource_slot_index: u32,
) -> CompiledMaterialResourceBinding {
    CompiledMaterialResourceBinding {
        node_id: resource.node_id.raw(),
        binding_key: resource.binding_key.clone(),
        bind_group: 1,
        texture_binding: resource_slot_index.saturating_mul(2),
        sampler_binding: resource_slot_index.saturating_mul(2).saturating_add(1),
        texture_dimension: texture_dimension_for_resource(resource),
    }
}

fn texture_dimension_for_resource(
    resource: &material_graph::MaterialResourceBinding,
) -> CompiledMaterialTextureDimension {
    match resource.reference.kind.as_str() {
        "asset.catalog.texture3d" | "asset.catalog.texture_3d" | "texture3d" | "texture_3d" => {
            CompiledMaterialTextureDimension::D3
        }
        _ => CompiledMaterialTextureDimension::D2,
    }
}

fn compiler_resource_identity(resource: &material_graph::MaterialResourceBinding) -> String {
    format!(
        "kind={}:dimension={:?}:ref={}",
        resource.reference.kind.as_str(),
        texture_dimension_for_resource(resource),
        resource.reference.canonical_component()
    )
}

pub(crate) fn material_resource_declarations(
    bindings: &[CompiledMaterialResourceBinding],
) -> String {
    if bindings.is_empty() {
        return String::new();
    }
    let mut lines = Vec::new();
    let mut declared = BTreeSet::<(u32, u32, u32)>::new();
    for binding in bindings {
        if !declared.insert((
            binding.bind_group,
            binding.texture_binding,
            binding.sampler_binding,
        )) {
            continue;
        }
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
    format!("rw_material_texture_{}", binding.texture_binding)
}

pub(crate) fn sampler_binding_variable(binding: &CompiledMaterialResourceBinding) -> String {
    format!("rw_material_sampler_{}", binding.sampler_binding)
}
