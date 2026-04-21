use editor_core::EntityId;
use scene::Vec3Value;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::ToolAction;
use crate::editor_tools_state::TranslateAxis;
use editor_core::EditorMutationError;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewportToolCommand {
    BeginTranslateDrag {
        entity: EntityId,
    },
    UpdateTranslateDrag {
        delta: Vec3Value,
    },
    BeginTranslateAxisDrag {
        entity: EntityId,
        axis: TranslateAxis,
    },
    UpdateTranslateAxisDrag {
        amount: f32,
    },
    CommitTranslateDrag,
    CancelTranslateDrag,
}

pub struct ViewportToolController;

impl ViewportToolController {
    pub fn dispatch(
        app: &mut RunenwerkEditorApp,
        command: ViewportToolCommand,
    ) -> Result<(), EditorMutationError> {
        match command {
            ViewportToolCommand::BeginTranslateDrag { entity } => {
                begin_translate_preview(app, entity)?;
            }
            ViewportToolCommand::UpdateTranslateDrag { delta } => {
                app.update_translation_preview(delta)?;
                app.dispatch_tool_action(ToolAction::UpdatePreview)?;
            }
            ViewportToolCommand::BeginTranslateAxisDrag { entity, axis } => {
                begin_translate_preview(app, entity)?;
                app.tool_runtime_state_mut()
                    .set_translate_axis(Some(axis))?;
            }
            ViewportToolCommand::UpdateTranslateAxisDrag { amount } => {
                let axis = app.tool_runtime_state().translate_axis().ok_or(
                    EditorMutationError::session_rejected("no active translate axis"),
                )?;

                let delta = axis_delta(axis, amount);
                app.update_translation_preview(delta)?;
                app.dispatch_tool_action(ToolAction::UpdatePreview)?;
            }
            ViewportToolCommand::CommitTranslateDrag => {
                app.dispatch_tool_action(ToolAction::CommitPreview)?;
            }
            ViewportToolCommand::CancelTranslateDrag => {
                app.dispatch_tool_action(ToolAction::CancelPreview)?;
            }
        }

        Ok(())
    }
}

fn begin_translate_preview(
    app: &mut RunenwerkEditorApp,
    entity: EntityId,
) -> Result<(), EditorMutationError> {
    app.dispatch_tool_action(ToolAction::SelectSingle(
        editor_core::SelectionTarget::Entity(entity),
    ))?;
    app.dispatch_tool_action(ToolAction::BeginPreview)?;
    Ok(())
}

fn axis_delta(axis: TranslateAxis, amount: f32) -> Vec3Value {
    match axis {
        TranslateAxis::X => Vec3Value::new(amount, 0.0, 0.0),
        TranslateAxis::Y => Vec3Value::new(0.0, amount, 0.0),
        TranslateAxis::Z => Vec3Value::new(0.0, 0.0, amount),
    }
}
