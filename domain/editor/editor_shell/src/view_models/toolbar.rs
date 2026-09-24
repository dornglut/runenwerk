//! File: domain/editor/editor_shell/src/view_models/toolbar.rs
//! Purpose: Toolbar view model for first shell slice.

use editor_core::ToolId;

use crate::WorkspaceProfileId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarMenuKind {
    File,
    Edit,
    Window,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarCommandKind {
    SaveScene,
    SaveSceneAs,
    OpenScene,
    OpenRecent,
    Undo,
    Redo,
    EditPreferences,
    NewWindow,
    NextWorkspace,
    PreviousWorkspace,
    SaveWorkspace,
    LoadWorkspaceProfile(WorkspaceProfileId),
    LoadCustomWorkspace,
    AddWorkspace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolbarButtonViewModel {
    pub id: ToolId,
    pub stable_name: &'static str,
    pub label: String,
    pub is_active: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ToolbarViewModel {
    pub buttons: Vec<ToolbarButtonViewModel>,
}
