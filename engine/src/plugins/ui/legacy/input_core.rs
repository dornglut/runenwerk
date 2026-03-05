// Owner: Grotto Quest Engine - UI Plugin
fn process_input_text_edit(
    manager: &mut SceneManager,
    input: &InputState,
    input_entity: ecs::Entity,
    input_visible: bool,
    input_nav_metrics: Option<(f32, f32)>,
) -> bool {
    let text_metrics = manager.overlay_runtime.ui.text_metrics.clone();
    let mut edited_text = false;
    let mut moved_cursor = false;
    if input_visible {
        let editor = &mut manager.overlay_runtime.ui.input_editor;
        let total_chars = char_count(&editor.text);
        editor.cursor_chars = editor.cursor_chars.min(total_chars);
        let mut reset_preferred_x = false;

        if !input.typed_text.is_empty() {
            let insert_at = byte_index_at_char(&editor.text, editor.cursor_chars);
            editor.text.insert_str(insert_at, &input.typed_text);
            editor.cursor_chars += char_count(&input.typed_text);
            edited_text = true;
            reset_preferred_x = true;
        }

        if input.insert_newline {
            let insert_at = byte_index_at_char(&editor.text, editor.cursor_chars);
            editor.text.insert(insert_at, '\n');
            editor.cursor_chars += 1;
            edited_text = true;
            reset_preferred_x = true;
        }

        if input.backspace && editor.cursor_chars > 0 {
            let remove_at = editor.cursor_chars - 1;
            let start = byte_index_at_char(&editor.text, remove_at);
            let end = byte_index_at_char(&editor.text, editor.cursor_chars);
            editor.text.replace_range(start..end, "");
            editor.cursor_chars = remove_at;
            edited_text = true;
            reset_preferred_x = true;
        }

        if input.delete && editor.cursor_chars < char_count(&editor.text) {
            let start = byte_index_at_char(&editor.text, editor.cursor_chars);
            let end = byte_index_at_char(&editor.text, editor.cursor_chars + 1);
            editor.text.replace_range(start..end, "");
            edited_text = true;
            reset_preferred_x = true;
        }

        if input.move_left {
            editor.cursor_chars = editor.cursor_chars.saturating_sub(1);
            moved_cursor = true;
            reset_preferred_x = true;
        }
        if input.move_right {
            editor.cursor_chars = (editor.cursor_chars + 1).min(char_count(&editor.text));
            moved_cursor = true;
            reset_preferred_x = true;
        }
        if input.move_home {
            editor.cursor_chars = 0;
            moved_cursor = true;
            reset_preferred_x = true;
        }
        if input.move_end {
            editor.cursor_chars = char_count(&editor.text);
            moved_cursor = true;
            reset_preferred_x = true;
        }

        if reset_preferred_x {
            editor.preferred_caret_x = None;
        }

        if let Some((input_text_size, input_content_w)) = input_nav_metrics {
            if input.move_up
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
            if input.move_down
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
        set_text_dirty(&mut manager.overlay_runtime.world, input_entity);
        manager.overlay_runtime.ui.caret_visible = true;
        manager.overlay_runtime.ui.caret_blink_timer = 0.0;
    }

    edited_text || moved_cursor
}

fn process_submit_and_pointer(
    manager: &mut SceneManager,
    input: &InputState,
    input_entity: ecs::Entity,
    button_entity: ecs::Entity,
    input_visible: bool,
    button_visible: bool,
) -> bool {
    let mut overlay_consumed = false;

    if input_visible && input.submitted {
        if let Ok(mut ui_entity) = manager.overlay_runtime.world.entity_mut(input_entity)
            && let Some(mut field) = ui_entity.get_mut::<UiInputField>()
        {
            field.submit_requested = true;
        }
        overlay_consumed = true;
    }

    let hovered = button_visible
        && manager
            .overlay_runtime
            .world
            .get::<UiTransform>(button_entity)
            .map(|rect| point_in_rect(input.mouse_position, rect))
            .unwrap_or(false);
    let clicked = hovered && input.left_mouse_pressed();
    let pressed = hovered && input.left_mouse_down();

    if let Ok(mut ui_entity) = manager.overlay_runtime.world.entity_mut(button_entity)
        && let Some(mut interaction) = ui_entity.get_mut::<UiInteraction>()
    {
        interaction.hovered = hovered;
        interaction.clicked = clicked;
        interaction.pressed = pressed;
    }

    if button_visible && clicked {
        if centered_demo_enabled(&manager.overlay_runtime.ui) {
            manager
                .overlay_runtime
                .world
                .emit_event(UiButtonRuntimeClickEvent {
                    entity: button_entity,
                });
        }
        if let Ok(mut ui_entity) = manager.overlay_runtime.world.entity_mut(input_entity)
            && let Some(mut field) = ui_entity.get_mut::<UiInputField>()
        {
            field.submit_requested = true;
        }
        overlay_consumed = true;
    }

    let input_clicked = input_visible
        && manager
            .overlay_runtime
            .world
            .get::<UiTransform>(input_entity)
            .map(|rect| point_in_rect(input.mouse_position, rect))
            .unwrap_or(false)
        && input.left_mouse_pressed();
    if input_clicked {
        if let Ok(mut ui_entity) = manager.overlay_runtime.world.entity_mut(input_entity)
            && let Some(mut interaction) = ui_entity.get_mut::<UiInteraction>()
        {
            interaction.focused = true;
        }
        manager.overlay_runtime.ui.caret_visible = true;
        manager.overlay_runtime.ui.caret_blink_timer = 0.0;
        overlay_consumed = true;
    }

    let mut should_submit = false;
    if let Ok(mut ui_entity) = manager.overlay_runtime.world.entity_mut(input_entity)
        && let Some(mut field) = ui_entity.get_mut::<UiInputField>()
        && field.submit_requested
    {
        field.submit_requested = false;
        should_submit = true;
    }

    if input_visible && should_submit {
        let line = format!(
            "{CONSOLE_PROMPT}{}",
            manager.overlay_runtime.ui.input_editor.text
        );
        manager
            .channels
            .overlay_submit
            .push(OverlaySubmitMessage::Line(line));

        manager.overlay_runtime.ui.input_editor.text.clear();
        manager.overlay_runtime.ui.input_editor.cursor_chars = 0;
        manager.overlay_runtime.ui.input_editor.viewport_row = 0;
        manager.overlay_runtime.ui.input_editor.preferred_caret_x = None;
        if let Ok(mut ui_entity) = manager.overlay_runtime.world.entity_mut(input_entity) {
            if let Some(mut field) = ui_entity.get_mut::<UiInputField>() {
                field.cursor = 0;
                field.focused = true;
            }
            if let Some(mut interaction) = ui_entity.get_mut::<UiInteraction>() {
                interaction.focused = true;
            }
        }
        set_text_dirty(&mut manager.overlay_runtime.world, input_entity);
        manager.overlay_runtime.ui.caret_visible = true;
        manager.overlay_runtime.ui.caret_blink_timer = 0.0;
        overlay_consumed = true;
    }

    if input_visible {
        if let Ok(mut ui_entity) = manager.overlay_runtime.world.entity_mut(input_entity) {
            if let Some(mut input_text) = ui_entity.get_mut::<UiText>() {
                input_text.content = format!(
                    "{CONSOLE_PROMPT}{}",
                    manager.overlay_runtime.ui.input_editor.text
                );
            }
            if let Some(mut field) = ui_entity.get_mut::<UiInputField>() {
                field.cursor = manager.overlay_runtime.ui.input_editor.cursor_chars;
            }
        }
    }

    overlay_consumed
}

fn process_scroll_routing(
    manager: &mut SceneManager,
    input: &InputState,
    scroll_entity: ecs::Entity,
    scroll_visible: bool,
    ui_scale: f32,
) -> bool {
    let console_hovered = scroll_visible
        && manager
            .overlay_runtime
            .world
            .get::<UiTransform>(scroll_entity)
            .map(|rect| point_in_rect(input.mouse_position, rect))
            .unwrap_or(false);
    let log_window = compute_log_window_rect(
        &manager.overlay_runtime.ui.layout,
        manager.overlay_runtime.ui.screen_size,
        ui_scale,
    );
    let log_hovered = point_in_rect(
        input.mouse_position,
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

    if input.scroll_delta != 0.0 {
        let scroll_step = if input.scroll_delta > 0.0 { 3 } else { -3 };
        if input.shift_down() {
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
    if input.page_up {
        if console_hovered {
            console_delta_lines += 12;
        } else if log_hovered {
            log_delta_lines += 12;
        } else {
            console_delta_lines += 12;
        }
    }
    if input.page_down {
        if console_hovered {
            console_delta_lines -= 12;
        } else if log_hovered {
            log_delta_lines -= 12;
        } else {
            console_delta_lines -= 12;
        }
    }

    manager.overlay_runtime.ui.scroll_lines_from_bottom = apply_scroll_delta(
        manager.overlay_runtime.ui.scroll_lines_from_bottom,
        console_delta_lines,
    );
    manager.overlay_runtime.ui.log_scroll_lines_from_bottom = apply_scroll_delta(
        manager.overlay_runtime.ui.log_scroll_lines_from_bottom,
        log_delta_lines,
    );
    manager.overlay_runtime.ui.scroll_horizontal_chars = apply_scroll_delta(
        manager.overlay_runtime.ui.scroll_horizontal_chars,
        console_delta_horizontal,
    );
    manager.overlay_runtime.ui.log_scroll_horizontal_chars = apply_scroll_delta(
        manager.overlay_runtime.ui.log_scroll_horizontal_chars,
        log_delta_horizontal,
    );

    console_delta_lines != 0
        || log_delta_lines != 0
        || console_delta_horizontal != 0
        || log_delta_horizontal != 0
}

