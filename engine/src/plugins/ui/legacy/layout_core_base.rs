// Owner: Grotto Quest Engine - UI Plugin
use crate::plugins::scene::SceneManager;
use crate::plugins::ui::domain::{
    ConsoleUiRuntimeState, UiBatchCmd, UiDirty, UiEditorNode, UiNode, UiPresentationMode,
    UiTransform,
};

pub(crate) const CONSOLE_PROMPT: &str = "grotto> ";
pub(crate) const CARET_BLINK_SECONDS: f32 = 0.5;
pub(crate) const INPUT_PADDING_X: f32 = 6.0;
pub(crate) const INPUT_PADDING_Y: f32 = 4.0;
pub(crate) const EDITOR_BASE_NUDGE_PX: f32 = 1.0;
pub(crate) const EDITOR_DRAG_SNAP_PX: f32 = 10.0;

#[derive(Debug, Copy, Clone)]
pub(crate) struct LogWindowRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
    pub(crate) body_x: f32,
    pub(crate) body_y: f32,
    pub(crate) body_w: f32,
    pub(crate) body_h: f32,
}

pub(crate) fn compute_log_window_rect(
    layout: &crate::plugins::ui::domain::UiLayoutConfig,
    screen_size: (f32, f32),
    ui_scale: f32,
) -> LogWindowRect {
    let log_margin = layout.logs_margin * ui_scale;
    let log_w = (screen_size.0 * layout.logs_width_ratio).clamp(
        layout.logs_min_width * ui_scale,
        (screen_size.0 - (log_margin * 2.0)).max(1.0),
    );
    let log_h = (screen_size.1 * layout.logs_height_ratio).clamp(
        layout.logs_min_height * ui_scale,
        (screen_size.1 - (log_margin * 2.0)).max(1.0),
    );
    let log_x = (screen_size.0 - log_w - log_margin).max(log_margin);
    let log_y = log_margin;
    let header_h = 24.0 * ui_scale;
    let body_y = log_y + header_h + (6.0 * ui_scale);
    let body_h = (log_h - header_h - (12.0 * ui_scale)).max(1.0);
    let body_x = log_x + (6.0 * ui_scale);
    let body_w = (log_w - (12.0 * ui_scale)).max(1.0);
    LogWindowRect {
        x: log_x,
        y: log_y,
        w: log_w,
        h: log_h,
        body_x,
        body_y,
        body_w,
        body_h,
    }
}

pub fn point_in_rect(point: (f32, f32), rect: &UiTransform) -> bool {
    point.0 >= rect.x
        && point.0 <= rect.x + rect.w
        && point.1 >= rect.y
        && point.1 <= rect.y + rect.h
}

#[derive(Debug, Copy, Clone)]
pub struct EditorNodeRect {
    pub node: UiEditorNode,
    pub z: i32,
    pub rect: UiTransform,
}

pub fn pick_editor_node_at(point: (f32, f32), nodes: &[EditorNodeRect]) -> Option<UiEditorNode> {
    nodes
        .iter()
        .filter(|item| point_in_rect(point, &item.rect))
        .max_by_key(|item| item.z)
        .map(|item| item.node)
}

pub(crate) fn editor_node_label(node: UiEditorNode) -> &'static str {
    match node {
        UiEditorNode::Root => "root",
        UiEditorNode::Scrollback => "scrollback",
        UiEditorNode::Input => "input",
        UiEditorNode::ConfirmButton => "confirm_button",
    }
}

pub fn snap_to_grid(value: f32, grid: f32) -> f32 {
    let g = grid.max(1.0);
    (value / g).round() * g
}

pub(crate) fn tint_color(color: [f32; 4], factor: f32) -> [f32; 4] {
    [
        (color[0] * factor).clamp(0.0, 1.0),
        (color[1] * factor).clamp(0.0, 1.0),
        (color[2] * factor).clamp(0.0, 1.0),
        color[3],
    ]
}

pub(crate) fn push_outline(
    commands: &mut Vec<UiBatchCmd>,
    rect: &UiTransform,
    thickness: f32,
    color: [f32; 4],
) {
    let t = thickness.max(1.0);
    commands.push(UiBatchCmd::Rect {
        x: rect.x,
        y: rect.y,
        w: rect.w,
        h: t,
        color,
        radius: 0.0,
    });
    commands.push(UiBatchCmd::Rect {
        x: rect.x,
        y: rect.y + rect.h - t,
        w: rect.w,
        h: t,
        color,
        radius: 0.0,
    });
    commands.push(UiBatchCmd::Rect {
        x: rect.x,
        y: rect.y,
        w: t,
        h: rect.h,
        color,
        radius: 0.0,
    });
    commands.push(UiBatchCmd::Rect {
        x: rect.x + rect.w - t,
        y: rect.y,
        w: t,
        h: rect.h,
        color,
        radius: 0.0,
    });
}

pub(crate) fn push_scroll_indicators(
    commands: &mut Vec<UiBatchCmd>,
    rect: &UiTransform,
    visible_rows: usize,
    total_rows: usize,
    lines_from_bottom: usize,
    horizontal_chars: usize,
    max_horizontal_chars: usize,
    ui_scale: f32,
    color: [f32; 4],
) {
    let track_w = (2.0 * ui_scale).max(1.0);
    let track_h = (2.0 * ui_scale).max(1.0);
    let inner_h = rect.h.max(1.0);
    let inner_w = rect.w.max(1.0);

    if total_rows > visible_rows.max(1) {
        let total = total_rows as f32;
        let visible = visible_rows.max(1) as f32;
        let thumb_h = ((visible / total) * inner_h).clamp(8.0 * ui_scale, inner_h);
        let max_scroll = total_rows.saturating_sub(visible_rows.max(1));
        let top_offset = max_scroll.saturating_sub(lines_from_bottom) as f32;
        let track_space = (inner_h - thumb_h).max(0.0);
        let thumb_y = rect.y + ((top_offset / max_scroll.max(1) as f32) * track_space);
        commands.push(UiBatchCmd::Rect {
            x: rect.x + rect.w - track_w,
            y: rect.y,
            w: track_w,
            h: rect.h,
            color: [color[0], color[1], color[2], 0.25],
            radius: 0.0,
        });
        commands.push(UiBatchCmd::Rect {
            x: rect.x + rect.w - track_w,
            y: thumb_y,
            w: track_w,
            h: thumb_h,
            color,
            radius: 0.0,
        });
    }

    if max_horizontal_chars > 0 {
        let visible_char_estimate = ((inner_w / (7.0 * ui_scale.max(1.0))).floor() as usize).max(1);
        let total_chars = max_horizontal_chars + visible_char_estimate;
        let thumb_w = ((visible_char_estimate as f32 / total_chars as f32) * inner_w)
            .clamp(16.0 * ui_scale, inner_w);
        let track_space = (inner_w - thumb_w).max(0.0);
        let thumb_x =
            rect.x + ((horizontal_chars as f32 / max_horizontal_chars.max(1) as f32) * track_space);
        commands.push(UiBatchCmd::Rect {
            x: rect.x,
            y: rect.y + rect.h - track_h,
            w: rect.w,
            h: track_h,
            color: [color[0], color[1], color[2], 0.25],
            radius: 0.0,
        });
        commands.push(UiBatchCmd::Rect {
            x: thumb_x,
            y: rect.y + rect.h - track_h,
            w: thumb_w,
            h: track_h,
            color,
            radius: 0.0,
        });
    }
}

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

pub(crate) fn fit_text_with_ellipsis_from_right(text: &str, size: f32, max_width: f32) -> String {
    if text.is_empty() || estimate_text_width(text, size) <= max_width {
        return text.to_string();
    }

    let ellipsis = "...";
    if estimate_text_width(ellipsis, size) >= max_width {
        return String::new();
    }

    let mut head = text.to_string();
    while !head.is_empty() && estimate_text_width(&format!("{head}{ellipsis}"), size) > max_width {
        head.pop();
    }

    if head.is_empty() {
        String::new()
    } else {
        format!("{head}{ellipsis}")
    }
}

pub(crate) fn line_height(size: f32) -> f32 {
    size * 1.25
}

pub fn visible_line_capacity(area_height: f32, text_size: f32) -> usize {
    let h = line_height(text_size).max(1.0);
    ((area_height / h).floor() as usize).max(1)
}

pub(crate) fn build_scrollback_view_lines(
    lines: &[String],
    lines_from_bottom: usize,
    visible_capacity: usize,
    max_line_width: f32,
    text_size: f32,
) -> Vec<String> {
    if lines.is_empty() {
        return Vec::new();
    }

    let end = lines.len().saturating_sub(lines_from_bottom);
    let start = end.saturating_sub(visible_capacity);
    lines[start..end]
        .iter()
        .map(|line| fit_text_with_ellipsis_from_right(line, text_size, max_line_width))
        .collect()
}

