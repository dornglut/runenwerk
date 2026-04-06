use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::scene_commands::execute_intent_with_history;
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
			execute_intent_with_history(
				app.runtime_mut(),
				"Rename Entity",
				editor_scene::SceneCommandIntent::RenameEntity {
					entity,
					new_display_name,
				},
			)?;
		}
		OutlinerPanelCommand::ReparentEntity { entity, new_parent } => {
			app.runtime().validate_reparent(entity, new_parent)?;

			execute_intent_with_history(
				app.runtime_mut(),
				"Reparent Entity",
				editor_scene::SceneCommandIntent::ReparentEntity { entity, new_parent },
			)?;
		}
		OutlinerPanelCommand::DeleteEntity { entity } => {
			execute_intent_with_history(
				app.runtime_mut(),
				"Delete Entity",
				editor_scene::SceneCommandIntent::DeleteEntity { entity },
			)?;
		}
	}

	Ok(OutlinerPanelCommandResult {
		state: OutlinerPanelPresenter::build_state(app.runtime()),
	})
}