//! File: domain/editor/editor_shell/src/workspace/viewport_embed_slot.rs
//! Purpose: Integration-edge mapping from viewport semantics to renderer-facing embed slot IDs.

use editor_viewport::ViewportSurfacePresentationSlot;
use ui_render_data::ViewportSurfaceEmbedSlotId;

const PRIMARY_COLOR_EMBED_SLOT_ID_RAW: u16 = 1;
const PICKING_IDS_EMBED_SLOT_ID_RAW: u16 = 2;
const OVERLAY_EMBED_SLOT_ID_RAW: u16 = 3;

pub fn viewport_embed_slot_for(
    slot: ViewportSurfacePresentationSlot,
) -> ViewportSurfaceEmbedSlotId {
    match slot {
        ViewportSurfacePresentationSlot::Primary => {
            ViewportSurfaceEmbedSlotId::new(PRIMARY_COLOR_EMBED_SLOT_ID_RAW)
        }
        ViewportSurfacePresentationSlot::Picking => {
            ViewportSurfaceEmbedSlotId::new(PICKING_IDS_EMBED_SLOT_ID_RAW)
        }
        ViewportSurfacePresentationSlot::Overlay => {
            ViewportSurfaceEmbedSlotId::new(OVERLAY_EMBED_SLOT_ID_RAW)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_keeps_viewport_semantics_outside_renderer_payload_type() {
        assert_eq!(
            viewport_embed_slot_for(ViewportSurfacePresentationSlot::Primary),
            ViewportSurfaceEmbedSlotId::new(PRIMARY_COLOR_EMBED_SLOT_ID_RAW)
        );
        assert_eq!(
            viewport_embed_slot_for(ViewportSurfacePresentationSlot::Picking),
            ViewportSurfaceEmbedSlotId::new(PICKING_IDS_EMBED_SLOT_ID_RAW)
        );
        assert_eq!(
            viewport_embed_slot_for(ViewportSurfacePresentationSlot::Overlay),
            ViewportSurfaceEmbedSlotId::new(OVERLAY_EMBED_SLOT_ID_RAW)
        );
    }
}
