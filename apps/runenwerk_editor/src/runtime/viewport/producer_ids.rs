//! File: apps/runenwerk_editor/src/runtime/viewport/producer_ids.rs
//! Purpose: Editor viewport render-product producer identities.

use engine::plugins::render::RenderFrameProducerId;

pub const EDITOR_VIEWPORT_RENDER_PRODUCT_PRODUCER_ID: RenderFrameProducerId =
    render_frame_producer_id(1);
pub const EDITOR_MATERIAL_PREVIEW_PRODUCT_PRODUCER_ID: RenderFrameProducerId =
    render_frame_producer_id(2);

const fn render_frame_producer_id(raw: u64) -> RenderFrameProducerId {
    match RenderFrameProducerId::try_from_raw(raw) {
        Ok(value) => value,
        Err(_) => panic!("render frame producer id must be non-zero"),
    }
}
