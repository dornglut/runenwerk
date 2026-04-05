//! File: domain/editor/editor_scene/src/component.rs
//! Purpose: Component authoring targets and descriptors.

use editor_core::{ComponentTypeId, EntityId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneComponentDescriptor {
	pub entity: EntityId,
	pub component_type: ComponentTypeId,
	pub display_name: String,
}

impl SceneComponentDescriptor {
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