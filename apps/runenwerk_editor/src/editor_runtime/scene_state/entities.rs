use editor_core::EntityId;

use crate::editor_runtime::EditorRuntimeIdRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneEntityView {
	pub id: EntityId,
	pub display_name: String,
	pub parent: Option<EntityId>,
}

impl SceneEntityView {
	/// File: apps/runenwerk_editor/src/editor_runtime/scene_state/entities.rs
	/// Method: new
	pub fn new(
		id: EntityId,
		display_name: impl Into<String>,
		parent: Option<EntityId>,
	) -> Self {
		Self {
			id,
			display_name: display_name.into(),
			parent,
		}
	}
}

/// File: apps/runenwerk_editor/src/editor_runtime/scene_state/entities.rs
/// Method: entity_view
pub fn entity_view(
	ids: &EditorRuntimeIdRegistry,
	entity: EntityId,
) -> Option<SceneEntityView> {
	let snapshot = ids.entity_snapshot(entity)?;
	Some(SceneEntityView::new(
		snapshot.id,
		snapshot.display_name,
		snapshot.parent,
	))
}

/// File: apps/runenwerk_editor/src/editor_runtime/scene_state/entities.rs
/// Method: all_entity_views
pub fn all_entity_views(ids: &EditorRuntimeIdRegistry) -> Vec<SceneEntityView> {
	let mut entities = ids
		.entity_ids()
		.filter_map(|entity| entity_view(ids, entity))
		.collect::<Vec<_>>();

	entities.sort_by(|left, right| {
		left.display_name
			.cmp(&right.display_name)
			.then_with(|| left.id.cmp(&right.id))
	});

	entities
}