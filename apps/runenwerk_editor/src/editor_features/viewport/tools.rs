use editor_core::EntityId;
use scene::Vec3Value;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::ToolAction;
use crate::editor_runtime::TransformToolKind;
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
    BeginRotateAxisDrag {
        entity: EntityId,
        axis: TranslateAxis,
    },
    UpdateRotateAxisDrag {
        radians: f32,
    },
    BeginScaleAxisDrag {
        entity: EntityId,
        axis: TranslateAxis,
    },
    UpdateScaleAxisDrag {
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
                begin_transform_preview(app, entity, TransformToolKind::Translate)?;
                app.tool_runtime_state_mut()
                    .set_translate_axis(Some(axis))?;
            }
            ViewportToolCommand::UpdateTranslateAxisDrag { amount } => {
                let axis = app.tool_runtime_state().translate_axis().ok_or(
                    EditorMutationError::session_rejected("no active translate axis"),
                )?;

                let delta = axis_delta(axis, snap_amount(app, amount, SnapChannel::Translate));
                app.update_translation_preview(delta)?;
                app.dispatch_tool_action(ToolAction::UpdatePreview)?;
            }
            ViewportToolCommand::BeginRotateAxisDrag { entity, axis } => {
                begin_transform_preview(app, entity, TransformToolKind::Rotate)?;
                app.tool_runtime_state_mut()
                    .set_translate_axis(Some(axis))?;
            }
            ViewportToolCommand::UpdateRotateAxisDrag { radians } => {
                let axis = app.tool_runtime_state().translate_axis().ok_or(
                    EditorMutationError::session_rejected("no active rotate axis"),
                )?;

                let delta = axis_delta(axis, snap_amount(app, radians, SnapChannel::Rotate));
                app.update_rotation_preview(delta)?;
                app.dispatch_tool_action(ToolAction::UpdatePreview)?;
            }
            ViewportToolCommand::BeginScaleAxisDrag { entity, axis } => {
                begin_transform_preview(app, entity, TransformToolKind::Scale)?;
                app.tool_runtime_state_mut()
                    .set_translate_axis(Some(axis))?;
            }
            ViewportToolCommand::UpdateScaleAxisDrag { amount } => {
                let axis = app.tool_runtime_state().translate_axis().ok_or(
                    EditorMutationError::session_rejected("no active scale axis"),
                )?;

                let delta = axis_delta(axis, snap_amount(app, amount, SnapChannel::Scale));
                app.update_scale_preview(delta)?;
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
    begin_transform_preview(app, entity, TransformToolKind::Translate)
}

fn begin_transform_preview(
    app: &mut RunenwerkEditorApp,
    entity: EntityId,
    tool: TransformToolKind,
) -> Result<(), EditorMutationError> {
    app.dispatch_tool_action(ToolAction::SelectSingle(
        editor_core::SelectionTarget::Entity(entity),
    ))?;
    app.dispatch_tool_action(ToolAction::BeginTransformPreview(tool))?;
    Ok(())
}

fn axis_delta(axis: TranslateAxis, amount: f32) -> Vec3Value {
    match axis {
        TranslateAxis::X => Vec3Value::new(amount, 0.0, 0.0),
        TranslateAxis::Y => Vec3Value::new(0.0, amount, 0.0),
        TranslateAxis::Z => Vec3Value::new(0.0, 0.0, amount),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnapChannel {
    Translate,
    Rotate,
    Scale,
}

fn snap_amount(app: &RunenwerkEditorApp, amount: f32, channel: SnapChannel) -> f32 {
    let snap = app.tool_runtime_state().snap_settings();
    if !snap.enabled {
        return amount;
    }
    let step = match channel {
        SnapChannel::Translate => snap.translate_step,
        SnapChannel::Rotate => snap.rotate_degrees.to_radians(),
        SnapChannel::Scale => snap.scale_step,
    };
    if step <= f32::EPSILON {
        return amount;
    }
    (amount / step).round() * step
}
