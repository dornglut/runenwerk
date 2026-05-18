//! File: apps/runenwerk_editor/src/shell/dispatch/outliner.rs
//! Purpose: Outliner surface command dispatch.

use editor_core::EditorMutationError;
use editor_shell::{OutlinerDomainMutation, StructuralCommandTarget, ToolSurfaceKind};
use ui_surface::{
    ObservationFrame, RatificationAdapter, RatificationDispatchError, RatificationOutcome,
    SessionScopeHandle, SurfaceCapability, SurfaceCapabilitySet, SurfaceIntent, SurfaceIntentKind,
    SurfacePresentationModel, ratify_surface_intent,
};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::OutlinerPanelCommand;
use crate::shell::RunenwerkEditorShellState;
use crate::shell::dispatch::{
    resolve_legacy_surface_command_contract, surface_capability_label, tool_surface_kind_label,
};

pub(crate) fn dispatch_domain_mutation(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: OutlinerDomainMutation,
) -> Result<(), EditorMutationError> {
    match mutation {
        OutlinerDomainMutation::SelectEntity { entity } => {
            let Some(surface_contract) = resolve_legacy_surface_command_contract(
                shell_state,
                target,
                ToolSurfaceKind::Outliner,
            ) else {
                app.append_console_line(
                    "[outliner] entity selection ignored (missing structural tool-surface target)"
                        .to_string(),
                );
                return Ok(());
            };
            if surface_contract.tool_surface_kind != ToolSurfaceKind::Outliner {
                app.append_console_line(format!(
                    "[outliner] entity selection ignored (surface-kind mismatch): expected=outliner actual={}",
                    tool_surface_kind_label(surface_contract.tool_surface_kind),
                ));
                return Ok(());
            }
            for required_capability in [SurfaceCapability::Observe, SurfaceCapability::Interact] {
                if !surface_contract.capabilities.allows(required_capability) {
                    app.append_console_line(format!(
                        "[outliner] entity selection ignored (missing capability): entity={} capability={}",
                        entity.0,
                        surface_capability_label(required_capability),
                    ));
                    return Ok(());
                }
            }

            let outliner_state = app.outliner_state();
            let presentation_model = build_outliner_surface_presentation_model(&outliner_state);
            if app.debug_logs_enabled() {
                app.append_console_line(format!(
                    "[surface.flow] observation outliner selected={:?}",
                    outliner_state.selected_entity.map(|value| value.0),
                ));
                app.append_console_line(format!(
                    "[surface.flow] presentation outliner selectable={:?}",
                    presentation_model
                        .selectable_primary_items()
                        .collect::<Vec<_>>(),
                ));
            }
            if !presentation_model.is_primary_selectable(entity.0) {
                app.append_console_line(format!(
                    "[outliner] entity selection ignored (unavailable): entity={}",
                    entity.0,
                ));
                return Ok(());
            }

            let _session_scope = SessionScopeHandle::new(
                surface_contract.surface_instance_id,
                target.panel_instance_id.raw(),
                surface_contract.retention_class,
            );
            let intent =
                SurfaceIntent::select_entity(surface_contract.surface_instance_id, entity.0);
            if app.debug_logs_enabled() {
                app.append_console_line(format!(
                    "[surface.flow] intent outliner surface_instance={} kind=SelectEntity entity_id={}",
                    intent.surface_instance_id.raw(),
                    entity.0,
                ));
            }
            let mut ratification_adapter =
                OutlinerEntitySelectionRatificationAdapter::new(app, surface_contract.capabilities);
            match ratify_surface_intent(&mut ratification_adapter, intent) {
                Ok(RatificationOutcome::Applied) => {
                    if app.debug_logs_enabled() {
                        app.append_console_line(
                            "[surface.flow] ratification outliner outcome=applied".to_string(),
                        );
                    }
                }
                Ok(RatificationOutcome::Ignored) => {
                    if app.debug_logs_enabled() {
                        app.append_console_line(
                            "[surface.flow] ratification outliner outcome=ignored".to_string(),
                        );
                    }
                }
                Err(RatificationDispatchError::MissingCapability(capability)) => {
                    app.append_console_line(format!(
                        "[outliner] entity selection ignored (missing capability): entity={} capability={}",
                        entity.0,
                        surface_capability_label(capability),
                    ));
                }
                Err(RatificationDispatchError::Adapter(error)) => return Err(error),
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct OutlinerObservationFrameAdapter {
    selected_entity_id: Option<u64>,
}

impl ObservationFrame<u64> for OutlinerObservationFrameAdapter {
    fn selected_primary_item(&self) -> Option<u64> {
        self.selected_entity_id
    }

    fn is_item_available(&self, _item_id: u64) -> bool {
        true
    }
}

fn build_outliner_surface_presentation_model(
    outliner_state: &crate::editor_panels::OutlinerPanelState,
) -> SurfacePresentationModel<u64> {
    let adapter = OutlinerObservationFrameAdapter {
        selected_entity_id: outliner_state.selected_entity.map(|value| value.0),
    };
    SurfacePresentationModel::from_observation_frame(
        &adapter,
        outliner_state.rows.iter().map(|row| row.entity.0),
    )
}

struct OutlinerEntitySelectionRatificationAdapter<'a> {
    app: &'a mut RunenwerkEditorApp,
    capabilities: SurfaceCapabilitySet,
}

impl<'a> OutlinerEntitySelectionRatificationAdapter<'a> {
    fn new(app: &'a mut RunenwerkEditorApp, capabilities: SurfaceCapabilitySet) -> Self {
        Self { app, capabilities }
    }
}

impl RatificationAdapter for OutlinerEntitySelectionRatificationAdapter<'_> {
    type Error = EditorMutationError;

    fn has_capability(&self, capability: SurfaceCapability) -> bool {
        self.capabilities.allows(capability)
    }

    fn ratify_intent(&mut self, intent: SurfaceIntent) -> Result<RatificationOutcome, Self::Error> {
        let entity_id = match intent.kind {
            SurfaceIntentKind::SelectEntity { entity_id } => entity_id,
            _ => return Ok(RatificationOutcome::Ignored),
        };
        self.app
            .dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
                entity: editor_core::EntityId(entity_id),
            })?;
        Ok(RatificationOutcome::Applied)
    }
}
