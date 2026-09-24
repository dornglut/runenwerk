use editor_core::EntityId;
use editor_viewport::{ViewportHitResult, ViewportHitTarget};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::{ViewportToolCommand, ViewportToolController};
use crate::editor_panels::ViewportPanelCommand;
use crate::editor_runtime::TransformToolKind;
use crate::editor_tools_state::TranslateAxis;
use editor_core::EditorMutationError;
use editor_shell::ViewportToolKind;
use ui_composition::MountedUnitId;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewportInteractionCommand {
    PointerDown { hit: ViewportHitResult },
    PointerDragAxis { amount: f32 },
    PointerUp,
    CancelInteraction,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewportInteractionState {
    active_entity: Option<EntityId>,
    active_axis: Option<TranslateAxis>,
    active_tool: Option<TransformToolKind>,
    drag_in_progress: bool,
}

impl ViewportInteractionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_entity(&self) -> Option<EntityId> {
        self.active_entity
    }

    pub fn active_axis(&self) -> Option<TranslateAxis> {
        self.active_axis
    }

    pub fn drag_in_progress(&self) -> bool {
        self.drag_in_progress
    }

    pub fn clear(&mut self) {
        self.active_entity = None;
        self.active_axis = None;
        self.active_tool = None;
        self.drag_in_progress = false;
    }
}

pub struct ViewportInteractionController;

impl ViewportInteractionController {
    pub fn dispatch(
        app: &mut RunenwerkEditorApp,
        mounted_unit_id: MountedUnitId,
        state: &mut ViewportInteractionState,
        command: ViewportInteractionCommand,
    ) -> Result<(), EditorMutationError> {
        match command {
            ViewportInteractionCommand::PointerDown { hit } => match hit.target {
                ViewportHitTarget::Entity(entity) => {
                    app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity { entity })?;
                    state.clear();
                }
                ViewportHitTarget::ComponentHandle { entity, .. } => {
                    app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity { entity })?;
                    state.clear();
                }
                ViewportHitTarget::GizmoAxis(axis) => {
                    let entity = app.runtime().selected_entity().ok_or(
                        EditorMutationError::session_rejected(
                            "cannot start gizmo drag without selected entity",
                        ),
                    )?;

                    let translate_axis = map_gizmo_axis(axis)?;
                    let tool = active_transform_tool(app, mounted_unit_id)?;
                    let begin = match tool {
                        TransformToolKind::Translate => {
                            ViewportToolCommand::BeginTranslateAxisDrag {
                                entity,
                                axis: translate_axis,
                            }
                        }
                        TransformToolKind::Rotate => ViewportToolCommand::BeginRotateAxisDrag {
                            entity,
                            axis: translate_axis,
                        },
                        TransformToolKind::Scale => ViewportToolCommand::BeginScaleAxisDrag {
                            entity,
                            axis: translate_axis,
                        },
                    };
                    ViewportToolController::dispatch(app, begin)?;

                    state.active_entity = Some(entity);
                    state.active_axis = Some(translate_axis);
                    state.active_tool = Some(tool);
                    state.drag_in_progress = true;
                }
                ViewportHitTarget::Grid | ViewportHitTarget::None => {
                    app.dispatch_viewport_command(ViewportPanelCommand::ClearSelection)?;
                    state.clear();
                }
            },
            ViewportInteractionCommand::PointerDragAxis { amount } => {
                if !state.drag_in_progress {
                    return Err(EditorMutationError::session_rejected(
                        "no active viewport drag",
                    ));
                }

                let tool = state
                    .active_tool
                    .ok_or(EditorMutationError::session_rejected(
                        "viewport drag has no captured tool",
                    ))?;
                let update = match tool {
                    TransformToolKind::Translate => {
                        ViewportToolCommand::UpdateTranslateAxisDrag { amount }
                    }
                    TransformToolKind::Rotate => {
                        ViewportToolCommand::UpdateRotateAxisDrag { radians: amount }
                    }
                    TransformToolKind::Scale => ViewportToolCommand::UpdateScaleAxisDrag { amount },
                };
                ViewportToolController::dispatch(app, update)?;
            }
            ViewportInteractionCommand::PointerUp => {
                if !state.drag_in_progress {
                    return Ok(());
                }

                ViewportToolController::dispatch(app, ViewportToolCommand::CommitTranslateDrag)?;
                state.clear();
            }
            ViewportInteractionCommand::CancelInteraction => {
                if state.drag_in_progress {
                    ViewportToolController::dispatch(
                        app,
                        ViewportToolCommand::CancelTranslateDrag,
                    )?;
                }
                state.clear();
            }
        }

        Ok(())
    }
}

fn active_transform_tool(
    app: &RunenwerkEditorApp,
    mounted_unit_id: MountedUnitId,
) -> Result<TransformToolKind, EditorMutationError> {
    match app.surface_sessions().viewport_tool(mounted_unit_id) {
        ViewportToolKind::Translate => Ok(TransformToolKind::Translate),
        ViewportToolKind::Rotate => Ok(TransformToolKind::Rotate),
        ViewportToolKind::Scale => Ok(TransformToolKind::Scale),
        ViewportToolKind::Select => Err(EditorMutationError::session_rejected(
            "gizmo axis interaction requires translate, rotate, or scale tool",
        )),
    }
}

fn map_gizmo_axis(axis: &'static str) -> Result<TranslateAxis, EditorMutationError> {
    match axis {
        "x" | "X" => Ok(TranslateAxis::X),
        "y" | "Y" => Ok(TranslateAxis::Y),
        "z" | "Z" => Ok(TranslateAxis::Z),
        _ => Err(EditorMutationError::runtime_rejected(
            "unsupported gizmo axis",
        )),
    }
}
