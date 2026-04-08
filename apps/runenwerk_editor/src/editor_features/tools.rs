use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::ToolAction;
use crate::editor_features::scene_commands::execute_intent_with_history;
use crate::editor_runtime::{
    TransformToolKind, clear_selection, commit_translation_preview_into_local_transform,
    select_single_component, select_single_entity,
};

pub fn dispatch_tool_action(
    app: &mut RunenwerkEditorApp,
    action: ToolAction,
) -> Result<(), &'static str> {
    match action {
        ToolAction::SelectSingle(target) => match target {
            editor_core::SelectionTarget::Entity(entity) => {
                select_single_entity(app.runtime_mut(), entity)?;
            }
            editor_core::SelectionTarget::Component {
                entity,
                component_type,
            } => {
                select_single_component(app.runtime_mut(), entity, component_type)?;
            }
            _ => return Err("unsupported selection target for tool action"),
        },
        ToolAction::ClearSelection => {
            clear_selection(app.runtime_mut());
        }
        ToolAction::Scene(intent) => {
            execute_intent_with_history(app.runtime_mut(), "Tool Scene Action", intent)?;
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
                .ok_or("cannot begin preview without a primary selection")?;

            app.tool_runtime_state_mut()
                .begin_preview(selection, TransformToolKind::Translate)?;
        }
        ToolAction::UpdatePreview => {
            app.tool_runtime_state_mut().update_preview()?;
        }
        ToolAction::CommitPreview => {
            let preview = app
                .tool_runtime_state_mut()
                .commit_preview()
                .ok_or("no active preview session")?;

            commit_translation_preview_into_local_transform(
                app.runtime_mut(),
                preview.entity,
                preview.translation_delta,
            )?;
        }
        ToolAction::CancelPreview => {
            let _ = app
                .tool_runtime_state_mut()
                .cancel_preview()
                .ok_or("no active preview session")?;
        }
    }

    Ok(())
}

pub fn dispatch_tool_actions(
    app: &mut RunenwerkEditorApp,
    actions: impl IntoIterator<Item = ToolAction>,
) -> Result<(), &'static str> {
    for action in actions {
        dispatch_tool_action(app, action)?;
    }

    Ok(())
}
