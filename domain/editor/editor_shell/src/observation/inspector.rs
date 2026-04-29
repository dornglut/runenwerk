//! File: domain/editor/editor_shell/src/observation/inspector.rs
//! Purpose: Inspector observation frame contracts.

use crate::ObservationFrameMetadata;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorObservedField {
    pub label: String,
    pub path_key: Option<String>,
    pub value_summary: String,
    pub is_focused: bool,
    pub editable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorObservedTarget {
    Empty,
    Entity {
        display_name: String,
    },
    Component {
        entity_display_name: String,
        component_display_name: String,
    },
    Resource {
        display_name: String,
    },
    Unsupported {
        label: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorObservationFrame {
    pub metadata: ObservationFrameMetadata,
    pub target: InspectorObservedTarget,
    pub fields: Vec<InspectorObservedField>,
}
