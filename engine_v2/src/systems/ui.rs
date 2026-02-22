use crate::runtime::EngineData;
use crate::ui::{
    UiBatchCmd, UiButton, UiDirty, UiDrawCmd, UiEditorNode, UiInputField, UiInteraction, UiStyle,
    UiSubmitEvent, UiText, UiTransform, reload_console_template_if_changed,
    save_console_template_to_disk,
};

const CONSOLE_PROMPT: &str = "grotto> ";
const CARET_BLINK_SECONDS: f32 = 0.5;
const INPUT_PADDING_X: f32 = 6.0;
const INPUT_PADDING_Y: f32 = 4.0;
const EDITOR_BASE_NUDGE_PX: f32 = 1.0;
const EDITOR_DRAG_SNAP_PX: f32 = 10.0;

pub(super) fn point_in_rect(point: (f32, f32), rect: &UiTransform) -> bool {
    point.0 >= rect.x
        && point.0 <= rect.x + rect.w
        && point.1 >= rect.y
        && point.1 <= rect.y + rect.h
}

#[derive(Debug, Copy, Clone)]
pub(super) struct EditorNodeRect {
    pub node: UiEditorNode,
    pub z: i32,
    pub rect: UiTransform,
}

pub(super) fn pick_editor_node_at(
    point: (f32, f32),
    nodes: &[EditorNodeRect],
) -> Option<UiEditorNode> {
    nodes
        .iter()
        .filter(|item| point_in_rect(point, &item.rect))
        .max_by_key(|item| item.z)
        .map(|item| item.node)
}

fn selected_editor_entity(ui: &crate::ui::ConsoleUiState) -> Option<ecs::EntityHandle> {
    match ui.editor.selected {
        Some(UiEditorNode::Root) => Some(ui.root),
        Some(UiEditorNode::Scrollback) => Some(ui.scrollback),
        Some(UiEditorNode::Input) => Some(ui.input),
        Some(UiEditorNode::ConfirmButton) => Some(ui.confirm_button),
        None => None,
    }
}

fn editor_node_label(node: UiEditorNode) -> &'static str {
    match node {
        UiEditorNode::Root => "root",
        UiEditorNode::Scrollback => "scrollback",
        UiEditorNode::Input => "input",
        UiEditorNode::ConfirmButton => "confirm_button",
    }
}

pub(super) fn snap_to_grid(value: f32, grid: f32) -> f32 {
    let g = grid.max(1.0);
    (value / g).round() * g
}

fn tint_color(color: [f32; 4], factor: f32) -> [f32; 4] {
    [
        (color[0] * factor).clamp(0.0, 1.0),
        (color[1] * factor).clamp(0.0, 1.0),
        (color[2] * factor).clamp(0.0, 1.0),
        color[3],
    ]
}

fn push_outline(
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

pub(super) fn estimate_text_width(text: &str, size: f32) -> f32 {
    // Monospace-biased estimate for centering labels in this MVP stage.
    text.chars().count() as f32 * size * 0.56
}

fn measure_text_advance_precise(metrics: &crate::ui::UiTextMetrics, text: &str, size: f32) -> f32 {
    let scale = text_scale(metrics, size);

    text.chars()
        .map(|ch| glyph_advance_with_scale(metrics, ch, scale))
        .sum()
}

fn char_count(text: &str) -> usize {
    text.chars().count()
}

fn byte_index_at_char(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

fn slice_chars(text: &str, start_char: usize, end_char: usize) -> String {
    let start = byte_index_at_char(text, start_char);
    let end = byte_index_at_char(text, end_char);
    text[start..end].to_string()
}

fn text_scale(metrics: &crate::ui::UiTextMetrics, size: f32) -> f32 {
    if metrics.base_size > 0.0 {
        (size / metrics.base_size).max(0.1)
    } else {
        1.0
    }
}

fn glyph_advance_with_scale(metrics: &crate::ui::UiTextMetrics, ch: char, scale: f32) -> f32 {
    let advance = metrics
        .glyphs
        .get(&ch)
        .map(|glyph| glyph.advance_px)
        .unwrap_or(metrics.fallback_advance);
    advance * scale
}

fn input_content_rect(transform: &UiTransform, ui_scale: f32) -> (f32, f32, f32, f32) {
    let pad_x = INPUT_PADDING_X * ui_scale;
    let pad_y = INPUT_PADDING_Y * ui_scale;
    let content_x = transform.x + pad_x;
    let content_y = transform.y + pad_y;
    let content_w = (transform.w - (pad_x * 2.0) - (2.0 * ui_scale)).max(1.0);
    let content_h = (transform.h - (pad_y * 2.0)).max(1.0);
    (content_x, content_y, content_w, content_h)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct WrappedEditorRow {
    pub start_char: usize,
    pub end_char: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct InputViewportLayout {
    pub content: String,
    pub caret_x: f32,
    pub caret_y: f32,
    pub viewport_row: usize,
    pub visible_rows: usize,
    pub total_rows: usize,
}

fn wrap_editor_rows_with_prompt(
    editor_text: &str,
    metrics: &crate::ui::UiTextMetrics,
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
pub(super) fn wrap_editor_rows(
    editor_text: &str,
    metrics: &crate::ui::UiTextMetrics,
    size: f32,
    max_width: f32,
) -> Vec<WrappedEditorRow> {
    let prompt_width = measure_text_advance_precise(metrics, CONSOLE_PROMPT, size);
    wrap_editor_rows_with_prompt(editor_text, metrics, size, max_width, prompt_width)
}

fn row_index_for_cursor(rows: &[WrappedEditorRow], cursor_chars: usize) -> usize {
    for (idx, row) in rows.iter().enumerate() {
        if cursor_chars >= row.start_char && cursor_chars <= row.end_char {
            return idx;
        }
    }
    rows.len().saturating_sub(1)
}

fn caret_x_for_row_cursor(
    text: &str,
    row: WrappedEditorRow,
    cursor_chars: usize,
    metrics: &crate::ui::UiTextMetrics,
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

fn cursor_for_row_x(
    text: &str,
    row: WrappedEditorRow,
    target_x: f32,
    metrics: &crate::ui::UiTextMetrics,
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

pub(super) fn move_cursor_vertical(
    editor: &mut crate::ui::EditorBuffer,
    metrics: &crate::ui::UiTextMetrics,
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

pub(super) fn build_visible_multiline_input(
    editor: &mut crate::ui::EditorBuffer,
    metrics: &crate::ui::UiTextMetrics,
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

fn fit_text_with_ellipsis_from_right(text: &str, size: f32, max_width: f32) -> String {
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

fn line_height(size: f32) -> f32 {
    size * 1.25
}

pub(super) fn visible_line_capacity(area_height: f32, text_size: f32) -> usize {
    let h = line_height(text_size).max(1.0);
    ((area_height / h).floor() as usize).max(1)
}

pub(super) fn build_scrollback_view_text(
    lines: &[String],
    lines_from_bottom: usize,
    visible_capacity: usize,
    max_line_width: f32,
    text_size: f32,
) -> String {
    if lines.is_empty() {
        return String::new();
    }

    let end = lines.len().saturating_sub(lines_from_bottom);
    let start = end.saturating_sub(visible_capacity);
    lines[start..end]
        .iter()
        .map(|line| fit_text_with_ellipsis_from_right(line, text_size, max_line_width))
        .collect::<Vec<_>>()
        .join("\n")
}

fn clamp_panel_dimension(target: f32, min_size: f32, max_size: f32) -> f32 {
    if max_size <= min_size {
        max_size.max(1.0)
    } else {
        target.clamp(min_size, max_size)
    }
}

pub(super) fn adaptive_footer_metrics(
    panel_available_w: f32,
    panel_h: f32,
    inner_padding: f32,
    scale: f32,
    base_footer_offset: f32,
    base_input_height: f32,
    base_button_width: f32,
    base_input_button_gap: f32,
) -> (f32, f32, f32, f32) {
    let min_input_h = (16.0 * scale).max(14.0);
    let input_h = (base_input_height * scale)
        .min((panel_h - inner_padding * 2.0).max(min_input_h))
        .max(min_input_h);

    let min_gap = (4.0 * scale).max(2.0);
    let min_button_w = (44.0 * scale).max(28.0);
    let min_input_w = (48.0 * scale).max(28.0);
    let footer_w = panel_available_w.max(1.0);
    let (button_w, input_w) = if footer_w > (min_button_w + min_input_w + min_gap) {
        let gap = (base_input_button_gap * scale).min((footer_w * 0.08).max(min_gap));
        let button_cap_by_footer = (footer_w - gap - min_input_w).max(min_button_w);
        let button_w = (base_button_width * scale)
            .min(button_cap_by_footer)
            .max(min_button_w);
        let input_w = (footer_w - button_w - gap).max(min_input_w);
        (button_w, input_w)
    } else {
        // Degrade gracefully on extremely small panels while keeping controls inside bounds.
        let gap = (footer_w * 0.06).clamp(1.0, min_gap);
        let button_w = (footer_w * 0.42).max(12.0).min((footer_w - gap).max(12.0));
        let input_w = (footer_w - button_w - gap).max(1.0);
        (button_w, input_w)
    };

    let footer_y = (base_footer_offset * scale)
        .min((panel_h - inner_padding - input_h).max(inner_padding))
        .max(inner_padding);

    (footer_y, input_h, button_w, input_w)
}

pub fn ui_hot_reload_system(data: &mut EngineData) -> anyhow::Result<()> {
    if let Err(err) = reload_console_template_if_changed(&mut data.world, &mut data.ui, false) {
        tracing::warn!(?err, "ui hot reload failed");
    }
    Ok(())
}

pub fn ui_editor_system(data: &mut EngineData) -> anyhow::Result<()> {
    if !data.ui.editor.enabled {
        return Ok(());
    }

    let candidates = [
        (UiEditorNode::Root, data.ui.root),
        (UiEditorNode::Scrollback, data.ui.scrollback),
        (UiEditorNode::Input, data.ui.input),
        (UiEditorNode::ConfirmButton, data.ui.confirm_button),
    ];

    if data.input.left_mouse_pressed() {
        let mut rects: Vec<EditorNodeRect> = Vec::new();
        for (node, entity) in candidates {
            if let (Some(transform), Some(ui_node)) = (
                data.world.get_component::<UiTransform>(entity),
                data.world.get_component::<crate::ui::UiNode>(entity),
            ) {
                rects.push(EditorNodeRect {
                    node,
                    z: ui_node.z,
                    rect: *transform,
                });
            }
        }

        data.ui.editor.selected = pick_editor_node_at(data.input.mouse_position, &rects);
        data.ui.editor.dragging = false;
        data.ui.editor.drag_pointer_offset = (0.0, 0.0);
        match data.ui.editor.selected {
            Some(node) => {
                if let Some(selected_rect) = rects.iter().find(|r| r.node == node).map(|r| r.rect) {
                    data.ui.editor.drag_pointer_offset = (
                        data.input.mouse_position.0 - selected_rect.x,
                        data.input.mouse_position.1 - selected_rect.y,
                    );
                    data.ui.editor.dragging = true;
                }
                data.ui.editor.status = format!("editor: selected {}", editor_node_label(node));
            }
            None => {
                data.ui.editor.status = "editor: nothing selected".to_string();
            }
        }
    }

    if data.input.left_mouse_released() {
        data.ui.editor.dragging = false;
    }

    if let Some(selected_node) = data.ui.editor.selected {
        let Some(selected) = selected_editor_entity(&data.ui) else {
            return Ok(());
        };
        let step = if data.input.shift_down() {
            10.0 * data.ui.scale.max(1.0)
        } else {
            EDITOR_BASE_NUDGE_PX * data.ui.scale.max(1.0)
        };
        let mut dx = 0.0;
        let mut dy = 0.0;
        if data.input.move_left {
            dx -= step;
        }
        if data.input.move_right {
            dx += step;
        }
        if data.input.move_up {
            dy -= step;
        }
        if data.input.move_down {
            dy += step;
        }
        if (dx != 0.0 || dy != 0.0)
            && let Some(transform) = data.world.get_component_mut::<UiTransform>(selected)
        {
            transform.x += dx;
            transform.y += dy;
            data.ui.editor.status = format!(
                "editor: nudged {} to ({:.0}, {:.0})",
                editor_node_label(selected_node),
                transform.x,
                transform.y
            );
        }

        if data.ui.editor.dragging
            && data.input.left_mouse_down()
            && let Some(transform) = data.world.get_component_mut::<UiTransform>(selected)
        {
            let mut next_x = data.input.mouse_position.0 - data.ui.editor.drag_pointer_offset.0;
            let mut next_y = data.input.mouse_position.1 - data.ui.editor.drag_pointer_offset.1;
            if data.input.shift_down() {
                let grid = EDITOR_DRAG_SNAP_PX * data.ui.scale.max(1.0);
                next_x = snap_to_grid(next_x, grid);
                next_y = snap_to_grid(next_y, grid);
            }
            transform.x = next_x;
            transform.y = next_y;
            data.ui.editor.status = format!(
                "editor: dragging {} ({:.0}, {:.0})",
                editor_node_label(selected_node),
                transform.x,
                transform.y
            );
        }
    }

    if data.input.save_ui_template {
        match save_console_template_to_disk(&data.world, &mut data.ui) {
            Ok(path) => {
                data.ui.editor.status = format!("editor: saved {}", path.display());
                let _ = reload_console_template_if_changed(&mut data.world, &mut data.ui, true);
            }
            Err(err) => {
                data.ui.editor.status = format!("editor: save failed: {err:#}");
            }
        }
    }

    Ok(())
}

pub fn ui_input_system(data: &mut EngineData) -> anyhow::Result<()> {
    if data.input.toggle_ui_editor_mode {
        data.ui.editor.enabled = !data.ui.editor.enabled;
        data.ui.editor.dragging = false;
        data.ui.editor.drag_pointer_offset = (0.0, 0.0);
        data.ui.editor.status = if data.ui.editor.enabled {
            "editor: on (click+drag move, Shift snap, arrows nudge, Cmd/Ctrl+S save, F1 off)"
                .to_string()
        } else {
            "editor: off (F1 to toggle)".to_string()
        };
    }

    if data.ui.editor.enabled {
        return Ok(());
    }

    let input_entity = data.ui.input;
    let button_entity = data.ui.confirm_button;
    let scroll_entity = data.ui.scrollback;
    let ui_scale = data.ui.scale.max(1.0);
    let input_nav_metrics = if let (Some(transform), Some(text)) = (
        data.world.get_component::<UiTransform>(input_entity),
        data.world.get_component::<UiText>(input_entity),
    ) {
        let (_, _, content_w, _) = input_content_rect(transform, ui_scale);
        Some((text.size * ui_scale, content_w))
    } else {
        None
    };
    let text_metrics = &data.ui.text_metrics;

    let mut edited_text = false;
    let mut moved_cursor = false;
    {
        let editor = &mut data.ui.input_editor;
        let total_chars = char_count(&editor.text);
        editor.cursor_chars = editor.cursor_chars.min(total_chars);
        let mut reset_preferred_x = false;

        if !data.input.typed_text.is_empty() {
            let insert_at = byte_index_at_char(&editor.text, editor.cursor_chars);
            editor.text.insert_str(insert_at, &data.input.typed_text);
            editor.cursor_chars += char_count(&data.input.typed_text);
            edited_text = true;
            reset_preferred_x = true;
        }

        if data.input.insert_newline {
            let insert_at = byte_index_at_char(&editor.text, editor.cursor_chars);
            editor.text.insert(insert_at, '\n');
            editor.cursor_chars += 1;
            edited_text = true;
            reset_preferred_x = true;
        }

        if data.input.backspace && editor.cursor_chars > 0 {
            let remove_at = editor.cursor_chars - 1;
            let start = byte_index_at_char(&editor.text, remove_at);
            let end = byte_index_at_char(&editor.text, editor.cursor_chars);
            editor.text.replace_range(start..end, "");
            editor.cursor_chars = remove_at;
            edited_text = true;
            reset_preferred_x = true;
        }

        if data.input.delete && editor.cursor_chars < char_count(&editor.text) {
            let start = byte_index_at_char(&editor.text, editor.cursor_chars);
            let end = byte_index_at_char(&editor.text, editor.cursor_chars + 1);
            editor.text.replace_range(start..end, "");
            edited_text = true;
            reset_preferred_x = true;
        }

        if data.input.move_left {
            editor.cursor_chars = editor.cursor_chars.saturating_sub(1);
            moved_cursor = true;
            reset_preferred_x = true;
        }
        if data.input.move_right {
            editor.cursor_chars = (editor.cursor_chars + 1).min(char_count(&editor.text));
            moved_cursor = true;
            reset_preferred_x = true;
        }
        if data.input.move_home {
            editor.cursor_chars = 0;
            moved_cursor = true;
            reset_preferred_x = true;
        }
        if data.input.move_end {
            editor.cursor_chars = char_count(&editor.text);
            moved_cursor = true;
            reset_preferred_x = true;
        }

        if reset_preferred_x {
            editor.preferred_caret_x = None;
        }

        if let Some((input_text_size, input_content_w)) = input_nav_metrics {
            if data.input.move_up
                && move_cursor_vertical(
                    editor,
                    text_metrics,
                    input_text_size,
                    input_content_w,
                    false,
                )
            {
                moved_cursor = true;
            }
            if data.input.move_down
                && move_cursor_vertical(
                    editor,
                    text_metrics,
                    input_text_size,
                    input_content_w,
                    true,
                )
            {
                moved_cursor = true;
            }
        }
    }

    if edited_text || moved_cursor {
        if let Some(dirty) = data.world.get_component_mut::<UiDirty>(input_entity) {
            dirty.text = true;
        }
        data.ui.caret_visible = true;
        data.ui.caret_blink_timer = 0.0;
    }

    if data.input.submitted {
        if let Some(field) = data.world.get_component_mut::<UiInputField>(input_entity) {
            field.submit_requested = true;
        }
    }

    let hovered = data
        .world
        .get_component::<UiTransform>(button_entity)
        .map(|rect| point_in_rect(data.input.mouse_position, rect))
        .unwrap_or(false);
    let clicked = hovered && data.input.left_mouse_pressed();
    let pressed = hovered && data.input.left_mouse_down();

    if let Some(interaction) = data.world.get_component_mut::<UiInteraction>(button_entity) {
        interaction.hovered = hovered;
        interaction.clicked = clicked;
        interaction.pressed = pressed;
    }

    if clicked {
        if let Some(field) = data.world.get_component_mut::<UiInputField>(input_entity) {
            field.submit_requested = true;
        }
    }

    let input_clicked = data
        .world
        .get_component::<UiTransform>(input_entity)
        .map(|rect| point_in_rect(data.input.mouse_position, rect))
        .unwrap_or(false)
        && data.input.left_mouse_pressed();
    if input_clicked {
        if let Some(interaction) = data.world.get_component_mut::<UiInteraction>(input_entity) {
            interaction.focused = true;
        }
        data.ui.caret_visible = true;
        data.ui.caret_blink_timer = 0.0;
    }

    let mut should_submit = false;
    if let Some(field) = data.world.get_component_mut::<UiInputField>(input_entity) {
        if field.submit_requested {
            field.submit_requested = false;
            should_submit = true;
        }
    }

    if should_submit {
        let line = format!("{CONSOLE_PROMPT}{}", data.ui.input_editor.text);
        data.world.spawn_entity_typed(UiSubmitEvent { line });

        data.ui.input_editor.text.clear();
        data.ui.input_editor.cursor_chars = 0;
        data.ui.input_editor.viewport_row = 0;
        data.ui.input_editor.preferred_caret_x = None;
        if let Some(field) = data.world.get_component_mut::<UiInputField>(input_entity) {
            field.cursor = 0;
            field.focused = true;
        }
        if let Some(dirty) = data.world.get_component_mut::<UiDirty>(input_entity) {
            dirty.text = true;
        }
        if let Some(interaction) = data.world.get_component_mut::<UiInteraction>(input_entity) {
            interaction.focused = true;
        }
        data.ui.caret_visible = true;
        data.ui.caret_blink_timer = 0.0;
    }

    if let Some(input_text) = data.world.get_component_mut::<UiText>(input_entity) {
        input_text.content = format!("{CONSOLE_PROMPT}{}", data.ui.input_editor.text);
    }
    if let Some(field) = data.world.get_component_mut::<UiInputField>(input_entity) {
        field.cursor = data.ui.input_editor.cursor_chars;
    }

    data.ui.caret_blink_timer += data.time.delta_seconds;
    while data.ui.caret_blink_timer >= CARET_BLINK_SECONDS {
        data.ui.caret_blink_timer -= CARET_BLINK_SECONDS;
        data.ui.caret_visible = !data.ui.caret_visible;
    }

    let wants_mouse_scroll = data
        .world
        .get_component::<UiTransform>(scroll_entity)
        .map(|rect| point_in_rect(data.input.mouse_position, rect))
        .unwrap_or(false);
    let mut delta_lines: i32 = 0;
    if wants_mouse_scroll {
        if data.input.scroll_delta > 0.0 {
            delta_lines += 3;
        } else if data.input.scroll_delta < 0.0 {
            delta_lines -= 3;
        }
    }
    if data.input.page_up {
        delta_lines += 12;
    }
    if data.input.page_down {
        delta_lines -= 12;
    }

    if delta_lines != 0 {
        if delta_lines > 0 {
            data.ui.scroll_lines_from_bottom = data
                .ui
                .scroll_lines_from_bottom
                .saturating_add(delta_lines as usize);
        } else {
            data.ui.scroll_lines_from_bottom = data
                .ui
                .scroll_lines_from_bottom
                .saturating_sub((-delta_lines) as usize);
        }
    }

    Ok(())
}

pub fn ui_layout_system(data: &mut EngineData) -> anyhow::Result<()> {
    if !data.ui.layout_dirty {
        return Ok(());
    }

    let (screen_w, screen_h) = data.ui.screen_size;
    let s = data.ui.scale.max(1.0);
    let outer_margin = data.ui.layout.outer_margin * s;
    let available_w = (screen_w - (outer_margin * 2.0)).max(1.0);
    let available_h = (screen_h - (outer_margin * 2.0)).max(1.0);
    let panel_w = clamp_panel_dimension(
        screen_w * data.ui.layout.panel_width_ratio,
        data.ui.layout.panel_min_width * s,
        available_w,
    );
    let panel_h = clamp_panel_dimension(
        screen_h * data.ui.layout.panel_height_ratio,
        data.ui.layout.panel_min_height * s,
        available_h,
    );
    let panel_x = outer_margin;
    let panel_y = (screen_h - panel_h - outer_margin).max(outer_margin);
    let inner_padding = data.ui.layout.inner_padding * s;
    let panel_inner_w = (panel_w - inner_padding * 2.0).max(1.0);
    let (footer_y, input_h, button_w, input_w) = adaptive_footer_metrics(
        panel_inner_w,
        panel_h,
        inner_padding,
        s,
        data.ui.layout.footer_offset,
        data.ui.layout.input_height,
        data.ui.layout.button_width,
        data.ui.layout.input_button_gap,
    );

    if let Some(root) = data.world.get_component_mut::<UiTransform>(data.ui.root) {
        root.x = panel_x;
        root.y = panel_y;
        root.w = panel_w;
        root.h = panel_h;
    }

    if let Some(scroll) = data
        .world
        .get_component_mut::<UiTransform>(data.ui.scrollback)
    {
        scroll.x = panel_x + inner_padding;
        scroll.y = panel_y + inner_padding;
        scroll.w = panel_inner_w;
        scroll.h = panel_h - footer_y - inner_padding;
    }

    if let Some(input) = data.world.get_component_mut::<UiTransform>(data.ui.input) {
        input.x = panel_x + inner_padding;
        input.y = panel_y + panel_h - footer_y;
        input.w = input_w;
        input.h = input_h;
    }

    if let Some(button) = data
        .world
        .get_component_mut::<UiTransform>(data.ui.confirm_button)
    {
        button.x = panel_x + panel_w - inner_padding - button_w;
        button.y = panel_y + panel_h - footer_y;
        button.w = button_w;
        button.h = input_h;
    }

    data.ui.layout_dirty = false;
    Ok(())
}

pub fn ui_build_batches_system(data: &mut EngineData) -> anyhow::Result<()> {
    let ui_scale = data.ui.scale.max(1.0);
    let mut commands: Vec<UiBatchCmd> = Vec::new();

    if let (Some(transform), Some(style)) = (
        data.world.get_component::<UiTransform>(data.ui.root),
        data.world.get_component::<UiStyle>(data.ui.root),
    ) {
        commands.push(UiBatchCmd::Rect {
            x: transform.x,
            y: transform.y,
            w: transform.w,
            h: transform.h,
            color: style.bg_color,
            radius: style.radius * ui_scale,
        });
    }

    if let (Some(transform), Some(text)) = (
        data.world.get_component::<UiTransform>(data.ui.scrollback),
        data.world.get_component::<UiText>(data.ui.scrollback),
    ) {
        let text_size = text.size * ui_scale;
        let visible_capacity = visible_line_capacity(transform.h, text_size);
        let max_scroll = data.ui.lines.len().saturating_sub(visible_capacity);
        data.ui.scroll_lines_from_bottom = data.ui.scroll_lines_from_bottom.min(max_scroll);
        let line_max_w = (transform.w - (8.0 * ui_scale)).max(1.0);
        let view = build_scrollback_view_text(
            &data.ui.lines,
            data.ui.scroll_lines_from_bottom,
            visible_capacity,
            line_max_w,
            text_size,
        );
        commands.push(UiBatchCmd::Text {
            x: transform.x,
            y: transform.y,
            content: view,
            color: text.color,
            size: text_size,
            clip: Some([transform.x, transform.y, transform.w, transform.h]),
        });
    }

    if let (Some(transform), Some(style)) = (
        data.world.get_component::<UiTransform>(data.ui.input),
        data.world.get_component::<UiStyle>(data.ui.root),
    ) {
        commands.push(UiBatchCmd::Rect {
            x: transform.x,
            y: transform.y,
            w: transform.w,
            h: transform.h,
            color: [
                style.bg_color[0] * 0.72,
                style.bg_color[1] * 0.74,
                style.bg_color[2] * 0.78,
                1.0,
            ],
            radius: 4.0 * ui_scale,
        });
    }

    if let (Some(transform), Some(text), Some(input_field)) = (
        data.world.get_component::<UiTransform>(data.ui.input),
        data.world.get_component::<UiText>(data.ui.input),
        data.world.get_component::<UiInputField>(data.ui.input),
    ) {
        let scaled_text_size = text.size * ui_scale;
        let (content_x, content_y, content_w, content_h) = input_content_rect(transform, ui_scale);
        let layout = build_visible_multiline_input(
            &mut data.ui.input_editor,
            &data.ui.text_metrics,
            scaled_text_size,
            content_w,
            content_h,
        );
        commands.push(UiBatchCmd::Text {
            x: content_x,
            y: content_y,
            content: layout.content,
            color: text.color,
            size: scaled_text_size,
            clip: Some([transform.x, transform.y, transform.w, transform.h]),
        });

        if data.ui.caret_visible && input_field.focused {
            let caret_w = (2.0 * ui_scale).max(1.0);
            let caret_h = (scaled_text_size * 0.9).min(content_h).max(1.0);
            let max_caret_x = (content_x + content_w - caret_w).max(content_x);
            let max_caret_y = (content_y + content_h - caret_h).max(content_y);
            let caret_x = (content_x + layout.caret_x).clamp(content_x, max_caret_x);
            let caret_y = (content_y + layout.caret_y).clamp(content_y, max_caret_y);
            commands.push(UiBatchCmd::Rect {
                x: caret_x,
                y: caret_y,
                w: caret_w,
                h: caret_h,
                color: [0.92, 0.95, 0.98, 0.95],
                radius: 1.0 * ui_scale,
            });
        }
    }

    if let (Some(transform), Some(style), Some(button), Some(interaction), Some(text)) = (
        data.world
            .get_component::<UiTransform>(data.ui.confirm_button),
        data.world.get_component::<UiStyle>(data.ui.confirm_button),
        data.world.get_component::<UiButton>(data.ui.confirm_button),
        data.world
            .get_component::<UiInteraction>(data.ui.confirm_button),
        data.world.get_component::<UiText>(data.ui.confirm_button),
    ) {
        let color = if !button.enabled {
            tint_color(style.bg_color, 0.6)
        } else if interaction.pressed {
            tint_color(style.bg_color, 0.82)
        } else if interaction.hovered {
            tint_color(style.bg_color, 1.18)
        } else {
            style.bg_color
        };

        commands.push(UiBatchCmd::Rect {
            x: transform.x,
            y: transform.y,
            w: transform.w,
            h: transform.h,
            color,
            radius: style.radius * ui_scale,
        });
        let scaled_text_size = text.size * ui_scale;
        let text_w = estimate_text_width(&text.content, scaled_text_size);
        let text_x = transform.x + ((transform.w - text_w) * 0.5).max(0.0);
        let text_y = transform.y + ((transform.h - scaled_text_size) * 0.5).max(0.0);
        commands.push(UiBatchCmd::Text {
            x: text_x,
            y: text_y,
            content: text.content.clone(),
            color: text.color,
            size: scaled_text_size,
            clip: Some([transform.x, transform.y, transform.w, transform.h]),
        });
    }

    if data.ui.editor.enabled {
        if let Some(selected_entity) = selected_editor_entity(&data.ui)
            && let Some(selected_rect) = data.world.get_component::<UiTransform>(selected_entity)
        {
            push_outline(
                &mut commands,
                selected_rect,
                2.0 * ui_scale,
                [0.95, 0.55, 0.15, 0.95],
            );
        }

        if let Some(root_rect) = data.world.get_component::<UiTransform>(data.ui.root) {
            commands.push(UiBatchCmd::Text {
                x: root_rect.x + (8.0 * ui_scale),
                y: root_rect.y + (8.0 * ui_scale),
                content: data.ui.editor.status.clone(),
                color: [0.98, 0.84, 0.52, 1.0],
                size: 12.0 * ui_scale,
                clip: Some([root_rect.x, root_rect.y, root_rect.w, root_rect.h]),
            });
        }
    }

    data.ui.batches.commands = commands;
    Ok(())
}

pub fn ui_render_extract_system(data: &mut EngineData) -> anyhow::Result<()> {
    let commands = data
        .ui
        .batches
        .commands
        .iter()
        .map(|cmd| match cmd {
            UiBatchCmd::Rect {
                x,
                y,
                w,
                h,
                color,
                radius,
            } => UiDrawCmd::Rect {
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                color: *color,
                radius: *radius,
            },
            UiBatchCmd::Text {
                x,
                y,
                content,
                color,
                size,
                clip,
            } => UiDrawCmd::Text {
                x: *x,
                y: *y,
                content: content.clone(),
                color: *color,
                size: *size,
                clip: *clip,
            },
        })
        .collect();

    data.ui.draw_list.commands = commands;
    Ok(())
}
