//! Editor toolbar definition bindings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorToolbarBinding {
    pub template: ui_definition::UiTemplateId,
    pub workspace_catalog: Option<crate::EditorWorkspaceCatalogDefinition>,
    pub routes: Vec<crate::EditorCommandRouteBinding>,
    pub availability: Vec<crate::EditorAvailabilityBinding>,
    pub menus: Vec<crate::EditorMenuBinding>,
    #[serde(default)]
    pub menu_items: Vec<EditorToolbarMenuItemBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorToolbarMenuItemBinding {
    pub menu_id: String,
    pub item_id: String,
    pub label: String,
    pub route: ui_definition::UiRouteSlotId,
    #[serde(default)]
    pub availability: Option<ui_definition::UiAvailabilityBinding>,
}
