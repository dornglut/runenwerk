//! File: domain/editor/editor_scene/src/scene_command.rs
//! Purpose: Scene authoring command intents.

use editor_core::{ComponentTypeId, EntityId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneCommandIntent {
	CreateEntity {
		parent: Option<EntityId>,
		display_name: String,
	},
	DeleteEntity {
		entity: EntityId,
	},
	DuplicateEntity {
		entity: EntityId,
	},
	ReparentEntity {
		entity: EntityId,
		new_parent: Option<EntityId>,
	},
	AddComponent {
		entity: EntityId,
		component_type: ComponentTypeId,
	},
	RemoveComponent {
		entity: EntityId,
		component_type: ComponentTypeId,
	},
}