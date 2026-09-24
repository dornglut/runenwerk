//! File: domain/editor/editor_scene/src/model/resource.rs
//! Purpose: Resource authoring targets and descriptors.

use editor_core::ResourceTypeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneResourceDescriptor {
    pub resource_type: ResourceTypeId,
    pub display_name: String,
}

impl SceneResourceDescriptor {
    pub fn new(resource_type: ResourceTypeId, display_name: impl Into<String>) -> Self {
        Self {
            resource_type,
            display_name: display_name.into(),
        }
    }
}
