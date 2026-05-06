//! File: domain/editor/editor_shell/src/commands/shell_command.rs
//! Purpose: Shell-level commands emitted from UI interactions.

use editor_core::{DocumentId, EntityId};
use editor_viewport::{ExpressionProductId, ViewportId};

use crate::{
    EntityTableSortKey, PanelInstanceId, TabStackId, ToolSurfaceInstanceId, ToolSurfaceKind,
    ToolbarCommandKind, ToolbarMenuKind, WorkspaceProfileId, WorkspaceSplitAxis,
};
use crate::{SurfaceLocalAction, SurfaceProviderId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralCommandTarget {
    pub panel_instance_id: PanelInstanceId,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
    pub tab_stack_id: TabStackId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabDropDestination {
    TabStack {
        tab_stack_id: TabStackId,
        insert_index: usize,
    },
    NewFloatingHost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellCommand {
    ActivateSelectTool,
    ActivateTranslateTool,
    ActivateRotateTool,
    ActivateScaleTool,
    ToggleToolbarMenu {
        menu: ToolbarMenuKind,
    },
    RunToolbarCommand {
        command: ToolbarCommandKind,
    },
    SwitchWorkspaceProfile {
        profile_id: WorkspaceProfileId,
    },
    Undo,
    Redo,
    SaveScene,
    LoadScene,
    ToggleDebugLogs,
    SetTabStackActivePanel {
        tab_stack_id: TabStackId,
        panel_instance_id: PanelInstanceId,
        projection_epoch: u64,
    },
    CommitTabDrop {
        panel_instance_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        destination: TabDropDestination,
        projection_epoch: u64,
    },
    SwitchPanelToolSurfaceKind {
        panel_instance_id: PanelInstanceId,
        tool_surface_kind: ToolSurfaceKind,
        projection_epoch: u64,
    },
    CreatePanelTab {
        tab_stack_id: TabStackId,
        tool_surface_kind: ToolSurfaceKind,
        projection_epoch: u64,
    },
    ClosePanelTab {
        tab_stack_id: TabStackId,
        panel_instance_id: PanelInstanceId,
        projection_epoch: u64,
    },
    CloseOtherPanelTabs {
        tab_stack_id: TabStackId,
        keep_panel_instance_id: PanelInstanceId,
        projection_epoch: u64,
    },
    SplitTabStackArea {
        tab_stack_id: TabStackId,
        axis: WorkspaceSplitAxis,
        tool_surface_kind: ToolSurfaceKind,
        projection_epoch: u64,
    },
    DuplicateTabStackArea {
        tab_stack_id: TabStackId,
        projection_epoch: u64,
    },
    CloseTabStackArea {
        tab_stack_id: TabStackId,
        projection_epoch: u64,
    },
    ResetTabStackArea {
        tab_stack_id: TabStackId,
        tool_surface_kind: ToolSurfaceKind,
        projection_epoch: u64,
    },
    LockTabStackAreaType {
        tab_stack_id: TabStackId,
        locked_tool_surface_kind: Option<ToolSurfaceKind>,
        projection_epoch: u64,
    },
    ActivateDocumentTab {
        document_id: DocumentId,
    },
    CloseDocumentTab {
        document_id: DocumentId,
    },
    SaveDocumentTab {
        document_id: DocumentId,
    },
    SelectEntityTableEntity {
        entity: EntityId,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    AppendEntityTableSearchText {
        text: String,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    BackspaceEntityTableSearch {
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    ToggleEntityTableSort {
        sort_key: EntityTableSortKey,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
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
    ToggleViewportDetails {
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    ActivateInspectorField {
        index: usize,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    FocusInspectorField {
        index: usize,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    AppendInspectorFieldText {
        index: usize,
        text: String,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    BackspaceInspectorFieldText {
        index: usize,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    CommitInspectorFieldText {
        index: usize,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    CancelInspectorFieldText {
        index: usize,
        target: StructuralCommandTarget,
        projection_epoch: u64,
    },
    DispatchSurfaceLocalAction {
        provider_id: SurfaceProviderId,
        tool_surface_instance_id: ToolSurfaceInstanceId,
        target: StructuralCommandTarget,
        action: SurfaceLocalAction,
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
            | Self::SetTabStackActivePanel {
                projection_epoch, ..
            }
            | Self::CommitTabDrop {
                projection_epoch, ..
            }
            | Self::SwitchPanelToolSurfaceKind {
                projection_epoch, ..
            }
            | Self::CreatePanelTab {
                projection_epoch, ..
            }
            | Self::ClosePanelTab {
                projection_epoch, ..
            }
            | Self::CloseOtherPanelTabs {
                projection_epoch, ..
            }
            | Self::SplitTabStackArea {
                projection_epoch, ..
            }
            | Self::DuplicateTabStackArea {
                projection_epoch, ..
            }
            | Self::CloseTabStackArea {
                projection_epoch, ..
            }
            | Self::ResetTabStackArea {
                projection_epoch, ..
            }
            | Self::LockTabStackAreaType {
                projection_epoch, ..
            }
            | Self::SelectEntityTableEntity {
                projection_epoch, ..
            }
            | Self::AppendEntityTableSearchText {
                projection_epoch, ..
            }
            | Self::BackspaceEntityTableSearch {
                projection_epoch, ..
            }
            | Self::ToggleEntityTableSort {
                projection_epoch, ..
            }
            | Self::SelectViewportProduct {
                projection_epoch, ..
            }
            | Self::ToggleViewportDetails {
                projection_epoch, ..
            }
            | Self::ActivateInspectorField {
                projection_epoch, ..
            }
            | Self::FocusInspectorField {
                projection_epoch, ..
            }
            | Self::AppendInspectorFieldText {
                projection_epoch, ..
            }
            | Self::BackspaceInspectorFieldText {
                projection_epoch, ..
            }
            | Self::CommitInspectorFieldText {
                projection_epoch, ..
            }
            | Self::CancelInspectorFieldText {
                projection_epoch, ..
            }
            | Self::DispatchSurfaceLocalAction {
                projection_epoch, ..
            } => Some(*projection_epoch),
            _ => None,
        }
    }
}
