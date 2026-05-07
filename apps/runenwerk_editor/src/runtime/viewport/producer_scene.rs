//! File: apps/runenwerk_editor/src/runtime/viewport/producer_scene.rs
//! Purpose: Producer-owned render target aliases for viewport expression products.

use crate::runtime::viewport::ViewportSurfaceSlot;

pub const EDITOR_MAIN_FLOW_ID: &str = "runenwerk.editor.main";
pub const EDITOR_VIEWPORT_SCENE_PRODUCT_UNIFORM_ID: &str =
    "runenwerk.editor.viewport.product.scene.uniform";

pub const VIEWPORT_TARGET_ALIAS_SCENE_COLOR: &str = "viewport.scene_color";
pub const VIEWPORT_TARGET_ALIAS_PICKING_IDS: &str = "viewport.picking_ids";
pub const VIEWPORT_TARGET_ALIAS_OVERLAY: &str = "viewport.overlay";

pub fn surface_slot_target_alias(slot: ViewportSurfaceSlot) -> &'static str {
    match slot {
        ViewportSurfaceSlot::PrimaryColor => VIEWPORT_TARGET_ALIAS_SCENE_COLOR,
        ViewportSurfaceSlot::PickingIds => VIEWPORT_TARGET_ALIAS_PICKING_IDS,
        ViewportSurfaceSlot::Overlay => VIEWPORT_TARGET_ALIAS_OVERLAY,
    }
}
