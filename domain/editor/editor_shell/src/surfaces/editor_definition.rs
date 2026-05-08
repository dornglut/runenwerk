//! File: domain/editor/editor_shell/src/surfaces/editor_definition.rs
//! Purpose: Editor-definition self-authoring surface workflow contracts.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorDefinitionSurfaceAction {
    SelectDocument { document_id: String },
    DuplicateSelected,
    RenameSelected { display_name: String },
    DeleteSelected,
    ExportSelected,
    ApplySelected,
    RollbackSelected,
    SelectUiNode { node_id: String },
    SetUiNodeText { node_id: String, text: String },
    SetThemeColor { token: String, value: String },
    AddWorkspaceLayoutTab { label: String, tool_surface: String },
    SplitWorkspaceLayoutRoot { axis: String },
    CloseWorkspaceLayoutLastTab,
}
