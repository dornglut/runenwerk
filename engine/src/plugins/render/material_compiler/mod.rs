//! File: engine/src/plugins/render/material_compiler/mod.rs
//! Purpose: Public surface for the engine-owned material IR to WGSL compiler.

mod bindings;
mod diagnostics;
mod identity;
#[cfg(test)]
mod tests;
mod types;
mod validation;
mod wgsl;

pub use diagnostics::MaterialShaderCompileError;
pub use types::{
    CompiledMaterialResourceBinding, CompiledMaterialShader, CompiledMaterialTextureDimension,
    CompiledSceneMaterialTableShader, MaterialPreviewFixture, MaterialShaderCompileRequest,
    SceneMaterialTableCompileRequest, SceneMaterialTableSlot,
};

use bindings::compiled_resource_bindings;
use identity::{
    material_scene_shader_identity, material_scene_table_shader_identity, material_shader_identity,
};
use validation::{validate_ir, validate_wgsl};
use wgsl::{
    WgslMaterialProgram, material_program_wgsl, material_scene_product_wgsl,
    material_scene_table_product_wgsl,
};

pub const MATERIAL_WGSL_COMPILER_CONTRACT_VERSION: u32 = 2;

pub fn compile_material_shader(
    request: MaterialShaderCompileRequest<'_>,
) -> Result<CompiledMaterialShader, MaterialShaderCompileError> {
    validate_ir(request.ir)?;
    let resource_bindings = compiled_resource_bindings(request.ir);
    let generated = WgslMaterialProgram::compile(request.ir, &resource_bindings)?;
    let wgsl = material_program_wgsl(request.ir, request.fixture, &generated);
    let scene_wgsl = material_scene_product_wgsl(request.ir, &generated);
    validate_wgsl(&wgsl)?;
    validate_wgsl(&scene_wgsl)?;
    let identity = material_shader_identity(request.ir, request.fixture);
    let scene_identity = material_scene_shader_identity(request.ir);
    Ok(CompiledMaterialShader {
        shader_id: format!("material.generated.{}", request.ir.document_id.raw()),
        wgsl,
        scene_wgsl,
        resource_bindings,
        identity,
        scene_identity,
        output_target: request.ir.output_target,
    })
}

pub fn compile_scene_material_table_shader(
    request: SceneMaterialTableCompileRequest<'_>,
) -> Result<CompiledSceneMaterialTableShader, MaterialShaderCompileError> {
    if request.slots.is_empty() {
        return Err(MaterialShaderCompileError::InvalidSceneMaterialTable(
            "scene material table requires at least one material slot".to_string(),
        ));
    }
    let mut compiled_slots = Vec::new();
    for slot in &request.slots {
        validate_ir(slot.ir)?;
        let resource_bindings = compiled_resource_bindings(slot.ir);
        if !resource_bindings.is_empty() {
            return Err(MaterialShaderCompileError::InvalidSceneMaterialTable(
                "scene material table compiler does not yet support texture resource bindings"
                    .to_string(),
            ));
        }
        let program = WgslMaterialProgram::compile(slot.ir, &resource_bindings)?;
        compiled_slots.push(wgsl::SceneMaterialTableProgramSlot {
            slot_index: slot.slot_index,
            material_instance_id: slot.material_instance_id.clone(),
            ir: slot.ir,
            program,
        });
    }
    let wgsl = material_scene_table_product_wgsl(&compiled_slots);
    validate_wgsl(&wgsl)?;
    let identity = material_scene_table_shader_identity(&request.slots);
    Ok(CompiledSceneMaterialTableShader {
        shader_id: format!("material.scene_table.{identity}"),
        wgsl,
        identity,
        slot_count: request.slots.len(),
    })
}
