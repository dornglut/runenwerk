//! Public material compiler request and output types.

use material_graph::{MaterialIr, MaterialOutputTarget};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledMaterialShader {
    pub shader_id: String,
    pub wgsl: String,
    pub scene_wgsl: String,
    pub resource_bindings: Vec<CompiledMaterialResourceBinding>,
    pub identity: String,
    pub scene_identity: String,
    pub output_target: MaterialOutputTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledMaterialResourceBinding {
    pub node_id: u64,
    pub binding_key: String,
    pub bind_group: u32,
    pub texture_binding: u32,
    pub sampler_binding: u32,
    pub texture_dimension: CompiledMaterialTextureDimension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledMaterialTextureDimension {
    D2,
    D3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialPreviewFixture {
    Sphere,
    Box,
    Plane,
    SdfPrimitive,
    FieldMaterial,
}

impl MaterialPreviewFixture {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sphere => "sphere",
            Self::Box => "box",
            Self::Plane => "plane",
            Self::SdfPrimitive => "sdf_primitive",
            Self::FieldMaterial => "field_material",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialShaderCompileRequest<'a> {
    pub ir: &'a MaterialIr,
    pub fixture: MaterialPreviewFixture,
}
