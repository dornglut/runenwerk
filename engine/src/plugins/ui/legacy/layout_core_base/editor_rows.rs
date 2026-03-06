// Owner: Grotto Quest Engine - UI Plugin
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct WrappedEditorRow {
    pub start_char: usize,
    pub end_char: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputViewportLayout {
    pub content: String,
    pub caret_x: f32,
    pub caret_y: f32,
    pub viewport_row: usize,
    pub visible_rows: usize,
    pub total_rows: usize,
}

pub(crate) fn wrap_editor_rows_with_prompt(
    editor_text: &str,
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
    size: f32,
    max_width: f32,
    prompt_width: f32,
) -> Vec<WrappedEditorRow> {
    let max_width = max_width.max(1.0);
    let mut rows = Vec::new();
    let mut row_start = 0usize;
    let mut row_width = 0.0f32;
    let mut row_index = 0usize;
    let mut char_index = 0usize;
    let total_chars = char_count(editor_text);
    let scale = text_scale(metrics, size);

    for ch in editor_text.chars() {
        if ch == '\n' {
            rows.push(WrappedEditorRow {
                start_char: row_start,
                end_char: char_index,
            });
            row_start = char_index + 1;
            row_width = 0.0;
            row_index += 1;
            char_index += 1;
            continue;
        }

        let row_limit = if row_index == 0 {
            (max_width - prompt_width).max(1.0)
        } else {
            max_width
        };
        let advance = glyph_advance_with_scale(metrics, ch, scale);
        if row_width > 0.0 && (row_width + advance) > row_limit {
            rows.push(WrappedEditorRow {
                start_char: row_start,
                end_char: char_index,
            });
            row_start = char_index;
            row_width = 0.0;
            row_index += 1;
        }

        row_width += advance;
        char_index += 1;
    }

    rows.push(WrappedEditorRow {
        start_char: row_start,
        end_char: total_chars,
    });
    if rows.is_empty() {
        rows.push(WrappedEditorRow {
            start_char: 0,
            end_char: 0,
        });
    }
    rows
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn wrap_editor_rows(
    editor_text: &str,
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
    size: f32,
    max_width: f32,
) -> Vec<WrappedEditorRow> {
    let prompt_width = measure_text_advance_precise(metrics, CONSOLE_PROMPT, size);
    wrap_editor_rows_with_prompt(editor_text, metrics, size, max_width, prompt_width)
}

pub(crate) fn row_index_for_cursor(rows: &[WrappedEditorRow], cursor_chars: usize) -> usize {
    for (idx, row) in rows.iter().enumerate() {
        if cursor_chars >= row.start_char && cursor_chars <= row.end_char {
            return idx;
        }
    }
    rows.len().saturating_sub(1)
}

pub(crate) fn caret_x_for_row_cursor(
    text: &str,
    row: WrappedEditorRow,
    cursor_chars: usize,
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
    size: f32,
    row_prefix: f32,
) -> f32 {
    let clamped_cursor = cursor_chars.clamp(row.start_char, row.end_char);
    if clamped_cursor == row.start_char {
        return row_prefix.max(0.0);
    }
    let prefix = slice_chars(text, row.start_char, clamped_cursor);
    row_prefix.max(0.0) + measure_text_advance_precise(metrics, &prefix, size)
}

pub(crate) fn cursor_for_row_x(
    text: &str,
    row: WrappedEditorRow,
    target_x: f32,
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
    size: f32,
    row_prefix: f32,
) -> usize {
    let mut x = row_prefix.max(0.0);
    if target_x <= x {
        return row.start_char;
    }

    let mut cursor = row.start_char;
    let segment = slice_chars(text, row.start_char, row.end_char);
    let scale = text_scale(metrics, size);
    for ch in segment.chars() {
        let advance = glyph_advance_with_scale(metrics, ch, scale);
        let midpoint = x + (advance * 0.5);
        if target_x <= midpoint {
            return cursor;
        }
        x += advance;
        cursor += 1;
    }
    row.end_char
}
