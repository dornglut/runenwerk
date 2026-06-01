//! File: domain/ui/ui_program/src/events/packet.rs
//! Crate: ui_program

use serde::{Deserialize, Serialize};
use ui_schema::{UiSchemaRef, UiSchemaValue};

use crate::events::payload::UiEventPayload;
use crate::events::route::{RouteCapability, RouteId, RouteSchemaVersion};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiEventPacket {
    pub route: RouteId,
    pub schema_version: RouteSchemaVersion,
    pub payload: UiEventPayload,
    pub capabilities: Vec<RouteCapability>,
}

impl UiEventPacket {
    pub fn new(
        route: RouteId,
        schema_version: RouteSchemaVersion,
        payload_schema: UiSchemaRef,
        payload: UiSchemaValue,
    ) -> Self {
        Self {
            route,
            schema_version,
            payload: UiEventPayload::new(payload_schema, payload),
            capabilities: Vec::new(),
        }
    }

    pub fn with_capability(mut self, capability: RouteCapability) -> Self {
        self.capabilities.push(capability);
        self
    }
}
