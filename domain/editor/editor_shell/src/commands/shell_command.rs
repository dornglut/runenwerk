//! File: domain/editor/editor_shell/src/commands/shell_command.rs
//! Purpose: Shell-level commands emitted from UI interactions.

use editor_core::EntityId;
use editor_viewport::{ExpressionProductId, ViewportId};

use crate::{PanelInstanceId, TabStackId, ToolSurfaceInstanceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralCommandTarget {
    pub panel_instance_id: PanelInstanceId,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
    pub tab_stack_id: TabStackId,
}

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
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    SelectViewportProduct {
        viewport_id: ViewportId,
        product_id: ExpressionProductId,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    ToggleViewportDetails,
    ActivateInspectorField {
        index: usize,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    NoOp,
}

impl ShellCommand {
    pub fn projection_epoch(&self) -> Option<u64> {
        match self {
            Self::SelectOutlinerEntity {
                projection_epoch, ..
            }
            | Self::SelectViewportProduct {
                projection_epoch, ..
            }
            | Self::ActivateInspectorField {
                projection_epoch, ..
            } => Some(*projection_epoch),
            _ => None,
        }
    }
}
