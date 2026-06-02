//! File: domain/ui/ui_controls/src/inspector_field/mod.rs
//! Crate: ui_controls

use ui_program::RouteCapability;
use ui_schema::{UiSchema, UiSchemaShape};

use crate::{ControlModuleDescriptor, RUNENWERK_CONTROL_PACKAGE_ID};

pub const INSPECTOR_FIELD_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.inspector-field";

pub fn control_module() -> ControlModuleDescriptor {
    crate::control_module_contract(
        "inspector-field",
        "InspectorField",
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.inspector-field.properties"),
            1,
        )
        .with_required_field("label", UiSchemaShape::String)
        .with_required_field("binding", UiSchemaShape::StableIdRef),
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.inspector-field.state"),
            1,
        )
        .with_optional_field("preview_value", UiSchemaShape::Object)
        .with_optional_field("dirty", UiSchemaShape::Bool),
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.inspector-field.event"),
            1,
        )
        .with_required_field("route", UiSchemaShape::RouteRef)
        .with_required_field("value", UiSchemaShape::Object),
        RouteCapability::new("runenwerk.ui.controls.inspect"),
    )
}
