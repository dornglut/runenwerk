//! Generic menu definitions.

use crate::{
    availability::UiAvailabilityBinding, identity::UiNodeId, slot::UiRouteSlotRef,
    value::UiValueBinding,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiMenuItemDefinition {
    pub id: UiNodeId,
    pub label: UiValueBinding,
    pub route: Option<UiRouteSlotRef>,
    pub availability: Option<UiAvailabilityBinding>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiMenuDefinition {
    pub id: String,
    pub items: Vec<UiMenuItemDefinition>,
}
