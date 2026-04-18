use editor_core::{ComponentTypeId, EditorMutationError, EntityId, SelectionTarget};
use editor_inspector::{
    InspectTarget, resolve_all_inspect_targets, resolve_primary_inspect_target,
};

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

    runtime.set_selection_single_with_origin(SelectionTarget::Entity(entity), origin);

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
        SelectionTarget::Component {
            entity,
            component_type,
        },
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
    match runtime.session().selection().primary() {
        Some(SelectionTarget::Entity(entity)) => Some(*entity),
        Some(SelectionTarget::Component { entity, .. }) => Some(*entity),
        _ => None,
    }
}

pub fn resolve_primary_inspect_target_from_runtime(
    runtime: &RunenwerkEditorRuntime,
) -> Option<InspectTarget> {
    resolve_primary_inspect_target(runtime.session().selection())
}

pub fn resolve_all_inspect_targets_from_runtime(
    runtime: &RunenwerkEditorRuntime,
) -> Vec<InspectTarget> {
    resolve_all_inspect_targets(runtime.session().selection())
}

pub fn sync_selection_after_scene_change(runtime: &mut RunenwerkEditorRuntime) {
    let should_clear = match runtime.session().selection().primary() {
        Some(SelectionTarget::Entity(entity)) => runtime.ids().resolve_entity(*entity).is_none(),
        Some(SelectionTarget::Component {
            entity,
            component_type,
        }) => {
            runtime.ids().resolve_entity(*entity).is_none()
                || !runtime.entity_has_component(*entity, *component_type)
        }
        _ => false,
    };

    if should_clear {
        clear_selection_with_origin(runtime, editor_core::ChangeOrigin::Runtime);
    }
}
