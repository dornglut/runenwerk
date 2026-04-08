//! File: domain/editor/editor_shell/src/commands/shell_command.rs
//! Purpose: Shell-level commands emitted from UI interactions.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellCommand {
	ActivateSelectTool,
	ActivateTranslateTool,
	NoOp,
}