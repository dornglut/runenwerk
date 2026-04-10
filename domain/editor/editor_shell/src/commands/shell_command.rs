//! File: domain/editor/editor_shell/src/commands/shell_command.rs
//! Purpose: Shell-level commands emitted from UI interactions.

use editor_core::EntityId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellCommand {
    ActivateSelectTool,
    ActivateTranslateTool,
    Undo,
    Redo,
    SaveScene,
    LoadScene,
    ToggleDebugLogs,
    SelectOutlinerEntity { entity: EntityId },
    ActivateInspectorField { index: usize },
    NoOp,
}
