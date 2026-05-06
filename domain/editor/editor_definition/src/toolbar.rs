//! Editor toolbar definition bindings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorToolbarBinding {
    pub template: ui_definition::UiTemplateId,
    pub workspace_catalog: Option<crate::EditorWorkspaceCatalogDefinition>,
    pub routes: Vec<crate::EditorCommandRouteBinding>,
    pub availability: Vec<crate::EditorAvailabilityBinding>,
    pub menus: Vec<crate::EditorMenuBinding>,
}
