//! WGSL program generation for material preview and scene entrypoints.

mod literals;
mod preview;
mod program;
mod scene;

pub(crate) use preview::material_program_wgsl;
pub(crate) use program::WgslMaterialProgram;
pub(crate) use scene::{
    SceneMaterialTableProgramSlot, material_scene_product_wgsl, material_scene_table_product_wgsl,
};
