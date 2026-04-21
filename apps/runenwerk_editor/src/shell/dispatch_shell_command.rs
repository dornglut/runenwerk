use std::path::PathBuf;

use editor_core::{ComponentTypeId, EditorMutationError};
use editor_inspector::{InspectorEditValue, InspectorValue};
use editor_shell::{
    FloatingHostPlaceholderState, PanelHostKind, ShellCommand, SplitHostState, WorkspaceMutation,
    WorkspaceSplitAxis,
};
use editor_viewport::{ProductAvailabilityState, ViewportPresentationState};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::{redo_last_scene_change, undo_last_scene_change};
use crate::editor_panels::{InspectorPanelCommand, InspectorPanelViewModel, OutlinerPanelCommand};
use crate::editor_runtime::{
    bootstrap_mvp_scene_if_empty, is_local_transform_component, register_mvp_component_types,
};
use crate::persistence::{
    load_scene_file_into_runtime_classified, read_retained_change_log, read_workspace_state_file,
    retained_change_log_path_for_scene, write_retained_change_log, write_scene_file,
    write_workspace_state_file,
};
use crate::runtime::viewport::{
    ViewportArtifactObservationResource, ViewportPresentationStateResource,
};
use crate::shell::{SELECT_TOOL_ID, TRANSLATE_TOOL_ID};

const TRANSFORM_STEPPER_INCREMENT: f64 = 0.25;
const DEFAULT_EDITOR_SCENE_PATH: &str = "editor-scenes/default.scene.ron";
const DEFAULT_EDITOR_WORKSPACE_PATH: &str = "editor-scenes/default.workspace.ron";

pub fn dispatch_shell_command(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&mut crate::shell::RunenwerkEditorShellState>,
    command: ShellCommand,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
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
            save_scene_to_default_path(app, shell_state.as_deref())?;
        }
        ShellCommand::LoadScene => {
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
            target: _,
            projection_epoch: _,
        } => match (viewport_presentations, viewport_observations) {
            (Some(viewport_presentations), Some(viewport_observations)) => {
                let selectable = viewport_observations
                    .frame_for(viewport_id)
                    .and_then(|frame| frame.availability_by_product.get(&product_id).copied())
                    .map(|availability| availability == ProductAvailabilityState::Available)
                    .unwrap_or(false);
                if selectable {
                    if let Some(state) = viewport_presentations.state_for_mut(viewport_id) {
                        state.select_primary_product(product_id);
                    } else {
                        viewport_presentations
                            .upsert_state(ViewportPresentationState::new(viewport_id, product_id));
                    }
                } else {
                    app.append_console_line(format!(
                        "[viewport] product selection ignored (unavailable): viewport={} product={}",
                        viewport_id.0, product_id.0
                    ));
                }
            }
            _ => {
                app.append_console_line(
                    "[viewport] product selection ignored (missing viewport runtime context)"
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
        ShellCommand::ActivateTab {
            tab_stack_id,
            panel_instance_id,
            projection_epoch: _,
        } => {
            if let Some(shell_state) = shell_state {
                shell_state
                    .apply_workspace_mutation(WorkspaceMutation::SetTabStackActivePanel {
                        tab_stack_id,
                        active_panel: Some(panel_instance_id),
                    })
                    .map_err(|_| {
                        EditorMutationError::runtime_rejected(
                            "failed to activate tab in workspace state",
                        )
                    })?;
            }
        }
        ShellCommand::FloatPanel {
            tab_stack_id,
            panel_instance_id,
            projection_epoch: _,
        } => {
            if let Some(shell_state) = shell_state {
                float_panel_from_tab_stack(shell_state, tab_stack_id, panel_instance_id)?;
            }
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
        ShellCommand::SelectOutlinerEntity { .. } => "SelectOutlinerEntity",
        ShellCommand::SelectViewportProduct { .. } => "SelectViewportProduct",
        ShellCommand::ToggleViewportDetails => "ToggleViewportDetails",
        ShellCommand::ActivateInspectorField { .. } => "ActivateInspectorField",
        ShellCommand::ActivateTab { .. } => "ActivateTab",
        ShellCommand::FloatPanel { .. } => "FloatPanel",
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

fn default_workspace_file_path() -> PathBuf {
    PathBuf::from(DEFAULT_EDITOR_WORKSPACE_PATH)
}

fn save_scene_to_default_path(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&crate::shell::RunenwerkEditorShellState>,
) -> Result<(), EditorMutationError> {
    let scene_path = default_scene_file_path();
    if let Some(parent) = scene_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to create editor scene folder")
        })?;
    }

    write_scene_file(&scene_path, app.runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save editor scene"))?;
    let retained_path = retained_change_log_path_for_scene(&scene_path);
    let entry_count = write_retained_change_log(&retained_path, app.runtime())
        .map_err(|_| EditorMutationError::runtime_rejected("failed to save retained change log"))?;
    let workspace_path = default_workspace_file_path();
    if let Some(shell_state) = shell_state {
        write_workspace_state_file(&workspace_path, shell_state.workspace_state()).map_err(
            |_| EditorMutationError::runtime_rejected("failed to save workspace layout"),
        )?;
        app.append_console_line(format!("[io] saved {}", workspace_path.display()));
    }
    app.runtime_mut()
        .record_workflow_event(editor_core::WorkflowEventKind::SceneSaved {
            path: scene_path.display().to_string(),
        });
    app.runtime_mut()
        .record_workflow_event(editor_core::WorkflowEventKind::RetainedChangesSaved {
            path: retained_path.display().to_string(),
            entry_count,
        });
    app.append_console_line(format!("[io] saved {}", scene_path.display()));
    app.append_console_line(format!(
        "[io] retained {} ratified changes at {}",
        entry_count,
        retained_path.display()
    ));
    Ok(())
}

fn load_scene_from_default_path(
    app: &mut RunenwerkEditorApp,
    shell_state: Option<&mut crate::shell::RunenwerkEditorShellState>,
) -> Result<(), EditorMutationError> {
    let scene_path = default_scene_file_path();
    if !scene_path.exists() {
        app.append_console_line(format!(
            "[io] scene file missing, skipping load: {}",
            scene_path.display()
        ));
        return Ok(());
    }

    {
        let runtime = app.runtime_mut();
        runtime.prepare_for_scene_load();
        register_mvp_component_types(runtime);
    }

    let migration = match load_scene_file_into_runtime_classified(&scene_path, app.runtime_mut()) {
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
    let retained_path = retained_change_log_path_for_scene(&scene_path);
    let retained = if retained_path.exists() {
        Some(read_retained_change_log(&retained_path).map_err(|_| {
            EditorMutationError::runtime_rejected("failed to load retained change log")
        })?)
    } else {
        None
    };
    if let Some(shell_state) = shell_state {
        let workspace_path = default_workspace_file_path();
        if workspace_path.exists() {
            let workspace_state = read_workspace_state_file(&workspace_path)
                .map_err(|_| EditorMutationError::runtime_rejected("failed to load workspace"))?;
            shell_state
                .replace_workspace_state(workspace_state)
                .map_err(|_| {
                    EditorMutationError::runtime_rejected("failed to apply workspace layout")
                })?;
            app.append_console_line(format!("[io] loaded {}", workspace_path.display()));
        }
    }
    bootstrap_mvp_scene_if_empty(app.runtime_mut())?;
    app.reset_transient_editor_ui_state();
    app.runtime_mut()
        .record_workflow_event(editor_core::WorkflowEventKind::SceneLoaded {
            path: scene_path.display().to_string(),
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
    app.append_console_line(format!("[io] loaded {}", scene_path.display()));
    Ok(())
}

fn float_panel_from_tab_stack(
    shell_state: &mut crate::shell::RunenwerkEditorShellState,
    tab_stack_id: editor_shell::TabStackId,
    panel_instance_id: editor_shell::PanelInstanceId,
) -> Result<(), EditorMutationError> {
    let source_tab_stack = shell_state
        .workspace_state()
        .tab_stack(tab_stack_id)
        .ok_or(EditorMutationError::runtime_rejected(
            "source tab stack is missing",
        ))?;
    if !source_tab_stack.ordered_panels.contains(&panel_instance_id) {
        return Err(EditorMutationError::runtime_rejected(
            "panel is not present in source tab stack",
        ));
    }

    let destination_tab_stack_id = shell_state.allocate_tab_stack_id();
    let floating_host_id = shell_state.allocate_panel_host_id();
    let new_root_host_id = shell_state.allocate_panel_host_id();
    let existing_root_host_id = shell_state.workspace_state().root_host_id();

    shell_state
        .apply_workspace_mutations([
            WorkspaceMutation::CreateTabStack {
                tab_stack_id: destination_tab_stack_id,
            },
            WorkspaceMutation::CreateHostNode {
                host_id: floating_host_id,
                kind: PanelHostKind::FloatingHostPlaceholder(FloatingHostPlaceholderState {
                    tab_stack_id: Some(destination_tab_stack_id),
                }),
            },
            WorkspaceMutation::CreateHostNode {
                host_id: new_root_host_id,
                kind: PanelHostKind::SplitHost(SplitHostState {
                    axis: WorkspaceSplitAxis::Horizontal,
                    fraction: 0.80,
                    first_child: existing_root_host_id,
                    second_child: floating_host_id,
                }),
            },
            WorkspaceMutation::SetRootHost {
                host_id: new_root_host_id,
            },
            WorkspaceMutation::MovePanelToTabStack {
                panel_id: panel_instance_id,
                source_tab_stack_id: tab_stack_id,
                destination_tab_stack_id,
                destination_index: Some(0),
                activate_in_destination: true,
            },
        ])
        .map_err(|_| EditorMutationError::runtime_rejected("failed to float panel"))?;

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
