//! File: apps/runenwerk_editor/src/runtime/viewport/producer_scene.rs
//! Purpose: Producer-owned render target aliases for viewport expression products.

use crate::runtime::viewport::ViewportSurfaceSlot;

pub const EDITOR_MAIN_FLOW_ID: &str = "runenwerk.editor.main";
pub const EDITOR_VIEWPORT_SCENE_PRODUCT_UNIFORM_ID: &str =
    "runenwerk.editor.viewport.product.scene.uniform";

pub const VIEWPORT_TARGET_ALIAS_SCENE_COLOR: &str = "viewport.scene_color";
pub const VIEWPORT_TARGET_ALIAS_PRIMARY_COLOR: &str = "viewport.primary_color";
pub const VIEWPORT_TARGET_ALIAS_PICKING_IDS: &str = "viewport.picking_ids";
pub const VIEWPORT_TARGET_ALIAS_OVERLAY: &str = "viewport.overlay";
pub const VIEWPORT_TARGET_ALIAS_SCALAR_FIELD: &str = "viewport.scalar_field";
pub const VIEWPORT_TARGET_ALIAS_VECTOR_FIELD: &str = "viewport.vector_field";
pub const VIEWPORT_TARGET_ALIAS_ATLAS: &str = "viewport.atlas";
pub const VIEWPORT_TARGET_ALIAS_VOLUME_SLICE: &str = "viewport.volume_slice";
pub const VIEWPORT_TARGET_ALIAS_BRICKMAP_DEBUG: &str = "viewport.brickmap_debug";
pub const VIEWPORT_TARGET_ALIAS_HISTORY_COLOR: &str = "viewport.history_color";
pub const VIEWPORT_TARGET_ALIAS_MATERIAL_PREVIEW: &str = "viewport.material_preview";

pub fn surface_slot_target_alias(slot: ViewportSurfaceSlot) -> &'static str {
    match slot {
        ViewportSurfaceSlot::PrimaryColor => VIEWPORT_TARGET_ALIAS_SCENE_COLOR,
        ViewportSurfaceSlot::PickingIds => VIEWPORT_TARGET_ALIAS_PICKING_IDS,
        ViewportSurfaceSlot::Overlay => VIEWPORT_TARGET_ALIAS_OVERLAY,
        ViewportSurfaceSlot::ScalarField => VIEWPORT_TARGET_ALIAS_SCALAR_FIELD,
        ViewportSurfaceSlot::VectorField => VIEWPORT_TARGET_ALIAS_VECTOR_FIELD,
        ViewportSurfaceSlot::Atlas => VIEWPORT_TARGET_ALIAS_ATLAS,
        ViewportSurfaceSlot::VolumeSlice => VIEWPORT_TARGET_ALIAS_VOLUME_SLICE,
        ViewportSurfaceSlot::BrickmapDebug => VIEWPORT_TARGET_ALIAS_BRICKMAP_DEBUG,
        ViewportSurfaceSlot::HistoryColor => VIEWPORT_TARGET_ALIAS_HISTORY_COLOR,
    }
}
