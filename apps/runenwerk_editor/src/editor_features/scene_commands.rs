use editor_core::{ChangeOrigin, GoverningChangeError, RatifiedChange};
use editor_scene::{SceneCommandIntent, SceneEditorCommand};

use crate::editor_runtime::{
    RunenwerkEditorRuntime, ratify_scene_command, ratify_scene_intent, ratify_scene_redo,
    ratify_scene_transaction, ratify_scene_undo,
};

pub fn execute_intent_with_history(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    intent: SceneCommandIntent,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        transaction_label,
        intent,
        ChangeOrigin::Runtime,
    )
}

pub fn execute_intent_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    intent: SceneCommandIntent,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    let result = ratify_scene_intent(runtime, transaction_label, intent, origin)?;

    if result.is_none() {
        return Ok(());
    }

    Ok(())
}

pub fn create_child_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    parent: editor_core::EntityId,
    display_name: impl Into<String>,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Create Child Entity",
        SceneCommandIntent::CreateChildEntity {
            parent,
            display_name: display_name.into(),
        },
        origin,
    )
}

pub fn rename_entity_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    entity: editor_core::EntityId,
    new_display_name: impl Into<String>,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Rename Entity",
        SceneCommandIntent::RenameEntity {
            entity,
            new_display_name: new_display_name.into(),
        },
        origin,
    )
}

pub fn reparent_entity_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    entity: editor_core::EntityId,
    new_parent: Option<editor_core::EntityId>,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Reparent Entity",
        SceneCommandIntent::ReparentEntity { entity, new_parent },
        origin,
    )
}

pub fn duplicate_entity_subtree_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    source: editor_core::EntityId,
    new_parent: Option<editor_core::EntityId>,
    name_suffix: impl Into<String>,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Duplicate Entity Subtree",
        SceneCommandIntent::DuplicateEntitySubtree {
            source,
            new_parent,
            name_suffix: name_suffix.into(),
        },
        origin,
    )
}

pub fn delete_entity_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    entity: editor_core::EntityId,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Delete Entity",
        SceneCommandIntent::DeleteEntity { entity },
        origin,
    )
}

pub fn delete_entities_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    entities: Vec<editor_core::EntityId>,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Delete Entities",
        SceneCommandIntent::DeleteEntities { entities },
        origin,
    )
}

pub fn add_component_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    entity: editor_core::EntityId,
    component_type: editor_core::ComponentTypeId,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Add Component",
        SceneCommandIntent::AddComponent {
            entity,
            component_type,
        },
        origin,
    )
}

pub fn remove_component_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    entity: editor_core::EntityId,
    component_type: editor_core::ComponentTypeId,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Remove Component",
        SceneCommandIntent::RemoveComponent {
            entity,
            component_type,
        },
        origin,
    )
}

pub fn create_sdf_primitive_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    parent: Option<editor_core::EntityId>,
    display_name: impl Into<String>,
    primitive: editor_scene::SdfPrimitiveSpec,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Create SDF Primitive",
        SceneCommandIntent::CreateSdfPrimitive {
            parent,
            display_name: display_name.into(),
            primitive,
        },
        origin,
    )
}

pub fn execute_command_with_history(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    command: SceneEditorCommand,
) -> Result<(), GoverningChangeError> {
    execute_command_with_history_from_origin(
        runtime,
        transaction_label,
        command,
        ChangeOrigin::Runtime,
    )
}

pub fn execute_command_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    command: SceneEditorCommand,
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    let result = ratify_scene_command(runtime, transaction_label, command, origin)?;

    if result.is_none() {
        return Ok(());
    }

    Ok(())
}

pub fn execute_transaction_with_history(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    commands: &mut [SceneEditorCommand],
) -> Result<(), GoverningChangeError> {
    execute_transaction_with_history_from_origin(
        runtime,
        transaction_label,
        commands,
        ChangeOrigin::Runtime,
    )
}

pub fn execute_transaction_with_history_from_origin(
    runtime: &mut RunenwerkEditorRuntime,
    transaction_label: impl Into<String>,
    commands: &mut [SceneEditorCommand],
    origin: ChangeOrigin,
) -> Result<(), GoverningChangeError> {
    let result = ratify_scene_transaction(runtime, transaction_label, commands, origin)?;

    if result.is_none() {
        return Ok(());
    }

    Ok(())
}

pub fn undo_last_scene_change(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    ratify_scene_undo(runtime, origin)
}

pub fn redo_last_scene_change(
    runtime: &mut RunenwerkEditorRuntime,
    origin: ChangeOrigin,
) -> Result<Option<RatifiedChange>, GoverningChangeError> {
    ratify_scene_redo(runtime, origin)
}
