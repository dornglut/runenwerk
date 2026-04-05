use editor_core::EntityId;

use crate::editor_runtime::EditorRuntimeIdRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HierarchyItem {
	pub entity: EntityId,
	pub display_name: String,
	pub parent: Option<EntityId>,
	pub children: Vec<HierarchyItem>,
}

impl HierarchyItem {
	/// File: apps/runenwerk_editor/src/editor_runtime/scene_state/hierarchy.rs
	/// Method: new
	pub fn new(
		entity: EntityId,
		display_name: impl Into<String>,
		parent: Option<EntityId>,
		children: Vec<HierarchyItem>,
	) -> Self {
		Self {
			entity,
			display_name: display_name.into(),
			parent,
			children,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HierarchySnapshot {
	pub roots: Vec<HierarchyItem>,
}

/// File: apps/runenwerk_editor/src/editor_runtime/scene_state/hierarchy.rs
/// Method: root_entities
pub fn root_entities(ids: &EditorRuntimeIdRegistry) -> Vec<EntityId> {
	let mut roots = ids.children_of(None);
	roots.sort();
	roots
}

/// File: apps/runenwerk_editor/src/editor_runtime/scene_state/hierarchy.rs
/// Method: children_of
pub fn children_of(
	ids: &EditorRuntimeIdRegistry,
	parent: EntityId,
) -> Vec<EntityId> {
	let mut children = ids.children_of(Some(parent));
	children.sort();
	children
}

/// File: apps/runenwerk_editor/src/editor_runtime/scene_state/hierarchy.rs
/// Method: build_hierarchy_snapshot
pub fn build_hierarchy_snapshot(ids: &EditorRuntimeIdRegistry) -> HierarchySnapshot {
	let roots = root_entities(ids)
		.into_iter()
		.filter_map(|entity| build_item(ids, entity))
		.collect::<Vec<_>>();

	HierarchySnapshot { roots }
}

/// File: apps/runenwerk_editor/src/editor_runtime/scene_state/hierarchy.rs
/// Method: validate_reparent
pub fn validate_reparent(
	ids: &EditorRuntimeIdRegistry,
	entity: EntityId,
	new_parent: Option<EntityId>,
) -> Result<(), &'static str> {
	if ids.resolve_entity(entity).is_none() {
		return Err("editor entity is not registered");
	}

	let Some(parent) = new_parent else {
		return Ok(());
	};

	if ids.resolve_entity(parent).is_none() {
		return Err("new parent entity is not registered");
	}

	if parent == entity {
		return Err("entity cannot be parented to itself");
	}

	if ids.would_create_cycle(entity, parent) {
		return Err("reparent would create a hierarchy cycle");
	}

	Ok(())
}

/// File: apps/runenwerk_editor/src/editor_runtime/scene_state/hierarchy.rs
/// Method: build_item
fn build_item(
	ids: &EditorRuntimeIdRegistry,
	entity: EntityId,
) -> Option<HierarchyItem> {
	let snapshot = ids.entity_snapshot(entity)?;
	let children = children_of(ids, entity)
		.into_iter()
		.filter_map(|child| build_item(ids, child))
		.collect::<Vec<_>>();

	Some(HierarchyItem::new(
		snapshot.id,
		snapshot.display_name,
		snapshot.parent,
		children,
	))
}