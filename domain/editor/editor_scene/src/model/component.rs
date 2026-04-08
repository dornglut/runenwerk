//! File: domain/editor/editor_scene/src/model/component.rs
//! Purpose: Component authoring targets, descriptors, and snapshots.

use editor_core::{ComponentTypeId, EntityId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneComponentDescriptor {
    pub entity: EntityId,
    pub component_type: ComponentTypeId,
    pub display_name: String,
}

impl SceneComponentDescriptor {
    /// File: domain/editor/editor_scene/src/model/component.rs
    /// Method: new
    pub fn new(
        entity: EntityId,
        component_type: ComponentTypeId,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            entity,
            component_type,
            display_name: display_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneComponentSnapshot {
    pub entity: EntityId,
    pub component_type: ComponentTypeId,
    pub display_name: String,
}

impl SceneComponentSnapshot {
    /// File: domain/editor/editor_scene/src/model/component.rs
    /// Method: new
    pub fn new(
        entity: EntityId,
        component_type: ComponentTypeId,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            entity,
            component_type,
            display_name: display_name.into(),
        }
    }
}
