use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::ToolAction;
use crate::editor_features::scene_commands::execute_intent_with_history_from_origin;
use crate::editor_runtime::{
    TransformToolKind, clear_selection_with_origin, commit_transform_preview_into_local_transform,
};
use editor_core::EditorMutationError;
use editor_scene::SceneSelectionTarget;

pub fn dispatch_tool_action(
    app: &mut RunenwerkEditorApp,
    action: ToolAction,
) -> Result<(), EditorMutationError> {
    match action {
        ToolAction::SelectSingle(address) => {
            app.runtime().validate_scene_selection_address(&address)?;
            match address.target() {
                SceneSelectionTarget::Entity(_) | SceneSelectionTarget::Component { .. } => {
                    app.runtime_mut().set_selection_single_with_origin(
                        address,
                        editor_core::ChangeOrigin::ToolInteraction,
                    );
                }
            }
        }
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
            begin_preview(app, TransformToolKind::Translate)?;
        }
        ToolAction::BeginTransformPreview(tool) => {
            begin_preview(app, tool)?;
        }
        ToolAction::UpdatePreview => {
            app.tool_runtime_state_mut().update_preview()?;
        }
        ToolAction::CommitPreview => {
            let preview = app.tool_runtime_state_mut().commit_preview().ok_or(
                EditorMutationError::session_rejected("no active preview session"),
            )?;

            commit_transform_preview_into_local_transform(app.runtime_mut(), &preview)?;
        }
        ToolAction::CancelPreview => {
            let _ = app.tool_runtime_state_mut().cancel_preview().ok_or(
                EditorMutationError::session_rejected("no active preview session"),
            )?;
        }
    }

    Ok(())
}

fn begin_preview(
    app: &mut RunenwerkEditorApp,
    tool: TransformToolKind,
) -> Result<(), EditorMutationError> {
    let selection = app.runtime().scene_selection().primary().cloned().ok_or(
        EditorMutationError::session_rejected("cannot begin preview without a primary selection"),
    )?;

    app.tool_runtime_state_mut()
        .begin_preview(selection, tool)?;
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
