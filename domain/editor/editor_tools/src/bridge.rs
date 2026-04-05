//! File: domain/editor/editor_tools/src/bridge.rs
//! Purpose: Convert tool results into editor-facing actions.

use editor_core::SelectionTarget;

use crate::{ToolIntent, ToolResult};

#[derive(Debug, Clone, PartialEq)]
pub enum ToolAction {
	SelectSingle(SelectionTarget),
	ClearSelection,
	Scene(editor_scene::SceneCommandIntent),
	HoverEntity(Option<editor_core::EntityId>),
	BeginPreview,
	UpdatePreview,
	CommitPreview,
	CancelPreview,
}

pub fn collect_tool_actions(result: ToolResult) -> Vec<ToolAction> {
	result
		.intents
		.into_iter()
		.map(|intent| match intent {
			ToolIntent::Scene(scene) => ToolAction::Scene(scene),
			ToolIntent::SelectEntity(entity) => {
				ToolAction::SelectSingle(SelectionTarget::Entity(entity))
			}
			ToolIntent::ClearSelection => ToolAction::ClearSelection,
			ToolIntent::SetHoverEntity(entity) => ToolAction::HoverEntity(entity),
			ToolIntent::BeginPreview => ToolAction::BeginPreview,
			ToolIntent::UpdatePreview => ToolAction::UpdatePreview,
			ToolIntent::CommitPreview => ToolAction::CommitPreview,
			ToolIntent::CancelPreview => ToolAction::CancelPreview,
		})
		.collect()
}