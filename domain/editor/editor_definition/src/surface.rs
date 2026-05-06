//! Editor surface template bindings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorSurfaceTemplateKind {
    ShellChrome,
    Inspector,
    Outliner,
    EntityTable,
    Console,
    Viewport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorSurfaceTemplateBinding {
    pub kind: EditorSurfaceTemplateKind,
    pub template: ui_definition::UiTemplateId,
    pub provider_slots: Vec<String>,
}
