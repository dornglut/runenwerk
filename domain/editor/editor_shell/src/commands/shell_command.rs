//! File: domain/editor/editor_shell/src/commands/shell_command.rs
//! Purpose: Shell-level commands emitted from UI interactions.

use editor_core::DocumentId;
use ui_composition::{MountedUnitId, RegionId, SplitFraction, StateRevision};

use crate::{
    DockSplitSide, EditorDomainMutation, MaterialSurfaceAction, PanelHostId, PanelInstanceId,
    SurfaceSessionMutation, TabStackId, TextureSurfaceAction, ToolSurfaceInstanceId,
    ToolSurfaceStableKey, ToolbarCommandKind, ToolbarMenuKind, WidgetId, WorkspaceProfileId,
    WorkspaceSplitAxis,
};
use crate::{SurfaceInteraction, SurfaceLocalAction, SurfaceProviderId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralCommandTarget {
    pub mounted_unit_id: Option<MountedUnitId>,
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
    SplitIntoArea {
        target_tab_stack_id: TabStackId,
        side: DockSplitSide,
    },
    SplitIntoHost {
        target_host_id: PanelHostId,
        side: DockSplitSide,
    },
    SplitIntoRoot {
        side: DockSplitSide,
    },
    NewFloatingHost,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShellCommand {
    ActivateSelectTool,
    ActivateTranslateTool,
    ActivateRotateTool,
    ActivateScaleTool,
    ToggleToolbarMenu {
        menu: ToolbarMenuKind,
    },
    ToggleTabStackActionMenu {
        tab_stack_id: TabStackId,
        anchor_widget_id: WidgetId,
    },
    ToggleTabStackSurfaceMenu {
        tab_stack_id: TabStackId,
        anchor_widget_id: WidgetId,
    },
    ToggleTabStackCreateSurfaceMenu {
        tab_stack_id: TabStackId,
        anchor_widget_id: WidgetId,
    },
    RunToolbarCommand {
        command: ToolbarCommandKind,
    },
    SwitchWorkspaceProfile {
        profile_id: WorkspaceProfileId,
    },
    CloseWorkspaceProfile {
        profile_id: WorkspaceProfileId,
    },
    Undo,
    Redo,
    UndoCompositionLayout,
    RedoCompositionLayout,
    SaveScene,
    LoadScene,
    ToggleDebugLogs,
    SelectAsset {
        asset_id: asset::AssetId,
        projection_epoch: u64,
    },
    LoadAssetCatalog {
        projection_epoch: u64,
    },
    SaveAssetCatalog {
        projection_epoch: u64,
    },
    ReimportAsset {
        asset_id: asset::AssetId,
        projection_epoch: u64,
    },
    ReimportSelectedAsset {
        projection_epoch: u64,
    },
    ClearAssetDiagnostics {
        projection_epoch: u64,
    },
    SelectMaterialAsset {
        asset_id: asset::AssetId,
        projection_epoch: u64,
    },
    BuildMaterialPreview {
        asset_id: asset::AssetId,
        projection_epoch: u64,
    },
    BuildSelectedMaterialPreview {
        projection_epoch: u64,
    },
    ClearMaterialDiagnostics {
        projection_epoch: u64,
    },
    ApplyMaterialSurfaceAction {
        action: MaterialSurfaceAction,
        projection_epoch: u64,
    },
    ApplyTextureSurfaceAction {
        action: TextureSurfaceAction,
        projection_epoch: u64,
    },
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
    CommitCompositionDock {
        intent: crate::EditorDockingIntent,
        projection_epoch: u64,
    },
    ResizeCompositionSplit {
        split: RegionId,
        fraction: SplitFraction,
        expected_revision: StateRevision,
        projection_epoch: u64,
    },
    CreatePanelTabStableKey {
        tab_stack_id: TabStackId,
        panel_kind: crate::PanelKind,
        stable_surface_key: ToolSurfaceStableKey,
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
    SplitTabStackAreaStableKey {
        tab_stack_id: TabStackId,
        axis: WorkspaceSplitAxis,
        panel_kind: crate::PanelKind,
        stable_surface_key: ToolSurfaceStableKey,
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
    ResetTabStackAreaStableKey {
        tab_stack_id: TabStackId,
        panel_kind: crate::PanelKind,
        stable_surface_key: ToolSurfaceStableKey,
        projection_epoch: u64,
    },
    LockTabStackAreaStableKey {
        tab_stack_id: TabStackId,
        locked_stable_surface_key: Option<ToolSurfaceStableKey>,
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
    SelectEditorDefinitionDocument {
        document_id: String,
    },
    DuplicateSelectedEditorDefinition,
    RenameSelectedEditorDefinition {
        display_name: String,
    },
    DeleteSelectedEditorDefinition,
    SaveEditorLabProjectPackage,
    ReloadEditorLabProjectPackage,
    ExportSelectedEditorDefinition,
    CreateEditorWorkbenchCompositionPackage,
    BuildSelectedEditorDefinitionApplyReview,
    RejectSelectedEditorDefinitionApplyReview,
    ApplySelectedEditorDefinition,
    ActivateSelectedEditorWorkbenchComposition,
    RollbackSelectedEditorDefinition,
    ReloadSelectedEditorDefinitionLastApplied,
    ApplyEditorLabOperation {
        operation: editor_definition::EditorLabOperation,
    },
    UndoEditorLabOperation,
    RedoEditorLabOperation,
    SelectEditorDefinitionUiNode {
        node_id: String,
    },
    InsertSelectedEditorDefinitionRecipe {
        recipe_id: String,
    },
    SetEditorDefinitionRecipeCatalogFilter {
        query: String,
    },
    CaptureUiDesignerScenarioEvidence,
    SetSelectedEditorDefinitionUiNodeText {
        node_id: String,
        text: String,
    },
    SetSelectedEditorThemeColor {
        token: String,
        value: String,
    },
    SetSelectedWorkbenchInstalledSuites {
        installed_suites: String,
    },
    SetSelectedWorkbenchProfileRefs {
        profile_refs: String,
    },
    SetSelectedWorkbenchDefaultProfileRef {
        profile_ref: String,
    },
    AddSelectedEditorWorkspaceLayoutTab {
        label: String,
        tool_surface: String,
    },
    SplitSelectedEditorWorkspaceLayoutRoot {
        axis: String,
    },
    CloseSelectedEditorWorkspaceLayoutLastTab,
    ApplySurfaceSessionMutation {
        target: StructuralCommandTarget,
        mutation: SurfaceSessionMutation,
        projection_epoch: u64,
    },
    ApplyEditorDomainMutation {
        target: StructuralCommandTarget,
        mutation: EditorDomainMutation,
        projection_epoch: u64,
    },
    DispatchSurfaceLocalAction {
        provider_id: SurfaceProviderId,
        tool_surface_instance_id: ToolSurfaceInstanceId,
        target: StructuralCommandTarget,
        action: SurfaceLocalAction,
        projection_epoch: u64,
    },
    DispatchSurfaceInteraction {
        provider_id: SurfaceProviderId,
        tool_surface_instance_id: ToolSurfaceInstanceId,
        target: StructuralCommandTarget,
        interaction: SurfaceInteraction,
        projection_epoch: u64,
    },
    NoOp,
}

impl ShellCommand {
    pub fn projection_epoch(&self) -> Option<u64> {
        match self {
            Self::SetTabStackActivePanel {
                projection_epoch, ..
            }
            | Self::CommitTabDrop {
                projection_epoch, ..
            }
            | Self::CommitCompositionDock {
                projection_epoch, ..
            }
            | Self::ResizeCompositionSplit {
                projection_epoch, ..
            }
            | Self::CreatePanelTabStableKey {
                projection_epoch, ..
            }
            | Self::ClosePanelTab {
                projection_epoch, ..
            }
            | Self::CloseOtherPanelTabs {
                projection_epoch, ..
            }
            | Self::SplitTabStackAreaStableKey {
                projection_epoch, ..
            }
            | Self::DuplicateTabStackArea {
                projection_epoch, ..
            }
            | Self::CloseTabStackArea {
                projection_epoch, ..
            }
            | Self::ResetTabStackAreaStableKey {
                projection_epoch, ..
            }
            | Self::LockTabStackAreaStableKey {
                projection_epoch, ..
            }
            | Self::SelectAsset {
                projection_epoch, ..
            }
            | Self::LoadAssetCatalog { projection_epoch }
            | Self::SaveAssetCatalog { projection_epoch }
            | Self::ReimportAsset {
                projection_epoch, ..
            }
            | Self::ReimportSelectedAsset { projection_epoch }
            | Self::ClearAssetDiagnostics { projection_epoch }
            | Self::SelectMaterialAsset {
                projection_epoch, ..
            }
            | Self::BuildMaterialPreview {
                projection_epoch, ..
            }
            | Self::BuildSelectedMaterialPreview { projection_epoch }
            | Self::ClearMaterialDiagnostics { projection_epoch }
            | Self::ApplyMaterialSurfaceAction {
                projection_epoch, ..
            }
            | Self::ApplyTextureSurfaceAction {
                projection_epoch, ..
            }
            | Self::ApplySurfaceSessionMutation {
                projection_epoch, ..
            }
            | Self::ApplyEditorDomainMutation {
                projection_epoch, ..
            }
            | Self::DispatchSurfaceLocalAction {
                projection_epoch, ..
            }
            | Self::DispatchSurfaceInteraction {
                projection_epoch, ..
            } => Some(*projection_epoch),
            _ => None,
        }
    }
}
