//! File: domain/editor/editor_shell/src/view_models/inspector.rs
//! Purpose: Inspector shell view model.

use crate::InspectorFieldControlKind;

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorFieldViewModel {
    pub label: String,
    pub path_key: Option<String>,
    pub value_summary: String,
    pub control: InspectorFieldControlKind,
    pub is_focused: bool,
    pub editable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorTargetViewModel {
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

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorViewModel {
    pub target: InspectorTargetViewModel,
    pub fields: Vec<InspectorFieldViewModel>,
}

impl Default for InspectorViewModel {
    fn default() -> Self {
        Self {
            target: InspectorTargetViewModel::Empty,
            fields: Vec::new(),
        }
    }
}
