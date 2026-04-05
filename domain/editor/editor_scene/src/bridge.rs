//! File: domain/editor/editor_scene/src/bridge.rs
//! Purpose: Convert scene authoring intents into executable editor commands.

use editor_core::CommandId;

use crate::{SceneCommandIntent, SceneEditorCommand};

pub fn scene_intent_label(intent: &SceneCommandIntent) -> String {
	match intent {
		SceneCommandIntent::CreateEntity { .. } => "Create Entity".to_string(),
		SceneCommandIntent::DeleteEntity { .. } => "Delete Entity".to_string(),
		SceneCommandIntent::DuplicateEntity { .. } => "Duplicate Entity".to_string(),
		SceneCommandIntent::ReparentEntity { .. } => "Reparent Entity".to_string(),
		SceneCommandIntent::AddComponent { .. } => "Add Component".to_string(),
		SceneCommandIntent::RemoveComponent { .. } => "Remove Component".to_string(),
	}
}

pub fn scene_intent_to_command(
	id: CommandId,
	intent: SceneCommandIntent,
) -> SceneEditorCommand {
	let label = scene_intent_label(&intent);
	SceneEditorCommand::new(id, label, intent)
}