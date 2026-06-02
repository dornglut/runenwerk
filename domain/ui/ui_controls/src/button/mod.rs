//! File: domain/ui/ui_controls/src/button/mod.rs
//! Crate: ui_controls

use ui_program::RouteCapability;
use ui_schema::{UiSchema, UiSchemaShape};

use crate::{ControlModuleDescriptor, RUNENWERK_CONTROL_PACKAGE_ID};

pub const BUTTON_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.button";

pub fn control_module() -> ControlModuleDescriptor {
    crate::control_module_contract(
        "button",
        "Button",
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.button.properties"),
            1,
        )
        .with_required_field("label", UiSchemaShape::String)
        .with_optional_field("disabled", UiSchemaShape::Bool),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.button.state"), 1)
            .with_optional_field("pressed", UiSchemaShape::Bool)
            .with_optional_field("focused", UiSchemaShape::Bool),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.button.event"), 1)
            .with_required_field("route", UiSchemaShape::RouteRef)
            .with_required_field("activated", UiSchemaShape::Bool),
        RouteCapability::new("runenwerk.ui.controls.activate"),
    )
}
