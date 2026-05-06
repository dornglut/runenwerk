//! Editor availability descriptors.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAvailabilityDescriptor {
    Always,
    RequiresActiveDocument,
    CanUndo,
    CanRedo,
    StaticDisabled { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorAvailabilityBinding {
    pub availability: ui_definition::UiAvailabilityId,
    pub descriptor: EditorAvailabilityDescriptor,
}
