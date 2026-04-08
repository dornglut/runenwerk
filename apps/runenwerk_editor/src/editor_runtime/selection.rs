use editor_core::{ComponentTypeId, EntityId, SelectionTarget};
use editor_inspector::{
    InspectTarget, resolve_all_inspect_targets, resolve_primary_inspect_target,
};

use crate::editor_runtime::RunenwerkEditorRuntime;

/// File: apps/runenwerk_editor/src/editor_runtime/selection.rs
/// Method: select_single_entity
pub fn select_single_entity(
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

/// File: apps/runenwerk_editor/src/editor_runtime/selection.rs
/// Method: select_single_component
pub fn select_single_component(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
    component_type: ComponentTypeId,
) -> Result<(), &'static str> {
    if runtime.ids().resolve_entity(entity).is_none() {
        return Err("editor entity is not registered");
    }

    if !runtime.entity_has_component(entity, component_type) {
        return Err("entity does not have the requested component");
    }

    runtime
        .session_mut()
        .select_single(SelectionTarget::Component {
            entity,
            component_type,
        });

    Ok(())
}

/// File: apps/runenwerk_editor/src/editor_runtime/selection.rs
/// Method: clear_selection
pub fn clear_selection(runtime: &mut RunenwerkEditorRuntime) {
    runtime.session_mut().clear_selection();
}

/// File: apps/runenwerk_editor/src/editor_runtime/selection.rs
/// Method: primary_selected_entity
pub fn primary_selected_entity(runtime: &RunenwerkEditorRuntime) -> Option<EntityId> {
    match runtime.session().selection().primary() {
        Some(SelectionTarget::Entity(entity)) => Some(*entity),
        Some(SelectionTarget::Component { entity, .. }) => Some(*entity),
        _ => None,
    }
}

/// File: apps/runenwerk_editor/src/editor_runtime/selection.rs
/// Method: resolve_primary_inspect_target_from_runtime
pub fn resolve_primary_inspect_target_from_runtime(
    runtime: &RunenwerkEditorRuntime,
) -> Option<InspectTarget> {
    resolve_primary_inspect_target(runtime.session().selection())
}

/// File: apps/runenwerk_editor/src/editor_runtime/selection.rs
/// Method: resolve_all_inspect_targets_from_runtime
pub fn resolve_all_inspect_targets_from_runtime(
    runtime: &RunenwerkEditorRuntime,
) -> Vec<InspectTarget> {
    resolve_all_inspect_targets(runtime.session().selection())
}

/// File: apps/runenwerk_editor/src/editor_runtime/selection.rs
/// Method: sync_selection_after_scene_change
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
        runtime.session_mut().clear_selection();
    }
}
