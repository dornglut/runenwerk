//! File: domain/editor/editor_scene/src/model/entity.rs
//! Purpose: Scene entity authoring targets, descriptors, and snapshots.

use editor_core::EntityId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneEntityDescriptor {
    pub id: EntityId,
    pub display_name: String,
    pub parent: Option<EntityId>,
}

impl SceneEntityDescriptor {
    /// File: domain/editor/editor_scene/src/model/entity.rs
    /// Method: new
    pub fn new(id: EntityId, display_name: impl Into<String>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            parent: None,
        }
    }

    /// File: domain/editor/editor_scene/src/model/entity.rs
    /// Method: with_parent
    pub fn with_parent(mut self, parent: Option<EntityId>) -> Self {
        self.parent = parent;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneEntitySnapshot {
    pub id: EntityId,
    pub display_name: String,
    pub parent: Option<EntityId>,
}

impl SceneEntitySnapshot {
    /// File: domain/editor/editor_scene/src/model/entity.rs
    /// Method: new
    pub fn new(id: EntityId, display_name: impl Into<String>, parent: Option<EntityId>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            parent,
        }
    }
}
