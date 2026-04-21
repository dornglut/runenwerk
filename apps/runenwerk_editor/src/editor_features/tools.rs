use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::ToolAction;
use crate::editor_features::scene_commands::execute_intent_with_history_from_origin;
use crate::editor_runtime::{
    TransformToolKind, clear_selection_with_origin,
    commit_translation_preview_into_local_transform, select_single_component_with_origin,
    select_single_entity_with_origin,
};
use editor_core::EditorMutationError;

pub fn dispatch_tool_action(
    app: &mut RunenwerkEditorApp,
    action: ToolAction,
) -> Result<(), EditorMutationError> {
    match action {
        ToolAction::SelectSingle(target) => match target {
            editor_core::SelectionTarget::Entity(entity) => {
                select_single_entity_with_origin(
                    app.runtime_mut(),
                    entity,
                    editor_core::ChangeOrigin::ToolInteraction,
                )?;
            }
            editor_core::SelectionTarget::Component {
                entity,
                component_type,
            } => {
                select_single_component_with_origin(
                    app.runtime_mut(),
                    entity,
                    component_type,
                    editor_core::ChangeOrigin::ToolInteraction,
                )?;
            }
            _ => {
                return Err(EditorMutationError::session_rejected(
                    "unsupported selection target for tool action",
                ));
            }
        },
        ToolAction::ClearSelection => {
            clear_selection_with_origin(
                app.runtime_mut(),
                editor_core::ChangeOrigin::ToolInteraction,
            );
        }
        ToolAction::Scene(intent) => {
            execute_intent_with_history_from_origin(
                app.runtime_mut(),
                "Tool Scene Action",
                intent,
                editor_core::ChangeOrigin::ToolInteraction,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
        }
        ToolAction::HoverEntity(entity) => {
            app.tool_runtime_state_mut().set_hovered_entity(entity);
        }
        ToolAction::BeginPreview => {
            let selection = app
                .runtime()
                .session()
                .selection()
                .primary()
                .cloned()
                .ok_or(EditorMutationError::session_rejected(
                    "cannot begin preview without a primary selection",
                ))?;

            app.tool_runtime_state_mut()
                .begin_preview(selection, TransformToolKind::Translate)?;
        }
        ToolAction::UpdatePreview => {
            app.tool_runtime_state_mut().update_preview()?;
        }
        ToolAction::CommitPreview => {
            let preview = app.tool_runtime_state_mut().commit_preview().ok_or(
                EditorMutationError::session_rejected("no active preview session"),
            )?;

            commit_translation_preview_into_local_transform(
                app.runtime_mut(),
                preview.entity,
                preview.translation_delta,
            )?;
        }
        ToolAction::CancelPreview => {
            let _ = app.tool_runtime_state_mut().cancel_preview().ok_or(
                EditorMutationError::session_rejected("no active preview session"),
            )?;
        }
    }

    Ok(())
}

pub fn dispatch_tool_actions(
    app: &mut RunenwerkEditorApp,
    actions: impl IntoIterator<Item = ToolAction>,
) -> Result<(), EditorMutationError> {
    for action in actions {
        dispatch_tool_action(app, action)?;
    }

    Ok(())
}
