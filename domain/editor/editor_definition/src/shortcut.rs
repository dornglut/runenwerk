//! Editor shortcut definition schemas.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorShortcutSetDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub shortcuts: Vec<EditorShortcutDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorShortcutDefinition {
    pub id: String,
    pub command: String,
    pub chord: String,
    #[serde(default)]
    pub context: Option<String>,
}
