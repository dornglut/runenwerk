// Owner: Grotto Quest Engine - UI Plugin
pub(crate) fn ui_layout_system(mut scene_resource: ResMut<SceneResource>) -> anyhow::Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    if !manager.overlay_visible() {
        return Ok(());
    }
    if !manager.overlay_runtime.ui.layout_dirty {
        return Ok(());
    }

    let (screen_w, screen_h) = manager.overlay_runtime.ui.screen_size;
    let s = manager.overlay_runtime.ui.scale.max(1.0);
    let outer_margin = manager.overlay_runtime.ui.layout.outer_margin * s;
    let available_w = (screen_w - (outer_margin * 2.0)).max(1.0);
    let available_h = (screen_h - (outer_margin * 2.0)).max(1.0);
    let panel_w = clamp_panel_dimension(
        screen_w * manager.overlay_runtime.ui.layout.panel_width_ratio,
        manager.overlay_runtime.ui.layout.panel_min_width * s,
        available_w,
    );
    let panel_h = clamp_panel_dimension(
        screen_h * manager.overlay_runtime.ui.layout.panel_height_ratio,
        manager.overlay_runtime.ui.layout.panel_min_height * s,
        available_h,
    );
    let centered_demo = centered_demo_enabled(&manager.overlay_runtime.ui);
    let panel_x = if centered_demo {
        ((screen_w - panel_w) * 0.5).max(outer_margin)
    } else {
        outer_margin
    };
    let panel_y = if centered_demo {
        ((screen_h - panel_h) * 0.5).max(outer_margin)
    } else {
        (screen_h - panel_h - outer_margin).max(outer_margin)
    };
    let inner_padding = manager.overlay_runtime.ui.layout.inner_padding * s;
    let panel_inner_w = (panel_w - inner_padding * 2.0).max(1.0);
    let (footer_y, input_h, button_w, input_w) = adaptive_footer_metrics(
        panel_inner_w,
        panel_h,
        inner_padding,
        s,
        manager.overlay_runtime.ui.layout.footer_offset,
        manager.overlay_runtime.ui.layout.input_height,
        manager.overlay_runtime.ui.layout.button_width,
        manager.overlay_runtime.ui.layout.input_button_gap,
    );

    if let Ok(mut ui_entity) = manager
        .overlay_runtime
        .world
        .entity_mut(manager.overlay_runtime.ui.root)
        && let Some(mut root) = ui_entity.get_mut::<UiTransform>()
    {
        root.x = panel_x;
        root.y = panel_y;
        root.w = panel_w;
        root.h = panel_h;
    }
    if let Ok(mut ui_entity) = manager
        .overlay_runtime
        .world
        .entity_mut(manager.overlay_runtime.ui.scrollback)
        && let Some(mut scroll) = ui_entity.get_mut::<UiTransform>()
    {
        if centered_demo {
            let scroll_h =
                (panel_h * 0.34).clamp(52.0 * s, (panel_h - (inner_padding * 2.0)).max(1.0));
            let scroll_y = (panel_y + ((panel_h - scroll_h) * 0.30)).clamp(
                panel_y + inner_padding,
                panel_y + panel_h - inner_padding - scroll_h,
            );
            scroll.x = panel_x + inner_padding;
            scroll.y = scroll_y;
            scroll.w = panel_inner_w;
            scroll.h = scroll_h;
        } else {
            scroll.x = panel_x + inner_padding;
            scroll.y = panel_y + inner_padding;
            scroll.w = panel_inner_w;
            scroll.h = panel_h - footer_y - inner_padding;
        }
    }
    if let Ok(mut ui_entity) = manager
        .overlay_runtime
        .world
        .entity_mut(manager.overlay_runtime.ui.input)
        && let Some(mut input_transform) = ui_entity.get_mut::<UiTransform>()
    {
        input_transform.x = if centered_demo {
            panel_x + ((panel_w - input_w) * 0.5).max(inner_padding)
        } else {
            panel_x + inner_padding
        };
        input_transform.y = panel_y + panel_h - footer_y;
        input_transform.w = input_w;
        input_transform.h = input_h;
    }
    if let Ok(mut ui_entity) = manager
        .overlay_runtime
        .world
        .entity_mut(manager.overlay_runtime.ui.confirm_button)
        && let Some(mut button) = ui_entity.get_mut::<UiTransform>()
    {
        button.x = if centered_demo {
            panel_x + ((panel_w - button_w) * 0.5).max(inner_padding)
        } else {
            panel_x + panel_w - inner_padding - button_w
        };
        button.y = panel_y + panel_h - footer_y;
        button.w = button_w;
        button.h = input_h;
    }

    manager.overlay_runtime.ui.layout_dirty = false;
    Ok(())
}

pub(crate) fn ui_build_batches_system(
    input: Res<InputState>,
    hud_stats: Res<UiWorldHudStats>,
    mut scene_resource: ResMut<SceneResource>,
) -> anyhow::Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    let ui_scale = manager.overlay_runtime.ui.scale.max(1.0);
    let centered_demo = centered_demo_enabled(&manager.overlay_runtime.ui);
    let mut commands: Vec<UiBatchCmd> = Vec::new();

    if !manager.overlay_visible() {
        if !centered_demo {
            build_diagnostics_batches(manager, &input, &hud_stats, &mut commands, ui_scale);
        }
        manager.overlay_runtime.ui.batches.commands = commands;
        return Ok(());
    }

    build_console_batches(manager, &mut commands, ui_scale);
    if !centered_demo {
        build_logs_batches(manager, &mut commands, ui_scale);
    }
    build_input_batches(manager, &mut commands, ui_scale);
    if !centered_demo {
        build_diagnostics_batches(manager, &input, &hud_stats, &mut commands, ui_scale);
    }
    manager.overlay_runtime.ui.batches.commands = commands;
    Ok(())
}

