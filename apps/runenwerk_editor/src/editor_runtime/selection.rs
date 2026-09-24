use editor_core::{ComponentTypeId, EditorMutationError, EntityId};
use editor_inspector::InspectTarget;
use editor_scene::{SceneSelectionAddress, SceneSelectionTarget};

use crate::editor_runtime::RunenwerkEditorRuntime;

pub fn select_single_entity(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
) -> Result<(), EditorMutationError> {
    select_single_entity_with_origin(runtime, entity, editor_core::ChangeOrigin::Runtime)
}

pub fn select_single_entity_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
    origin: editor_core::ChangeOrigin,
) -> Result<(), EditorMutationError> {
    if runtime.ids().resolve_entity(entity).is_none() {
        return Err(EditorMutationError::session_rejected(
            "editor entity is not registered",
        ));
    }

    runtime.set_selection_single_with_origin(
        SceneSelectionAddress::entity(runtime.scene_selection().scope(), entity),
        origin,
    );

    Ok(())
}

pub fn select_single_component(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
    component_type: ComponentTypeId,
) -> Result<(), EditorMutationError> {
    select_single_component_with_origin(
        runtime,
        entity,
        component_type,
        editor_core::ChangeOrigin::Runtime,
    )
}

pub fn select_single_component_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
    component_type: ComponentTypeId,
    origin: editor_core::ChangeOrigin,
) -> Result<(), EditorMutationError> {
    if runtime.ids().resolve_entity(entity).is_none() {
        return Err(EditorMutationError::session_rejected(
            "editor entity is not registered",
        ));
    }

    if !runtime.entity_has_component(entity, component_type) {
        return Err(EditorMutationError::session_rejected(
            "entity does not have the requested component",
        ));
    }

    runtime.set_selection_single_with_origin(
        SceneSelectionAddress::component(runtime.scene_selection().scope(), entity, component_type),
        origin,
    );

    Ok(())
}

pub fn clear_selection(runtime: &mut RunenwerkEditorRuntime) {
    clear_selection_with_origin(runtime, editor_core::ChangeOrigin::Runtime);
}

pub fn clear_selection_with_origin(
    runtime: &mut RunenwerkEditorRuntime,
    origin: editor_core::ChangeOrigin,
) {
    let _ = runtime.clear_selection_with_origin(origin);
}

pub fn primary_selected_entity(runtime: &RunenwerkEditorRuntime) -> Option<EntityId> {
    runtime
        .scene_selection()
        .primary()
        .map(|address| address.entity_id())
}

pub fn resolve_primary_inspect_target_from_runtime(
    runtime: &RunenwerkEditorRuntime,
) -> Option<InspectTarget> {
    runtime
        .scene_selection()
        .primary()
        .map(scene_selection_to_inspect_target)
}

pub fn resolve_all_inspect_targets_from_runtime(
    runtime: &RunenwerkEditorRuntime,
) -> Vec<InspectTarget> {
    runtime
        .scene_selection()
        .iter()
        .map(scene_selection_to_inspect_target)
        .collect()
}

pub fn sync_selection_after_scene_change(runtime: &mut RunenwerkEditorRuntime) {
    let should_clear = runtime
        .scene_selection()
        .primary()
        .is_some_and(|address| runtime.validate_scene_selection_address(address).is_err());

    if should_clear {
        clear_selection_with_origin(runtime, editor_core::ChangeOrigin::Runtime);
    }
}

fn scene_selection_to_inspect_target(address: &SceneSelectionAddress) -> InspectTarget {
    match address.target() {
        SceneSelectionTarget::Entity(entity) => InspectTarget::Entity(*entity),
        SceneSelectionTarget::Component {
            entity,
            component_type,
        } => InspectTarget::Component {
            entity: *entity,
            component_type: *component_type,
        },
    }
}
