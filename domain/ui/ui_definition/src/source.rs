//! Authored source map metadata.

use crate::identity::AuthoredUiNodePath;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSourceLocation {
    pub source_name: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiSourceMap {
    pub locations_by_path: BTreeMap<AuthoredUiNodePath, UiSourceLocation>,
}
