//! Editor surface template bindings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorSurfaceTemplateKind {
    ShellChrome,
    Inspector,
    Outliner,
    EntityTable,
    Console,
    Viewport,
    EditorDesignOutliner,
    UiHierarchy,
    UiCanvas,
    StyleInspector,
    Bindings,
    DockLayoutPreview,
    ThemeEditor,
    ShortcutEditor,
    MenuEditor,
    DefinitionValidation,
    CommandDiff,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorSurfaceTemplateBinding {
    pub kind: EditorSurfaceTemplateKind,
    pub template: ui_definition::UiTemplateId,
    pub provider_slots: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorToolSurfaceDefinition {
    pub id: String,
    pub label: String,
    pub provider_family: String,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub allowed_document_kinds: Vec<String>,
    #[serde(default)]
    pub allowed_workspace_profiles: Vec<String>,
}
