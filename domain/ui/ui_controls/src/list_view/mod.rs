//! File: domain/ui/ui_controls/src/list_view/mod.rs
//! Crate: ui_controls

use ui_program::RouteCapability;
use ui_schema::{UiSchema, UiSchemaShape};

use crate::{ControlModuleDescriptor, RUNENWERK_CONTROL_PACKAGE_ID};

pub const LIST_VIEW_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.list-view";

pub fn control_module() -> ControlModuleDescriptor {
    crate::control_module_contract(
        "list-view",
        "ListView",
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.list-view.properties"),
            1,
        )
        .with_required_field("items", UiSchemaShape::list(UiSchemaShape::Object))
        .with_optional_field("selection_route", UiSchemaShape::RouteRef),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.list-view.state"), 1)
            .with_optional_field("selected_index", UiSchemaShape::UnsignedInteger)
            .with_optional_field("scroll_offset", UiSchemaShape::Number),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.list-view.event"), 1)
            .with_required_field("route", UiSchemaShape::RouteRef)
            .with_required_field("selected_index", UiSchemaShape::UnsignedInteger),
        RouteCapability::new("runenwerk.ui.controls.list.select"),
    )
}
