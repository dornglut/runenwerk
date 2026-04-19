//! File: apps/runenwerk_editor/src/runtime/viewport/producer_scene.rs
//! Purpose: Producer-owned render resource identities for viewport expression products.

use editor_viewport::ViewportSurfaceSlot;

pub const EDITOR_MAIN_FLOW_ID: &str = "runenwerk.editor.main";

pub const VIEWPORT_RESOURCE_SCENE_COLOR: &str = "editor.viewport.v1.scene_color";
pub const VIEWPORT_RESOURCE_PICKING_IDS: &str = "editor.viewport.v1.picking_ids";
pub const VIEWPORT_RESOURCE_OVERLAY: &str = "editor.viewport.v1.overlay";

pub fn surface_slot_resource_id(slot: ViewportSurfaceSlot) -> &'static str {
    match slot {
        ViewportSurfaceSlot::PrimaryColor => VIEWPORT_RESOURCE_SCENE_COLOR,
        ViewportSurfaceSlot::PickingIds => VIEWPORT_RESOURCE_PICKING_IDS,
        ViewportSurfaceSlot::Overlay => VIEWPORT_RESOURCE_OVERLAY,
    }
}
