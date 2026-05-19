//! Material IR and generated WGSL validation.

use std::collections::BTreeSet;

use material_graph::{
    MATERIAL_IR_CONTRACT_VERSION, MaterialIr, MaterialIrInputSource, MaterialNodeOp,
};

use super::{MaterialShaderCompileError, SceneMaterialTableSlot};

pub(super) fn validate_ir(ir: &MaterialIr) -> Result<(), MaterialShaderCompileError> {
    if ir.contract_version != MATERIAL_IR_CONTRACT_VERSION {
        return Err(MaterialShaderCompileError::UnsupportedIrVersion {
            found: ir.contract_version,
            expected: MATERIAL_IR_CONTRACT_VERSION,
        });
    }
    let output_nodes = ir
        .nodes
        .iter()
        .filter(|node| node.op == MaterialNodeOp::PbrOutput)
        .count();
    if output_nodes == 0 {
        return Err(MaterialShaderCompileError::MissingOutputNode);
    }
    if output_nodes > 1 {
        return Err(MaterialShaderCompileError::DuplicateOutputNode);
    }

    let mut outputs = BTreeSet::<(u64, String)>::new();
    for node in &ir.nodes {
        if !node.outputs.is_empty() && node.op == MaterialNodeOp::PbrOutput {
            return Err(MaterialShaderCompileError::InvalidNodeContract {
                node_id: node.node_id.raw(),
                message: "pbr.output must not declare value outputs".to_string(),
            });
        }
        for output in &node.outputs {
            outputs.insert((node.node_id.raw(), output.name.clone()));
        }
    }
    for node in &ir.nodes {
        for input in &node.inputs {
            if let MaterialIrInputSource::Connected {
                node_id,
                output_name,
            } = &input.source
            {
                if !outputs.contains(&(node_id.raw(), output_name.clone())) {
                    return Err(MaterialShaderCompileError::MissingConnectedOutput {
                        node_id: node.node_id.raw(),
                        input: input.name.clone(),
                        source_node_id: node_id.raw(),
                        output: output_name.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

pub(super) fn validate_scene_material_table_slots(
    slots: &[SceneMaterialTableSlot<'_>],
) -> Result<(), MaterialShaderCompileError> {
    if slots.is_empty() {
        return Err(MaterialShaderCompileError::InvalidSceneMaterialTable(
            "scene material table requires at least one material slot".to_string(),
        ));
    }
    let mut seen_slot_indices = BTreeSet::new();
    for slot in slots {
        if !seen_slot_indices.insert(slot.slot_index) {
            return Err(MaterialShaderCompileError::InvalidSceneMaterialTable(
                format!("duplicate material slot index {}", slot.slot_index),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_wgsl(source: &str) -> Result<(), MaterialShaderCompileError> {
    let module = naga::front::wgsl::parse_str(source)
        .map_err(|error| MaterialShaderCompileError::InvalidWgsl(error.to_string()))?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map(|_| ())
    .map_err(|error| MaterialShaderCompileError::InvalidWgsl(format!("{error:?}")))
}
