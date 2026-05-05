use std::path::PathBuf;

use editor_core::{ComponentTypeId, EditorMutationError};
use editor_inspector::{InspectorEditValue, InspectorValue};
use editor_shell::{
    FloatingHostBounds, ShellCommand, StructuralCommandTarget, TabDropDestination, ToolSurfaceKind,
    WorkspaceMutation, tool_surface_capability_set, tool_surface_session_retention_class,
};
use editor_viewport::{
    ArtifactObservationFrame, ExpressionProductId, ViewportId, ViewportPresentationState,
};
use ui_surface::{
    ObservationFrame, RatificationAdapter, RatificationDispatchError, RatificationOutcome,
    SessionRetentionClass, SessionScopeHandle, SurfaceCapability, SurfaceCapabilitySet,
    SurfaceInstanceId, SurfaceIntent, SurfaceIntentKind, SurfacePresentationModel,
    ratify_surface_intent,
};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::{
    execute_intent_with_history_from_origin, redo_last_scene_change, undo_last_scene_change,
};
use crate::editor_panels::{
    EntityTablePanelPresenter, InspectorPanelPresenter, InspectorPanelViewModel,
    OutlinerPanelCommand,
};
use crate::editor_runtime::{
    EditorInspectorUiState, bootstrap_mvp_scene_if_empty, is_local_transform_component,
    register_mvp_component_types, select_single_component_with_origin,
    select_single_entity_with_origin,
};
use crate::persistence::{
    default_workspace_layout_path_for_profile, legacy_workspace_layout_path_for_scene,
    load_scene_file_into_runtime_classified, read_retained_change_log, read_workspace_layout,
    retained_change_log_path_for_scene, write_retained_change_log, write_scene_file,
    write_workspace_layout,
};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportPresentationStateResource,
};
use crate::shell::{RunenwerkEditorShellState, SELECT_TOOL_ID, TRANSLATE_TOOL_ID};

const TRANSFORM_STEPPER_INCREMENT: f64 = 0.25;
const DEFAULT_EDITOR_SCENE_PATH: &str = "editor-scenes/default.scene.ron";

pub fn dispatch_shell_command(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    command: ShellCommand,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    current_projection_epoch: Option<u64>,
) -> Result<(), EditorMutationError> {
    if let (Some(command_epoch), Some(expected_epoch)) =
        (command.projection_epoch(), current_projection_epoch)
        && command_epoch != expected_epoch
    {
        return Ok(());
    }

    app.runtime_mut().record_workflow_event(
        editor_core::WorkflowEventKind::ShellCommandDispatched {
            command: shell_command_label(&command),
        },
    );

    match command {
        ShellCommand::ActivateSelectTool => {
            app.runtime_mut().set_active_tool_with_origin(
                Some(SELECT_TOOL_ID),
                editor_core::ChangeOrigin::EditorShell,
            );
        }
        ShellCommand::ActivateTranslateTool => {
            app.runtime_mut().set_active_tool_with_origin(
                Some(TRANSLATE_TOOL_ID),
                editor_core::ChangeOrigin::EditorShell,
            );
        }
        ShellCommand::Undo => {
            if let Some(entry) =
                undo_last_scene_change(app.runtime_mut(), editor_core::ChangeOrigin::EditorShell)
                    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?
            {
                app.append_console_line(format!("[history] undo: {}", entry.transaction.label));
            }
        }
        ShellCommand::Redo => {
            if let Some(entry) =
                redo_last_scene_change(app.runtime_mut(), editor_core::ChangeOrigin::EditorShell)
                    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?
            {
                app.append_console_line(format!("[history] redo: {}", entry.transaction.label));
            }
        }
        ShellCommand::SaveScene => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for save command",
                    ))?;
            save_scene_to_default_path(app, shell_state)?;
        }
        ShellCommand::LoadScene => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for load command",
                    ))?;
            load_scene_from_default_path(app, shell_state)?;
        }
        ShellCommand::ToggleDebugLogs => {
            app.toggle_debug_logs_enabled();
            app.append_console_line(format!(
                "[debug] interaction logs {}",
                if app.debug_logs_enabled() {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
        }
        ShellCommand::SetTabStackActivePanel {
            tab_stack_id,
            panel_instance_id,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            shell_state
                .apply_workspace_mutation(WorkspaceMutation::SetTabStackActivePanel {
                    tab_stack_id,
                    active_panel: Some(panel_instance_id),
                })
                .map_err(|_| EditorMutationError::runtime_rejected("workspace mutation failed"))?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::CommitTabDrop {
            panel_instance_id,
            source_tab_stack_id,
            destination,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for workspace command",
                    ))?;
            apply_tab_drop(
                shell_state,
                panel_instance_id,
                source_tab_stack_id,
                destination,
            )
            .map_err(|_| EditorMutationError::runtime_rejected("workspace tab drop failed"))?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::SwitchPanelToolSurfaceKind {
            panel_instance_id,
            tool_surface_kind,
            projection_epoch: _,
        } => {
            let shell_state =
                shell_state
                    .as_deref_mut()
                    .ok_or(EditorMutationError::runtime_rejected(
                        "missing shell state for surface switch command",
                    ))?;
            shell_state
                .switch_panel_tool_surface_kind(panel_instance_id, tool_surface_kind)
                .map_err(|_| EditorMutationError::runtime_rejected("surface switch failed"))?;
            app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
        }
        ShellCommand::SelectEntityTableEntity {
            entity,
            target,
            projection_epoch: _,
        } => {
            let Some(surface_contract) = resolve_surface_command_contract(
                shell_state.as_deref(),
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
        ShellCommand::AppendEntityTableSearchText {
            text,
            target,
            projection_epoch: _,
        } => {
            if resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::EntityTable,
            )
            .is_some()
            {
                let Some(surface_id) = target.active_tool_surface else {
                    app.append_console_line(
                        "[entity_table] search ignored (missing tool-surface session target)"
                            .to_string(),
                    );
                    return Ok(());
                };
                app.surface_sessions_mut()
                    .session_mut(surface_id)
                    .entity_table_ui_state
                    .append_search_text(&text);
            }
        }
        ShellCommand::BackspaceEntityTableSearch {
            target,
            projection_epoch: _,
        } => {
            if resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::EntityTable,
            )
            .is_some()
            {
                let Some(surface_id) = target.active_tool_surface else {
                    app.append_console_line(
                        "[entity_table] search backspace ignored (missing tool-surface session target)"
                            .to_string(),
                    );
                    return Ok(());
                };
                app.surface_sessions_mut()
                    .session_mut(surface_id)
                    .entity_table_ui_state
                    .backspace_search_query();
            }
        }
        ShellCommand::ToggleEntityTableSort {
            sort_key,
            target,
            projection_epoch: _,
        } => {
            if resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::EntityTable,
            )
            .is_some()
            {
                let Some(surface_id) = target.active_tool_surface else {
                    app.append_console_line(
                        "[entity_table] sort ignored (missing tool-surface session target)"
                            .to_string(),
                    );
                    return Ok(());
                };
                app.surface_sessions_mut()
                    .session_mut(surface_id)
                    .entity_table_ui_state
                    .toggle_sort(sort_key);
            }
        }
        ShellCommand::SelectOutlinerEntity {
            entity,
            target,
            projection_epoch: _,
        } => {
            let Some(surface_contract) = resolve_surface_command_contract(
                shell_state.as_deref(),
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
        ShellCommand::SelectViewportProduct {
            viewport_id,
            product_id,
            target,
            projection_epoch: _,
        } => match (
            viewport_presentations,
            viewport_observations,
            tool_surface_bindings,
        ) {
            (
                Some(viewport_presentations),
                Some(viewport_observations),
                Some(tool_surface_bindings),
            ) => {
                let resolved_binding =
                    match tool_surface_bindings.resolve_command_target(target, viewport_id) {
                        Ok(binding) => binding,
                        Err(error) => {
                            app.append_console_line(format!(
                                "[viewport.binding] product selection ignored: {error}"
                            ));
                            return Ok(());
                        }
                    };
                let resolved_viewport_id = resolved_binding.viewport_id;
                let Some(observation_frame) = viewport_observations.frame_for(resolved_viewport_id)
                else {
                    app.append_console_line(format!(
                        "[viewport] product selection ignored (missing observation frame): viewport={}",
                        resolved_viewport_id.0
                    ));
                    return Ok(());
                };
                let Some(surface_contract) = resolve_surface_command_contract(
                    shell_state.as_deref(),
                    target,
                    ToolSurfaceKind::Viewport,
                ) else {
                    app.append_console_line(
                        "[viewport] product selection ignored (missing structural tool-surface target)"
                            .to_string(),
                    );
                    return Ok(());
                };
                if surface_contract.tool_surface_kind != ToolSurfaceKind::Viewport {
                    app.append_console_line(format!(
                        "[viewport] product selection ignored (surface-kind mismatch): expected=viewport actual={}",
                        tool_surface_kind_label(surface_contract.tool_surface_kind),
                    ));
                    return Ok(());
                }
                let capabilities = surface_contract.capabilities;
                for required_capability in [SurfaceCapability::Observe, SurfaceCapability::Interact]
                {
                    if !capabilities.allows(required_capability) {
                        app.append_console_line(format!(
                            "[viewport] product selection ignored (missing capability): viewport={} product={} capability={}",
                            resolved_viewport_id.0,
                            product_id.0,
                            surface_capability_label(required_capability),
                        ));
                        return Ok(());
                    }
                }

                let presentation_model =
                    build_viewport_surface_presentation_model(observation_frame);
                if app.debug_logs_enabled() {
                    app.append_console_line(format!(
                        "[surface.flow] observation viewport={} selected={:?}",
                        resolved_viewport_id.0,
                        observation_frame
                            .selected_primary_product_id
                            .map(|value| value.0),
                    ));
                    app.append_console_line(format!(
                        "[surface.flow] presentation viewport={} selectable={:?}",
                        resolved_viewport_id.0,
                        presentation_model
                            .selectable_primary_items()
                            .map(|value| value.0)
                            .collect::<Vec<_>>(),
                    ));
                }
                if !presentation_model.is_primary_selectable(product_id) {
                    app.append_console_line(format!(
                        "[viewport] product selection ignored (unavailable): viewport={} product={}",
                        resolved_viewport_id.0, product_id.0
                    ));
                    return Ok(());
                }

                let _session_scope = SessionScopeHandle::new(
                    surface_contract.surface_instance_id,
                    resolved_viewport_id.0,
                    surface_contract.retention_class,
                );
                let mut ratification_adapter = ViewportProductSelectionRatificationAdapter::new(
                    viewport_presentations,
                    resolved_viewport_id,
                    capabilities,
                );
                let intent = SurfaceIntent::select_primary_item(
                    surface_contract.surface_instance_id,
                    product_id.0,
                );
                if app.debug_logs_enabled() {
                    app.append_console_line(format!(
                        "[surface.flow] intent viewport={} surface_instance={} kind=SelectPrimaryItem item_id={}",
                        resolved_viewport_id.0,
                        intent.surface_instance_id.raw(),
                        product_id.0,
                    ));
                }

                match ratify_surface_intent(&mut ratification_adapter, intent) {
                    Ok(RatificationOutcome::Applied) => {
                        if app.debug_logs_enabled() {
                            app.append_console_line(format!(
                                "[surface.flow] ratification viewport={} outcome=applied",
                                resolved_viewport_id.0
                            ));
                        }
                    }
                    Ok(RatificationOutcome::Ignored) => {
                        if app.debug_logs_enabled() {
                            app.append_console_line(format!(
                                "[surface.flow] ratification viewport={} outcome=ignored",
                                resolved_viewport_id.0
                            ));
                        }
                    }
                    Err(RatificationDispatchError::MissingCapability(capability)) => {
                        app.append_console_line(format!(
                            "[viewport] product selection ignored (missing capability): viewport={} product={} capability={}",
                            resolved_viewport_id.0,
                            product_id.0,
                            surface_capability_label(capability),
                        ));
                    }
                    Err(RatificationDispatchError::Adapter(error)) => return Err(error),
                }
            }
            _ => {
                app.append_console_line(
                    "[viewport.binding] product selection ignored (missing runtime binding context)"
                        .to_string(),
                );
            }
        },
        ShellCommand::ToggleViewportDetails {
            target,
            projection_epoch: _,
        } => {
            let Some(surface_contract) = resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::Viewport,
            ) else {
                app.append_console_line(
                    "[viewport] details toggle ignored (missing structural tool-surface target)"
                        .to_string(),
                );
                return Ok(());
            };
            if surface_contract.tool_surface_kind != ToolSurfaceKind::Viewport {
                app.append_console_line(format!(
                    "[viewport] details toggle ignored (surface-kind mismatch): expected=viewport actual={}",
                    tool_surface_kind_label(surface_contract.tool_surface_kind),
                ));
                return Ok(());
            }
            let Some(surface_id) = target.active_tool_surface else {
                app.append_console_line(
                    "[viewport] details toggle ignored (missing tool-surface session target)"
                        .to_string(),
                );
                return Ok(());
            };
            let session = app.surface_sessions_mut().session_mut(surface_id);
            session.viewport_details_visible = !session.viewport_details_visible;
        }
        ShellCommand::ActivateInspectorField {
            index,
            target,
            projection_epoch: _,
        } => {
            let Some(surface_contract) = resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::Inspector,
            ) else {
                app.append_console_line(
                    "[inspector] field activation ignored (missing structural tool-surface target)"
                        .to_string(),
                );
                return Ok(());
            };
            if surface_contract.tool_surface_kind != ToolSurfaceKind::Inspector {
                app.append_console_line(format!(
                    "[inspector] field activation ignored (surface-kind mismatch): expected=inspector actual={}",
                    tool_surface_kind_label(surface_contract.tool_surface_kind),
                ));
                return Ok(());
            }
            for required_capability in [SurfaceCapability::Observe, SurfaceCapability::Interact] {
                if !surface_contract.capabilities.allows(required_capability) {
                    app.append_console_line(format!(
                        "[inspector] field activation ignored (missing capability): index={} capability={}",
                        index,
                        surface_capability_label(required_capability),
                    ));
                    return Ok(());
                }
            }

            let Some(tool_surface_id) = target.active_tool_surface else {
                app.append_console_line(
                    "[inspector] field activation ignored (missing tool-surface session target)"
                        .to_string(),
                );
                return Ok(());
            };
            let mut session = app.surface_sessions().session_or_default(tool_surface_id);
            let inspector_view = InspectorPanelPresenter::build_view_model(
                app.runtime(),
                &session.inspector_ui_state,
            );
            let presentation_model =
                build_inspector_surface_presentation_model(app.runtime(), &inspector_view);
            if app.debug_logs_enabled() {
                app.append_console_line(format!(
                    "[surface.flow] observation inspector selected_field={:?}",
                    presentation_model.selected_primary_item,
                ));
                app.append_console_line(format!(
                    "[surface.flow] presentation inspector selectable={:?}",
                    presentation_model
                        .selectable_primary_items()
                        .collect::<Vec<_>>(),
                ));
            }
            let field_index = index as u64;
            if !presentation_model.is_primary_selectable(field_index) {
                app.append_console_line(format!(
                    "[inspector] field activation ignored (unavailable): index={}",
                    index,
                ));
                return Ok(());
            }

            let _session_scope = SessionScopeHandle::new(
                surface_contract.surface_instance_id,
                target.panel_instance_id.raw(),
                surface_contract.retention_class,
            );
            let intent =
                SurfaceIntent::activate_field(surface_contract.surface_instance_id, field_index);
            if app.debug_logs_enabled() {
                app.append_console_line(format!(
                    "[surface.flow] intent inspector surface_instance={} kind=ActivateField field_index={}",
                    intent.surface_instance_id.raw(),
                    field_index,
                ));
            }
            let mut ratification_adapter = InspectorFieldActivationRatificationAdapter::new(
                app,
                &mut session.inspector_ui_state,
                surface_contract.capabilities,
            );
            match ratify_surface_intent(&mut ratification_adapter, intent) {
                Ok(RatificationOutcome::Applied) => {
                    if app.debug_logs_enabled() {
                        app.append_console_line(
                            "[surface.flow] ratification inspector outcome=applied".to_string(),
                        );
                    }
                }
                Ok(RatificationOutcome::Ignored) => {
                    if app.debug_logs_enabled() {
                        app.append_console_line(
                            "[surface.flow] ratification inspector outcome=ignored".to_string(),
                        );
                    }
                }
                Err(RatificationDispatchError::MissingCapability(capability)) => {
                    app.append_console_line(format!(
                        "[inspector] field activation ignored (missing capability): index={} capability={}",
                        index,
                        surface_capability_label(capability),
                    ));
                }
                Err(RatificationDispatchError::Adapter(error)) => return Err(error),
            }
            *app.surface_sessions_mut().session_mut(tool_surface_id) = session;
        }
        ShellCommand::FocusInspectorField {
            index,
            target,
            projection_epoch: _,
        } => {
            if resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::Inspector,
            )
            .is_some()
            {
                mutate_inspector_surface_session(app, target, |app, state| {
                    focus_inspector_field(app, state, index)
                })?;
                if let Some(state) = shell_state.as_deref_mut() {
                    state
                        .runtime_mut()
                        .set_focused_widget(Some(editor_shell::inspector_field_widget_id(index)));
                }
            }
        }
        ShellCommand::AppendInspectorFieldText {
            index,
            text,
            target,
            projection_epoch: _,
        } => {
            if resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::Inspector,
            )
            .is_some()
            {
                mutate_inspector_surface_session(app, target, |app, state| {
                    append_inspector_field_text(app, state, index, &text)
                })?;
            }
        }
        ShellCommand::BackspaceInspectorFieldText {
            index,
            target,
            projection_epoch: _,
        } => {
            if resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::Inspector,
            )
            .is_some()
            {
                mutate_inspector_surface_session(app, target, |app, state| {
                    backspace_inspector_field_text(app, state, index)
                })?;
            }
        }
        ShellCommand::CommitInspectorFieldText {
            index,
            target,
            projection_epoch: _,
        } => {
            if resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::Inspector,
            )
            .is_some()
            {
                mutate_inspector_surface_session(app, target, |app, state| {
                    commit_inspector_field_text(app, state, index)
                })?;
            }
        }
        ShellCommand::CancelInspectorFieldText {
            index: _,
            target,
            projection_epoch: _,
        } => {
            if resolve_surface_command_contract(
                shell_state.as_deref(),
                target,
                ToolSurfaceKind::Inspector,
            )
            .is_some()
            {
                mutate_inspector_surface_session(app, target, |_app, state| {
                    state.cancel_field_draft();
                    Ok(())
                })?;
            }
        }
        ShellCommand::DispatchSurfaceLocalAction { .. } => {
            return Err(EditorMutationError::session_rejected(
                "surface-local command must be resolved through provider registry before dispatch",
            ));
        }
        ShellCommand::NoOp => {}
    }

    Ok(())
}

struct ArtifactObservationFrameAdapter<'a> {
    frame: &'a ArtifactObservationFrame,
}

impl ObservationFrame<ExpressionProductId> for ArtifactObservationFrameAdapter<'_> {
    fn selected_primary_item(&self) -> Option<ExpressionProductId> {
        self.frame.selected_primary_product_id
    }

    fn is_item_available(&self, item_id: ExpressionProductId) -> bool {
        self.frame
            .availability_by_product
            .get(&item_id)
            .copied()
            .map(|availability| {
                availability == editor_viewport::ProductAvailabilityState::Available
            })
            .unwrap_or(false)
    }
}

fn build_viewport_surface_presentation_model(
    observation_frame: &ArtifactObservationFrame,
) -> SurfacePresentationModel<ExpressionProductId> {
    let adapter = ArtifactObservationFrameAdapter {
        frame: observation_frame,
    };
    SurfacePresentationModel::from_observation_frame(
        &adapter,
        observation_frame.availability_by_product.keys().copied(),
    )
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

fn build_entity_table_surface_presentation_model(
    table_state: &crate::editor_panels::EntityTablePanelState,
) -> SurfacePresentationModel<u64> {
    let adapter = OutlinerObservationFrameAdapter {
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

#[derive(Debug, Clone, Copy)]
struct InspectorObservationFrameAdapter {
    selected_field_index: Option<u64>,
}

impl ObservationFrame<u64> for InspectorObservationFrameAdapter {
    fn selected_primary_item(&self) -> Option<u64> {
        self.selected_field_index
    }

    fn is_item_available(&self, _item_id: u64) -> bool {
        true
    }
}

fn build_inspector_surface_presentation_model(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
    inspector_view: &InspectorPanelViewModel,
) -> SurfacePresentationModel<u64> {
    let selectable_indices: Vec<u64> = match inspector_view {
        InspectorPanelViewModel::Entity {
            components,
            available_component_types,
            ..
        } => (0..components.len() + available_component_types.len())
            .map(|index| index as u64)
            .collect(),
        InspectorPanelViewModel::Component {
            component_type,
            widget_fields,
            ..
        } => widget_fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                next_shell_edit_value(runtime, *component_type, field).map(|_| index as u64)
            })
            .collect(),
        InspectorPanelViewModel::Empty
        | InspectorPanelViewModel::Resource { .. }
        | InspectorPanelViewModel::Unsupported { .. }
        | InspectorPanelViewModel::Error { .. } => Vec::new(),
    };

    let adapter = InspectorObservationFrameAdapter {
        selected_field_index: None,
    };
    SurfacePresentationModel::from_observation_frame(&adapter, selectable_indices)
}

#[derive(Debug, Clone, Copy)]
struct ResolvedSurfaceCommandContract {
    surface_instance_id: SurfaceInstanceId,
    tool_surface_kind: ToolSurfaceKind,
    capabilities: SurfaceCapabilitySet,
    retention_class: SessionRetentionClass,
}

fn resolve_surface_command_contract(
    shell_state: Option<&RunenwerkEditorShellState>,
    target: StructuralCommandTarget,
    fallback_kind: ToolSurfaceKind,
) -> Option<ResolvedSurfaceCommandContract> {
    let tool_surface_id = target.active_tool_surface?;
    let resolved_kind = if let Some(state) = shell_state {
        let surface = state.workspace_state().tool_surface(tool_surface_id)?;
        surface.tool_surface_kind
    } else {
        fallback_kind
    };
    Some(ResolvedSurfaceCommandContract {
        surface_instance_id: SurfaceInstanceId::new(tool_surface_id.raw()),
        tool_surface_kind: resolved_kind,
        capabilities: tool_surface_capability_set(resolved_kind),
        retention_class: tool_surface_session_retention_class(resolved_kind),
    })
}

fn tool_surface_kind_label(kind: ToolSurfaceKind) -> &'static str {
    match kind {
        ToolSurfaceKind::Outliner => "outliner",
        ToolSurfaceKind::EntityTable => "entity_table",
        ToolSurfaceKind::Viewport => "viewport",
        ToolSurfaceKind::Inspector => "inspector",
        ToolSurfaceKind::Console => "console",
        ToolSurfaceKind::Placeholder => "placeholder",
    }
}

fn surface_capability_label(capability: SurfaceCapability) -> &'static str {
    match capability {
        SurfaceCapability::Observe => "observe",
        SurfaceCapability::Interact => "interact",
        SurfaceCapability::RequestMutation => "request_mutation",
        SurfaceCapability::Ratify => "ratify",
    }
}

struct ViewportProductSelectionRatificationAdapter<'a> {
    viewport_presentations: &'a mut ViewportPresentationStateResource,
    viewport_id: ViewportId,
    capabilities: SurfaceCapabilitySet,
}

impl<'a> ViewportProductSelectionRatificationAdapter<'a> {
    fn new(
        viewport_presentations: &'a mut ViewportPresentationStateResource,
        viewport_id: ViewportId,
        capabilities: SurfaceCapabilitySet,
    ) -> Self {
        Self {
            viewport_presentations,
            viewport_id,
            capabilities,
        }
    }
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

struct InspectorFieldActivationRatificationAdapter<'a> {
    app: &'a mut RunenwerkEditorApp,
    inspector_state: &'a mut EditorInspectorUiState,
    capabilities: SurfaceCapabilitySet,
}

impl<'a> InspectorFieldActivationRatificationAdapter<'a> {
    fn new(
        app: &'a mut RunenwerkEditorApp,
        inspector_state: &'a mut EditorInspectorUiState,
        capabilities: SurfaceCapabilitySet,
    ) -> Self {
        Self {
            app,
            inspector_state,
            capabilities,
        }
    }
}

impl RatificationAdapter for InspectorFieldActivationRatificationAdapter<'_> {
    type Error = EditorMutationError;

    fn has_capability(&self, capability: SurfaceCapability) -> bool {
        self.capabilities.allows(capability)
    }

    fn ratify_intent(&mut self, intent: SurfaceIntent) -> Result<RatificationOutcome, Self::Error> {
        let field_index = match intent.kind {
            SurfaceIntentKind::ActivateField { field_index } => field_index,
            _ => return Ok(RatificationOutcome::Ignored),
        };
        let index = usize::try_from(field_index).map_err(|_| {
            EditorMutationError::inspector_rejected("inspector field index overflow")
        })?;
        activate_inspector_field(self.app, self.inspector_state, index)?;
        Ok(RatificationOutcome::Applied)
    }
}

impl RatificationAdapter for ViewportProductSelectionRatificationAdapter<'_> {
    type Error = EditorMutationError;

    fn has_capability(&self, capability: SurfaceCapability) -> bool {
        self.capabilities.allows(capability)
    }

    fn ratify_intent(&mut self, intent: SurfaceIntent) -> Result<RatificationOutcome, Self::Error> {
        let item_id = match intent.kind {
            SurfaceIntentKind::SelectPrimaryItem { item_id } => item_id,
            SurfaceIntentKind::SelectEntity { .. } | SurfaceIntentKind::ActivateField { .. } => {
                return Ok(RatificationOutcome::Ignored);
            }
        };
        let product_id = ExpressionProductId(item_id);
        if let Some(state) = self.viewport_presentations.state_for_mut(self.viewport_id) {
            state.select_primary_product(product_id);
        } else {
            self.viewport_presentations
                .upsert_state(ViewportPresentationState::new(self.viewport_id, product_id));
        }
        Ok(RatificationOutcome::Applied)
    }
}

fn shell_command_label(command: &ShellCommand) -> &'static str {
    match command {
        ShellCommand::ActivateSelectTool => "ActivateSelectTool",
        ShellCommand::ActivateTranslateTool => "ActivateTranslateTool",
        ShellCommand::Undo => "Undo",
        ShellCommand::Redo => "Redo",
        ShellCommand::SaveScene => "SaveScene",
        ShellCommand::LoadScene => "LoadScene",
        ShellCommand::ToggleDebugLogs => "ToggleDebugLogs",
        ShellCommand::SetTabStackActivePanel { .. } => "SetTabStackActivePanel",
        ShellCommand::CommitTabDrop { .. } => "CommitTabDrop",
        ShellCommand::SwitchPanelToolSurfaceKind { .. } => "SwitchPanelToolSurfaceKind",
        ShellCommand::SelectEntityTableEntity { .. } => "SelectEntityTableEntity",
        ShellCommand::AppendEntityTableSearchText { .. } => "AppendEntityTableSearchText",
        ShellCommand::BackspaceEntityTableSearch { .. } => "BackspaceEntityTableSearch",
        ShellCommand::ToggleEntityTableSort { .. } => "ToggleEntityTableSort",
        ShellCommand::SelectOutlinerEntity { .. } => "SelectOutlinerEntity",
        ShellCommand::SelectViewportProduct { .. } => "SelectViewportProduct",
        ShellCommand::ToggleViewportDetails { .. } => "ToggleViewportDetails",
        ShellCommand::ActivateInspectorField { .. } => "ActivateInspectorField",
        ShellCommand::FocusInspectorField { .. } => "FocusInspectorField",
        ShellCommand::AppendInspectorFieldText { .. } => "AppendInspectorFieldText",
        ShellCommand::BackspaceInspectorFieldText { .. } => "BackspaceInspectorFieldText",
        ShellCommand::CommitInspectorFieldText { .. } => "CommitInspectorFieldText",
        ShellCommand::CancelInspectorFieldText { .. } => "CancelInspectorFieldText",
        ShellCommand::DispatchSurfaceLocalAction { .. } => "DispatchSurfaceLocalAction",
        ShellCommand::NoOp => "NoOp",
    }
}

fn activate_inspector_field(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
) -> Result<(), EditorMutationError> {
    let inspector_view = InspectorPanelPresenter::build_view_model(app.runtime(), inspector_state);

    match inspector_view {
        InspectorPanelViewModel::Entity {
            entity,
            components,
            available_component_types,
            ..
        } => {
            if let Some(component) = components.get(index) {
                select_single_component_with_origin(
                    app.runtime_mut(),
                    component.entity,
                    component.component_type,
                    editor_core::ChangeOrigin::InspectorPanel,
                )?;
                inspector_state.clear_draft();
                inspector_state.clear_focus();
                return Ok(());
            }

            let offset = index.saturating_sub(components.len());
            let Some(candidate) = available_component_types.get(offset) else {
                return Err(EditorMutationError::inspector_rejected(
                    "inspector field index out of range",
                ));
            };

            if candidate.already_attached {
                return Ok(());
            }

            execute_intent_with_history_from_origin(
                app.runtime_mut(),
                "Add Component",
                editor_scene::SceneCommandIntent::AddComponent {
                    entity,
                    component_type: candidate.component_type,
                },
                editor_core::ChangeOrigin::InspectorPanel,
            )
            .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
            inspector_state.clear_draft();
            inspector_state.clear_focus();
            Ok(())
        }
        InspectorPanelViewModel::Component {
            entity,
            component_type,
            widget_fields,
            ..
        } => {
            let field = widget_fields
                .get(index)
                .ok_or(EditorMutationError::inspector_rejected(
                    "inspector field index out of range",
                ))?;

            let next_value = next_shell_edit_value(app.runtime(), component_type, field).ok_or(
                EditorMutationError::inspector_rejected("inspector field is not editable"),
            )?;

            commit_inspector_component_field_edit(
                app,
                inspector_state,
                entity,
                component_type,
                field.path.clone(),
                next_value,
            )?;

            Ok(())
        }
        InspectorPanelViewModel::Empty
        | InspectorPanelViewModel::Resource { .. }
        | InspectorPanelViewModel::Unsupported { .. }
        | InspectorPanelViewModel::Error { .. } => Err(EditorMutationError::inspector_rejected(
            "shell inspector field activation requires entity/component target",
        )),
    }
}

fn mutate_inspector_surface_session(
    app: &mut RunenwerkEditorApp,
    target: StructuralCommandTarget,
    mutate: impl FnOnce(
        &mut RunenwerkEditorApp,
        &mut EditorInspectorUiState,
    ) -> Result<(), EditorMutationError>,
) -> Result<(), EditorMutationError> {
    let Some(surface_id) = target.active_tool_surface else {
        return Err(EditorMutationError::inspector_rejected(
            "inspector command requires surface instance target",
        ));
    };
    let mut session = app.surface_sessions().session_or_default(surface_id);
    let result = mutate(app, &mut session.inspector_ui_state);
    *app.surface_sessions_mut().session_mut(surface_id) = session;
    result
}

fn focus_inspector_field(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let text = inspector_current_draft_text(&field, true);
    apply_inspector_draft_text(inspector_state, entity, component_type, &field, text)
}

fn append_inspector_field_text(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
    text: &str,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let mut next_text = inspector_current_draft_text(&field, false);
    next_text.push_str(text);
    apply_inspector_draft_text(inspector_state, entity, component_type, &field, next_text)
}

fn backspace_inspector_field_text(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let mut next_text = inspector_current_draft_text(&field, true);
    let _ = next_text.pop();
    apply_inspector_draft_text(inspector_state, entity, component_type, &field, next_text)
}

fn commit_inspector_field_text(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    index: usize,
) -> Result<(), EditorMutationError> {
    let (entity, component_type, field) =
        inspector_component_field_at_index(app, inspector_state, index)?;
    let text = inspector_current_draft_text(&field, true);
    let value = parse_inspector_field_text(&field, &text).ok_or(
        EditorMutationError::inspector_rejected("inspector field text is invalid for target type"),
    )?;
    commit_inspector_component_field_edit(
        app,
        inspector_state,
        entity,
        component_type,
        field.path,
        value,
    )
}

fn inspector_component_field_at_index(
    app: &mut RunenwerkEditorApp,
    inspector_state: &EditorInspectorUiState,
    index: usize,
) -> Result<
    (
        editor_core::EntityId,
        editor_core::ComponentTypeId,
        crate::editor_panels::InspectorWidgetField,
    ),
    EditorMutationError,
> {
    let inspector_view = InspectorPanelPresenter::build_view_model(app.runtime(), inspector_state);
    match inspector_view {
        InspectorPanelViewModel::Component {
            entity,
            component_type,
            widget_fields,
            ..
        } => {
            let field = widget_fields
                .get(index)
                .ok_or(EditorMutationError::inspector_rejected(
                    "inspector field index out of range",
                ))?
                .clone();
            Ok((entity, component_type, field))
        }
        _ => Err(EditorMutationError::inspector_rejected(
            "inspector text editing requires component target",
        )),
    }
}

fn inspector_current_draft_text(
    field: &crate::editor_panels::InspectorWidgetField,
    include_base_value: bool,
) -> String {
    if let Some(text) = &field.draft_text {
        return text.clone();
    }
    if include_base_value {
        return inspector_value_text(&field.value);
    }
    String::new()
}

fn apply_inspector_draft_text(
    inspector_state: &mut EditorInspectorUiState,
    entity: editor_core::EntityId,
    component_type: editor_core::ComponentTypeId,
    field: &crate::editor_panels::InspectorWidgetField,
    text: String,
) -> Result<(), EditorMutationError> {
    let parsed_value = parse_inspector_field_text(field, &text);
    let initial_value = parsed_value
        .clone()
        .or_else(|| editable_value_from_field(field))
        .ok_or(EditorMutationError::inspector_rejected(
            "inspector field is not editable",
        ))?;

    inspector_state.begin_field_edit(
        entity,
        component_type,
        field.path.clone(),
        initial_value,
        text.clone(),
    );
    inspector_state.update_field_draft_text(text)?;
    if let Some(value) = parsed_value {
        inspector_state.update_field_draft(value)?;
    }
    Ok(())
}

fn commit_inspector_component_field_edit(
    app: &mut RunenwerkEditorApp,
    inspector_state: &mut EditorInspectorUiState,
    entity: editor_core::EntityId,
    component_type: editor_core::ComponentTypeId,
    path: editor_inspector::InspectorPath,
    value: InspectorEditValue,
) -> Result<(), EditorMutationError> {
    execute_intent_with_history_from_origin(
        app.runtime_mut(),
        "Edit Component Field",
        editor_scene::SceneCommandIntent::EditComponentField {
            entity,
            component_type,
            path,
            value,
        },
        editor_core::ChangeOrigin::InspectorPanel,
    )
    .map_err(|error| EditorMutationError::runtime_rejected(error.as_static_str()))?;
    inspector_state.clear_draft();
    inspector_state.clear_focus();
    Ok(())
}

fn parse_inspector_field_text(
    field: &crate::editor_panels::InspectorWidgetField,
    text: &str,
) -> Option<InspectorEditValue> {
    match &field.value {
        InspectorValue::Bool(_) => {
            let normalized = text.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "true" | "1" => Some(InspectorEditValue::Bool(true)),
                "false" | "0" => Some(InspectorEditValue::Bool(false)),
                _ => None,
            }
        }
        InspectorValue::Integer(_) => text
            .trim()
            .parse::<i64>()
            .ok()
            .map(InspectorEditValue::Integer),
        InspectorValue::Float(_) => text
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map(InspectorEditValue::Float),
        InspectorValue::Text(_) => Some(InspectorEditValue::Text(text.to_string())),
        InspectorValue::ReadOnlyText(_)
        | InspectorValue::Enum { .. }
        | InspectorValue::Group
        | InspectorValue::Unsupported { .. } => None,
    }
}

fn editable_value_from_field(
    field: &crate::editor_panels::InspectorWidgetField,
) -> Option<InspectorEditValue> {
    if let Some(value) = &field.draft_value {
        return Some(value.clone());
    }

    match &field.value {
        InspectorValue::Bool(value) => Some(InspectorEditValue::Bool(*value)),
        InspectorValue::Integer(value) => Some(InspectorEditValue::Integer(*value)),
        InspectorValue::Float(value) => Some(InspectorEditValue::Float(*value)),
        InspectorValue::Text(value) => Some(InspectorEditValue::Text(value.clone())),
        InspectorValue::ReadOnlyText(_)
        | InspectorValue::Enum { .. }
        | InspectorValue::Group
        | InspectorValue::Unsupported { .. } => None,
    }
}

fn inspector_value_text(value: &InspectorValue) -> String {
    match value {
        InspectorValue::Bool(value) => value.to_string(),
        InspectorValue::Integer(value) => value.to_string(),
        InspectorValue::Float(value) => value.to_string(),
        InspectorValue::Text(value) => value.clone(),
        InspectorValue::ReadOnlyText(value) => value.clone(),
        InspectorValue::Enum { current, .. } => current.clone(),
        InspectorValue::Group => "group".to_string(),
        InspectorValue::Unsupported { type_name } => format!("unsupported<{type_name}>"),
    }
}

fn next_shell_edit_value(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
    component_type: ComponentTypeId,
    field: &crate::editor_panels::InspectorWidgetField,
) -> Option<InspectorEditValue> {
    if is_local_transform_component(runtime, component_type)
        && let Some(stepper_value) = transform_stepper_value(field)
    {
        return Some(stepper_value);
    }

    if let Some(draft) = &field.draft_value {
        return match draft {
            InspectorEditValue::Bool(value) => Some(InspectorEditValue::Bool(!value)),
            InspectorEditValue::Integer(value) => {
                Some(InspectorEditValue::Integer(value.saturating_add(1)))
            }
            InspectorEditValue::Float(value) => Some(InspectorEditValue::Float(value + 1.0)),
            InspectorEditValue::Text(value) => Some(InspectorEditValue::Text(format!("{value}*"))),
        };
    }

    match &field.value {
        InspectorValue::Bool(value) => Some(InspectorEditValue::Bool(!value)),
        InspectorValue::Integer(value) => {
            Some(InspectorEditValue::Integer(value.saturating_add(1)))
        }
        InspectorValue::Float(value) => Some(InspectorEditValue::Float(value + 1.0)),
        InspectorValue::Text(value) => Some(InspectorEditValue::Text(format!("{value}*"))),
        InspectorValue::ReadOnlyText(_)
        | InspectorValue::Enum { .. }
        | InspectorValue::Group
        | InspectorValue::Unsupported { .. } => None,
    }
}

fn transform_stepper_value(
    field: &crate::editor_panels::InspectorWidgetField,
) -> Option<InspectorEditValue> {
    let path = field.path.stable_key();
    if path != "translation.x" && path != "translation.y" && path != "translation.z" {
        return None;
    }

    let current = match field.draft_value.as_ref() {
        Some(InspectorEditValue::Float(value)) => *value,
        Some(_) => return None,
        None => match &field.value {
            InspectorValue::Float(value) => *value,
            _ => return None,
        },
    };

    Some(InspectorEditValue::Float(
        current + TRANSFORM_STEPPER_INCREMENT,
    ))
}

fn default_scene_file_path() -> PathBuf {
    PathBuf::from(DEFAULT_EDITOR_SCENE_PATH)
}

fn apply_tab_drop(
    shell_state: &mut RunenwerkEditorShellState,
    panel_instance_id: editor_shell::PanelInstanceId,
    source_tab_stack_id: editor_shell::TabStackId,
    destination: TabDropDestination,
) -> Result<(), editor_shell::WorkspaceStateError> {
    match destination {
        TabDropDestination::TabStack {
            tab_stack_id,
            insert_index,
        } => shell_state.apply_workspace_mutation(WorkspaceMutation::MovePanelBetweenTabStacks {
            panel_id: panel_instance_id,
            source_tab_stack_id,
            destination_tab_stack_id: tab_stack_id,
            destination_index: insert_index,
            activate_panel: true,
        }),
        TabDropDestination::NewFloatingHost => {
            let floating_host_id = shell_state.allocate_panel_host_id();
            let floating_tab_stack_id = shell_state.allocate_tab_stack_id();
            shell_state.apply_workspace_mutation(WorkspaceMutation::MovePanelToNewFloatingHost {
                panel_id: panel_instance_id,
                source_tab_stack_id,
                floating_host_id,
                floating_tab_stack_id,
                bounds: default_floating_host_bounds(shell_state),
            })
        }
    }
}

fn default_floating_host_bounds(shell_state: &RunenwerkEditorShellState) -> FloatingHostBounds {
    let bounds = shell_state
        .last_bounds()
        .unwrap_or(ui_math::UiRect::new(0.0, 0.0, 1280.0, 720.0));
    let width = (bounds.width * 0.46).clamp(360.0, 920.0);
    let height = (bounds.height * 0.42).clamp(240.0, 680.0);
    let x = (bounds.width - width).max(0.0) * 0.5;
    let y = (bounds.height - height).max(0.0) * 0.33;
    FloatingHostBounds::new(x, y, width, height)
}

fn save_scene_to_default_path(
    app: &mut RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
) -> Result<(), EditorMutationError> {
    let path = default_scene_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to create editor scene folder")
        })?;
    }

    write_scene_file(&path, app.runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save editor scene"))?;
    let retained_path = retained_change_log_path_for_scene(&path);
    let workspace_layout_path =
        default_workspace_layout_path_for_profile(shell_state.active_workspace_profile_id());
    let entry_count = write_retained_change_log(&retained_path, app.runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save retained change log"))?;
    if let Some(parent) = workspace_layout_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to create workspace layout folder")
        })?;
    }
    write_workspace_layout(&workspace_layout_path, shell_state.workspace_state())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save workspace layout"))?;
    app.runtime_mut()
        .record_workflow_event(editor_core::WorkflowEventKind::SceneSaved {
            path: path.display().to_string(),
        });
    app.runtime_mut()
        .record_workflow_event(editor_core::WorkflowEventKind::RetainedChangesSaved {
            path: retained_path.display().to_string(),
            entry_count,
        });
    app.append_console_line(format!("[io] saved {}", path.display()));
    app.append_console_line(format!(
        "[io] retained {} ratified changes at {}",
        entry_count,
        retained_path.display()
    ));
    app.append_console_line(format!(
        "[io] saved workspace layout {}",
        workspace_layout_path.display()
    ));
    Ok(())
}

fn load_scene_from_default_path(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
) -> Result<(), EditorMutationError> {
    let path = default_scene_file_path();
    if !path.exists() {
        app.append_console_line(format!(
            "[io] scene file missing, skipping load: {}",
            path.display()
        ));
        return Ok(());
    }

    {
        let runtime = app.runtime_mut();
        runtime.prepare_for_scene_load();
        register_mvp_component_types(runtime);
    }

    let migration = match load_scene_file_into_runtime_classified(&path, app.runtime_mut()) {
        Ok(migration) => migration,
        Err(class) => {
            app.append_console_line(format!(
                "[io] load failed ({})",
                migration_failure_class_label(class)
            ));
            return Err(EditorMutationError::runtime_rejected(
                "failed to load editor scene",
            ));
        }
    };
    let retained_path = retained_change_log_path_for_scene(&path);
    let workspace_layout_path =
        default_workspace_layout_path_for_profile(shell_state.active_workspace_profile_id());
    let legacy_workspace_layout_path = legacy_workspace_layout_path_for_scene(&path);
    let retained = if retained_path.exists() {
        Some(read_retained_change_log(&retained_path).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to load retained change log")
        })?)
    } else {
        None
    };
    bootstrap_mvp_scene_if_empty(app.runtime_mut())?;
    app.reset_transient_editor_ui_state();
    app.runtime_mut()
        .record_workflow_event(editor_core::WorkflowEventKind::SceneLoaded {
            path: path.display().to_string(),
            migration_path: migration,
        });
    if let Some(migration_path) = migration {
        app.append_console_line(format!(
            "[io] scene migration applied: {}",
            editor_core::migration_path_label(migration_path)
        ));
    }
    if let Some(retained) = retained {
        app.runtime_mut().record_workflow_event(
            editor_core::WorkflowEventKind::RetainedChangesLoaded {
                path: retained_path.display().to_string(),
                entry_count: retained.entries.len(),
            },
        );
        app.append_console_line(format!(
            "[io] loaded retained change log: {} entries ({})",
            retained.entries.len(),
            retained_path.display()
        ));
    }
    let layout_path_to_load = if workspace_layout_path.exists() {
        Some(workspace_layout_path.clone())
    } else if legacy_workspace_layout_path.exists() {
        Some(legacy_workspace_layout_path.clone())
    } else {
        None
    };

    if let Some(layout_path_to_load) = layout_path_to_load {
        match read_workspace_layout(&layout_path_to_load) {
            Ok(workspace_state) => {
                shell_state.replace_workspace_state(workspace_state);
                app.prune_surface_sessions_for_workspace(shell_state.workspace_state());
                app.append_console_line(format!(
                    "[io] loaded workspace layout {}",
                    layout_path_to_load.display()
                ));
            }
            Err(error) => {
                app.append_console_line(format!(
                    "[io] workspace layout load failed, keeping current layout: {} ({error})",
                    layout_path_to_load.display()
                ));
            }
        }
    } else {
        app.append_console_line(format!(
            "[io] workspace layout missing, keeping current layout: {}",
            workspace_layout_path.display()
        ));
    }
    app.append_console_line(format!("[io] loaded {}", path.display()));
    Ok(())
}

fn migration_failure_class_label(class: editor_core::MigrationFailureClass) -> &'static str {
    match class {
        editor_core::MigrationFailureClass::DecodeFailure => "decode-failure",
        editor_core::MigrationFailureClass::NormalizationFailure => "normalization-failure",
        editor_core::MigrationFailureClass::FormationFailure => "formation-failure",
        editor_core::MigrationFailureClass::ApplyFailure => "apply-failure",
    }
}
