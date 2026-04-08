//! File: domain/editor/editor_inspector/src/model/node.rs
//! Purpose: Inspector field/node model.

use crate::{InspectorPath, InspectorValue, ValidationMessage};

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorField {
    pub stable_name: String,
    pub display_name: String,
    pub path: InspectorPath,
    pub value: InspectorValue,
    pub is_read_only: bool,
    pub validation: Vec<ValidationMessage>,
    pub children: Vec<InspectorField>,
}

impl InspectorField {
    /// File: domain/editor/editor_inspector/src/model/node.rs
    /// Method: new
    pub fn new(
        stable_name: impl Into<String>,
        display_name: impl Into<String>,
        path: InspectorPath,
        value: InspectorValue,
    ) -> Self {
        Self {
            stable_name: stable_name.into(),
            display_name: display_name.into(),
            path,
            value,
            is_read_only: false,
            validation: Vec::new(),
            children: Vec::new(),
        }
    }

    /// File: domain/editor/editor_inspector/src/model/node.rs
    /// Method: read_only
    pub fn read_only(mut self, is_read_only: bool) -> Self {
        self.is_read_only = is_read_only;
        self
    }

    /// File: domain/editor/editor_inspector/src/model/node.rs
    /// Method: with_validation
    pub fn with_validation(mut self, message: ValidationMessage) -> Self {
        self.validation.push(message);
        self
    }

    /// File: domain/editor/editor_inspector/src/model/node.rs
    /// Method: with_child
    pub fn with_child(mut self, child: InspectorField) -> Self {
        self.children.push(child);
        self
    }
}
