//! File: domain/ui/ui_controls/src/action_prompt/mod.rs
//! Crate: ui_controls

use ui_program::RouteCapability;
use ui_schema::{UiSchema, UiSchemaShape};

use crate::{ControlModuleDescriptor, RUNENWERK_CONTROL_PACKAGE_ID};

pub const ACTION_PROMPT_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.action-prompt";

pub fn control_module() -> ControlModuleDescriptor {
    crate::control_module_contract(
        "action-prompt",
        "ActionPrompt",
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.action-prompt.properties"),
            1,
        )
        .with_required_field("title", UiSchemaShape::String)
        .with_required_field("primary_route", UiSchemaShape::RouteRef),
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.action-prompt.state"),
            1,
        )
        .with_optional_field("open", UiSchemaShape::Bool)
        .with_optional_field("focused_action", UiSchemaShape::String),
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.action-prompt.event"),
            1,
        )
        .with_required_field("route", UiSchemaShape::RouteRef)
        .with_required_field("accepted", UiSchemaShape::Bool),
        RouteCapability::new("runenwerk.ui.controls.prompt.answer"),
    )
}
