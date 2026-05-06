//! Editor workspace catalog definitions.

use serde::{Deserialize, Serialize};

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
