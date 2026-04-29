use editor_core::{ComponentTypeId, EditorMutationError, EntityId};
use editor_inspector::{InspectorEditValue, InspectorPath};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::scene_commands::execute_intent_with_history_from_origin;
use crate::editor_panels::{
    InspectorPanelCommand, InspectorPanelCommandResult, InspectorPanelPresenter,
};
use crate::editor_runtime::{RunenwerkEditorRuntime, select_single_component_with_origin};

pub fn dispatch_inspector_command(
    app: &mut RunenwerkEditorApp,
    command: InspectorPanelCommand,
) -> Result<InspectorPanelCommandResult, EditorMutationError> {
    match command {
        InspectorPanelCommand::SelectComponent {
            entity,
            component_type,
        } => {
            select_single_component_with_origin(
                app.runtime_mut(),
                entity,
                component_type,
                editor_core::ChangeOrigin::InspectorPanel,
            )?;
            app.inspector_ui_state_mut().clear_draft();
            app.inspector_ui_state_mut().clear_focus();
        }
        InspectorPanelCommand::AddComponentToEntity {
            entity,
            component_type,
        } => {
            execute_intent_with_history_from_origin(
                app.runtime_mut(),
                "Add Component",
                editor_scene::SceneCommandIntent::AddComponent {
                    entity,
                    component_type,
                },
                editor_core::ChangeOrigin::InspectorPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
            app.inspector_ui_state_mut().clear_draft();
            app.inspector_ui_state_mut().clear_focus();
        }
        InspectorPanelCommand::RemoveComponentFromEntity {
            entity,
            component_type,
        } => {
            execute_intent_with_history_from_origin(
                app.runtime_mut(),
                "Remove Component",
                editor_scene::SceneCommandIntent::RemoveComponent {
                    entity,
                    component_type,
                },
                editor_core::ChangeOrigin::InspectorPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
            app.inspector_ui_state_mut().clear_draft();
            app.inspector_ui_state_mut().clear_focus();
        }
        InspectorPanelCommand::EditComponentField {
            entity,
            component_type,
            path,
            value,
        } => {
            commit_component_field_edit(app.runtime_mut(), entity, component_type, path, value)?;
            app.inspector_ui_state_mut().clear_draft();
            app.inspector_ui_state_mut().clear_focus();
        }
        InspectorPanelCommand::BeginEditComponentField {
            entity,
            component_type,
            path,
            value,
            text,
        } => {
            app.inspector_ui_state_mut().begin_field_edit(
                entity,
                component_type,
                path,
                value,
                text,
            );
        }
        InspectorPanelCommand::UpdateDraftComponentField { value } => {
            app.inspector_ui_state_mut().update_field_draft(value)?;
        }
        InspectorPanelCommand::UpdateDraftComponentFieldText { text } => {
            app.inspector_ui_state_mut().update_field_draft_text(text)?;
        }
        InspectorPanelCommand::CommitDraftComponentField => {
            let draft = app.inspector_ui_state_mut().take_active_draft().ok_or(
                EditorMutationError::inspector_rejected("no active inspector field draft"),
            )?;

            commit_component_field_edit(
                app.runtime_mut(),
                draft.entity,
                draft.component_type,
                draft.path,
                draft.value,
            )?;
        }
        InspectorPanelCommand::CancelDraftComponentField => {
            app.inspector_ui_state_mut().cancel_field_draft();
        }
        InspectorPanelCommand::ToggleSectionExpanded { key } => {
            app.inspector_ui_state_mut().toggle_expanded(key);
        }
    }

    Ok(InspectorPanelCommandResult {
        view_model: InspectorPanelPresenter::build_view_model(
            app.runtime(),
            app.inspector_ui_state(),
        ),
    })
}

fn commit_component_field_edit(
    runtime: &mut RunenwerkEditorRuntime,
    entity: EntityId,
    component_type: ComponentTypeId,
    path: InspectorPath,
    value: InspectorEditValue,
) -> Result<(), EditorMutationError> {
    execute_intent_with_history_from_origin(
        runtime,
        "Edit Component Field",
        editor_scene::SceneCommandIntent::EditComponentField {
            entity,
            component_type,
            path,
            value,
        },
        editor_core::ChangeOrigin::InspectorPanel,
    )
    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))
}
