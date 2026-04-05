use editor_core::{EntityId, SelectionTarget};
use editor_inspector::InspectTarget;
use editor_scene::SceneCommandIntent;

use crate::editor_runtime::RunenwerkEditorRuntime;

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/actions.rs
/// Method: select_entity_from_outliner
pub fn select_entity_from_outliner(
	runtime: &mut RunenwerkEditorRuntime,
	entity: EntityId,
) -> Result<(), &'static str> {
	if runtime.ids().resolve_entity(entity).is_none() {
		return Err("editor entity is not registered");
	}

	runtime
		.session_mut()
		.select_single(SelectionTarget::Entity(entity));

	Ok(())
}

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/actions.rs
/// Method: clear_outliner_selection
pub fn clear_outliner_selection(runtime: &mut RunenwerkEditorRuntime) {
	runtime.session_mut().clear_selection();
}

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/actions.rs
/// Method: rename_entity_from_outliner
pub fn rename_entity_from_outliner(
	runtime: &mut RunenwerkEditorRuntime,
	entity: EntityId,
	new_display_name: impl Into<String>,
) -> Result<(), &'static str> {
	let command_id = runtime.allocate_command_id();
	let transaction_id = runtime.allocate_transaction_id();

	let result = crate::editor_runtime::execute_scene_command_and_push_history(
		runtime,
		editor_scene::scene_intent_to_command(
			command_id,
			SceneCommandIntent::RenameEntity {
				entity,
				new_display_name: new_display_name.into(),
			},
		),
		"Rename Entity",
		transaction_id,
	)?;

	if result.is_none() {
		return Ok(());
	}

	Ok(())
}

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/actions.rs
/// Method: reparent_entity_from_outliner
pub fn reparent_entity_from_outliner(
	runtime: &mut RunenwerkEditorRuntime,
	entity: EntityId,
	new_parent: Option<EntityId>,
) -> Result<(), &'static str> {
	runtime.validate_reparent(entity, new_parent)?;

	let command_id = runtime.allocate_command_id();
	let transaction_id = runtime.allocate_transaction_id();

	let result = crate::editor_runtime::execute_scene_command_and_push_history(
		runtime,
		editor_scene::scene_intent_to_command(
			command_id,
			SceneCommandIntent::ReparentEntity { entity, new_parent },
		),
		"Reparent Entity",
		transaction_id,
	)?;

	if result.is_none() {
		return Ok(());
	}

	Ok(())
}

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/actions.rs
/// Method: delete_entity_from_outliner
pub fn delete_entity_from_outliner(
	runtime: &mut RunenwerkEditorRuntime,
	entity: EntityId,
) -> Result<(), &'static str> {
	let command_id = runtime.allocate_command_id();
	let transaction_id = runtime.allocate_transaction_id();

	let result = crate::editor_runtime::execute_scene_command_and_push_history(
		runtime,
		editor_scene::scene_intent_to_command(
			command_id,
			SceneCommandIntent::DeleteEntity { entity },
		),
		"Delete Entity",
		transaction_id,
	)?;

	if result.is_none() {
		return Ok(());
	}

	Ok(())
}

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/actions.rs
/// Method: selected_outliner_entity
pub fn selected_outliner_entity(
	runtime: &RunenwerkEditorRuntime,
) -> Option<EntityId> {
	match runtime.session().selection().primary() {
		Some(SelectionTarget::Entity(entity)) => Some(*entity),
		_ => None,
	}
}

/// File: apps/runenwerk_editor/src/editor_runtime/outliner/actions.rs
/// Method: selected_outliner_inspect_target
pub fn selected_outliner_inspect_target(
	runtime: &RunenwerkEditorRuntime,
) -> Option<InspectTarget> {
	crate::editor_runtime::resolve_primary_inspect_target_from_runtime(runtime)
}