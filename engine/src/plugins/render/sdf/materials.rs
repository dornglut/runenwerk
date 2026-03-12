use crate::plugins::render::domain::{RenderMaterialDescriptor, RenderMaterialId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdfMaterialBinding {
    pub material: RenderMaterialDescriptor,
}

impl Default for SdfMaterialBinding {
    fn default() -> Self {
        Self {
            material: RenderMaterialDescriptor {
                id: RenderMaterialId("sdf.default".to_string()),
                label: "SDF Default".to_string(),
            },
        }
    }
}
