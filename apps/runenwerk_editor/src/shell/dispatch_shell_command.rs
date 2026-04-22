use std::path::PathBuf;

use editor_core::{ComponentTypeId, EditorMutationError};
use editor_inspector::{InspectorEditValue, InspectorValue};
use editor_shell::{FloatingHostBounds, ShellCommand, TabDropDestination, WorkspaceMutation};
use editor_viewport::{ProductAvailabilityState, ViewportPresentationState};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::{redo_last_scene_change, undo_last_scene_change};
use crate::editor_panels::{InspectorPanelCommand, InspectorPanelViewModel, OutlinerPanelCommand};
use crate::editor_runtime::{
    bootstrap_mvp_scene_if_empty, is_local_transform_component, register_mvp_component_types,
};
use crate::persistence::{
    load_scene_file_into_runtime_classified, read_retained_change_log, read_workspace_layout,
    retained_change_log_path_for_scene, workspace_layout_path_for_scene, write_retained_change_log,
    write_scene_file, write_workspace_layout,
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
        }
        ShellCommand::SelectOutlinerEntity {
            entity,
            target: _,
            projection_epoch: _,
        } => {
            app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity { entity })?;
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
                let selectable = viewport_observations
                    .frame_for(resolved_viewport_id)
                    .and_then(|frame| frame.availability_by_product.get(&product_id).copied())
                    .map(|availability| availability == ProductAvailabilityState::Available)
                    .unwrap_or(false);
                if selectable {
                    if let Some(state) = viewport_presentations.state_for_mut(resolved_viewport_id)
                    {
                        state.select_primary_product(product_id);
                    } else {
                        viewport_presentations.upsert_state(ViewportPresentationState::new(
                            resolved_viewport_id,
                            product_id,
                        ));
                    }
                } else {
                    app.append_console_line(format!(
                        "[viewport] product selection ignored (unavailable): viewport={} product={}",
                        resolved_viewport_id.0, product_id.0
                    ));
                }
            }
            _ => {
                app.append_console_line(
                    "[viewport.binding] product selection ignored (missing runtime binding context)"
                        .to_string(),
                );
            }
        },
        ShellCommand::ToggleViewportDetails => {
            app.toggle_viewport_details_visible();
        }
        ShellCommand::ActivateInspectorField {
            index,
            target: _,
            projection_epoch: _,
        } => {
            activate_inspector_field(app, index)?;
        }
        ShellCommand::NoOp => {}
    }

    Ok(())
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
        ShellCommand::SelectOutlinerEntity { .. } => "SelectOutlinerEntity",
        ShellCommand::SelectViewportProduct { .. } => "SelectViewportProduct",
        ShellCommand::ToggleViewportDetails => "ToggleViewportDetails",
        ShellCommand::ActivateInspectorField { .. } => "ActivateInspectorField",
        ShellCommand::NoOp => "NoOp",
    }
}

fn activate_inspector_field(
    app: &mut RunenwerkEditorApp,
    index: usize,
) -> Result<(), EditorMutationError> {
    let inspector_view = app.inspector_view_model();

    match inspector_view {
        InspectorPanelViewModel::Entity {
            entity,
            components,
            available_component_types,
            ..
        } => {
            if let Some(component) = components.get(index) {
                app.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
                    entity: component.entity,
                    component_type: component.component_type,
                })?;
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

            app.dispatch_inspector_command(InspectorPanelCommand::AddComponentToEntity {
                entity,
                component_type: candidate.component_type,
            })?;
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

            app.dispatch_inspector_command(InspectorPanelCommand::EditComponentField {
                entity,
                component_type,
                path: field.path.clone(),
                value: next_value,
            })?;

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

fn next_shell_edit_value(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
    component_type: ComponentTypeId,
    field: &crate::editor_panels::InspectorWidgetField,
) -> Option<InspectorEditValue> {
    if is_local_transform_component(runtime, component_type) {
        if let Some(stepper_value) = transform_stepper_value(field) {
            return Some(stepper_value);
        }
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
    let workspace_layout_path = workspace_layout_path_for_scene(&path);
    let entry_count = write_retained_change_log(&retained_path, app.runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save retained change log"))?;
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
    let workspace_layout_path = workspace_layout_path_for_scene(&path);
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
    if workspace_layout_path.exists() {
        match read_workspace_layout(&workspace_layout_path) {
            Ok(workspace_state) => {
                shell_state.replace_workspace_state(workspace_state);
                app.append_console_line(format!(
                    "[io] loaded workspace layout {}",
                    workspace_layout_path.display()
                ));
            }
            Err(error) => {
                app.append_console_line(format!(
                    "[io] workspace layout load failed, keeping current layout: {} ({error})",
                    workspace_layout_path.display()
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
