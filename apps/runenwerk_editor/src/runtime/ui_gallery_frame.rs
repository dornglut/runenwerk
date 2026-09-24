use ui_math::{UiPoint, UiRect, UiSize};
use ui_render_data::{UiFrame, UiFrameFragment, UiFramePlacement, compose_frame_fragments};

const GALLERY_PREVIEW_TILE_WIDTH: f32 = 320.0;
const GALLERY_PREVIEW_TILE_HEIGHT: f32 = 128.0;
const GALLERY_PREVIEW_PADDING: f32 = 16.0;
const GALLERY_PREVIEW_GAP: f32 = 12.0;

pub(super) fn default_gallery_proof_size() -> UiSize {
    UiSize::new(GALLERY_PREVIEW_TILE_WIDTH, GALLERY_PREVIEW_TILE_HEIGHT)
}

pub(super) fn compose_gallery_preview_frame(
    output_size: UiSize,
    previews: &[UiFrame],
) -> Option<UiFrame> {
    if previews.is_empty() {
        return None;
    }

    let columns = gallery_preview_columns(output_size.width);
    let fragments = previews.iter().enumerate().map(|(index, frame)| {
        let column = index % columns;
        let row = index / columns;
        let x = GALLERY_PREVIEW_PADDING
            + column as f32 * (GALLERY_PREVIEW_TILE_WIDTH + GALLERY_PREVIEW_GAP);
        let y = GALLERY_PREVIEW_PADDING
            + row as f32 * (GALLERY_PREVIEW_TILE_HEIGHT + GALLERY_PREVIEW_GAP);
        let origin = UiPoint::new(x, y);
        let clip = UiRect::new(
            x,
            y,
            GALLERY_PREVIEW_TILE_WIDTH,
            GALLERY_PREVIEW_TILE_HEIGHT,
        );
        UiFrameFragment::new(frame, UiFramePlacement::new(origin, clip, index as u32))
    });
    let frame = compose_frame_fragments(output_size, fragments);
    (!frame.is_empty()).then_some(frame)
}

fn gallery_preview_columns(output_width: f32) -> usize {
    let usable_width = (output_width - GALLERY_PREVIEW_PADDING * 2.0 + GALLERY_PREVIEW_GAP)
        .max(GALLERY_PREVIEW_TILE_WIDTH);
    ((usable_width / (GALLERY_PREVIEW_TILE_WIDTH + GALLERY_PREVIEW_GAP)).floor() as usize).max(1)
}
