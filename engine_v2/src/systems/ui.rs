use crate::runtime::EngineData;
use crate::ui::{
    UiBatchCmd, UiButton, UiDirty, UiDrawCmd, UiInputField, UiInteraction, UiStyle,
    UiSubmitEvent, UiText, UiTransform,
};

const CONSOLE_PROMPT: &str = "grotto> ";
const CARET_BLINK_SECONDS: f32 = 0.5;

pub(super) fn point_in_rect(point: (f32, f32), rect: &UiTransform) -> bool {
    point.0 >= rect.x
        && point.0 <= rect.x + rect.w
        && point.1 >= rect.y
        && point.1 <= rect.y + rect.h
}

fn tint_color(color: [f32; 4], factor: f32) -> [f32; 4] {
    [
        (color[0] * factor).clamp(0.0, 1.0),
        (color[1] * factor).clamp(0.0, 1.0),
        (color[2] * factor).clamp(0.0, 1.0),
        color[3],
    ]
}

pub(super) fn estimate_text_width(text: &str, size: f32) -> f32 {
    // Monospace-biased estimate for centering labels in this MVP stage.
    text.chars().count() as f32 * size * 0.56
}

fn fit_text_with_ellipsis_from_left(text: &str, size: f32, max_width: f32) -> String {
    if text.is_empty() || estimate_text_width(text, size) <= max_width {
        return text.to_string();
    }

    let ellipsis = "...";
    if estimate_text_width(ellipsis, size) >= max_width {
        return String::new();
    }

    let mut tail = text.to_string();
    while !tail.is_empty() && estimate_text_width(&format!("{ellipsis}{tail}"), size) > max_width {
        tail.remove(0);
    }

    if tail.is_empty() {
        String::new()
    } else {
        format!("{ellipsis}{tail}")
    }
}

pub(super) fn visible_input_text(full_line: &str, size: f32, max_width: f32) -> String {
    if estimate_text_width(full_line, size) <= max_width {
        return full_line.to_string();
    }

    let Some(rest) = full_line.strip_prefix(CONSOLE_PROMPT) else {
        return fit_text_with_ellipsis_from_left(full_line, size, max_width);
    };

    let prompt_width = estimate_text_width(CONSOLE_PROMPT, size);
    if prompt_width >= max_width {
        return fit_text_with_ellipsis_from_left(CONSOLE_PROMPT, size, max_width);
    }

    let clipped = fit_text_with_ellipsis_from_left(rest, size, max_width - prompt_width);
    format!("{CONSOLE_PROMPT}{clipped}")
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
) -> String {
    if lines.is_empty() {
        return String::new();
    }

    let end = lines.len().saturating_sub(lines_from_bottom);
    let start = end.saturating_sub(visible_capacity);
    lines[start..end].join("\n")
}

fn clamp_panel_dimension(target: f32, min_size: f32, max_size: f32) -> f32 {
    if max_size <= min_size {
        max_size.max(1.0)
    } else {
        target.clamp(min_size, max_size)
    }
}

pub fn ui_input_system(data: &mut EngineData) -> anyhow::Result<()> {
    let input_entity = data.ui.input;
    let button_entity = data.ui.confirm_button;
    let scroll_entity = data.ui.scrollback;

    if !data.input.typed_text.is_empty() {
        if let Some(text) = data.world.get_component_mut::<UiText>(input_entity) {
            text.content.push_str(&data.input.typed_text);
        }
        if let Some(field) = data.world.get_component_mut::<UiInputField>(input_entity) {
            field.cursor += data.input.typed_text.chars().count();
        }
        if let Some(dirty) = data.world.get_component_mut::<UiDirty>(input_entity) {
            dirty.text = true;
        }
        data.ui.caret_visible = true;
        data.ui.caret_blink_timer = 0.0;
    }

    if data.input.backspace {
        if let Some(text) = data.world.get_component_mut::<UiText>(input_entity) {
            if text.content.len() > CONSOLE_PROMPT.len() {
                text.content.pop();
                if let Some(field) = data.world.get_component_mut::<UiInputField>(input_entity) {
                    field.cursor = field.cursor.saturating_sub(1);
                }
                if let Some(dirty) = data.world.get_component_mut::<UiDirty>(input_entity) {
                    dirty.text = true;
                }
            }
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
        let submitted_line = data
            .world
            .get_component::<UiText>(input_entity)
            .map(|input_text| input_text.content.clone());
        if let Some(line) = submitted_line {
            data.world.spawn_entity_typed(UiSubmitEvent { line });
        }

        if let Some(input_text) = data.world.get_component_mut::<UiText>(input_entity) {
            input_text.content = CONSOLE_PROMPT.to_string();
        }
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
    let outer_margin = 24.0 * s;
    let available_w = (screen_w - (outer_margin * 2.0)).max(1.0);
    let available_h = (screen_h - (outer_margin * 2.0)).max(1.0);
    let panel_w = clamp_panel_dimension(screen_w * 0.6, 480.0 * s, available_w);
    let panel_h = clamp_panel_dimension(screen_h * 0.45, 280.0 * s, available_h);
    let panel_x = (screen_w - panel_w - outer_margin).max(outer_margin);
    let panel_y = (screen_h - panel_h - outer_margin).max(outer_margin);
    let inner_padding = 12.0 * s;
    let footer_y = 40.0 * s;
    let input_h = 28.0 * s;
    let button_w = 100.0 * s;
    let input_button_gap = 8.0 * s;

    if let Some(root) = data.world.get_component_mut::<UiTransform>(data.ui.root) {
        root.x = panel_x;
        root.y = panel_y;
        root.w = panel_w;
        root.h = panel_h;
    }

    if let Some(scroll) = data.world.get_component_mut::<UiTransform>(data.ui.scrollback) {
        scroll.x = panel_x + inner_padding;
        scroll.y = panel_y + inner_padding;
        scroll.w = panel_w - (inner_padding * 2.0);
        scroll.h = panel_h - footer_y - inner_padding;
    }

    if let Some(input) = data.world.get_component_mut::<UiTransform>(data.ui.input) {
        input.x = panel_x + inner_padding;
        input.y = panel_y + panel_h - footer_y;
        input.w = panel_w - (inner_padding * 2.0) - button_w - input_button_gap;
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
        let view = build_scrollback_view_text(
            &data.ui.lines,
            data.ui.scroll_lines_from_bottom,
            visible_capacity,
        );
        commands.push(UiBatchCmd::Text {
            x: transform.x,
            y: transform.y,
            content: view,
            color: text.color,
            size: text_size,
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
        let padded_x = transform.x + (6.0 * ui_scale);
        let content_max_w = (transform.w - (10.0 * ui_scale)).max(1.0);
        let content = visible_input_text(&text.content, scaled_text_size, content_max_w);
        commands.push(UiBatchCmd::Text {
            x: padded_x,
            y: transform.y,
            content: content.clone(),
            color: text.color,
            size: scaled_text_size,
        });

        if data.ui.caret_visible && input_field.focused {
            let caret_x = (padded_x + estimate_text_width(&content, scaled_text_size) + ui_scale)
                .min(transform.x + transform.w - (2.0 * ui_scale));
            let caret_h = (scaled_text_size * 0.92).min(transform.h);
            let caret_y = transform.y + ((transform.h - caret_h) * 0.5).max(0.0);
            commands.push(UiBatchCmd::Rect {
                x: caret_x,
                y: caret_y,
                w: 2.0 * ui_scale,
                h: caret_h,
                color: [0.92, 0.95, 0.98, 0.95],
                radius: 1.0 * ui_scale,
            });
        }
    }

    if let (Some(transform), Some(style), Some(button), Some(interaction), Some(text)) = (
        data.world.get_component::<UiTransform>(data.ui.confirm_button),
        data.world.get_component::<UiStyle>(data.ui.confirm_button),
        data.world.get_component::<UiButton>(data.ui.confirm_button),
        data.world.get_component::<UiInteraction>(data.ui.confirm_button),
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
        });
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
            } => UiDrawCmd::Text {
                x: *x,
                y: *y,
                content: content.clone(),
                color: *color,
                size: *size,
            },
        })
        .collect();

    data.ui.draw_list.commands = commands;
    Ok(())
}
