//! File: domain/ui/ui_controls/src/label/mod.rs
//! Crate: ui_controls

use ui_program::RouteCapability;
use ui_schema::{UiSchema, UiSchemaShape};

use crate::{ControlModuleDescriptor, RUNENWERK_CONTROL_PACKAGE_ID};

pub const LABEL_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.label";

pub fn control_module() -> ControlModuleDescriptor {
    crate::control_module_contract(
        "label",
        "Label",
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.label.properties"),
            1,
        )
        .with_required_field("text", UiSchemaShape::String)
        .with_optional_field("style_slot", UiSchemaShape::String),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.label.state"), 1)
            .with_optional_field("measured_width", UiSchemaShape::Number)
            .with_optional_field("measured_height", UiSchemaShape::Number),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.label.event"), 1)
            .with_optional_field("route", UiSchemaShape::RouteRef),
        RouteCapability::new("runenwerk.ui.controls.read"),
    )
}
