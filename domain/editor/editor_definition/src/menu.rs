//! Editor menu definitions and bindings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMenuBinding {
    pub menu_id: String,
    pub menu_slot: ui_definition::UiMenuSlotId,
}
