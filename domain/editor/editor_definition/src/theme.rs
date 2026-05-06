//! Editor-owned theme definition schemas.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorThemeDefinition {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub colors: BTreeMap<String, String>,
    #[serde(default)]
    pub spacing: BTreeMap<String, f32>,
    #[serde(default)]
    pub typography: BTreeMap<String, EditorTypographyTokenDefinition>,
    #[serde(default)]
    pub radius: BTreeMap<String, f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorTypographyTokenDefinition {
    pub font_family: String,
    pub size: f32,
    pub weight: u16,
}
