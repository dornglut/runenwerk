//! File: domain/editor/editor_scene/src/bridge/command_builder.rs
//! Purpose: Convert scene authoring intents into executable commands.

use editor_core::CommandId;

use crate::{SceneCommandIntent, SceneEditorCommand};

/// File: domain/editor/editor_scene/src/bridge/command_builder.rs
/// Method: scene_intent_label
pub fn scene_intent_label(intent: &SceneCommandIntent) -> String {
	match intent {
		SceneCommandIntent::CreateEntity { .. } => "Create Entity".to_string(),
		SceneCommandIntent::DeleteEntity { .. } => "Delete Entity".to_string(),
		SceneCommandIntent::DuplicateEntity { .. } => "Duplicate Entity".to_string(),
		SceneCommandIntent::ReparentEntity { .. } => "Reparent Entity".to_string(),
		SceneCommandIntent::AddComponent { .. } => "Add Component".to_string(),
		SceneCommandIntent::RemoveComponent { .. } => "Remove Component".to_string(),
		SceneCommandIntent::EditComponentField { .. } => "Edit Component Field".to_string(),
		SceneCommandIntent::EditResourceField { .. } => "Edit Resource Field".to_string(),
		SceneCommandIntent::RenameEntity { .. } => "Rename Entity".to_string(),
	}
}

/// File: domain/editor/editor_scene/src/bridge/command_builder.rs
/// Method: scene_intent_to_command
pub fn scene_intent_to_command(
	id: CommandId,
	intent: SceneCommandIntent,
) -> SceneEditorCommand {
	let label = scene_intent_label(&intent);
	SceneEditorCommand::from_intent(id, label, intent)
}