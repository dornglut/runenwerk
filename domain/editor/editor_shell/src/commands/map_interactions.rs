//! File: domain/editor/editor_shell/src/commands/map_interactions.rs
//! Purpose: Map semantic UI interactions to shell commands.

use crate::{EditorShellViewModel, UiInteraction, UiInteractionResults};

use crate::{
    ShellCommand, TOOLBAR_DEBUG_LOGS_BUTTON_WIDGET_ID, TOOLBAR_LOAD_BUTTON_WIDGET_ID,
    TOOLBAR_REDO_BUTTON_WIDGET_ID, TOOLBAR_SAVE_BUTTON_WIDGET_ID, TOOLBAR_SELECT_BUTTON_WIDGET_ID,
    TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID, TOOLBAR_UNDO_BUTTON_WIDGET_ID, inspector_field_index,
    outliner_row_index, viewport_product_button_index,
};

pub fn map_interactions_to_shell_commands(
    interactions: &UiInteractionResults,
    view_model: &EditorShellViewModel,
) -> Vec<ShellCommand> {
    let mut commands = Vec::new();

    for interaction in &interactions.items {
        if let UiInteraction::Activated(widget_id) = interaction {
            let toolbar_enabled = |stable_name: &str| {
                view_model
                    .toolbar
                    .buttons
                    .iter()
                    .find(|button| button.stable_name == stable_name)
                    .map(|button| button.enabled)
                    .unwrap_or(false)
            };

            let command = if *widget_id == TOOLBAR_SELECT_BUTTON_WIDGET_ID {
                ShellCommand::ActivateSelectTool
            } else if *widget_id == TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID {
                ShellCommand::ActivateTranslateTool
            } else if *widget_id == TOOLBAR_UNDO_BUTTON_WIDGET_ID {
                if toolbar_enabled("undo") {
                    ShellCommand::Undo
                } else {
                    ShellCommand::NoOp
                }
            } else if *widget_id == TOOLBAR_REDO_BUTTON_WIDGET_ID {
                if toolbar_enabled("redo") {
                    ShellCommand::Redo
                } else {
                    ShellCommand::NoOp
                }
            } else if *widget_id == TOOLBAR_SAVE_BUTTON_WIDGET_ID {
                if toolbar_enabled("save") {
                    ShellCommand::SaveScene
                } else {
                    ShellCommand::NoOp
                }
            } else if *widget_id == TOOLBAR_LOAD_BUTTON_WIDGET_ID {
                if toolbar_enabled("load") {
                    ShellCommand::LoadScene
                } else {
                    ShellCommand::NoOp
                }
            } else if *widget_id == TOOLBAR_DEBUG_LOGS_BUTTON_WIDGET_ID {
                ShellCommand::ToggleDebugLogs
            } else if let Some(index) = viewport_product_button_index(*widget_id) {
                view_model
                    .viewport
                    .product_choices
                    .get(index)
                    .map(|choice| {
                        if choice.enabled {
                            ShellCommand::SelectViewportProduct {
                                viewport_id: choice.viewport_id,
                                product_id: choice.product_id,
                            }
                        } else {
                            ShellCommand::NoOp
                        }
                    })
                    .unwrap_or(ShellCommand::NoOp)
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
