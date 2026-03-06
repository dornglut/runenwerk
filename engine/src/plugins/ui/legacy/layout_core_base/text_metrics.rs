// Owner: Grotto Quest Engine - UI Plugin
pub fn estimate_text_width(text: &str, size: f32) -> f32 {
    // Monospace-biased estimate for centering labels in this MVP stage.
    text.chars().count() as f32 * size * 0.56
}

pub(crate) fn measure_text_advance_precise(
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
    text: &str,
    size: f32,
) -> f32 {
    let scale = text_scale(metrics, size);

    text.chars()
        .map(|ch| glyph_advance_with_scale(metrics, ch, scale))
        .sum()
}

pub(crate) fn char_count(text: &str) -> usize {
    text.chars().count()
}

pub(crate) fn byte_index_at_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

pub(crate) fn slice_chars(text: &str, start_char: usize, end_char: usize) -> String {
    let start = byte_index_at_char(text, start_char);
    let end = byte_index_at_char(text, end_char);
    text[start..end].to_string()
}

pub(crate) fn text_scale(metrics: &crate::plugins::ui::domain::UiTextMetrics, size: f32) -> f32 {
    if metrics.base_size > 0.0 {
        (size / metrics.base_size).max(0.1)
    } else {
        1.0
    }
}

pub(crate) fn glyph_advance_with_scale(
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
    ch: char,
    scale: f32,
) -> f32 {
    let advance = metrics
        .glyphs
        .get(&ch)
        .map(|glyph| glyph.advance_px)
        .unwrap_or(metrics.fallback_advance);
    advance * scale
}

pub(crate) fn input_content_rect(transform: &UiTransform, ui_scale: f32) -> (f32, f32, f32, f32) {
    let pad_x = INPUT_PADDING_X * ui_scale;
    let pad_y = INPUT_PADDING_Y * ui_scale;
    let content_x = transform.x + pad_x;
    let content_y = transform.y + pad_y;
    let content_w = (transform.w - (pad_x * 2.0) - (2.0 * ui_scale)).max(1.0);
    let content_h = (transform.h - (pad_y * 2.0)).max(1.0);
    (content_x, content_y, content_w, content_h)
}
