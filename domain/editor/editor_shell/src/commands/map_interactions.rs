//! File: domain/editor/editor_shell/src/commands/map_interactions.rs
//! Purpose: Map semantic UI interactions to shell commands.

use crate::{EditorShellViewModel, UiInteraction, UiInteractionResults};

use crate::{
    ShellCommand, TOOLBAR_SELECT_BUTTON_WIDGET_ID, TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
    inspector_field_index, outliner_row_index,
};

pub fn map_interactions_to_shell_commands(
    interactions: &UiInteractionResults,
    view_model: &EditorShellViewModel,
) -> Vec<ShellCommand> {
    let mut commands = Vec::new();

    for interaction in &interactions.items {
        if let UiInteraction::Activated(widget_id) = interaction {
            let command = if *widget_id == TOOLBAR_SELECT_BUTTON_WIDGET_ID {
                ShellCommand::ActivateSelectTool
            } else if *widget_id == TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID {
                ShellCommand::ActivateTranslateTool
            } else if let Some(index) = inspector_field_index(*widget_id) {
                ShellCommand::ActivateInspectorField { index }
            } else if let Some(index) = outliner_row_index(*widget_id) {
                view_model
                    .outliner
                    .rows
                    .get(index)
                    .map(|row| ShellCommand::SelectOutlinerEntity { entity: row.entity })
                    .unwrap_or(ShellCommand::NoOp)
            } else {
                ShellCommand::NoOp
            };

            commands.push(command);
        }
    }

    commands
}
