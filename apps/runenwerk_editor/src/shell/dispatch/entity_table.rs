//! File: apps/runenwerk_editor/src/shell/dispatch/entity_table.rs
//! Purpose: Entity-table surface command dispatch.

use editor_core::EditorMutationError;
use editor_shell::{
    EntityTableDomainMutation, EntityTableSessionMutation, StructuralCommandTarget, ToolSurfaceKind,
};
use ui_surface::{
    ObservationFrame, RatificationAdapter, RatificationDispatchError, RatificationOutcome,
    SessionScopeHandle, SurfaceCapability, SurfaceCapabilitySet, SurfaceIntent, SurfaceIntentKind,
    SurfacePresentationModel, ratify_surface_intent,
};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::EntityTablePanelPresenter;
use crate::editor_runtime::select_single_entity_with_origin;
use crate::shell::RunenwerkEditorShellState;
use crate::shell::dispatch::{
    resolve_legacy_surface_command_contract, surface_capability_label, tool_surface_kind_label,
};

pub(crate) fn dispatch_session_mutation(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: EntityTableSessionMutation,
) -> Result<(), EditorMutationError> {
    let Some(surface_contract) =
        resolve_legacy_surface_command_contract(shell_state, target, ToolSurfaceKind::EntityTable)
    else {
        app.append_console_line(
            "[entity_table] session mutation ignored (missing structural tool-surface target)"
                .to_string(),
        );
        return Ok(());
    };
    if surface_contract.tool_surface_kind != ToolSurfaceKind::EntityTable {
        app.append_console_line(format!(
            "[entity_table] session mutation ignored (surface-kind mismatch): expected=entity_table actual={}",
            tool_surface_kind_label(surface_contract.tool_surface_kind),
        ));
        return Ok(());
    }
    for required_capability in [SurfaceCapability::Observe, SurfaceCapability::Interact] {
        if !surface_contract.capabilities.allows(required_capability) {
            app.append_console_line(format!(
                "[entity_table] session mutation ignored (missing capability): capability={}",
                surface_capability_label(required_capability),
            ));
            return Ok(());
        }
    }
    let Some(surface_id) = target.active_tool_surface else {
        app.append_console_line(
            "[entity_table] session mutation ignored (missing tool-surface session target)"
                .to_string(),
        );
        return Ok(());
    };
    let state = &mut app
        .surface_sessions_mut()
        .session_mut(surface_id)
        .entity_table_ui_state;
    match mutation {
        EntityTableSessionMutation::AppendSearchText { text } => {
            state.append_search_text(&text);
        }
        EntityTableSessionMutation::BackspaceSearch => {
            state.backspace_search_query();
        }
        EntityTableSessionMutation::ClearSearch => {
            state.clear_search_query();
        }
        EntityTableSessionMutation::SetSelectedOnly { selected_only } => {
            state.set_selected_only(selected_only);
        }
        EntityTableSessionMutation::SetHierarchyFilter { filter } => {
            state.set_hierarchy_filter(filter);
        }
        EntityTableSessionMutation::SetComponentFilter { filter } => {
            state.set_component_filter(filter);
        }
        EntityTableSessionMutation::ToggleSort { sort_key } => {
            state.toggle_sort(sort_key);
        }
    }
    Ok(())
}

pub(crate) fn dispatch_domain_mutation(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    mutation: EntityTableDomainMutation,
) -> Result<(), EditorMutationError> {
    match mutation {
        EntityTableDomainMutation::SelectRow { entities } => {
            let Some(entity) = entities.first().copied() else {
                app.append_console_line("[entity_table] selection ignored (empty row)".to_string());
                return Ok(());
            };
            let Some(surface_contract) = resolve_legacy_surface_command_contract(
                shell_state,
                target,
                ToolSurfaceKind::EntityTable,
            ) else {
                app.append_console_line(
                    "[entity_table] selection ignored (missing structural tool-surface target)"
                        .to_string(),
                );
                return Ok(());
            };
            if surface_contract.tool_surface_kind != ToolSurfaceKind::EntityTable {
                app.append_console_line(format!(
                    "[entity_table] selection ignored (surface-kind mismatch): expected=entity_table actual={}",
                    tool_surface_kind_label(surface_contract.tool_surface_kind),
                ));
                return Ok(());
            }
            for required_capability in [SurfaceCapability::Observe, SurfaceCapability::Interact] {
                if !surface_contract.capabilities.allows(required_capability) {
                    app.append_console_line(format!(
                        "[entity_table] selection ignored (missing capability): entity={} capability={}",
                        entity.0,
                        surface_capability_label(required_capability),
                    ));
                    return Ok(());
                }
            }

            let Some(surface_id) = target.active_tool_surface else {
                app.append_console_line(
                    "[entity_table] selection ignored (missing tool-surface session target)"
                        .to_string(),
                );
                return Ok(());
            };
            let session = app.surface_sessions().session_or_default(surface_id);
            let table_state = EntityTablePanelPresenter::build_state(
                app.runtime(),
                &session.entity_table_ui_state,
            );
            let presentation_model = build_entity_table_surface_presentation_model(&table_state);
            if !presentation_model.is_primary_selectable(entity.0) {
                app.append_console_line(format!(
                    "[entity_table] selection ignored (unavailable): entity={}",
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
            let mut ratification_adapter =
                EntityTableSelectionRatificationAdapter::new(app, surface_contract.capabilities);
            match ratify_surface_intent(&mut ratification_adapter, intent) {
                Ok(RatificationOutcome::Applied) | Ok(RatificationOutcome::Ignored) => {}
                Err(RatificationDispatchError::MissingCapability(capability)) => {
                    app.append_console_line(format!(
                        "[entity_table] selection ignored (missing capability): entity={} capability={}",
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
struct EntityTableObservationFrameAdapter {
    selected_entity_id: Option<u64>,
}

impl ObservationFrame<u64> for EntityTableObservationFrameAdapter {
    fn selected_primary_item(&self) -> Option<u64> {
        self.selected_entity_id
    }

    fn is_item_available(&self, _item_id: u64) -> bool {
        true
    }
}

fn build_entity_table_surface_presentation_model(
    table_state: &crate::editor_panels::EntityTablePanelState,
) -> SurfacePresentationModel<u64> {
    let adapter = EntityTableObservationFrameAdapter {
        selected_entity_id: table_state
            .rows
            .iter()
            .find(|row| row.is_selected)
            .map(|row| row.entity.0),
    };
    SurfacePresentationModel::from_observation_frame(
        &adapter,
        table_state.rows.iter().map(|row| row.entity.0),
    )
}

struct EntityTableSelectionRatificationAdapter<'a> {
    app: &'a mut RunenwerkEditorApp,
    capabilities: SurfaceCapabilitySet,
}

impl<'a> EntityTableSelectionRatificationAdapter<'a> {
    fn new(app: &'a mut RunenwerkEditorApp, capabilities: SurfaceCapabilitySet) -> Self {
        Self { app, capabilities }
    }
}

impl RatificationAdapter for EntityTableSelectionRatificationAdapter<'_> {
    type Error = EditorMutationError;

    fn has_capability(&self, capability: SurfaceCapability) -> bool {
        self.capabilities.allows(capability)
    }

    fn ratify_intent(&mut self, intent: SurfaceIntent) -> Result<RatificationOutcome, Self::Error> {
        let entity_id = match intent.kind {
            SurfaceIntentKind::SelectEntity { entity_id } => entity_id,
            _ => return Ok(RatificationOutcome::Ignored),
        };
        select_single_entity_with_origin(
            self.app.runtime_mut(),
            editor_core::EntityId(entity_id),
            editor_core::ChangeOrigin::EntityTablePanel,
        )?;
        Ok(RatificationOutcome::Applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::{PanelInstanceId, TabStackId, ToolSurfaceInstanceId};

    fn target_for_surface_kind(
        shell_state: &RunenwerkEditorShellState,
        kind: ToolSurfaceKind,
    ) -> StructuralCommandTarget {
        let (panel_instance_id, tool_surface_id) = shell_state
            .workspace_state()
            .panels()
            .filter_map(|panel| {
                let tool_surface_id = panel.active_tool_surface?;
                let surface = shell_state
                    .workspace_state()
                    .tool_surface(tool_surface_id)?;
                editor_shell::stable_key_for_tool_surface_kind(kind)
                    .as_ref()
                    .is_some_and(|key| surface.stable_surface_key() == key)
                    .then_some((panel.id, tool_surface_id))
            })
            .next()
            .expect("default shell state should contain requested surface kind");

        StructuralCommandTarget {
            panel_instance_id,
            active_tool_surface: Some(tool_surface_id),
            tab_stack_id: TabStackId::try_from_raw(1).unwrap(),
        }
    }

    #[test]
    fn entity_table_session_mutation_ignores_wrong_surface_kind() {
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let console_target = target_for_surface_kind(&shell_state, ToolSurfaceKind::Console);
        let console_surface = console_target
            .active_tool_surface
            .expect("target should carry a concrete surface");

        dispatch_session_mutation(
            &mut app,
            Some(&shell_state),
            console_target,
            EntityTableSessionMutation::AppendSearchText {
                text: "wrong surface".to_string(),
            },
        )
        .expect("wrong surface kind should fail closed without mutation error");

        assert!(
            app.surface_sessions().session(console_surface).is_none(),
            "entity-table session mutation must not create or mutate a console surface session",
        );
    }

    #[test]
    fn entity_table_session_mutation_updates_entity_table_surface_only() {
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let entity_table_target =
            target_for_surface_kind(&shell_state, ToolSurfaceKind::EntityTable);
        let entity_table_surface = entity_table_target
            .active_tool_surface
            .expect("target should carry a concrete surface");

        dispatch_session_mutation(
            &mut app,
            Some(&shell_state),
            entity_table_target,
            EntityTableSessionMutation::AppendSearchText {
                text: "owned surface".to_string(),
            },
        )
        .expect("entity table session mutation should dispatch");

        assert_eq!(
            app.surface_sessions()
                .session(entity_table_surface)
                .expect("entity table session should be created")
                .entity_table_ui_state
                .search_query(),
            "owned surface",
        );
    }

    #[test]
    fn entity_table_session_mutation_requires_structural_surface_target() {
        let mut app = RunenwerkEditorApp::new();

        dispatch_session_mutation(
            &mut app,
            None,
            StructuralCommandTarget {
                panel_instance_id: PanelInstanceId::try_from_raw(1).unwrap(),
                active_tool_surface: None,
                tab_stack_id: TabStackId::try_from_raw(1).unwrap(),
            },
            EntityTableSessionMutation::AppendSearchText {
                text: "missing target".to_string(),
            },
        )
        .expect("missing structural target should fail closed without mutation error");

        assert!(
            app.surface_sessions()
                .session(ToolSurfaceInstanceId::try_from_raw(1).unwrap())
                .is_none(),
            "missing structural target must not create a fallback surface session",
        );
    }
}
