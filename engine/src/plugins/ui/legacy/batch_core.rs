// Owner: Grotto Quest Engine - UI Plugin
fn build_console_batches(
    manager: &mut SceneManager,
    commands: &mut Vec<UiBatchCmd>,
    ui_scale: f32,
) {
    let centered_demo = centered_demo_enabled(&manager.overlay_runtime.ui);
    if let (Some(transform), Some(style)) = (
        manager
            .overlay_runtime
            .world
            .get::<UiTransform>(manager.overlay_runtime.ui.root),
        manager
            .overlay_runtime
            .world
            .get::<UiStyle>(manager.overlay_runtime.ui.root),
    ) && ui_node_visible(manager, manager.overlay_runtime.ui.root)
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
        manager
            .overlay_runtime
            .world
            .get::<UiTransform>(manager.overlay_runtime.ui.scrollback),
        manager
            .overlay_runtime
            .world
            .get::<UiText>(manager.overlay_runtime.ui.scrollback),
    ) && ui_node_visible(manager, manager.overlay_runtime.ui.scrollback)
    {
        let text_size = text.size * ui_scale;
        let visible_capacity = visible_line_capacity(transform.h, text_size);
        let line_max_w = (transform.w - (8.0 * ui_scale)).max(1.0);
        let viewport = build_scroll_viewport(ScrollViewportSpec {
            lines: &manager.overlay_runtime.ui.lines,
            lines_from_bottom: manager.overlay_runtime.ui.scroll_lines_from_bottom,
            visible_capacity,
            horizontal_chars: manager.overlay_runtime.ui.scroll_horizontal_chars,
            max_width: line_max_w,
            text_size,
            metrics: &manager.overlay_runtime.ui.text_metrics,
        });
        manager.overlay_runtime.ui.scroll_horizontal_chars = viewport.clamped_horizontal_chars;
        manager.overlay_runtime.ui.scroll_lines_from_bottom = viewport.clamped_lines_from_bottom;

        for (line_idx, line) in viewport.view_rows.iter().enumerate() {
            let (line_color, stripped) = scrollback_line_style(line, text.color);
            let line_x = if centered_demo {
                let text_w = measure_text_advance_precise(
                    &manager.overlay_runtime.ui.text_metrics,
                    stripped,
                    text_size,
                );
                transform.x + ((transform.w - text_w) * 0.5).max(0.0)
            } else {
                transform.x
            };
            commands.push(UiBatchCmd::Text {
                x: line_x,
                y: transform.y + (line_idx as f32 * line_height(text_size)),
                content: stripped.to_string(),
                color: line_color,
                size: text_size,
                clip: Some([transform.x, transform.y, transform.w, transform.h]),
            });
        }
        if !centered_demo && manager.overlay_runtime.ui.layout.show_scroll_indicators {
            push_scroll_indicators(
                commands,
                transform,
                visible_capacity,
                viewport.wrapped_rows.len(),
                viewport.clamped_lines_from_bottom,
                viewport.clamped_horizontal_chars,
                viewport.max_horizontal_chars,
                ui_scale,
                [0.62, 0.74, 0.86, 0.95],
            );
        }
        if !centered_demo && manager.overlay_runtime.ui.layout.show_scroll_hints {
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
}

fn build_logs_batches(manager: &mut SceneManager, commands: &mut Vec<UiBatchCmd>, ui_scale: f32) {
    let log_rect = compute_log_window_rect(
        &manager.overlay_runtime.ui.layout,
        manager.overlay_runtime.ui.screen_size,
        ui_scale,
    );
    let log_text_size = 12.0 * ui_scale;
    let log_header_h = 24.0 * ui_scale;
    let log_visible_capacity = visible_line_capacity(log_rect.body_h, log_text_size);
    let viewport = build_scroll_viewport(ScrollViewportSpec {
        lines: &manager.overlay_runtime.ui.log_lines,
        lines_from_bottom: manager.overlay_runtime.ui.log_scroll_lines_from_bottom,
        visible_capacity: log_visible_capacity,
        horizontal_chars: manager.overlay_runtime.ui.log_scroll_horizontal_chars,
        max_width: log_rect.body_w,
        text_size: log_text_size,
        metrics: &manager.overlay_runtime.ui.text_metrics,
    });
    manager.overlay_runtime.ui.log_scroll_horizontal_chars = viewport.clamped_horizontal_chars;
    manager.overlay_runtime.ui.log_scroll_lines_from_bottom = viewport.clamped_lines_from_bottom;

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
    let pause_status = if manager.overlay_runtime.ui.logs_paused {
        format!(
            "PAUSED ({})",
            manager.overlay_runtime.ui.log_paused_lines.len()
        )
    } else {
        "LIVE".to_string()
    };
    commands.push(UiBatchCmd::Text {
        x: log_rect.x + (8.0 * ui_scale),
        y: log_rect.y + (6.0 * ui_scale),
        content: format!("Logs [{pause_status}]"),
        color: if manager.overlay_runtime.ui.logs_paused {
            [0.98, 0.72, 0.42, 1.0]
        } else {
            [0.72, 0.91, 0.78, 1.0]
        },
        size: 12.0 * ui_scale,
        clip: Some([log_rect.x, log_rect.y, log_rect.w, log_rect.h]),
    });
    for (line_idx, line) in viewport.view_rows.iter().enumerate() {
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
    if manager.overlay_runtime.ui.layout.show_scroll_indicators {
        let log_content_rect = UiTransform {
            x: log_rect.x + (4.0 * ui_scale),
            y: log_rect.body_y,
            w: log_rect.w - (8.0 * ui_scale),
            h: log_rect.body_h,
        };
        push_scroll_indicators(
            commands,
            &log_content_rect,
            log_visible_capacity,
            viewport.wrapped_rows.len(),
            viewport.clamped_lines_from_bottom,
            viewport.clamped_horizontal_chars,
            viewport.max_horizontal_chars,
            ui_scale,
            [0.72, 0.88, 0.78, 0.95],
        );
    }
}

fn build_input_batches(manager: &mut SceneManager, commands: &mut Vec<UiBatchCmd>, ui_scale: f32) {
    if let (Some(transform), Some(style)) = (
        manager
            .overlay_runtime
            .world
            .get::<UiTransform>(manager.overlay_runtime.ui.input),
        manager
            .overlay_runtime
            .world
            .get::<UiStyle>(manager.overlay_runtime.ui.root),
    ) && ui_node_visible(manager, manager.overlay_runtime.ui.input)
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
        manager
            .overlay_runtime
            .world
            .get::<UiTransform>(manager.overlay_runtime.ui.input),
        manager
            .overlay_runtime
            .world
            .get::<UiText>(manager.overlay_runtime.ui.input),
        manager
            .overlay_runtime
            .world
            .get::<UiInputField>(manager.overlay_runtime.ui.input),
    ) && ui_node_visible(manager, manager.overlay_runtime.ui.input)
    {
        let scaled_text_size = text.size * ui_scale;
        let (content_x, content_y, content_w, content_h) = input_content_rect(transform, ui_scale);
        let layout = build_visible_multiline_input(
            &mut manager.overlay_runtime.ui.input_editor,
            &manager.overlay_runtime.ui.text_metrics,
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

        if manager.overlay_runtime.ui.caret_visible && input_field.focused {
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
        manager
            .overlay_runtime
            .world
            .get::<UiTransform>(manager.overlay_runtime.ui.confirm_button),
        manager
            .overlay_runtime
            .world
            .get::<UiStyle>(manager.overlay_runtime.ui.confirm_button),
        manager
            .overlay_runtime
            .world
            .get::<UiButton>(manager.overlay_runtime.ui.confirm_button),
        manager
            .overlay_runtime
            .world
            .get::<UiInteraction>(manager.overlay_runtime.ui.confirm_button),
        manager
            .overlay_runtime
            .world
            .get::<UiText>(manager.overlay_runtime.ui.confirm_button),
    ) && ui_node_visible(manager, manager.overlay_runtime.ui.confirm_button)
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
}

fn push_world_stats_panel(
    commands: &mut Vec<UiBatchCmd>,
    stats: &UiWorldHudStats,
    ui: &ConsoleUiRuntimeState,
    ui_scale: f32,
) {
    if !stats.visible {
        return;
    }

    let margin = 10.0 * ui_scale;
    let panel_w = 300.0 * ui_scale;
    let base_lines = 2_usize;
    let total_lines = base_lines + stats.lines.len();
    let panel_h = (40.0 + total_lines as f32 * 18.0).max(74.0) * ui_scale;
    let panel_x = margin;
    let panel_y = margin;
    let panel_w = panel_w.min((ui.screen_size.0 - (margin * 2.0)).max(80.0));
    let panel_h = panel_h.min((ui.screen_size.1 - (margin * 2.0)).max(40.0));
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
        content: stats.panel_title.clone(),
        color: [0.88, 0.95, 1.0, 1.0],
        size: 12.0 * ui_scale,
        clip,
    });
    commands.push(UiBatchCmd::Text {
        x: panel_x + (10.0 * ui_scale),
        y: panel_y + (30.0 * ui_scale),
        content: format!("player=({:.1}, {:.1})", stats.player_x, stats.player_y),
        color: [0.56, 0.94, 0.66, 1.0],
        size: 11.0 * ui_scale,
        clip,
    });
    commands.push(UiBatchCmd::Text {
        x: panel_x + (10.0 * ui_scale),
        y: panel_y + (48.0 * ui_scale),
        content: format!(
            "enemies={} slain={}",
            stats.enemies_alive, stats.enemy_kills
        ),
        color: [0.98, 0.74, 0.42, 1.0],
        size: 11.0 * ui_scale,
        clip,
    });
    for (index, line) in stats.lines.iter().enumerate() {
        commands.push(UiBatchCmd::Text {
            x: panel_x + (10.0 * ui_scale),
            y: panel_y + ((66.0 + index as f32 * 16.0) * ui_scale),
            content: line.clone(),
            color: [0.84, 0.9, 0.98, 1.0],
            size: 10.0 * ui_scale,
            clip,
        });
    }
}

fn build_diagnostics_batches(
    manager: &SceneManager,
    input: &InputState,
    stats: &UiWorldHudStats,
    commands: &mut Vec<UiBatchCmd>,
    ui_scale: f32,
) {
    push_world_stats_panel(commands, stats, &manager.overlay_runtime.ui, ui_scale);

    if manager.overlay_runtime.ui.editor.enabled {
        if let Some(selected_entity) = selected_editor_entity(&manager.overlay_runtime.ui)
            && let Some(selected_rect) = manager
                .overlay_runtime
                .world
                .get::<UiTransform>(selected_entity)
        {
            push_outline(
                commands,
                selected_rect,
                2.0 * ui_scale,
                [0.95, 0.55, 0.15, 0.95],
            );
        }

        if let Some(root_rect) = manager
            .overlay_runtime
            .world
            .get::<UiTransform>(manager.overlay_runtime.ui.root)
        {
            let stack_labels = manager
                .overlays
                .iter()
                .map(|slot| slot.active.label())
                .collect::<Vec<_>>()
                .join(" > ");
            let debug_pos = manager
                .world_runtime
                .ctx
                .world
                .get::<WorldDebugPosition>(manager.world_runtime.ctx.debug_entity)
                .map(|p| format!("({:.1}, {:.1})", p.x, p.y))
                .unwrap_or_else(|| "(n/a)".to_string());
            commands.push(UiBatchCmd::Text {
                x: root_rect.x + (8.0 * ui_scale),
                y: root_rect.y + (8.0 * ui_scale),
                content: manager.overlay_runtime.ui.editor.status.clone(),
                color: [0.98, 0.84, 0.52, 1.0],
                size: 12.0 * ui_scale,
                clip: Some([root_rect.x, root_rect.y, root_rect.w, root_rect.h]),
            });
            commands.push(UiBatchCmd::Text {
                x: root_rect.x + (8.0 * ui_scale),
                y: root_rect.y + (24.0 * ui_scale),
                content: format!(
                    "scene world={} paused={} overlays=[{}]",
                    manager.world.active.label(),
                    manager.world.paused,
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
                    manager.world_runtime.ctx.frame_count, input.overlay_consumed, debug_pos
                ),
                color: [0.72, 0.86, 0.84, 1.0],
                size: 11.0 * ui_scale,
                clip: Some([root_rect.x, root_rect.y, root_rect.w, root_rect.h]),
            });
        }
    }
}

