use std::{fs, path::Path};

use ui_controls::button::control_module;
use ui_definition::authored_control_schema::authored_control_properties_to_schema_value;
use ui_definition::{AuthoredControlValue, UiNodeDefinition};

#[test]
fn ui_gallery_button_basic_fixture_parses_as_generic_control() {
    let node = load_node("assets/ui_gallery/button/basic.ron");

    let UiNodeDefinition::Control {
        id,
        kind,
        properties,
        bindings,
        route,
        accessibility,
    } = node
    else {
        panic!("expected generic Control node");
    };

    assert_eq!(id.as_str(), "button_basic");
    assert_eq!(kind.as_str(), "runenwerk.ui.controls.button");

    assert_eq!(
        properties.get("label"),
        Some(&AuthoredControlValue::String("Press me".to_owned()))
    );
    assert_eq!(
        properties.get("variant"),
        Some(&AuthoredControlValue::String("primary".to_owned()))
    );
    assert_eq!(
        properties.get("tone"),
        Some(&AuthoredControlValue::String("accent".to_owned()))
    );
    assert_eq!(
        properties.get("density"),
        Some(&AuthoredControlValue::String("normal".to_owned()))
    );
    assert_eq!(
        properties.get("size"),
        Some(&AuthoredControlValue::String("md".to_owned()))
    );

    assert!(bindings.is_empty());
    assert_eq!(
        route.as_ref().map(|route| route.as_str()),
        Some("ui_gallery.button.basic.activate")
    );

    let accessibility = accessibility.expect("basic button must define accessibility intent");
    assert_eq!(accessibility.role, "button");
    assert_eq!(accessibility.label.as_deref(), Some("Press demo button"));
}

#[test]
fn ui_gallery_button_selected_fixture_parses_selected_binding() {
    let node = load_node("assets/ui_gallery/button/selected.ron");

    let UiNodeDefinition::Control {
        id,
        kind,
        properties,
        bindings,
        route,
        accessibility,
    } = node
    else {
        panic!("expected generic Control node");
    };

    assert_eq!(id.as_str(), "button_selected");
    assert_eq!(kind.as_str(), "runenwerk.ui.controls.button");

    assert_eq!(
        properties.get("label"),
        Some(&AuthoredControlValue::String("Selected".to_owned()))
    );
    assert_eq!(
        properties.get("variant"),
        Some(&AuthoredControlValue::String("secondary".to_owned()))
    );

    assert_eq!(
        bindings.get("selected").map(|binding| binding.as_str()),
        Some("ui_gallery.button.selected.active")
    );

    assert_eq!(
        route.as_ref().map(|route| route.as_str()),
        Some("ui_gallery.button.selected.activate")
    );

    let accessibility = accessibility.expect("selected button must define accessibility intent");
    assert_eq!(accessibility.role, "button");
    assert_eq!(accessibility.label.as_deref(), Some("Selected demo button"));
}

#[test]
fn ui_gallery_button_basic_fixture_validates_against_button_package_schema() {
    let node = load_node("assets/ui_gallery/button/basic.ron");

    let UiNodeDefinition::Control {
        kind, properties, ..
    } = node
    else {
        panic!("expected generic Control node");
    };

    assert_eq!(kind.as_str(), "runenwerk.ui.controls.button");

    let value = authored_control_properties_to_schema_value(&properties);
    let report = button_property_schema().validate(&value);

    assert!(report.is_valid(), "{:?}", report.diagnostics);
}

#[test]
fn ui_gallery_button_selected_fixture_validates_against_button_package_schema() {
    let node = load_node("assets/ui_gallery/button/selected.ron");

    let UiNodeDefinition::Control {
        kind, properties, ..
    } = node
    else {
        panic!("expected generic Control node");
    };

    assert_eq!(kind.as_str(), "runenwerk.ui.controls.button");

    let value = authored_control_properties_to_schema_value(&properties);
    let report = button_property_schema().validate(&value);

    assert!(report.is_valid(), "{:?}", report.diagnostics);
}

#[test]
fn ui_gallery_button_invalid_density_reports_schema_diagnostic() {
    let node = load_node("assets/ui_gallery/button/basic.ron");

    let UiNodeDefinition::Control {
        kind,
        mut properties,
        ..
    } = node
    else {
        panic!("expected generic Control node");
    };

    assert_eq!(kind.as_str(), "runenwerk.ui.controls.button");

    properties.insert(
        "density".to_owned(),
        AuthoredControlValue::String("tiny".to_owned()),
    );

    let value = authored_control_properties_to_schema_value(&properties);
    let report = button_property_schema().validate(&value);

    assert!(!report.is_valid());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.field_path == ["density"]
            && diagnostic.diagnostic_id.as_str() == "ui.schema.string_value_not_allowed"
    }));
}

fn load_node(relative_repo_path: &str) -> UiNodeDefinition {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative_repo_path);

    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {:?}: {error}", path));

    ron::from_str(&source).unwrap_or_else(|error| panic!("failed to parse {:?}: {error}", path))
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
