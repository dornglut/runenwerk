// Owner: Grotto Quest Engine - UI Plugin
pub(crate) fn ui_hot_reload_system(mut scene_resource: ResMut<SceneResource>) -> anyhow::Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    if !manager
        .overlay_runtime
        .world
        .has_resource::<UiRenderShaderConfig>()
    {
        manager
            .overlay_runtime
            .world
            .insert_resource(UiRenderShaderConfig::default());
    }
    if let Err(err) = reload_console_template_if_changed(
        &mut manager.overlay_runtime.world,
        &mut manager.overlay_runtime.ui,
        false,
    ) {
        tracing::warn!(?err, "ui hot reload failed");
    }
    Ok(())
}

pub(crate) fn ui_input_system(
    time: Res<Time>,
    mut input: ResMut<InputState>,
    mut scene_resource: ResMut<SceneResource>,
) -> anyhow::Result<()> {
    input.overlay_consumed = false;
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    if !manager.overlay_visible() {
        return Ok(());
    }

    if input.toggle_ui_editor_mode {
        manager.overlay_runtime.ui.editor.enabled = !manager.overlay_runtime.ui.editor.enabled;
        manager.overlay_runtime.ui.editor.dragging = false;
        manager.overlay_runtime.ui.editor.drag_pointer_offset = (0.0, 0.0);
        manager.overlay_runtime.ui.editor.status = if manager.overlay_runtime.ui.editor.enabled {
            "editor: on (click+drag move, Shift snap, arrows nudge, X hide node, A restore, Cmd/Ctrl+S save, F1 off)"
                .to_string()
        } else {
            "editor: off (F1 to toggle)".to_string()
        };
    }

    if manager.overlay_runtime.ui.editor.enabled {
        input.overlay_consumed = true;
        return Ok(());
    }

    let input_entity = manager.overlay_runtime.ui.input;
    let button_entity = manager.overlay_runtime.ui.confirm_button;
    let scroll_entity = manager.overlay_runtime.ui.scrollback;
    let input_visible = ui_node_visible(manager, input_entity);
    let button_visible = ui_node_visible(manager, button_entity);
    let scroll_visible = ui_node_visible(manager, scroll_entity);
    let ui_scale = manager.overlay_runtime.ui.scale.max(1.0);
    let input_nav_metrics = if let (Some(transform), Some(text)) = (
        manager
            .overlay_runtime
            .world
            .get::<UiTransform>(input_entity),
        manager.overlay_runtime.world.get::<UiText>(input_entity),
    ) {
        let (_, _, content_w, _) = input_content_rect(transform, ui_scale);
        Some((text.size * ui_scale, content_w))
    } else {
        None
    };

    let input_ref: &InputState = &input;
    let mut overlay_consumed = false;
    overlay_consumed |= process_input_text_edit(
        manager,
        input_ref,
        input_entity,
        input_visible,
        input_nav_metrics,
    );
    overlay_consumed |= process_submit_and_pointer(
        manager,
        input_ref,
        input_entity,
        button_entity,
        input_visible,
        button_visible,
    );

    manager.overlay_runtime.ui.caret_blink_timer += time.delta_seconds;
    while manager.overlay_runtime.ui.caret_blink_timer >= CARET_BLINK_SECONDS {
        manager.overlay_runtime.ui.caret_blink_timer -= CARET_BLINK_SECONDS;
        manager.overlay_runtime.ui.caret_visible = !manager.overlay_runtime.ui.caret_visible;
    }
    overlay_consumed |=
        process_scroll_routing(manager, input_ref, scroll_entity, scroll_visible, ui_scale);

    input.overlay_consumed = overlay_consumed;
    Ok(())
}

