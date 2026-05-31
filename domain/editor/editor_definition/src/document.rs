//! Durable editor-definition document lifecycle schemas.

use crate::{
    EditorCommandBindingSetDefinition, EditorDefinitionBindings, EditorMenuDefinition,
    EditorPanelRegistryDefinition, EditorShortcutSetDefinition, EditorThemeDefinition,
    EditorToolSurfaceRegistryDefinition, EditorWorkbenchCompositionDefinition,
    EditorWorkspaceLayoutDefinition, EditorWorkspaceProfileDefinition,
};
use serde::{Deserialize, Serialize};
use ui_definition::AuthoredUiTemplate;

pub const CURRENT_EDITOR_DEFINITION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EditorDefinitionId(pub String);

impl EditorDefinitionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for EditorDefinitionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorDefinitionLifecycleState {
    Draft,
    Validated,
    Applied,
    RolledBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorDefinitionDocumentKind {
    UiLayout,
    WorkspaceDefinition,
    Theme,
    Shortcut,
    Menu,
    CommandBinding,
    PanelRegistry,
    ToolSurfaceDefinition,
    WorkbenchComposition,
    EditorBindings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorDefinitionDocument {
    pub schema_version: u32,
    pub id: EditorDefinitionId,
    pub display_name: String,
    pub kind: EditorDefinitionDocumentKind,
    pub lifecycle_state: EditorDefinitionLifecycleState,
    pub content: EditorDefinitionDocumentContent,
}

impl EditorDefinitionDocument {
    pub fn current(
        id: EditorDefinitionId,
        display_name: impl Into<String>,
        kind: EditorDefinitionDocumentKind,
        content: EditorDefinitionDocumentContent,
    ) -> Self {
        Self {
            schema_version: CURRENT_EDITOR_DEFINITION_SCHEMA_VERSION,
            id,
            display_name: display_name.into(),
            kind,
            lifecycle_state: EditorDefinitionLifecycleState::Draft,
            content,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditorDefinitionDocumentContent {
    UiTemplate(AuthoredUiTemplate),
    WorkspaceProfile(EditorWorkspaceProfileDefinition),
    WorkspaceLayout(EditorWorkspaceLayoutDefinition),
    Menu(EditorMenuDefinition),
    Theme(EditorThemeDefinition),
    Shortcuts(EditorShortcutSetDefinition),
    CommandBindings(EditorCommandBindingSetDefinition),
    PanelRegistry(EditorPanelRegistryDefinition),
    ToolSurfaceRegistry(EditorToolSurfaceRegistryDefinition),
    WorkbenchComposition(EditorWorkbenchCompositionDefinition),
    EditorBindings(EditorDefinitionBindings),
}
