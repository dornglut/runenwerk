//! File: domain/editor/editor_ui/src/shortcut.rs
//! Purpose: Editor UI shortcut bindings to editor commands/tools.

use editor_core::{CommandId, ToolId};
use ui_input::Shortcut;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutAction {
	RunCommand(CommandId),
	ActivateTool(ToolId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBinding {
	pub shortcut: Shortcut,
	pub action: ShortcutAction,
}

impl ShortcutBinding {
	pub fn new(shortcut: Shortcut, action: ShortcutAction) -> Self {
		Self { shortcut, action }
	}
}