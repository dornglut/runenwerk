use std::path::PathBuf;

use editor_core::ComponentTypeId;
use editor_inspector::{InspectorEditValue, InspectorValue};
use editor_shell::ShellCommand;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::{InspectorPanelCommand, InspectorPanelViewModel, OutlinerPanelCommand};
use crate::editor_runtime::{
    bootstrap_mvp_scene_if_empty, is_local_transform_component, redo_last_scene_transaction,
    register_mvp_component_types, undo_last_scene_transaction,
};
use crate::persistence::{load_scene_file_into_runtime, write_scene_file};
use crate::shell::{SELECT_TOOL_ID, TRANSLATE_TOOL_ID};

const TRANSFORM_STEPPER_INCREMENT: f64 = 0.25;
const DEFAULT_EDITOR_SCENE_PATH: &str = "editor-scenes/default.scene.ron";

pub fn dispatch_shell_command(
    app: &mut RunenwerkEditorApp,
    command: ShellCommand,
) -> Result<(), &'static str> {
    match command {
        ShellCommand::ActivateSelectTool => {
            app.runtime_mut()
                .session_mut()
                .set_active_tool(Some(SELECT_TOOL_ID));
        }
        ShellCommand::ActivateTranslateTool => {
            app.runtime_mut()
                .session_mut()
                .set_active_tool(Some(TRANSLATE_TOOL_ID));
        }
        ShellCommand::Undo => {
            if let Some(entry) = undo_last_scene_transaction(app.runtime_mut())? {
                app.append_console_line(format!("[history] undo: {}", entry.transaction.label));
            }
        }
        ShellCommand::Redo => {
            if let Some(entry) = redo_last_scene_transaction(app.runtime_mut())? {
                app.append_console_line(format!("[history] redo: {}", entry.transaction.label));
            }
        }
        ShellCommand::SaveScene => {
            save_scene_to_default_path(app)?;
        }
        ShellCommand::LoadScene => {
            load_scene_from_default_path(app)?;
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
        ShellCommand::SelectOutlinerEntity { entity } => {
            app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity { entity })?;
        }
        ShellCommand::ActivateInspectorField { index } => {
            activate_inspector_field(app, index)?;
        }
        ShellCommand::NoOp => {}
    }

    Ok(())
}

fn activate_inspector_field(
    app: &mut RunenwerkEditorApp,
    index: usize,
) -> Result<(), &'static str> {
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
                return Err("inspector field index out of range");
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
                .ok_or("inspector field index out of range")?;

            let next_value = next_shell_edit_value(app.runtime(), component_type, field)
                .ok_or("inspector field is not editable")?;

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
        | InspectorPanelViewModel::Error { .. } => {
            Err("shell inspector field activation requires entity/component target")
        }
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

fn save_scene_to_default_path(app: &mut RunenwerkEditorApp) -> Result<(), &'static str> {
    let path = default_scene_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "failed to create editor scene folder")?;
    }

    write_scene_file(&path, app.runtime()).map_err(|_| "failed to save editor scene")?;
    app.append_console_line(format!("[io] saved {}", path.display()));
    Ok(())
}

fn load_scene_from_default_path(app: &mut RunenwerkEditorApp) -> Result<(), &'static str> {
    let path = default_scene_file_path();
    if !path.exists() {
        app.append_console_line(format!(
            "[io] scene file missing, skipping load: {}",
            path.display()
        ));
        return Ok(());
    }

    let mut runtime = crate::editor_runtime::RunenwerkEditorRuntime::new();
    register_mvp_component_types(&mut runtime);
    load_scene_file_into_runtime(&path, &mut runtime).map_err(|_| "failed to load editor scene")?;
    bootstrap_mvp_scene_if_empty(&mut runtime)?;
    app.runtime = runtime;
    app.inspector_ui_state.clear_draft();
    app.inspector_ui_state.clear_focus();
    app.viewport_interaction_state.clear();
    app.append_console_line(format!("[io] loaded {}", path.display()));
    Ok(())
}
