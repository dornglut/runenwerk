//! Top-level editor definition binding document.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorDefinitionBindings {
    pub toolbar: crate::EditorToolbarBinding,
    pub shell_chrome_template: ui_definition::UiTemplateId,
    pub surface_templates: Vec<crate::EditorSurfaceTemplateBinding>,
}
