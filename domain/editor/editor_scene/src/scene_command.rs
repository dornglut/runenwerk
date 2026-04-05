//! File: domain/editor/editor_scene/src/scene_command.rs
//! Purpose: Scene authoring command intents.

use editor_core::{ComponentTypeId, EntityId, ResourceTypeId};
use editor_inspector::{InspectorEditValue, InspectorPath};

#[derive(Debug, Clone, PartialEq)]
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
	EditComponentField {
		entity: EntityId,
		component_type: ComponentTypeId,
		path: InspectorPath,
		value: InspectorEditValue,
	},
	EditResourceField {
		resource_type: ResourceTypeId,
		path: InspectorPath,
		value: InspectorEditValue,
	},
	RenameEntity {
		entity: EntityId,
		new_display_name: String,
	},
}