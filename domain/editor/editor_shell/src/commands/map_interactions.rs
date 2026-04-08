//! File: domain/editor/editor_shell/src/commands/map_interactions.rs
//! Purpose: Map semantic UI interactions to shell commands.

use ui_runtime::{UiInteraction, UiInteractionResults};

use crate::{
	ShellCommand, TOOLBAR_SELECT_BUTTON_WIDGET_ID, TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
};

pub fn map_interactions_to_shell_commands(
	interactions: &UiInteractionResults,
) -> Vec<ShellCommand> {
	let mut commands = Vec::new();

	for interaction in &interactions.items {
		if let UiInteraction::Activated(widget_id) = interaction {
			let command = if *widget_id == TOOLBAR_SELECT_BUTTON_WIDGET_ID {
				ShellCommand::ActivateSelectTool
			} else if *widget_id == TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID {
				ShellCommand::ActivateTranslateTool
			} else {
				ShellCommand::NoOp
			};

			commands.push(command);
		}
	}

	commands
}