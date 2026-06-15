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
        .with_optional_field(
            "variant",
            UiSchemaShape::string_enum([
                "toolbar",
                "primary",
                "secondary",
                "ghost",
                "danger",
                "inspector",
                "icon",
            ]),
        )
        .with_optional_field(
            "tone",
            UiSchemaShape::string_enum(["neutral", "accent", "success", "warning", "danger"]),
        )
        .with_optional_field(
            "density",
            UiSchemaShape::string_enum(["compact", "normal", "spacious"]),
        )
        .with_optional_field("size", UiSchemaShape::string_enum(["xs", "sm", "md", "lg"]))
        .with_optional_field("leading_icon", UiSchemaShape::String)
        .with_optional_field("trailing_icon", UiSchemaShape::String)
        .with_optional_field("show_label", UiSchemaShape::Bool)
        .with_optional_field("tooltip", UiSchemaShape::String)
        .with_optional_field("disabled", UiSchemaShape::Bool),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.button.state"), 1)
            .with_optional_field("selected", UiSchemaShape::Bool)
            .with_optional_field("pressed", UiSchemaShape::Bool)
            .with_optional_field("focused", UiSchemaShape::Bool),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.button.event"), 1)
            .with_required_field("route", UiSchemaShape::RouteRef)
            .with_required_field("activated", UiSchemaShape::Bool),
        RouteCapability::new("runenwerk.ui.controls.activate"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_schema::UiSchemaValue;

    #[test]
    fn button_property_schema_accepts_final_shape_style_axes() {
        let schema = button_property_schema();

        let value = UiSchemaValue::object([
            ("label", UiSchemaValue::string("Press me")),
            ("variant", UiSchemaValue::string("primary")),
            ("tone", UiSchemaValue::string("accent")),
            ("density", UiSchemaValue::string("normal")),
            ("size", UiSchemaValue::string("md")),
            (
                "leading_icon",
                UiSchemaValue::string("runenwerk.icons.play"),
            ),
            (
                "trailing_icon",
                UiSchemaValue::string("runenwerk.icons.chevron_right"),
            ),
            ("show_label", UiSchemaValue::bool(true)),
        ]);

        let report = schema.validate(&value);

        assert!(report.is_valid(), "{:?}", report.diagnostics);
    }

    #[test]
    fn button_property_schema_rejects_invalid_density_and_size() {
        let schema = button_property_schema();

        let value = UiSchemaValue::object([
            ("label", UiSchemaValue::string("Broken")),
            ("density", UiSchemaValue::string("tiny")),
            ("size", UiSchemaValue::string("xxl")),
        ]);

        let report = schema.validate(&value);

        assert!(!report.is_valid());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.field_path == ["density"]
                && diagnostic.diagnostic_id.as_str() == "ui.schema.string_value_not_allowed"
        }));
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.field_path == ["size"]
                && diagnostic.diagnostic_id.as_str() == "ui.schema.string_value_not_allowed"
        }));
    }

    #[test]
    fn button_property_schema_rejects_unknown_properties() {
        let schema = button_property_schema();

        let value = UiSchemaValue::object([
            ("label", UiSchemaValue::string("Broken")),
            ("background_color", UiSchemaValue::string("#ff00ff")),
        ]);

        let report = schema.validate(&value);

        assert!(!report.is_valid());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.field_path == ["background_color"]
                && diagnostic.diagnostic_id.as_str() == "ui.schema.unknown_field"
        }));
    }

    #[test]
    fn button_property_schema_requires_label() {
        let schema = button_property_schema();

        let value = UiSchemaValue::object([
            ("variant", UiSchemaValue::string("primary")),
            ("tone", UiSchemaValue::string("accent")),
            ("density", UiSchemaValue::string("normal")),
            ("size", UiSchemaValue::string("md")),
        ]);

        let report = schema.validate(&value);

        assert!(!report.is_valid());
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.field_path == ["label"]
                && diagnostic.diagnostic_id.as_str() == "ui.schema.required_field_missing"
        }));
    }

    fn button_property_schema() -> ui_schema::UiSchema {
        control_module()
            .schemas
            .into_iter()
            .find(|schema| {
                schema.schema.schema_ref.id.as_str() == "runenwerk.ui.controls.button.properties"
            })
            .expect("button property schema should exist")
            .schema
    }
}
