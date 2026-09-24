use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::scene_commands::{
    create_child_with_history_from_origin, delete_entities_with_history_from_origin,
    delete_entity_with_history_from_origin, duplicate_entity_subtree_with_history_from_origin,
    rename_entity_with_history_from_origin, reparent_entity_with_history_from_origin,
};
use crate::editor_panels::{
    OutlinerPanelCommand, OutlinerPanelCommandResult, OutlinerPanelPresenter,
};
use crate::editor_runtime::select_entity_from_outliner;
use editor_core::EditorMutationError;

pub fn dispatch_outliner_command(
    app: &mut RunenwerkEditorApp,
    command: OutlinerPanelCommand,
) -> Result<OutlinerPanelCommandResult, EditorMutationError> {
    match command {
        OutlinerPanelCommand::SelectEntity { entity } => {
            select_entity_from_outliner(app.runtime_mut(), entity)?;
        }
        OutlinerPanelCommand::CreateChild {
            parent,
            display_name,
        } => {
            create_child_with_history_from_origin(
                app.runtime_mut(),
                parent,
                display_name,
                editor_core::ChangeOrigin::OutlinerPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
        }
        OutlinerPanelCommand::RenameEntity {
            entity,
            new_display_name,
        } => {
            rename_entity_with_history_from_origin(
                app.runtime_mut(),
                entity,
                new_display_name,
                editor_core::ChangeOrigin::OutlinerPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
        }
        OutlinerPanelCommand::DuplicateSubtree {
            source,
            new_parent,
            name_suffix,
        } => {
            duplicate_entity_subtree_with_history_from_origin(
                app.runtime_mut(),
                source,
                new_parent,
                name_suffix,
                editor_core::ChangeOrigin::OutlinerPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
        }
        OutlinerPanelCommand::ReparentEntity { entity, new_parent } => {
            app.runtime().validate_reparent(entity, new_parent)?;

            reparent_entity_with_history_from_origin(
                app.runtime_mut(),
                entity,
                new_parent,
                editor_core::ChangeOrigin::OutlinerPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
        }
        OutlinerPanelCommand::DeleteEntity { entity } => {
            delete_entity_with_history_from_origin(
                app.runtime_mut(),
                entity,
                editor_core::ChangeOrigin::OutlinerPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
        }
        OutlinerPanelCommand::DeleteEntities { entities } => {
            delete_entities_with_history_from_origin(
                app.runtime_mut(),
                entities,
                editor_core::ChangeOrigin::OutlinerPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
        }
    }

    Ok(OutlinerPanelCommandResult {
        state: OutlinerPanelPresenter::build_state(app.runtime()),
    })
}
