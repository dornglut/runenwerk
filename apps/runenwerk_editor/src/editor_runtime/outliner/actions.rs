use editor_core::{EntityId, SelectionTarget};
use editor_inspector::InspectTarget;

use crate::editor_runtime::RunenwerkEditorRuntime;

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

pub fn clear_outliner_selection(runtime: &mut RunenwerkEditorRuntime) {
    runtime.session_mut().clear_selection();
}

pub fn selected_outliner_entity(runtime: &RunenwerkEditorRuntime) -> Option<EntityId> {
    match runtime.session().selection().primary() {
        Some(SelectionTarget::Entity(entity)) => Some(*entity),
        _ => None,
    }
}

pub fn selected_outliner_inspect_target(runtime: &RunenwerkEditorRuntime) -> Option<InspectTarget> {
    crate::editor_runtime::resolve_primary_inspect_target_from_runtime(runtime)
}
