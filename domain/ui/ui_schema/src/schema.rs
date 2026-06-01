//! File: domain/ui/ui_schema/src/schema.rs
//! Crate: ui_schema

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UiSchemaId(String);

impl UiSchemaId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiSchemaVersion(u32);

impl UiSchemaVersion {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UiSchemaRef {
    pub id: UiSchemaId,
    pub version: UiSchemaVersion,
}

impl UiSchemaRef {
    pub fn new(id: impl Into<String>, version: u32) -> Self {
        Self {
            id: UiSchemaId::new(id),
            version: UiSchemaVersion::new(version),
        }
    }
}
