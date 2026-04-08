use editor_inspector::{InspectorEditValue, InspectorValue};
use editor_shell::ShellCommand;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::{InspectorPanelCommand, InspectorPanelViewModel, OutlinerPanelCommand};
use crate::shell::{SELECT_TOOL_ID, TRANSLATE_TOOL_ID};

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

            let next_value =
                next_shell_edit_value(field).ok_or("inspector field is not editable")?;

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
    field: &crate::editor_panels::InspectorWidgetField,
) -> Option<InspectorEditValue> {
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
