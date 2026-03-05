// Owner: Grotto Quest Engine - UI Plugin
pub(crate) fn wrapped_line_rows(
    line: &str,
    horizontal_chars: usize,
    max_width: f32,
    text_size: f32,
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
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

pub(crate) fn flatten_wrapped_rows(
    lines: &[String],
    horizontal_chars: usize,
    max_width: f32,
    text_size: f32,
    metrics: &crate::plugins::ui::domain::UiTextMetrics,
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

pub(crate) struct ScrollViewportSpec<'a> {
    pub(crate) lines: &'a [String],
    pub(crate) lines_from_bottom: usize,
    pub(crate) visible_capacity: usize,
    pub(crate) horizontal_chars: usize,
    pub(crate) max_width: f32,
    pub(crate) text_size: f32,
    pub(crate) metrics: &'a crate::plugins::ui::domain::UiTextMetrics,
}

pub(crate) struct ScrollViewport {
    pub(crate) wrapped_rows: Vec<String>,
    pub(crate) view_rows: Vec<String>,
    pub(crate) max_horizontal_chars: usize,
    pub(crate) clamped_lines_from_bottom: usize,
    pub(crate) clamped_horizontal_chars: usize,
}

pub(crate) fn build_scroll_viewport(spec: ScrollViewportSpec<'_>) -> ScrollViewport {
    let max_horizontal_chars = spec
        .lines
        .iter()
        .map(|line| char_count(line))
        .max()
        .unwrap_or(0)
        .saturating_sub(1);
    let clamped_horizontal_chars = spec.horizontal_chars.min(max_horizontal_chars);
    let wrapped_rows = flatten_wrapped_rows(
        spec.lines,
        clamped_horizontal_chars,
        spec.max_width,
        spec.text_size,
        spec.metrics,
    );
    let max_scroll = wrapped_rows.len().saturating_sub(spec.visible_capacity);
    let clamped_lines_from_bottom = spec.lines_from_bottom.min(max_scroll);
    let view_rows = if wrapped_rows.is_empty() {
        Vec::new()
    } else {
        let end = wrapped_rows.len().saturating_sub(clamped_lines_from_bottom);
        let start = end.saturating_sub(spec.visible_capacity);
        wrapped_rows[start..end].to_vec()
    };

    ScrollViewport {
        wrapped_rows,
        view_rows,
        max_horizontal_chars,
        clamped_lines_from_bottom,
        clamped_horizontal_chars,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn build_scrollback_view_text(
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

pub fn scrollback_line_style(line: &str, default: [f32; 4]) -> ([f32; 4], &str) {
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

pub(crate) fn clamp_panel_dimension(target: f32, min_size: f32, max_size: f32) -> f32 {
    if max_size <= min_size {
        max_size.max(1.0)
    } else {
        target.clamp(min_size, max_size)
    }
}

pub fn adaptive_footer_metrics(
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

pub(crate) fn apply_scroll_delta(current: usize, delta: i32) -> usize {
    if delta >= 0 {
        current.saturating_add(delta as usize)
    } else {
        current.saturating_sub((-delta) as usize)
    }
}

pub(crate) fn centered_demo_enabled(ui: &ConsoleUiRuntimeState) -> bool {
    matches!(ui.presentation_mode, UiPresentationMode::CenteredDemo)
}

pub(crate) fn selected_editor_entity(ui: &ConsoleUiRuntimeState) -> Option<ecs::Entity> {
    match ui.editor.selected {
        Some(UiEditorNode::Root) => Some(ui.root),
        Some(UiEditorNode::Scrollback) => Some(ui.scrollback),
        Some(UiEditorNode::Input) => Some(ui.input),
        Some(UiEditorNode::ConfirmButton) => Some(ui.confirm_button),
        None => None,
    }
}

pub(crate) fn ui_node_visible(manager: &SceneManager, entity: ecs::Entity) -> bool {
    manager
        .overlay_runtime
        .world
        .get::<UiNode>(entity)
        .map(|node| node.visible)
        .unwrap_or(true)
}

pub(crate) fn set_text_dirty(world: &mut ecs::World, entity: ecs::Entity) {
    if let Ok(mut ui_entity) = world.entity_mut(entity)
        && let Some(mut dirty) = ui_entity.get_mut::<UiDirty>()
    {
        dirty.text = true;
    }
}

pub(crate) fn apply_editor_translation(
    manager: &mut SceneManager,
    selected_node: UiEditorNode,
    dx: f32,
    dy: f32,
) {
    if dx == 0.0 && dy == 0.0 {
        return;
    }

    let translate = |world: &mut ecs::World, entity: ecs::Entity| {
        if let Ok(mut ui_entity) = world.entity_mut(entity)
            && let Some(mut transform) = ui_entity.get_mut::<UiTransform>()
        {
            transform.x += dx;
            transform.y += dy;
        }
    };

    if let Some(selected) = selected_editor_entity(&manager.overlay_runtime.ui) {
        translate(&mut manager.overlay_runtime.world, selected);
        if selected_node == UiEditorNode::Root {
            let scrollback = manager.overlay_runtime.ui.scrollback;
            let input = manager.overlay_runtime.ui.input;
            let confirm = manager.overlay_runtime.ui.confirm_button;
            translate(&mut manager.overlay_runtime.world, scrollback);
            translate(&mut manager.overlay_runtime.world, input);
            translate(&mut manager.overlay_runtime.world, confirm);
        }
    }
}
