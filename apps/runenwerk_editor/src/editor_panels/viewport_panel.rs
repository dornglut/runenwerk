use editor_core::EntityId;
use editor_viewport::{ViewportHitResult, ViewportHitTarget};

use crate::editor_runtime::{
    RunenwerkEditorRuntime, clear_selection, select_single_component, select_single_entity,
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
    ) -> Result<ViewportPanelState, &'static str> {
        match command {
            ViewportPanelCommand::SelectEntity { entity } => {
                select_single_entity(runtime, entity)?;
            }
            ViewportPanelCommand::SelectFromHit { hit } => {
                apply_viewport_hit_selection(runtime, &hit)?;
            }
            ViewportPanelCommand::ClearSelection => {
                clear_selection(runtime);
            }
        }

        Ok(Self::build_state(runtime))
    }
}

fn apply_viewport_hit_selection(
    runtime: &mut RunenwerkEditorRuntime,
    hit: &ViewportHitResult,
) -> Result<(), &'static str> {
    match &hit.target {
        ViewportHitTarget::Entity(entity) => {
            select_single_entity(runtime, *entity)?;
        }
        ViewportHitTarget::ComponentHandle {
            entity,
            component_type,
        } => {
            select_single_component(runtime, *entity, *component_type)?;
        }
        ViewportHitTarget::GizmoAxis(_) | ViewportHitTarget::Grid | ViewportHitTarget::None => {
            clear_selection(runtime);
        }
    }

    Ok(())
}
