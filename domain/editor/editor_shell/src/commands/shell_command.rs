//! File: domain/editor/editor_shell/src/commands/shell_command.rs
//! Purpose: Shell-level commands emitted from UI interactions.

use editor_core::EntityId;
use editor_viewport::{ExpressionProductId, ViewportId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellCommand {
    ActivateSelectTool,
    ActivateTranslateTool,
    Undo,
    Redo,
    SaveScene,
    LoadScene,
    ToggleDebugLogs,
    SelectOutlinerEntity {
        entity: EntityId,
    },
    SelectViewportProduct {
        viewport_id: ViewportId,
        product_id: ExpressionProductId,
    },
    ToggleViewportDetails,
    ActivateInspectorField {
        index: usize,
    },
    NoOp,
}
