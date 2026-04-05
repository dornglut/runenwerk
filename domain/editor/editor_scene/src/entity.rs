//! File: domain/editor/editor_scene/src/entity.rs
//! Purpose: Scene entity authoring targets and descriptors.

use editor_core::EntityId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneEntityDescriptor {
	pub id: EntityId,
	pub display_name: String,
	pub parent: Option<EntityId>,
}

impl SceneEntityDescriptor {
	pub fn new(id: EntityId, display_name: impl Into<String>) -> Self {
		Self {
			id,
			display_name: display_name.into(),
			parent: None,
		}
	}

	pub fn with_parent(mut self, parent: Option<EntityId>) -> Self {
		self.parent = parent;
		self
	}
}