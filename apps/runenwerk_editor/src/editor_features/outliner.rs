use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::scene_commands::execute_intent_with_history_from_origin;
use crate::editor_panels::{
    OutlinerPanelCommand, OutlinerPanelCommandResult, OutlinerPanelPresenter,
};
use crate::editor_runtime::select_entity_from_outliner;

pub fn dispatch_outliner_command(
    app: &mut RunenwerkEditorApp,
    command: OutlinerPanelCommand,
) -> Result<OutlinerPanelCommandResult, &'static str> {
    match command {
        OutlinerPanelCommand::SelectEntity { entity } => {
            select_entity_from_outliner(app.runtime_mut(), entity)?;
        }
        OutlinerPanelCommand::RenameEntity {
            entity,
            new_display_name,
        } => {
            execute_intent_with_history_from_origin(
                app.runtime_mut(),
                "Rename Entity",
                editor_scene::SceneCommandIntent::RenameEntity {
                    entity,
                    new_display_name,
                },
                editor_core::ChangeOrigin::OutlinerPanel,
            )
            .map_err(editor_core::GoverningChangeError::as_static_str)?;
        }
        OutlinerPanelCommand::ReparentEntity { entity, new_parent } => {
            app.runtime().validate_reparent(entity, new_parent)?;

            execute_intent_with_history_from_origin(
                app.runtime_mut(),
                "Reparent Entity",
                editor_scene::SceneCommandIntent::ReparentEntity { entity, new_parent },
                editor_core::ChangeOrigin::OutlinerPanel,
            )
            .map_err(editor_core::GoverningChangeError::as_static_str)?;
        }
        OutlinerPanelCommand::DeleteEntity { entity } => {
            execute_intent_with_history_from_origin(
                app.runtime_mut(),
                "Delete Entity",
                editor_scene::SceneCommandIntent::DeleteEntity { entity },
                editor_core::ChangeOrigin::OutlinerPanel,
            )
            .map_err(editor_core::GoverningChangeError::as_static_str)?;
        }
    }

    Ok(OutlinerPanelCommandResult {
        state: OutlinerPanelPresenter::build_state(app.runtime()),
    })
}
