// Owner: Grotto Quest Engine - UI Plugin
pub fn move_cursor_vertical(
    editor: &mut crate::plugins::ui::domain::EditorBuffer,
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
    size: f32,
    max_width: f32,
    move_down: bool,
) -> bool {
    let total_chars = char_count(&editor.text);
    editor.cursor_chars = editor.cursor_chars.min(total_chars);
    let prompt_width = measure_text_advance_precise(metrics, CONSOLE_PROMPT, size);
    let rows = wrap_editor_rows_with_prompt(&editor.text, metrics, size, max_width, prompt_width);
    if rows.is_empty() {
        return false;
    }

    let cursor_row = row_index_for_cursor(&rows, editor.cursor_chars);
    let target_row = if move_down {
        (cursor_row + 1).min(rows.len().saturating_sub(1))
    } else {
        cursor_row.saturating_sub(1)
    };
    if target_row == cursor_row {
        return false;
    }

    let current_row_prefix = if cursor_row == 0 { prompt_width } else { 0.0 };
    let desired_x = editor.preferred_caret_x.unwrap_or_else(|| {
        caret_x_for_row_cursor(
            &editor.text,
            rows[cursor_row],
            editor.cursor_chars,
            metrics,
            size,
            current_row_prefix,
        )
    });
    let target_row_prefix = if target_row == 0 { prompt_width } else { 0.0 };
    editor.cursor_chars = cursor_for_row_x(
        &editor.text,
        rows[target_row],
        desired_x,
        metrics,
        size,
        target_row_prefix,
    );
    editor.preferred_caret_x = Some(desired_x);
    true
}

pub fn build_visible_multiline_input(
    editor: &mut crate::plugins::ui::domain::EditorBuffer,
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
    size: f32,
    max_width: f32,
    max_height: f32,
) -> InputViewportLayout {
    let total_chars = char_count(&editor.text);
    editor.cursor_chars = editor.cursor_chars.min(total_chars);
    let prompt_width = measure_text_advance_precise(metrics, CONSOLE_PROMPT, size);
    let rows = wrap_editor_rows_with_prompt(&editor.text, metrics, size, max_width, prompt_width);
    let total_rows = rows.len().max(1);
    let cursor_row = row_index_for_cursor(&rows, editor.cursor_chars);

    let visible_rows = visible_line_capacity(max_height, size);
    let max_viewport_row = total_rows.saturating_sub(visible_rows);
    let mut viewport_row = editor.viewport_row.min(max_viewport_row);
    if cursor_row < viewport_row {
        viewport_row = cursor_row;
    } else if cursor_row >= viewport_row + visible_rows {
        viewport_row = cursor_row + 1 - visible_rows;
    }
    viewport_row = viewport_row.min(max_viewport_row);
    editor.viewport_row = viewport_row;

    let visible_end = (viewport_row + visible_rows).min(total_rows);
    let mut lines = Vec::with_capacity(visible_end.saturating_sub(viewport_row));
    for idx in viewport_row..visible_end {
        let row = rows[idx];
        let row_text = slice_chars(&editor.text, row.start_char, row.end_char);
        if idx == 0 {
            lines.push(format!("{CONSOLE_PROMPT}{row_text}"));
        } else {
            lines.push(row_text);
        }
    }

    let row_prefix = if cursor_row == 0 { prompt_width } else { 0.0 };
    let caret_x = caret_x_for_row_cursor(
        &editor.text,
        rows[cursor_row],
        editor.cursor_chars,
        metrics,
        size,
        row_prefix,
    );
    let line_h = line_height(size).max(1.0);
    let caret_h = (size * 0.9).max(1.0);
    let caret_row_in_view = cursor_row.saturating_sub(viewport_row);
    let caret_line_top = (caret_row_in_view as f32) * line_h + ((line_h - size) * 0.5).max(0.0);
    let caret_y = caret_line_top + ((size - caret_h) * 0.5).max(0.0);

    InputViewportLayout {
        content: lines.join("\n"),
        caret_x,
        caret_y,
        viewport_row,
        visible_rows,
        total_rows,
    }
}
