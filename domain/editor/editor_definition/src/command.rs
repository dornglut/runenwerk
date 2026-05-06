//! Editor command route identifiers.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EditorRouteId(pub String);

impl EditorRouteId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for EditorRouteId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for EditorRouteId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorCommandRouteBinding {
    pub route: EditorRouteId,
    pub ui_route_slot: ui_definition::UiRouteSlotId,
}
