//! File: domain/editor/editor_scene/src/hierarchy.rs
//! Purpose: Scene hierarchy model for outliner/tree authoring.

use editor_core::EntityId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HierarchyNode {
	pub entity: EntityId,
	pub parent: Option<EntityId>,
	pub children: Vec<EntityId>,
}

impl HierarchyNode {
	pub fn new(entity: EntityId) -> Self {
		Self {
			entity,
			parent: None,
			children: Vec::new(),
		}
	}

	pub fn with_parent(mut self, parent: Option<EntityId>) -> Self {
		self.parent = parent;
		self
	}

	pub fn with_children(mut self, children: Vec<EntityId>) -> Self {
		self.children = children;
		self
	}
}