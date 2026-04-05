//! File: domain/editor/editor_tools/src/intent.rs

use editor_core::EntityId;
use editor_scene::SceneCommandIntent;

#[derive(Debug, Clone, PartialEq)]
pub enum ToolIntent {
	Scene(SceneCommandIntent),
	SelectEntity(EntityId),
	ClearSelection,
	SetHoverEntity(Option<EntityId>),
	BeginPreview,
	UpdatePreview,
	CommitPreview,
	CancelPreview,
}