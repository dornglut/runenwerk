//! File: domain/ui/ui_controls/src/tree_view/mod.rs
//! Crate: ui_controls

use ui_program::RouteCapability;
use ui_schema::{UiSchema, UiSchemaShape};

use crate::{ControlModuleDescriptor, RUNENWERK_CONTROL_PACKAGE_ID};

pub const TREE_VIEW_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.tree-view";

pub fn control_module() -> ControlModuleDescriptor {
    crate::control_module_contract(
        "tree-view",
        "TreeView",
        UiSchema::object(
            format!("{RUNENWERK_CONTROL_PACKAGE_ID}.tree-view.properties"),
            1,
        )
        .with_required_field("roots", UiSchemaShape::list(UiSchemaShape::Object))
        .with_optional_field("expand_route", UiSchemaShape::RouteRef),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.tree-view.state"), 1)
            .with_optional_field(
                "expanded_ids",
                UiSchemaShape::list(UiSchemaShape::StableIdRef),
            )
            .with_optional_field("selected_id", UiSchemaShape::StableIdRef),
        UiSchema::object(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.tree-view.event"), 1)
            .with_required_field("route", UiSchemaShape::RouteRef)
            .with_required_field("node_id", UiSchemaShape::StableIdRef),
        RouteCapability::new("runenwerk.ui.controls.tree.navigate"),
    )
}
