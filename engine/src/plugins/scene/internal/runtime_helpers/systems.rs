// Owner: Engine Scene Plugin - Runtime Helpers
fn scene_setup_system(
    window: Res<WindowState>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    session: Res<SessionRuntimeState>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    if scene_resource.manager.is_none() {
        scene_resource.manager = Some(SceneManager::new(&window)?);
    }
    if let Some(manager) = scene_resource.manager.as_mut() {
        sync_overlay_viewport(manager, &window);
        sync_world_scene_context_from_session(manager, &session);
        publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    }
    Ok(())
}

fn scene_transition_system(
    input: Res<InputState>,
    window: Res<WindowState>,
    time: Res<Time>,
    fixed_time: Res<FixedTimeConfig>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };

    sync_overlay_viewport(manager, &window);
    sync_world_scene_context_from_input(
        manager,
        &input,
        time.delta_seconds,
        fixed_time.step_seconds,
    );

    if input.toggle_pause_menu {
        let show_overlay = !manager.overlay_visible();
        manager.set_active_overlay_visible(show_overlay);
        manager.queue(SceneCommand::PauseWorld(show_overlay));
        if show_overlay && manager.active_overlay() != SceneId::HudUi {
            manager.queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
        }
    }
    if input.scene_next {
        let next = manager.active_overlay().next_overlay();
        manager.queue(SceneCommand::ReplaceOverlay(next));
    }
    if input.scene_prev {
        let prev = manager.active_overlay().previous_overlay();
        manager.queue(SceneCommand::ReplaceOverlay(prev));
    }
    if input.scene_console {
        manager.queue(SceneCommand::ReplaceOverlay(SceneId::ConsoleUi));
    }
    if input.scene_hud {
        manager.queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
    }
    if input.scene_overlay_push {
        let next = manager.active_overlay().next_overlay();
        manager.queue(SceneCommand::PushOverlay(next));
    }
    if input.scene_overlay_pop {
        manager.queue(SceneCommand::PopOverlay);
    }

    let result = manager.apply_pending()?;
    if result.world_changed {
        manager.overlay_runtime.ui.editor.status = format!(
            "editor: world scene switched to {}",
            manager.world.active.label()
        );
    }
    if result.overlay_changed {
        let active = manager.active_overlay();
        let path = manager.registry.ui_template_path(active).unwrap_or("<none>");
        manager.overlay_runtime.ui.editor.status = format!(
            "editor: overlay scene switched to {} ({}) [stack={}]",
            active.label(),
            path,
            manager.overlays.len()
        );
    }
    if result.world_pause_changed {
        manager.overlay_runtime.ui.editor.status = if manager.world.paused {
            "editor: world scene paused".to_string()
        } else {
            "editor: world scene resumed".to_string()
        };
    }

    flush_lifecycle_status(manager);
    publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    Ok(())
}

fn world_scene_update_system(
    fixed_time: Res<FixedTimeConfig>,
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    session: Res<SessionRuntimeState>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    sync_world_scene_context_from_session(manager, &session);
    if !manager.world.visible || manager.world.paused {
        publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
        return Ok(());
    }

    manager.world_runtime.ctx.delta_seconds = fixed_time.step_seconds.clamp(1.0 / 240.0, 1.0 / 30.0);
    manager.world_runtime.ctx.fixed_step_seconds = manager.world_runtime.ctx.delta_seconds;
    manager.world_runtime.scheduler.run(&mut manager.world_runtime.ctx)?;
    let outbound = std::mem::take(&mut manager.world_runtime.ctx.outbound_notifications);
    manager.channels.world_to_overlay.extend(outbound);
    publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    Ok(())
}

fn scene_overlay_update_system(
    mut scene_resource: ResMut<SceneResource>,
    mut scene_state: ResMut<SceneRuntimeState>,
    mut gameplay: ResMut<GameplayRuntimeConfig>,
    mut overlay: ResMut<UiOverlayState>,
) -> Result<()> {
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };

    apply_overlay_messages(manager);
    publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
    Ok(())
}
