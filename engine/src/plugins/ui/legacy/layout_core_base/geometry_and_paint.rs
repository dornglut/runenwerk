// Owner: Grotto Quest Engine - UI Plugin
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
