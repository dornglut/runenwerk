use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::{
    EntityTablePanelCommand, EntityTablePanelCommandResult, EntityTablePanelPresenter,
};
use crate::editor_runtime::select_single_entity_with_origin;
use editor_core::EditorMutationError;

pub fn dispatch_entity_table_command(
    app: &mut RunenwerkEditorApp,
    command: EntityTablePanelCommand,
) -> Result<EntityTablePanelCommandResult, EditorMutationError> {
    match command {
        EntityTablePanelCommand::SelectEntity { entity } => {
            select_single_entity_with_origin(
                app.runtime_mut(),
                entity,
                editor_core::ChangeOrigin::EntityTablePanel,
            )?;
        }
        EntityTablePanelCommand::AppendSearchText { text } => {
            app.entity_table_ui_state_mut().append_search_text(&text);
        }
        EntityTablePanelCommand::BackspaceSearchQuery => {
            app.entity_table_ui_state_mut().backspace_search_query();
        }
        EntityTablePanelCommand::ToggleSort { sort_key } => {
            app.entity_table_ui_state_mut().toggle_sort(sort_key);
        }
    }

    Ok(EntityTablePanelCommandResult {
        state: EntityTablePanelPresenter::build_state(app.runtime(), app.entity_table_ui_state()),
    })
}
