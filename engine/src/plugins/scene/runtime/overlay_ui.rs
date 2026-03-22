use crate::plugins::SceneManager;
use crate::plugins::ui::domain::{
    UiDrawCmd, UiNode, UiPresentationMode, UiStyle, UiText, UiTransform,
    reload_console_template_if_changed,
};

const TEXT_PADDING_X: f32 = 10.0;
const TEXT_PADDING_Y: f32 = 8.0;
const MIN_INPUT_WIDTH: f32 = 120.0;

// Owner: Engine Scene Plugin - Overlay UI Composition
pub(crate) fn rebuild_overlay_draw_list(manager: &mut SceneManager) -> anyhow::Result<()> {
    let overlay_visible = manager.overlay_visible();
    let world = &mut manager.overlay_runtime.world;
    let ui = &mut manager.overlay_runtime.ui;

    if let Err(err) = reload_console_template_if_changed(world, ui, false) {
        ui.log_lines
            .push(format!("[ui] template reload failed: {err:#}"));
    }

    apply_runtime_layout(world, ui);
    sync_runtime_text(world, ui);

    if !overlay_visible {
        ui.draw_list.commands.clear();
        return Ok(());
    }

    let mut commands = Vec::new();
    push_panel_commands(world, ui.root, &mut commands);
    push_text_block_commands(world, ui.scrollback, true, false, &mut commands);
    push_text_block_commands(world, ui.input, true, false, &mut commands);
    push_text_block_commands(world, ui.confirm_button, false, true, &mut commands);
    ui.draw_list.commands = commands;
    Ok(())
}

fn apply_runtime_layout(
    world: &mut ecs::World,
    ui: &mut crate::plugins::ui::domain::ConsoleUiRuntimeState,
) {
    if !ui.layout_dirty {
        return;
    }

    let safe_width = ui.screen_size.0.max(1.0);
    let safe_height = ui.screen_size.1.max(1.0);
    let outer_margin = ui.layout.outer_margin.max(0.0);
    let inner_padding = ui.layout.inner_padding.max(0.0);
    let footer_offset = ui.layout.footer_offset.max(0.0);
    let input_height = ui.layout.input_height.max(1.0);
    let button_width = ui.layout.button_width.max(1.0);
    let input_button_gap = ui.layout.input_button_gap.max(0.0);

    let available_w = (safe_width - outer_margin * 2.0).max(1.0);
    let available_h = (safe_height - outer_margin * 2.0).max(1.0);
    let panel_w = (safe_width * ui.layout.panel_width_ratio)
        .max(ui.layout.panel_min_width)
        .min(available_w);
    let panel_h = (safe_height * ui.layout.panel_height_ratio)
        .max(ui.layout.panel_min_height)
        .min(available_h);

    let (panel_x, panel_y) = match ui.presentation_mode {
        UiPresentationMode::CenteredDemo => {
            ((safe_width - panel_w) * 0.5, (safe_height - panel_h) * 0.5)
        }
        UiPresentationMode::Standard => (outer_margin, outer_margin),
    };

    let logs_x = panel_x + inner_padding;
    let logs_y = panel_y + inner_padding;
    let logs_w = (panel_w - inner_padding * 2.0).max(1.0);
    let logs_h = (panel_h - inner_padding * 3.0 - input_height - footer_offset).max(1.0);

    let input_y = logs_y + logs_h + inner_padding;
    let target_input_width = (logs_w - button_width - input_button_gap)
        .clamp(MIN_INPUT_WIDTH, logs_w.max(MIN_INPUT_WIDTH));
    let final_button_width = (logs_w - target_input_width - input_button_gap).max(1.0);

    set_transform(
        world,
        ui.root,
        UiTransform {
            x: panel_x,
            y: panel_y,
            w: panel_w,
            h: panel_h,
        },
    );
    set_transform(
        world,
        ui.scrollback,
        UiTransform {
            x: logs_x,
            y: logs_y,
            w: logs_w,
            h: logs_h,
        },
    );
    set_transform(
        world,
        ui.input,
        UiTransform {
            x: logs_x,
            y: input_y,
            w: target_input_width,
            h: input_height,
        },
    );
    set_transform(
        world,
        ui.confirm_button,
        UiTransform {
            x: logs_x + target_input_width + input_button_gap,
            y: input_y,
            w: final_button_width,
            h: input_height,
        },
    );

    ui.layout_dirty = false;
}

fn set_transform(world: &mut ecs::World, entity: ecs::Entity, transform: UiTransform) {
    if let Ok(mut entity_ref) = world.entity_mut(entity)
        && let Some(mut current) = entity_ref.get_mut::<UiTransform>()
    {
        *current = transform;
    }
}

fn sync_runtime_text(
    world: &mut ecs::World,
    ui: &mut crate::plugins::ui::domain::ConsoleUiRuntimeState,
) {
    if ui.log_lines.is_empty() {
        ui.log_lines.push("[world] scene overlay ready".to_string());
    }

    let scrollback_transform = world.get::<UiTransform>(ui.scrollback).copied();
    let scrollback_style = world.get::<UiText>(ui.scrollback).cloned();
    let visible_lines = scrollback_transform
        .zip(scrollback_style.as_ref())
        .map(|(transform, text)| {
            let line_height = (text.size.max(1.0) * 1.25).max(1.0);
            ((transform.h / line_height).floor() as usize).max(1)
        })
        .unwrap_or(8);
    let scroll_offset = ui
        .log_scroll_lines_from_bottom
        .min(ui.log_lines.len().saturating_sub(1));
    let end = ui.log_lines.len().saturating_sub(scroll_offset);
    let start = end.saturating_sub(visible_lines);
    let scrollback_content = ui.log_lines[start..end].join("\n");

    if let Ok(mut entity_ref) = world.entity_mut(ui.scrollback)
        && let Some(mut text) = entity_ref.get_mut::<UiText>()
    {
        text.content = scrollback_content;
    }

    let input_prefix = if ui.input_editor.text.is_empty() {
        "grotto> ".to_string()
    } else {
        format!("grotto> {}", ui.input_editor.text)
    };
    if let Ok(mut entity_ref) = world.entity_mut(ui.input)
        && let Some(mut text) = entity_ref.get_mut::<UiText>()
    {
        text.content = input_prefix;
    }
}

fn push_panel_commands(world: &ecs::World, entity: ecs::Entity, commands: &mut Vec<UiDrawCmd>) {
    let Some(node) = world.get::<UiNode>(entity) else {
        return;
    };
    if !node.visible {
        return;
    }
    let Some(transform) = world.get::<UiTransform>(entity) else {
        return;
    };
    if let Some(style) = world.get::<UiStyle>(entity) {
        commands.push(UiDrawCmd::Rect {
            x: transform.x,
            y: transform.y,
            w: transform.w.max(0.0),
            h: transform.h.max(0.0),
            color: style.bg_color,
            radius: style.radius.max(0.0),
        });
    }
}

fn push_text_block_commands(
    world: &ecs::World,
    entity: ecs::Entity,
    clip_text: bool,
    center_text: bool,
    commands: &mut Vec<UiDrawCmd>,
) {
    let Some(node) = world.get::<UiNode>(entity) else {
        return;
    };
    if !node.visible {
        return;
    }
    let Some(transform) = world.get::<UiTransform>(entity) else {
        return;
    };

    if let Some(style) = world.get::<UiStyle>(entity) {
        commands.push(UiDrawCmd::Rect {
            x: transform.x,
            y: transform.y,
            w: transform.w.max(0.0),
            h: transform.h.max(0.0),
            color: style.bg_color,
            radius: style.radius.max(0.0),
        });
    }

    let Some(text) = world.get::<UiText>(entity) else {
        return;
    };

    let (x, y) = if center_text {
        let approx_text_w = text.content.chars().count() as f32 * text.size.max(1.0) * 0.55;
        let centered_x = transform.x + (transform.w - approx_text_w).max(0.0) * 0.5;
        let centered_y = transform.y + (transform.h - text.size.max(1.0)).max(0.0) * 0.5;
        (centered_x, centered_y)
    } else {
        (transform.x + TEXT_PADDING_X, transform.y + TEXT_PADDING_Y)
    };

    let clip = clip_text.then_some([transform.x, transform.y, transform.w, transform.h]);
    commands.push(UiDrawCmd::Text {
        x,
        y,
        content: text.content.clone(),
        color: text.color,
        size: text.size.max(1.0),
        clip,
    });
}
