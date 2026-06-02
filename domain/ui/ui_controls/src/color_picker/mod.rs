//! File: domain/ui/ui_controls/src/color_picker/mod.rs
//! Crate: ui_controls

use ui_program::RouteCapability;
use ui_schema::{UiSchema, UiSchemaShape};

use crate::{ControlModuleDescriptor, RUNENWERK_CONTROL_PACKAGE_ID};

pub const COLOR_PICKER_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.color-picker";

pub fn control_module() -> ControlModuleDescriptor {
    crate::control_module_contract(
        "color-picker",
        "ColorPicker",
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.color-picker.properties"),
            1,
        )
        .with_required_field("committed_rgba", UiSchemaShape::Object)
        .with_optional_field("allow_alpha", UiSchemaShape::Bool),
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.color-picker.state"),
            1,
        )
        .with_required_field("hue_degrees", UiSchemaShape::Number)
        .with_required_field("triangle_saturation", UiSchemaShape::Number)
        .with_required_field("triangle_value", UiSchemaShape::Number)
        .with_required_field("preview_rgba", UiSchemaShape::Object),
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.color-picker.event"),
            1,
        )
        .with_required_field("route", UiSchemaShape::RouteRef)
        .with_required_field("rgba", UiSchemaShape::Object)
        .with_required_field("preview", UiSchemaShape::Bool)
        .with_required_field("committed", UiSchemaShape::Bool),
        RouteCapability::new("runenwerk.ui.controls.color.write"),
    )
}
