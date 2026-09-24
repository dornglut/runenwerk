use editor_core::EditorMutationError;
use editor_core::EntityId;
use editor_inspector::InspectTarget;

use crate::editor_runtime::{
    RunenwerkEditorRuntime, clear_selection_with_origin, select_single_entity_with_origin,
};

pub fn select_entity_from_outliner(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
) -> Result<(), EditorMutationError> {
    select_single_entity_with_origin(runtime, entity, editor_core::ChangeOrigin::OutlinerPanel)
}

pub fn clear_outliner_selection(runtime: &mut RunenwerkEditorRuntime) {
    clear_selection_with_origin(runtime, editor_core::ChangeOrigin::OutlinerPanel);
}

pub fn selected_outliner_entity(runtime: &RunenwerkEditorRuntime) -> Option<EntityId> {
    runtime
        .scene_selection()
        .primary()
        .and_then(|address| match address.target() {
            editor_scene::SceneSelectionTarget::Entity(entity) => Some(entity),
            editor_scene::SceneSelectionTarget::Component { .. } => None,
        })
        .copied()
}

pub fn selected_outliner_inspect_target(runtime: &RunenwerkEditorRuntime) -> Option<InspectTarget> {
    crate::editor_runtime::resolve_primary_inspect_target_from_runtime(runtime)
}
