//! File: domain/ui/ui_program/src/events/payload.rs
//! Crate: ui_program

use serde::{Deserialize, Serialize};
use ui_schema::{UiSchemaRef, UiSchemaValue};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiEventPayload {
    pub schema: UiSchemaRef,
    pub value: UiSchemaValue,
}

impl UiEventPayload {
    pub fn new(schema: UiSchemaRef, value: UiSchemaValue) -> Self {
        Self { schema, value }
    }
}
