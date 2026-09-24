//! Editor menu definitions and bindings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMenuBinding {
    pub menu_id: String,
    pub menu_slot: ui_definition::UiMenuSlotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMenuDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub items: Vec<EditorMenuItemDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMenuItemDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub children: Vec<EditorMenuItemDefinition>,
    #[serde(default)]
    pub availability: Option<String>,
}
