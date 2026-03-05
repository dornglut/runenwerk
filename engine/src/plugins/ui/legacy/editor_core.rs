// Owner: Grotto Quest Engine - UI Plugin
pub(crate) fn ui_editor_system(
    input: Res<InputState>,
    mut scene_resource: ResMut<SceneResource>,
) -> anyhow::Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    if !manager.overlay_runtime.ui.editor.enabled {
        return Ok(());
    }

    let candidates = [
        (UiEditorNode::Root, manager.overlay_runtime.ui.root),
        (
            UiEditorNode::Scrollback,
            manager.overlay_runtime.ui.scrollback,
        ),
        (UiEditorNode::Input, manager.overlay_runtime.ui.input),
        (
            UiEditorNode::ConfirmButton,
            manager.overlay_runtime.ui.confirm_button,
        ),
    ];

    if input.left_mouse_pressed() {
        let mut rects: Vec<EditorNodeRect> = Vec::new();
        for (node, entity) in candidates {
            if let (Some(transform), Some(ui_node)) = (
                manager.overlay_runtime.world.get::<UiTransform>(entity),
                manager.overlay_runtime.world.get::<UiNode>(entity),
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

        manager.overlay_runtime.ui.editor.selected =
            pick_editor_node_at(input.mouse_position, &rects);
        manager.overlay_runtime.ui.editor.dragging = false;
        manager.overlay_runtime.ui.editor.drag_pointer_offset = (0.0, 0.0);
        match manager.overlay_runtime.ui.editor.selected {
            Some(node) => {
                if let Some(selected_rect) = rects.iter().find(|r| r.node == node).map(|r| r.rect) {
                    manager.overlay_runtime.ui.editor.drag_pointer_offset = (
                        input.mouse_position.0 - selected_rect.x,
                        input.mouse_position.1 - selected_rect.y,
                    );
                    manager.overlay_runtime.ui.editor.dragging = true;
                }
                manager.overlay_runtime.ui.editor.status =
                    format!("editor: selected {}", editor_node_label(node));
            }
            None => {
                manager.overlay_runtime.ui.editor.status = "editor: nothing selected".to_string();
            }
        }
    }

    if input.left_mouse_released() {
        manager.overlay_runtime.ui.editor.dragging = false;
    }

    if let Some(selected_node) = manager.overlay_runtime.ui.editor.selected {
        let Some(selected) = selected_editor_entity(&manager.overlay_runtime.ui) else {
            return Ok(());
        };
        let step = if input.shift_down() {
            10.0 * manager.overlay_runtime.ui.scale.max(1.0)
        } else {
            EDITOR_BASE_NUDGE_PX * manager.overlay_runtime.ui.scale.max(1.0)
        };
        let mut dx = 0.0;
        let mut dy = 0.0;
        if input.move_left {
            dx -= step;
        }
        if input.move_right {
            dx += step;
        }
        if input.move_up {
            dy -= step;
        }
        if input.move_down {
            dy += step;
        }
        if (dx != 0.0 || dy != 0.0)
            && manager
                .overlay_runtime
                .world
                .get::<UiTransform>(selected)
                .is_some()
        {
            apply_editor_translation(manager, selected_node, dx, dy);
            let pos = manager
                .overlay_runtime
                .world
                .get::<UiTransform>(selected)
                .map(|t| (t.x, t.y))
                .unwrap_or((0.0, 0.0));
            manager.overlay_runtime.ui.editor.status = format!(
                "editor: nudged {} to ({:.0}, {:.0})",
                editor_node_label(selected_node),
                pos.0,
                pos.1
            );
        }

        if manager.overlay_runtime.ui.editor.dragging
            && input.left_mouse_down()
            && let Some(current) = manager
                .overlay_runtime
                .world
                .get::<UiTransform>(selected)
                .copied()
        {
            let mut next_x =
                input.mouse_position.0 - manager.overlay_runtime.ui.editor.drag_pointer_offset.0;
            let mut next_y =
                input.mouse_position.1 - manager.overlay_runtime.ui.editor.drag_pointer_offset.1;
            if input.shift_down() {
                let grid = EDITOR_DRAG_SNAP_PX * manager.overlay_runtime.ui.scale.max(1.0);
                next_x = snap_to_grid(next_x, grid);
                next_y = snap_to_grid(next_y, grid);
            }
            let dx = next_x - current.x;
            let dy = next_y - current.y;
            apply_editor_translation(manager, selected_node, dx, dy);
            let pos = manager
                .overlay_runtime
                .world
                .get::<UiTransform>(selected)
                .map(|t| (t.x, t.y))
                .unwrap_or((next_x, next_y));
            manager.overlay_runtime.ui.editor.status = format!(
                "editor: dragging {} ({:.0}, {:.0})",
                editor_node_label(selected_node),
                pos.0,
                pos.1
            );
        }

        if input.editor_hide_selected {
            let can_hide = selected_node != UiEditorNode::Root;
            if can_hide {
                if let Ok(mut ui_entity) = manager.overlay_runtime.world.entity_mut(selected)
                    && let Some(mut node) = ui_entity.get_mut::<UiNode>()
                {
                    node.visible = false;
                    manager.overlay_runtime.ui.editor.status = format!(
                        "editor: hid {} (A restores hidden nodes)",
                        editor_node_label(selected_node)
                    );
                }
                manager.overlay_runtime.ui.editor.selected = Some(UiEditorNode::Root);
                manager.overlay_runtime.ui.editor.dragging = false;
            } else {
                manager.overlay_runtime.ui.editor.status =
                    "editor: root cannot be hidden".to_string();
            }
        }
    }

    if input.editor_restore_all {
        for entity in [
            manager.overlay_runtime.ui.root,
            manager.overlay_runtime.ui.scrollback,
            manager.overlay_runtime.ui.input,
            manager.overlay_runtime.ui.confirm_button,
        ] {
            if let Ok(mut ui_entity) = manager.overlay_runtime.world.entity_mut(entity)
                && let Some(mut node) = ui_entity.get_mut::<UiNode>()
            {
                node.visible = true;
            }
        }
        manager.overlay_runtime.ui.editor.status = "editor: restored all nodes".to_string();
    }

    if input.save_ui_template {
        match save_console_template_to_disk(
            &manager.overlay_runtime.world,
            &mut manager.overlay_runtime.ui,
        ) {
            Ok(path) => {
                manager.overlay_runtime.ui.editor.status =
                    format!("editor: saved {}", path.display());
                let _ = reload_console_template_if_changed(
                    &mut manager.overlay_runtime.world,
                    &mut manager.overlay_runtime.ui,
                    true,
                );
            }
            Err(err) => {
                manager.overlay_runtime.ui.editor.status = format!("editor: save failed: {err:#}");
            }
        }
    }

    Ok(())
}

