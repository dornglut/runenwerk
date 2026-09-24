//! Editor workspace catalog definitions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorWorkspaceSplitAxisDefinition {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorWorkspaceCatalogEntry {
    pub id: String,
    pub label: String,
    pub route: ui_definition::UiRouteSlotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorWorkspaceCatalogDefinition {
    pub collection_slot: ui_definition::UiCollectionSlotId,
    pub active_selection_slot: ui_definition::UiSelectionSlotId,
    pub entries: Vec<EditorWorkspaceCatalogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorWorkspaceProfileDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub default_modes: Vec<String>,
    #[serde(default)]
    pub document_kind_filters: Vec<String>,
    pub default_layout: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorWorkspaceLayoutDefinition {
    pub id: String,
    pub label: String,
    pub root: EditorWorkspaceHostDefinition,
    #[serde(default)]
    pub floating_hosts: Vec<EditorWorkspaceFloatingHostDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditorWorkspaceHostDefinition {
    Split {
        id: String,
        axis: EditorWorkspaceSplitAxisDefinition,
        fraction: f32,
        first: Box<EditorWorkspaceHostDefinition>,
        second: Box<EditorWorkspaceHostDefinition>,
    },
    TabStack {
        id: String,
        tabs: Vec<EditorWorkspacePanelTabDefinition>,
        active_tab: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorWorkspacePanelTabDefinition {
    pub id: String,
    pub label: String,
    pub tool_surface: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorWorkspaceFloatingHostDefinition {
    pub id: String,
    pub host: EditorWorkspaceHostDefinition,
}
