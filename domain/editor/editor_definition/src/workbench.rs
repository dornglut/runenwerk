//! Workbench composition definition contracts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorWorkbenchCompositionDefinition {
    pub id: String,
    pub label: String,
    pub installed_suites: Vec<String>,
    pub profile_refs: Vec<String>,
    pub default_profile_ref: String,
    #[serde(default)]
    pub host_policy: EditorWorkbenchHostPolicyDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EditorWorkbenchHostPolicyDefinition {
    AllowAll,
    #[default]
    DenyAll,
    Explicit {
        #[serde(default)]
        allow_all: bool,
        #[serde(default)]
        allowed_commands: Vec<String>,
        #[serde(default)]
        denied_commands: Vec<String>,
        #[serde(default)]
        allowed_products: Vec<String>,
        #[serde(default)]
        denied_products: Vec<String>,
        #[serde(default)]
        allowed_resources: Vec<String>,
        #[serde(default)]
        denied_resources: Vec<String>,
    },
}
