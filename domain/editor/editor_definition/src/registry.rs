//! Editor panel and tool-surface registry definition schemas.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorPanelRegistryDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub panels: Vec<EditorPanelDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorPanelDefinition {
    pub id: String,
    pub label: String,
    pub default_tool_surface: String,
    #[serde(default)]
    pub allowed_document_kinds: Vec<String>,
    #[serde(default)]
    pub allowed_workspace_profiles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorToolSurfaceRegistryDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub tool_surfaces: Vec<crate::EditorToolSurfaceDefinition>,
}
