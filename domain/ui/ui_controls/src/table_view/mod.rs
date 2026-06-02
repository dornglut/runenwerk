//! File: domain/ui/ui_controls/src/table_view/mod.rs
//! Crate: ui_controls

use ui_program::RouteCapability;
use ui_schema::{UiSchema, UiSchemaShape};

use crate::{ControlModuleDescriptor, RUNENWERK_CONTROL_PACKAGE_ID};

pub const TABLE_VIEW_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.table-view";

pub fn control_module() -> ControlModuleDescriptor {
    crate::control_module_contract(
        "table-view",
        "TableView",
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.table-view.properties"),
            1,
        )
        .with_required_field("columns", UiSchemaShape::list(UiSchemaShape::Object))
        .with_required_field("rows", UiSchemaShape::list(UiSchemaShape::Object)),
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.table-view.state"),
            1,
        )
        .with_optional_field("sort_column", UiSchemaShape::String)
        .with_optional_field("selected_row", UiSchemaShape::UnsignedInteger),
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.table-view.event"),
            1,
        )
        .with_required_field("route", UiSchemaShape::RouteRef)
        .with_required_field("row", UiSchemaShape::UnsignedInteger),
        RouteCapability::new("runenwerk.ui.controls.table.select"),
    )
}
