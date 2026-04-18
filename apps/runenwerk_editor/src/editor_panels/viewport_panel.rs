use editor_core::EntityId;
use editor_viewport::{ViewportHitResult, ViewportHitTarget};
use editor_core::EditorMutationError;

use crate::editor_runtime::{
    RunenwerkEditorRuntime, clear_selection_with_origin, select_single_component_with_origin,
    select_single_entity_with_origin,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportPanelState {
    pub selected_entity: Option<EntityId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewportPanelCommand {
    SelectEntity { entity: EntityId },
    SelectFromHit { hit: ViewportHitResult },
    ClearSelection,
}

pub struct ViewportPanelPresenter;

impl ViewportPanelPresenter {
    pub fn build_state(runtime: &RunenwerkEditorRuntime) -> ViewportPanelState {
        ViewportPanelState {
            selected_entity: runtime.selected_entity(),
        }
    }

    pub fn dispatch(
        runtime: &mut RunenwerkEditorRuntime,
        command: ViewportPanelCommand,
    ) -> Result<ViewportPanelState, EditorMutationError> {
        match command {
            ViewportPanelCommand::SelectEntity { entity } => {
                select_single_entity_with_origin(
                    runtime,
                    entity,
                    editor_core::ChangeOrigin::ViewportInteraction,
                )?;
            }
            ViewportPanelCommand::SelectFromHit { hit } => {
                apply_viewport_hit_selection(runtime, &hit)?;
            }
            ViewportPanelCommand::ClearSelection => {
                clear_selection_with_origin(
                    runtime,
                    editor_core::ChangeOrigin::ViewportInteraction,
                );
            }
        }

        Ok(Self::build_state(runtime))
    }
}

fn apply_viewport_hit_selection(
    runtime: &mut RunenwerkEditorRuntime,
    hit: &ViewportHitResult,
) -> Result<(), EditorMutationError> {
    match &hit.target {
        ViewportHitTarget::Entity(entity) => {
            select_single_entity_with_origin(
                runtime,
                *entity,
                editor_core::ChangeOrigin::ViewportInteraction,
            )?;
        }
        ViewportHitTarget::ComponentHandle {
            entity,
            component_type,
        } => {
            select_single_component_with_origin(
                runtime,
                *entity,
                *component_type,
                editor_core::ChangeOrigin::ViewportInteraction,
            )?;
        }
        ViewportHitTarget::GizmoAxis(_) | ViewportHitTarget::Grid | ViewportHitTarget::None => {
            clear_selection_with_origin(runtime, editor_core::ChangeOrigin::ViewportInteraction);
        }
    }

    Ok(())
}
