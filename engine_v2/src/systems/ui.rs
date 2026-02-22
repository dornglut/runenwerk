use crate::runtime::{AgentHealth, AgentPosition, AgentTeam, EngineData, OverlaySubmitMessage};
use crate::ui::{
    UiBatchCmd, UiButton, UiDirty, UiDrawCmd, UiEditorNode, UiInputField, UiInteraction, UiNode,
    UiStyle, UiText, UiTransform, reload_console_template_if_changed,
    save_console_template_to_disk,
};

const CONSOLE_PROMPT: &str = "grotto> ";
const CARET_BLINK_SECONDS: f32 = 0.5;
const INPUT_PADDING_X: f32 = 6.0;
const INPUT_PADDING_Y: f32 = 4.0;
const EDITOR_BASE_NUDGE_PX: f32 = 1.0;
const EDITOR_DRAG_SNAP_PX: f32 = 10.0;

#[derive(Debug, Copy, Clone)]
struct LogWindowRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    body_x: f32,
    body_y: f32,
    body_w: f32,
    body_h: f32,
}

fn compute_log_window_rect(
    layout: &crate::ui::UiLayoutConfig,
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

fn world_hud_stats(data: &EngineData) -> Option<(f32, f32, usize, u32)> {
    let world = &data.scene.world_runtime.ctx.world;
    let mut player_pos: Option<(f32, f32)> = None;
    let mut enemy_count: usize = 0;
    for entity in world.entities_with::<AgentTeam>() {
        let Some(team) = world.get_component::<AgentTeam>(entity).copied() else {
            continue;
        };
        if team == AgentTeam::Enemy {
            let alive = world
                .get_component::<AgentHealth>(entity)
                .map(|h| h.current > 0)
                .unwrap_or(false);
            if alive {
                enemy_count = enemy_count.saturating_add(1);
            }
        } else if player_pos.is_none()
            && let Some(pos) = world.get_component::<AgentPosition>(entity).copied()
        {
            player_pos = Some((pos.x, pos.y));
        }
    }
    player_pos.map(|(x, y)| (x, y, enemy_count, data.scene.world_runtime.ctx.enemy_kills))
}

fn push_world_stats_panel(commands: &mut Vec<UiBatchCmd>, data: &EngineData, ui_scale: f32) {
    let Some((px, py, enemies, slain)) = world_hud_stats(data) else {
        return;
    };

    let margin = 10.0 * ui_scale;
    let panel_w = 300.0 * ui_scale;
    let panel_h = 74.0 * ui_scale;
    let panel_x = margin;
    let panel_y = margin;
    let panel_w =
        panel_w.min((data.scene.overlay_runtime.ui.screen_size.0 - (margin * 2.0)).max(80.0));
    let panel_h =
        panel_h.min((data.scene.overlay_runtime.ui.screen_size.1 - (margin * 2.0)).max(40.0));
    let clip = Some([panel_x, panel_y, panel_w, panel_h]);

    commands.push(UiBatchCmd::Rect {
        x: panel_x,
        y: panel_y,
        w: panel_w,
        h: panel_h,
        color: [0.05, 0.08, 0.12, 0.86],
        radius: 8.0 * ui_scale,
    });
    commands.push(UiBatchCmd::Text {
        x: panel_x + (10.0 * ui_scale),
        y: panel_y + (10.0 * ui_scale),
        content: "World Stats".to_string(),
        color: [0.88, 0.95, 1.0, 1.0],
        size: 12.0 * ui_scale,
        clip,
    });
    commands.push(UiBatchCmd::Text {
        x: panel_x + (10.0 * ui_scale),
        y: panel_y + (30.0 * ui_scale),
        content: format!("player=({:.1}, {:.1})", px, py),
        color: [0.56, 0.94, 0.66, 1.0],
        size: 11.0 * ui_scale,
        clip,
    });
    commands.push(UiBatchCmd::Text {
        x: panel_x + (10.0 * ui_scale),
        y: panel_y + (48.0 * ui_scale),
        content: format!("enemies={} slain={}", enemies, slain),
        color: [0.98, 0.74, 0.42, 1.0],
        size: 11.0 * ui_scale,
        clip,
    });
}

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

fn ui_node_visible(data: &EngineData, entity: ecs::EntityHandle) -> bool {
    data.scene
        .overlay_runtime
        .world
        .get_component::<UiNode>(entity)
        .map(|n| n.visible)
        .unwrap_or(true)
}

fn editor_node_label(node: UiEditorNode) -> &'static str {
    match node {
        UiEditorNode::Root => "root",
        UiEditorNode::Scrollback => "scrollback",
        UiEditorNode::Input => "input",
        UiEditorNode::ConfirmButton => "confirm_button",
    }
}

fn apply_editor_translation(data: &mut EngineData, selected_node: UiEditorNode, dx: f32, dy: f32) {
    if dx == 0.0 && dy == 0.0 {
        return;
    }

    let mut translate = |entity: ecs::EntityHandle| {
        if let Some(transform) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiTransform>(entity)
        {
            transform.x += dx;
            transform.y += dy;
        }
    };

    translate(
        selected_editor_entity(&data.scene.overlay_runtime.ui)
            .expect("selected node should have entity"),
    );
    if selected_node == UiEditorNode::Root {
        translate(data.scene.overlay_runtime.ui.scrollback);
        translate(data.scene.overlay_runtime.ui.input);
        translate(data.scene.overlay_runtime.ui.confirm_button);
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

fn push_scroll_indicators(
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

fn build_scrollback_view_lines(
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

fn wrapped_line_rows(
    line: &str,
    horizontal_chars: usize,
    max_width: f32,
    text_size: f32,
    metrics: &crate::ui::UiTextMetrics,
) -> Vec<String> {
    let start = byte_index_at_char(line, horizontal_chars.min(char_count(line)));
    let shifted = &line[start..];
    if shifted.is_empty() {
        return vec![String::new()];
    }
    let mut rows = Vec::new();
    let chars: Vec<char> = shifted.chars().collect();
    let scale = text_scale(metrics, text_size);
    let mut idx = 0usize;
    while idx < chars.len() {
        let mut row = String::new();
        let mut width = 0.0_f32;
        while idx < chars.len() {
            let ch = chars[idx];
            let advance = glyph_advance_with_scale(metrics, ch, scale);
            if !row.is_empty() && (width + advance) > max_width {
                break;
            }
            row.push(ch);
            width += advance;
            idx += 1;
            if row.is_empty() {
                break;
            }
        }
        if row.is_empty() {
            row.push(chars[idx]);
            idx += 1;
        }
        rows.push(row);
    }
    rows
}

fn flatten_wrapped_rows(
    lines: &[String],
    horizontal_chars: usize,
    max_width: f32,
    text_size: f32,
    metrics: &crate::ui::UiTextMetrics,
) -> Vec<String> {
    let mut rows = Vec::new();
    for line in lines {
        rows.extend(wrapped_line_rows(
            line,
            horizontal_chars,
            max_width,
            text_size,
            metrics,
        ));
    }
    rows
}

fn build_wrapped_view_rows(
    lines: &[String],
    lines_from_bottom: usize,
    visible_capacity: usize,
    horizontal_chars: usize,
    max_width: f32,
    text_size: f32,
    metrics: &crate::ui::UiTextMetrics,
) -> Vec<String> {
    let wrapped = flatten_wrapped_rows(lines, horizontal_chars, max_width, text_size, metrics);
    if wrapped.is_empty() {
        return Vec::new();
    }
    let end = wrapped.len().saturating_sub(lines_from_bottom);
    let start = end.saturating_sub(visible_capacity);
    wrapped[start..end].to_vec()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn build_scrollback_view_text(
    lines: &[String],
    lines_from_bottom: usize,
    visible_capacity: usize,
    max_line_width: f32,
    text_size: f32,
) -> String {
    build_scrollback_view_lines(
        lines,
        lines_from_bottom,
        visible_capacity,
        max_line_width,
        text_size,
    )
    .join("\n")
}

pub(super) fn scrollback_line_style(line: &str, default: [f32; 4]) -> ([f32; 4], &str) {
    if let Some(rest) = line.strip_prefix("[world] ") {
        return ([0.70, 0.86, 0.98, 1.0], rest);
    }
    if let Some(rest) = line.strip_prefix("[combat] ") {
        return ([0.98, 0.58, 0.46, 1.0], rest);
    }
    if let Some(rest) = line.strip_prefix("[loot] ") {
        return ([0.58, 0.92, 0.62, 1.0], rest);
    }
    if let Some(rest) = line.strip_prefix("[quest] ") {
        return ([0.95, 0.86, 0.50, 1.0], rest);
    }
    (default, line)
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
    if let Err(err) = reload_console_template_if_changed(
        &mut data.scene.overlay_runtime.world,
        &mut data.scene.overlay_runtime.ui,
        false,
    ) {
        tracing::warn!(?err, "ui hot reload failed");
    }
    Ok(())
}

pub fn ui_editor_system(data: &mut EngineData) -> anyhow::Result<()> {
    if !data.scene.overlay_runtime.ui.editor.enabled {
        return Ok(());
    }

    let candidates = [
        (UiEditorNode::Root, data.scene.overlay_runtime.ui.root),
        (
            UiEditorNode::Scrollback,
            data.scene.overlay_runtime.ui.scrollback,
        ),
        (UiEditorNode::Input, data.scene.overlay_runtime.ui.input),
        (
            UiEditorNode::ConfirmButton,
            data.scene.overlay_runtime.ui.confirm_button,
        ),
    ];

    if data.input.left_mouse_pressed() {
        let mut rects: Vec<EditorNodeRect> = Vec::new();
        for (node, entity) in candidates {
            if let (Some(transform), Some(ui_node)) = (
                data.scene
                    .overlay_runtime
                    .world
                    .get_component::<UiTransform>(entity),
                data.scene
                    .overlay_runtime
                    .world
                    .get_component::<crate::ui::UiNode>(entity),
            ) {
                if !ui_node.visible {
                    continue;
                }
                rects.push(EditorNodeRect {
                    node,
                    z: ui_node.z,
                    rect: *transform,
                });
            }
        }

        data.scene.overlay_runtime.ui.editor.selected =
            pick_editor_node_at(data.input.mouse_position, &rects);
        data.scene.overlay_runtime.ui.editor.dragging = false;
        data.scene.overlay_runtime.ui.editor.drag_pointer_offset = (0.0, 0.0);
        match data.scene.overlay_runtime.ui.editor.selected {
            Some(node) => {
                if let Some(selected_rect) = rects.iter().find(|r| r.node == node).map(|r| r.rect) {
                    data.scene.overlay_runtime.ui.editor.drag_pointer_offset = (
                        data.input.mouse_position.0 - selected_rect.x,
                        data.input.mouse_position.1 - selected_rect.y,
                    );
                    data.scene.overlay_runtime.ui.editor.dragging = true;
                }
                data.scene.overlay_runtime.ui.editor.status =
                    format!("editor: selected {}", editor_node_label(node));
            }
            None => {
                data.scene.overlay_runtime.ui.editor.status =
                    "editor: nothing selected".to_string();
            }
        }
    }

    if data.input.left_mouse_released() {
        data.scene.overlay_runtime.ui.editor.dragging = false;
    }

    if let Some(selected_node) = data.scene.overlay_runtime.ui.editor.selected {
        let Some(selected) = selected_editor_entity(&data.scene.overlay_runtime.ui) else {
            return Ok(());
        };
        let step = if data.input.shift_down() {
            10.0 * data.scene.overlay_runtime.ui.scale.max(1.0)
        } else {
            EDITOR_BASE_NUDGE_PX * data.scene.overlay_runtime.ui.scale.max(1.0)
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
            && data
                .scene
                .overlay_runtime
                .world
                .get_component::<UiTransform>(selected)
                .is_some()
        {
            apply_editor_translation(data, selected_node, dx, dy);
            let pos = data
                .scene
                .overlay_runtime
                .world
                .get_component::<UiTransform>(selected)
                .map(|t| (t.x, t.y))
                .unwrap_or((0.0, 0.0));
            data.scene.overlay_runtime.ui.editor.status = format!(
                "editor: nudged {} to ({:.0}, {:.0})",
                editor_node_label(selected_node),
                pos.0,
                pos.1
            );
        }

        if data.scene.overlay_runtime.ui.editor.dragging
            && data.input.left_mouse_down()
            && let Some(transform) = data
                .scene
                .overlay_runtime
                .world
                .get_component_mut::<UiTransform>(selected)
        {
            let mut next_x = data.input.mouse_position.0
                - data.scene.overlay_runtime.ui.editor.drag_pointer_offset.0;
            let mut next_y = data.input.mouse_position.1
                - data.scene.overlay_runtime.ui.editor.drag_pointer_offset.1;
            if data.input.shift_down() {
                let grid = EDITOR_DRAG_SNAP_PX * data.scene.overlay_runtime.ui.scale.max(1.0);
                next_x = snap_to_grid(next_x, grid);
                next_y = snap_to_grid(next_y, grid);
            }
            let dx = next_x - transform.x;
            let dy = next_y - transform.y;
            let _ = transform;
            apply_editor_translation(data, selected_node, dx, dy);
            let pos = data
                .scene
                .overlay_runtime
                .world
                .get_component::<UiTransform>(selected)
                .map(|t| (t.x, t.y))
                .unwrap_or((next_x, next_y));
            data.scene.overlay_runtime.ui.editor.status = format!(
                "editor: dragging {} ({:.0}, {:.0})",
                editor_node_label(selected_node),
                pos.0,
                pos.1
            );
        }

        if data.input.editor_hide_selected {
            let can_hide = selected_node != UiEditorNode::Root;
            if can_hide {
                if let Some(node) = data
                    .scene
                    .overlay_runtime
                    .world
                    .get_component_mut::<UiNode>(selected)
                {
                    node.visible = false;
                    data.scene.overlay_runtime.ui.editor.status = format!(
                        "editor: hid {} (A restores hidden nodes)",
                        editor_node_label(selected_node)
                    );
                }
                data.scene.overlay_runtime.ui.editor.selected = Some(UiEditorNode::Root);
                data.scene.overlay_runtime.ui.editor.dragging = false;
            } else {
                data.scene.overlay_runtime.ui.editor.status =
                    "editor: root cannot be hidden".to_string();
            }
        }
    }

    if data.input.editor_restore_all {
        for entity in [
            data.scene.overlay_runtime.ui.root,
            data.scene.overlay_runtime.ui.scrollback,
            data.scene.overlay_runtime.ui.input,
            data.scene.overlay_runtime.ui.confirm_button,
        ] {
            if let Some(node) = data
                .scene
                .overlay_runtime
                .world
                .get_component_mut::<UiNode>(entity)
            {
                node.visible = true;
            }
        }
        data.scene.overlay_runtime.ui.editor.status = "editor: restored all nodes".to_string();
    }

    if data.input.save_ui_template {
        match save_console_template_to_disk(
            &data.scene.overlay_runtime.world,
            &mut data.scene.overlay_runtime.ui,
        ) {
            Ok(path) => {
                data.scene.overlay_runtime.ui.editor.status =
                    format!("editor: saved {}", path.display());
                let _ = reload_console_template_if_changed(
                    &mut data.scene.overlay_runtime.world,
                    &mut data.scene.overlay_runtime.ui,
                    true,
                );
            }
            Err(err) => {
                data.scene.overlay_runtime.ui.editor.status =
                    format!("editor: save failed: {err:#}");
            }
        }
    }

    Ok(())
}

fn process_input_text_edit(
    data: &mut EngineData,
    input_entity: ecs::EntityHandle,
    input_visible: bool,
    input_nav_metrics: Option<(f32, f32)>,
) -> bool {
    let text_metrics = data.scene.overlay_runtime.ui.text_metrics.clone();
    let mut edited_text = false;
    let mut moved_cursor = false;
    if input_visible {
        let editor = &mut data.scene.overlay_runtime.ui.input_editor;
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
                    &text_metrics,
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
                    &text_metrics,
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
        if let Some(dirty) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiDirty>(input_entity)
        {
            dirty.text = true;
        }
        data.scene.overlay_runtime.ui.caret_visible = true;
        data.scene.overlay_runtime.ui.caret_blink_timer = 0.0;
    }

    edited_text || moved_cursor
}

fn process_submit_and_pointer(
    data: &mut EngineData,
    input_entity: ecs::EntityHandle,
    button_entity: ecs::EntityHandle,
    input_visible: bool,
    button_visible: bool,
) -> bool {
    let mut overlay_consumed = false;

    if input_visible && data.input.submitted {
        if let Some(field) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiInputField>(input_entity)
        {
            field.submit_requested = true;
        }
        overlay_consumed = true;
    }

    let hovered = button_visible
        && data
            .scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(button_entity)
            .map(|rect| point_in_rect(data.input.mouse_position, rect))
            .unwrap_or(false);
    let clicked = hovered && data.input.left_mouse_pressed();
    let pressed = hovered && data.input.left_mouse_down();

    if let Some(interaction) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiInteraction>(button_entity)
    {
        interaction.hovered = hovered;
        interaction.clicked = clicked;
        interaction.pressed = pressed;
    }

    if button_visible && clicked {
        if let Some(field) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiInputField>(input_entity)
        {
            field.submit_requested = true;
        }
        overlay_consumed = true;
    }

    let input_clicked = input_visible
        && data
            .scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(input_entity)
            .map(|rect| point_in_rect(data.input.mouse_position, rect))
            .unwrap_or(false)
        && data.input.left_mouse_pressed();
    if input_clicked {
        if let Some(interaction) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiInteraction>(input_entity)
        {
            interaction.focused = true;
        }
        data.scene.overlay_runtime.ui.caret_visible = true;
        data.scene.overlay_runtime.ui.caret_blink_timer = 0.0;
        overlay_consumed = true;
    }

    let mut should_submit = false;
    if let Some(field) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiInputField>(input_entity)
        && field.submit_requested
    {
        field.submit_requested = false;
        should_submit = true;
    }

    if input_visible && should_submit {
        let line = format!(
            "{CONSOLE_PROMPT}{}",
            data.scene.overlay_runtime.ui.input_editor.text
        );
        data.scene
            .channels
            .overlay_submit
            .push(OverlaySubmitMessage::Line(line));

        data.scene.overlay_runtime.ui.input_editor.text.clear();
        data.scene.overlay_runtime.ui.input_editor.cursor_chars = 0;
        data.scene.overlay_runtime.ui.input_editor.viewport_row = 0;
        data.scene.overlay_runtime.ui.input_editor.preferred_caret_x = None;
        if let Some(field) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiInputField>(input_entity)
        {
            field.cursor = 0;
            field.focused = true;
        }
        if let Some(dirty) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiDirty>(input_entity)
        {
            dirty.text = true;
        }
        if let Some(interaction) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiInteraction>(input_entity)
        {
            interaction.focused = true;
        }
        data.scene.overlay_runtime.ui.caret_visible = true;
        data.scene.overlay_runtime.ui.caret_blink_timer = 0.0;
        overlay_consumed = true;
    }

    if input_visible
        && let Some(input_text) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiText>(input_entity)
    {
        input_text.content = format!(
            "{CONSOLE_PROMPT}{}",
            data.scene.overlay_runtime.ui.input_editor.text
        );
    }
    if input_visible
        && let Some(field) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiInputField>(input_entity)
    {
        field.cursor = data.scene.overlay_runtime.ui.input_editor.cursor_chars;
    }

    overlay_consumed
}

fn process_scroll_routing(
    data: &mut EngineData,
    scroll_entity: ecs::EntityHandle,
    scroll_visible: bool,
    ui_scale: f32,
) -> bool {
    let console_hovered = scroll_visible
        && data
            .scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(scroll_entity)
            .map(|rect| point_in_rect(data.input.mouse_position, rect))
            .unwrap_or(false);
    let log_window = compute_log_window_rect(
        &data.scene.overlay_runtime.ui.layout,
        data.scene.overlay_runtime.ui.screen_size,
        ui_scale,
    );
    let log_hovered = point_in_rect(
        data.input.mouse_position,
        &UiTransform {
            x: log_window.x,
            y: log_window.y,
            w: log_window.w,
            h: log_window.h,
        },
    );

    let mut console_delta_lines: i32 = 0;
    let mut log_delta_lines: i32 = 0;
    let mut console_delta_horizontal: i32 = 0;
    let mut log_delta_horizontal: i32 = 0;

    if data.input.scroll_delta != 0.0 {
        let scroll_step = if data.input.scroll_delta > 0.0 { 3 } else { -3 };
        if data.input.shift_down() {
            if console_hovered {
                console_delta_horizontal -= scroll_step;
            } else if log_hovered {
                log_delta_horizontal -= scroll_step;
            }
        } else if console_hovered {
            console_delta_lines += scroll_step;
        } else if log_hovered {
            log_delta_lines += scroll_step;
        }
    }
    if data.input.page_up {
        if console_hovered {
            console_delta_lines += 12;
        } else if log_hovered {
            log_delta_lines += 12;
        } else {
            console_delta_lines += 12;
        }
    }
    if data.input.page_down {
        if console_hovered {
            console_delta_lines -= 12;
        } else if log_hovered {
            log_delta_lines -= 12;
        } else {
            console_delta_lines -= 12;
        }
    }

    if console_delta_lines > 0 {
        data.scene.overlay_runtime.ui.scroll_lines_from_bottom = data
            .scene
            .overlay_runtime
            .ui
            .scroll_lines_from_bottom
            .saturating_add(console_delta_lines as usize);
    } else if console_delta_lines < 0 {
        data.scene.overlay_runtime.ui.scroll_lines_from_bottom = data
            .scene
            .overlay_runtime
            .ui
            .scroll_lines_from_bottom
            .saturating_sub((-console_delta_lines) as usize);
    }
    if log_delta_lines > 0 {
        data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = data
            .scene
            .overlay_runtime
            .ui
            .log_scroll_lines_from_bottom
            .saturating_add(log_delta_lines as usize);
    } else if log_delta_lines < 0 {
        data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = data
            .scene
            .overlay_runtime
            .ui
            .log_scroll_lines_from_bottom
            .saturating_sub((-log_delta_lines) as usize);
    }
    if console_delta_horizontal > 0 {
        data.scene.overlay_runtime.ui.scroll_horizontal_chars = data
            .scene
            .overlay_runtime
            .ui
            .scroll_horizontal_chars
            .saturating_add(console_delta_horizontal as usize);
    } else if console_delta_horizontal < 0 {
        data.scene.overlay_runtime.ui.scroll_horizontal_chars = data
            .scene
            .overlay_runtime
            .ui
            .scroll_horizontal_chars
            .saturating_sub((-console_delta_horizontal) as usize);
    }
    if log_delta_horizontal > 0 {
        data.scene.overlay_runtime.ui.log_scroll_horizontal_chars = data
            .scene
            .overlay_runtime
            .ui
            .log_scroll_horizontal_chars
            .saturating_add(log_delta_horizontal as usize);
    } else if log_delta_horizontal < 0 {
        data.scene.overlay_runtime.ui.log_scroll_horizontal_chars = data
            .scene
            .overlay_runtime
            .ui
            .log_scroll_horizontal_chars
            .saturating_sub((-log_delta_horizontal) as usize);
    }

    console_delta_lines != 0
        || log_delta_lines != 0
        || console_delta_horizontal != 0
        || log_delta_horizontal != 0
}

pub fn ui_input_system(data: &mut EngineData) -> anyhow::Result<()> {
    data.input.overlay_consumed = false;
    if !data.scene.overlay_visible() {
        return Ok(());
    }

    if data.input.toggle_ui_editor_mode {
        data.scene.overlay_runtime.ui.editor.enabled =
            !data.scene.overlay_runtime.ui.editor.enabled;
        data.scene.overlay_runtime.ui.editor.dragging = false;
        data.scene.overlay_runtime.ui.editor.drag_pointer_offset = (0.0, 0.0);
        data.scene.overlay_runtime.ui.editor.status = if data
            .scene
            .overlay_runtime
            .ui
            .editor
            .enabled
        {
            "editor: on (click+drag move, Shift snap, arrows nudge, X hide node, A restore, Cmd/Ctrl+S save, F1 off)"
                .to_string()
        } else {
            "editor: off (F1 to toggle)".to_string()
        };
    }

    if data.scene.overlay_runtime.ui.editor.enabled {
        data.input.overlay_consumed = true;
        return Ok(());
    }

    let input_entity = data.scene.overlay_runtime.ui.input;
    let button_entity = data.scene.overlay_runtime.ui.confirm_button;
    let scroll_entity = data.scene.overlay_runtime.ui.scrollback;
    let input_visible = ui_node_visible(data, input_entity);
    let button_visible = ui_node_visible(data, button_entity);
    let scroll_visible = ui_node_visible(data, scroll_entity);
    let ui_scale = data.scene.overlay_runtime.ui.scale.max(1.0);
    let input_nav_metrics = if let (Some(transform), Some(text)) = (
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(input_entity),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiText>(input_entity),
    ) {
        let (_, _, content_w, _) = input_content_rect(transform, ui_scale);
        Some((text.size * ui_scale, content_w))
    } else {
        None
    };
    let mut overlay_consumed = false;
    overlay_consumed |=
        process_input_text_edit(data, input_entity, input_visible, input_nav_metrics);
    overlay_consumed |= process_submit_and_pointer(
        data,
        input_entity,
        button_entity,
        input_visible,
        button_visible,
    );

    data.scene.overlay_runtime.ui.caret_blink_timer += data.time.delta_seconds;
    while data.scene.overlay_runtime.ui.caret_blink_timer >= CARET_BLINK_SECONDS {
        data.scene.overlay_runtime.ui.caret_blink_timer -= CARET_BLINK_SECONDS;
        data.scene.overlay_runtime.ui.caret_visible = !data.scene.overlay_runtime.ui.caret_visible;
    }
    overlay_consumed |= process_scroll_routing(data, scroll_entity, scroll_visible, ui_scale);

    data.input.overlay_consumed = overlay_consumed;

    Ok(())
}

pub fn ui_layout_system(data: &mut EngineData) -> anyhow::Result<()> {
    if !data.scene.overlay_visible() {
        return Ok(());
    }
    if !data.scene.overlay_runtime.ui.layout_dirty {
        return Ok(());
    }

    let (screen_w, screen_h) = data.scene.overlay_runtime.ui.screen_size;
    let s = data.scene.overlay_runtime.ui.scale.max(1.0);
    let outer_margin = data.scene.overlay_runtime.ui.layout.outer_margin * s;
    let available_w = (screen_w - (outer_margin * 2.0)).max(1.0);
    let available_h = (screen_h - (outer_margin * 2.0)).max(1.0);
    let panel_w = clamp_panel_dimension(
        screen_w * data.scene.overlay_runtime.ui.layout.panel_width_ratio,
        data.scene.overlay_runtime.ui.layout.panel_min_width * s,
        available_w,
    );
    let panel_h = clamp_panel_dimension(
        screen_h * data.scene.overlay_runtime.ui.layout.panel_height_ratio,
        data.scene.overlay_runtime.ui.layout.panel_min_height * s,
        available_h,
    );
    let panel_x = outer_margin;
    let panel_y = (screen_h - panel_h - outer_margin).max(outer_margin);
    let inner_padding = data.scene.overlay_runtime.ui.layout.inner_padding * s;
    let panel_inner_w = (panel_w - inner_padding * 2.0).max(1.0);
    let (footer_y, input_h, button_w, input_w) = adaptive_footer_metrics(
        panel_inner_w,
        panel_h,
        inner_padding,
        s,
        data.scene.overlay_runtime.ui.layout.footer_offset,
        data.scene.overlay_runtime.ui.layout.input_height,
        data.scene.overlay_runtime.ui.layout.button_width,
        data.scene.overlay_runtime.ui.layout.input_button_gap,
    );

    if let Some(root) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiTransform>(data.scene.overlay_runtime.ui.root)
    {
        root.x = panel_x;
        root.y = panel_y;
        root.w = panel_w;
        root.h = panel_h;
    }

    if let Some(scroll) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiTransform>(data.scene.overlay_runtime.ui.scrollback)
    {
        scroll.x = panel_x + inner_padding;
        scroll.y = panel_y + inner_padding;
        scroll.w = panel_inner_w;
        scroll.h = panel_h - footer_y - inner_padding;
    }

    if let Some(input) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiTransform>(data.scene.overlay_runtime.ui.input)
    {
        input.x = panel_x + inner_padding;
        input.y = panel_y + panel_h - footer_y;
        input.w = input_w;
        input.h = input_h;
    }

    if let Some(button) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiTransform>(data.scene.overlay_runtime.ui.confirm_button)
    {
        button.x = panel_x + panel_w - inner_padding - button_w;
        button.y = panel_y + panel_h - footer_y;
        button.w = button_w;
        button.h = input_h;
    }

    data.scene.overlay_runtime.ui.layout_dirty = false;
    Ok(())
}

pub fn ui_build_batches_system(data: &mut EngineData) -> anyhow::Result<()> {
    if !data.scene.overlay_visible() {
        let ui_scale = data.scene.overlay_runtime.ui.scale.max(1.0);
        let mut commands: Vec<UiBatchCmd> = Vec::new();
        push_world_stats_panel(&mut commands, data, ui_scale);
        data.scene.overlay_runtime.ui.batches.commands = commands;
        return Ok(());
    }
    let ui_scale = data.scene.overlay_runtime.ui.scale.max(1.0);
    let mut commands: Vec<UiBatchCmd> = Vec::new();

    if let (Some(transform), Some(style)) = (
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(data.scene.overlay_runtime.ui.root),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiStyle>(data.scene.overlay_runtime.ui.root),
    ) && ui_node_visible(data, data.scene.overlay_runtime.ui.root)
    {
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
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(data.scene.overlay_runtime.ui.scrollback),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiText>(data.scene.overlay_runtime.ui.scrollback),
    ) && ui_node_visible(data, data.scene.overlay_runtime.ui.scrollback)
    {
        let text_size = text.size * ui_scale;
        let visible_capacity = visible_line_capacity(transform.h, text_size);
        let line_max_w = (transform.w - (8.0 * ui_scale)).max(1.0);
        let wrapped_rows = flatten_wrapped_rows(
            &data.scene.overlay_runtime.ui.lines,
            data.scene.overlay_runtime.ui.scroll_horizontal_chars,
            line_max_w,
            text_size,
            &data.scene.overlay_runtime.ui.text_metrics,
        );
        let max_scroll = wrapped_rows.len().saturating_sub(visible_capacity);
        let max_horizontal = data
            .scene
            .overlay_runtime
            .ui
            .lines
            .iter()
            .map(|line| char_count(line))
            .max()
            .unwrap_or(0)
            .saturating_sub(1);
        data.scene.overlay_runtime.ui.scroll_horizontal_chars = data
            .scene
            .overlay_runtime
            .ui
            .scroll_horizontal_chars
            .min(max_horizontal);
        data.scene.overlay_runtime.ui.scroll_lines_from_bottom = data
            .scene
            .overlay_runtime
            .ui
            .scroll_lines_from_bottom
            .min(max_scroll);
        let view_lines = build_wrapped_view_rows(
            &data.scene.overlay_runtime.ui.lines,
            data.scene.overlay_runtime.ui.scroll_lines_from_bottom,
            visible_capacity,
            data.scene.overlay_runtime.ui.scroll_horizontal_chars,
            line_max_w,
            text_size,
            &data.scene.overlay_runtime.ui.text_metrics,
        );
        for (line_idx, line) in view_lines.iter().enumerate() {
            let (line_color, stripped) = scrollback_line_style(line, text.color);
            commands.push(UiBatchCmd::Text {
                x: transform.x,
                y: transform.y + (line_idx as f32 * line_height(text_size)),
                content: stripped.to_string(),
                color: line_color,
                size: text_size,
                clip: Some([transform.x, transform.y, transform.w, transform.h]),
            });
        }
        if data.scene.overlay_runtime.ui.layout.show_scroll_indicators {
            push_scroll_indicators(
                &mut commands,
                transform,
                visible_capacity,
                wrapped_rows.len(),
                data.scene.overlay_runtime.ui.scroll_lines_from_bottom,
                data.scene.overlay_runtime.ui.scroll_horizontal_chars,
                max_horizontal,
                ui_scale,
                [0.62, 0.74, 0.86, 0.95],
            );
        }
        if data.scene.overlay_runtime.ui.layout.show_scroll_hints {
            commands.push(UiBatchCmd::Text {
                x: transform.x + (4.0 * ui_scale),
                y: (transform.y + transform.h - (14.0 * ui_scale)).max(transform.y),
                content: "Wheel: vertical  Shift+Wheel: horizontal".to_string(),
                color: [0.62, 0.70, 0.78, 0.92],
                size: 10.0 * ui_scale,
                clip: Some([transform.x, transform.y, transform.w, transform.h]),
            });
        }
    }

    let log_rect = compute_log_window_rect(
        &data.scene.overlay_runtime.ui.layout,
        data.scene.overlay_runtime.ui.screen_size,
        ui_scale,
    );
    let log_text_size = 12.0 * ui_scale;
    let log_header_h = 24.0 * ui_scale;
    let log_visible_capacity = visible_line_capacity(log_rect.body_h, log_text_size);
    let wrapped_log_rows = flatten_wrapped_rows(
        &data.scene.overlay_runtime.ui.log_lines,
        data.scene.overlay_runtime.ui.log_scroll_horizontal_chars,
        log_rect.body_w,
        log_text_size,
        &data.scene.overlay_runtime.ui.text_metrics,
    );
    let log_max_scroll = wrapped_log_rows.len().saturating_sub(log_visible_capacity);
    let log_max_horizontal = data
        .scene
        .overlay_runtime
        .ui
        .log_lines
        .iter()
        .map(|line| char_count(line))
        .max()
        .unwrap_or(0)
        .saturating_sub(1);
    data.scene.overlay_runtime.ui.log_scroll_horizontal_chars = data
        .scene
        .overlay_runtime
        .ui
        .log_scroll_horizontal_chars
        .min(log_max_horizontal);
    data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = data
        .scene
        .overlay_runtime
        .ui
        .log_scroll_lines_from_bottom
        .min(log_max_scroll);

    commands.push(UiBatchCmd::Rect {
        x: log_rect.x,
        y: log_rect.y,
        w: log_rect.w,
        h: log_rect.h,
        color: [0.04, 0.06, 0.08, 0.92],
        radius: 6.0 * ui_scale,
    });
    commands.push(UiBatchCmd::Rect {
        x: log_rect.x,
        y: log_rect.y,
        w: log_rect.w,
        h: log_header_h,
        color: [0.11, 0.17, 0.23, 0.96],
        radius: 6.0 * ui_scale,
    });
    let pause_status = if data.scene.overlay_runtime.ui.logs_paused {
        format!(
            "PAUSED ({})",
            data.scene.overlay_runtime.ui.log_paused_lines.len()
        )
    } else {
        "LIVE".to_string()
    };
    commands.push(UiBatchCmd::Text {
        x: log_rect.x + (8.0 * ui_scale),
        y: log_rect.y + (6.0 * ui_scale),
        content: format!("Logs [{pause_status}]"),
        color: if data.scene.overlay_runtime.ui.logs_paused {
            [0.98, 0.72, 0.42, 1.0]
        } else {
            [0.72, 0.91, 0.78, 1.0]
        },
        size: 12.0 * ui_scale,
        clip: Some([log_rect.x, log_rect.y, log_rect.w, log_rect.h]),
    });
    let log_lines = build_wrapped_view_rows(
        &data.scene.overlay_runtime.ui.log_lines,
        data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom,
        log_visible_capacity,
        data.scene.overlay_runtime.ui.log_scroll_horizontal_chars,
        log_rect.body_w,
        log_text_size,
        &data.scene.overlay_runtime.ui.text_metrics,
    );
    for (line_idx, line) in log_lines.iter().enumerate() {
        let (line_color, stripped) = scrollback_line_style(line, [0.80, 0.88, 0.94, 1.0]);
        commands.push(UiBatchCmd::Text {
            x: log_rect.body_x,
            y: log_rect.body_y + (line_idx as f32 * line_height(log_text_size)),
            content: stripped.to_string(),
            color: line_color,
            size: log_text_size,
            clip: Some([
                log_rect.x + (4.0 * ui_scale),
                log_rect.body_y,
                log_rect.w - (8.0 * ui_scale),
                log_rect.body_h,
            ]),
        });
    }
    if data.scene.overlay_runtime.ui.layout.show_scroll_indicators {
        let log_content_rect = UiTransform {
            x: log_rect.x + (4.0 * ui_scale),
            y: log_rect.body_y,
            w: log_rect.w - (8.0 * ui_scale),
            h: log_rect.body_h,
        };
        push_scroll_indicators(
            &mut commands,
            &log_content_rect,
            log_visible_capacity,
            wrapped_log_rows.len(),
            data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom,
            data.scene.overlay_runtime.ui.log_scroll_horizontal_chars,
            log_max_horizontal,
            ui_scale,
            [0.72, 0.88, 0.78, 0.95],
        );
    }

    if let (Some(transform), Some(style)) = (
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(data.scene.overlay_runtime.ui.input),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiStyle>(data.scene.overlay_runtime.ui.root),
    ) && ui_node_visible(data, data.scene.overlay_runtime.ui.input)
    {
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
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(data.scene.overlay_runtime.ui.input),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiText>(data.scene.overlay_runtime.ui.input),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiInputField>(data.scene.overlay_runtime.ui.input),
    ) && ui_node_visible(data, data.scene.overlay_runtime.ui.input)
    {
        let scaled_text_size = text.size * ui_scale;
        let (content_x, content_y, content_w, content_h) = input_content_rect(transform, ui_scale);
        let layout = build_visible_multiline_input(
            &mut data.scene.overlay_runtime.ui.input_editor,
            &data.scene.overlay_runtime.ui.text_metrics,
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

        if data.scene.overlay_runtime.ui.caret_visible && input_field.focused {
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
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(data.scene.overlay_runtime.ui.confirm_button),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiStyle>(data.scene.overlay_runtime.ui.confirm_button),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiButton>(data.scene.overlay_runtime.ui.confirm_button),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiInteraction>(data.scene.overlay_runtime.ui.confirm_button),
        data.scene
            .overlay_runtime
            .world
            .get_component::<UiText>(data.scene.overlay_runtime.ui.confirm_button),
    ) && ui_node_visible(data, data.scene.overlay_runtime.ui.confirm_button)
    {
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

    push_world_stats_panel(&mut commands, data, ui_scale);

    if data.scene.overlay_runtime.ui.editor.enabled {
        if let Some(selected_entity) = selected_editor_entity(&data.scene.overlay_runtime.ui)
            && let Some(selected_rect) = data
                .scene
                .overlay_runtime
                .world
                .get_component::<UiTransform>(selected_entity)
        {
            push_outline(
                &mut commands,
                selected_rect,
                2.0 * ui_scale,
                [0.95, 0.55, 0.15, 0.95],
            );
        }

        if let Some(root_rect) = data
            .scene
            .overlay_runtime
            .world
            .get_component::<UiTransform>(data.scene.overlay_runtime.ui.root)
        {
            let stack_labels = data
                .scene
                .overlays
                .iter()
                .map(|slot| slot.active.label())
                .collect::<Vec<_>>()
                .join(" > ");
            let debug_pos = data
                .scene
                .world_runtime
                .ctx
                .world
                .get_component::<crate::runtime::WorldDebugPosition>(
                    data.scene.world_runtime.ctx.debug_entity,
                )
                .map(|p| format!("({:.1}, {:.1})", p.x, p.y))
                .unwrap_or_else(|| "(n/a)".to_string());
            commands.push(UiBatchCmd::Text {
                x: root_rect.x + (8.0 * ui_scale),
                y: root_rect.y + (8.0 * ui_scale),
                content: data.scene.overlay_runtime.ui.editor.status.clone(),
                color: [0.98, 0.84, 0.52, 1.0],
                size: 12.0 * ui_scale,
                clip: Some([root_rect.x, root_rect.y, root_rect.w, root_rect.h]),
            });
            commands.push(UiBatchCmd::Text {
                x: root_rect.x + (8.0 * ui_scale),
                y: root_rect.y + (24.0 * ui_scale),
                content: format!(
                    "scene world={} paused={} overlays=[{}]",
                    data.scene.world.active.label(),
                    data.scene.world.paused,
                    stack_labels
                ),
                color: [0.76, 0.87, 0.98, 1.0],
                size: 11.0 * ui_scale,
                clip: Some([root_rect.x, root_rect.y, root_rect.w, root_rect.h]),
            });
            commands.push(UiBatchCmd::Text {
                x: root_rect.x + (8.0 * ui_scale),
                y: root_rect.y + (39.0 * ui_scale),
                content: format!(
                    "world_tick={} overlay_consumed={} debug_pos={}",
                    data.scene.world_runtime.ctx.frame_count,
                    data.input.overlay_consumed,
                    debug_pos
                ),
                color: [0.72, 0.86, 0.84, 1.0],
                size: 11.0 * ui_scale,
                clip: Some([root_rect.x, root_rect.y, root_rect.w, root_rect.h]),
            });
        }
    }

    data.scene.overlay_runtime.ui.batches.commands = commands;
    Ok(())
}

pub fn ui_render_extract_system(data: &mut EngineData) -> anyhow::Result<()> {
    let commands = data
        .scene
        .overlay_runtime
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

    data.scene.overlay_runtime.ui.draw_list.commands = commands;
    Ok(())
}
