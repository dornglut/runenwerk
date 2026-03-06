// Owner: Engine Scene Plugin - Runtime Helpers
fn rebuild_overlay_stack(manager: &mut SceneManager, slots: &[SceneSlot]) -> Result<()> {
    let slots = if slots.is_empty() {
        vec![SceneSlot {
            active: SceneId::ConsoleUi,
            paused: false,
            visible: false,
        }]
    } else {
        slots.to_vec()
    };
    let screen_size = manager.overlay_runtime.ui.screen_size;
    let scale = manager.overlay_runtime.ui.scale;
    let last_index = slots.len().saturating_sub(1);
    let mut stack = Vec::new();
    for slot in slots.iter().take(last_index) {
        stack.push((
            *slot,
            build_overlay_runtime(slot.active, screen_size, scale, &manager.registry)?,
        ));
    }
    let active_slot = slots[last_index];
    manager.overlay_runtime =
        build_overlay_runtime(active_slot.active, screen_size, scale, &manager.registry)?;
    manager.overlay_back_stack = stack;
    manager.overlays = slots;
    Ok(())
}

fn snapshot_public_scene_state(
    manager: &SceneManager,
) -> (
    SceneRuntimeState,
    GameplayRuntimeConfig,
    UiOverlayState,
    SessionRuntimeState,
) {
    let gameplay = GameplayRuntimeConfig {
        chunk_size: manager.world_runtime.ctx.gameplay_config.chunk_size,
        chunk_load_radius: manager.world_runtime.ctx.gameplay_config.chunk_load_radius,
        infinite_world: manager.world_runtime.ctx.gameplay_config.infinite_world,
    };
    let scene_state = SceneRuntimeState {
        world_scene_label: manager.world.active.label().to_string(),
        overlay_scene_label: manager.active_overlay().label().to_string(),
        overlay_visible: manager.overlay_visible(),
        world_paused: manager.world.paused,
        enemy_kills: manager.world_runtime.ctx.enemy_kills,
        gameplay,
    };
    let overlay = UiOverlayState {
        screen_size: manager.overlay_runtime.ui.screen_size,
        scale: manager.overlay_runtime.ui.scale,
        ..UiOverlayState::default()
    };
    let session = SessionRuntimeState {
        admitted: manager.world_runtime.ctx.session_admitted,
        lobby_id: manager.world_runtime.ctx.session_lobby_id.clone(),
        roster_player_codes: manager
            .world_runtime
            .ctx
            .session_roster_player_codes
            .clone(),
        max_players: manager.world_runtime.ctx.session_max_players,
        ai_fill_target: manager.world_runtime.ctx.session_ai_fill_target,
        settings_json: manager.world_runtime.ctx.session_settings_json.clone(),
    };
    (scene_state, gameplay, overlay, session)
}

fn system_time_to_millis(value: Option<SystemTime>) -> Option<u64> {
    value.and_then(|time| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
    })
}

fn millis_to_system_time(value: Option<u64>) -> Option<SystemTime> {
    value.map(|millis| UNIX_EPOCH + Duration::from_millis(millis))
}

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
        let path = manager
            .registry
            .ui_template_path(active)
            .unwrap_or("<none>");
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

    manager.world_runtime.ctx.delta_seconds =
        fixed_time.step_seconds.clamp(1.0 / 240.0, 1.0 / 30.0);
    manager.world_runtime.ctx.fixed_step_seconds = manager.world_runtime.ctx.delta_seconds;
    manager
        .world_runtime
        .scheduler
        .run(&mut manager.world_runtime.ctx)?;
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

fn sync_overlay_viewport(manager: &mut SceneManager, window: &WindowState) {
    manager.set_overlay_viewport(
        (window.size_px.0 as f32, window.size_px.1 as f32),
        window.scale_factor as f32,
    );
}

fn sync_world_scene_context_from_input(
    manager: &mut SceneManager,
    input: &InputState,
    frame_delta_seconds: f32,
    fixed_step_seconds: f32,
) {
    let active_overlay_label = manager.active_overlay().label().to_string();
    let overlay_visible = manager.overlay_visible();
    let world_paused = manager.world.paused;
    let runtime = &mut manager.world_runtime;
    runtime.ctx.overlay_consumed = input.overlay_consumed;
    runtime.ctx.overlay_scene_label = active_overlay_label;
    runtime.ctx.player_move_x = (if input.world_move_right { 1.0 } else { 0.0 })
        - (if input.world_move_left { 1.0 } else { 0.0 });
    runtime.ctx.player_move_y = (if input.world_move_up { 1.0 } else { 0.0 })
        - (if input.world_move_down { 1.0 } else { 0.0 });
    runtime.ctx.fixed_step_seconds = fixed_step_seconds;

    if !overlay_visible && !world_paused {
        let camera_cfg = &runtime.ctx.gameplay_config.camera;
        let rotate_sensitivity = camera_cfg.rotate_sensitivity.max(0.0);
        let yaw_sign = if camera_cfg.invert_x { 1.0 } else { -1.0 };
        let pitch_sign = if camera_cfg.invert_y { -1.0 } else { 1.0 };
        runtime.ctx.camera_yaw += input.mouse_delta.0 * rotate_sensitivity * yaw_sign;
        runtime.ctx.camera_pitch += input.mouse_delta.1 * rotate_sensitivity * pitch_sign;
    }
    if !overlay_visible && input.scroll_delta.abs() > f32::EPSILON {
        let camera_cfg = &runtime.ctx.gameplay_config.camera;
        let zoom_sensitivity = camera_cfg.zoom_sensitivity.max(0.0);
        let zoom_sign = if camera_cfg.invert_zoom { 1.0 } else { -1.0 };
        runtime.ctx.camera_distance += input.scroll_delta * zoom_sensitivity * zoom_sign;
    }
    let camera_cfg = &runtime.ctx.gameplay_config.camera;
    let pitch_min = camera_cfg.pitch_min.min(camera_cfg.pitch_max);
    let pitch_max = camera_cfg.pitch_min.max(camera_cfg.pitch_max);
    let distance_min = camera_cfg
        .distance_min
        .min(camera_cfg.distance_max)
        .max(0.1);
    let distance_max = camera_cfg
        .distance_min
        .max(camera_cfg.distance_max)
        .max(distance_min);
    runtime.ctx.camera_pitch = runtime.ctx.camera_pitch.clamp(pitch_min, pitch_max);
    runtime.ctx.camera_distance = runtime
        .ctx
        .camera_distance
        .clamp(distance_min, distance_max);

    let latest_modified = gameplay_config_modified();
    if should_reload(
        true,
        false,
        runtime.ctx.gameplay_config_modified,
        latest_modified,
    ) {
        let (config, modified, error) = load_gameplay_config_with_modified_and_error();
        runtime.ctx.gameplay_config = config;
        runtime.ctx.gameplay_config_modified = modified;
        runtime.ctx.gameplay_config_revision =
            runtime.ctx.gameplay_config_revision.saturating_add(1);
        let payload = ReloadStatusPayload::new(
            "gameplay_config",
            manager.world.active.label(),
            if error.is_some() {
                "fallback"
            } else {
                "reloaded"
            },
            GAMEPLAY_CONFIG_PATH,
            runtime.ctx.gameplay_config_revision,
            true,
            modified,
            error,
            None,
        );
        manager
            .channels
            .overlay_console_lines
            .push(format!("[world] {}", payload.line()));
    }

    runtime.ctx.delta_seconds = frame_delta_seconds.max(0.0);
}

fn sync_world_scene_context_from_session(
    manager: &mut SceneManager,
    session: &SessionRuntimeState,
) {
    let runtime = &mut manager.world_runtime;
    runtime.ctx.session_admitted = session.admitted;
    runtime.ctx.session_lobby_id = session.lobby_id.clone();
    runtime.ctx.session_roster_player_codes = session.roster_player_codes.clone();
    runtime.ctx.session_max_players = session.max_players.max(1);
    runtime.ctx.session_ai_fill_target = session
        .ai_fill_target
        .clamp(1, runtime.ctx.session_max_players);
    runtime.ctx.session_settings_json = session.settings_json.clone();
}

fn publish_scene_state(
    manager: &SceneManager,
    scene_state: &mut SceneRuntimeState,
    gameplay: &mut GameplayRuntimeConfig,
    overlay: &mut UiOverlayState,
) {
    let (scene_state_value, gameplay_value, overlay_value, _) =
        snapshot_public_scene_state(manager);
    *scene_state = scene_state_value;
    *gameplay = gameplay_value;
    *overlay = overlay_value;
}

fn with_scene_manager_mut<T>(
    world: &mut ecs::World,
    f: impl FnOnce(&mut SceneManager) -> Result<T>,
) -> Result<T> {
    if !world.has_resource::<SceneResource>() {
        return Err(anyhow!("ScenePlugin is not installed"));
    }
    let window = world.resource::<WindowState>().ok().cloned();
    let mut scene_resource = world
        .resource_mut::<SceneResource>()
        .map_err(|_| anyhow!("ScenePlugin resource is not available"))?;
    if scene_resource.manager.is_none() {
        let window = window.ok_or_else(|| anyhow!("WindowState is not available"))?;
        scene_resource.manager = Some(SceneManager::new(&window)?);
    }
    let manager = scene_resource
        .manager
        .as_mut()
        .ok_or_else(|| anyhow!("scene manager failed to initialize"))?;
    f(manager)
}

pub fn switch_scene_by_id(world: &mut ecs::World, scene_id: &str) -> Result<bool> {
    let normalized = normalize_scene_label_alias(scene_id);
    let Some(scene) = SceneId::from_label(&normalized) else {
        return Ok(false);
    };
    with_scene_manager_mut(world, |manager| {
        match scene.layer() {
            SceneLayer::World => {
                manager.queue(SceneCommand::ReplaceWorldByLabel(normalized));
                manager.queue(SceneCommand::PauseWorld(false));
            }
            SceneLayer::OverlayUi => {
                manager.queue(SceneCommand::ReplaceOverlayByLabel(normalized));
                manager.queue(SceneCommand::PauseWorld(true));
            }
        }
        Ok(true)
    })
}

pub fn set_world_by_id(world: &mut ecs::World, scene_id: &str) -> Result<bool> {
    let normalized = normalize_scene_label_alias(scene_id);
    let Some(scene) = SceneId::from_label(&normalized) else {
        return Ok(false);
    };
    if scene.layer() != SceneLayer::World {
        return Ok(false);
    }
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::ReplaceWorldByLabel(normalized));
        manager.queue(SceneCommand::PauseWorld(false));
        Ok(true)
    })
}

pub fn push_overlay_by_id(world: &mut ecs::World, scene_id: &str) -> Result<bool> {
    let normalized = normalize_scene_label_alias(scene_id);
    let Some(scene) = SceneId::from_label(&normalized) else {
        return Ok(false);
    };
    if scene.layer() != SceneLayer::OverlayUi {
        return Ok(false);
    }
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::PushOverlayByLabel(normalized));
        manager.queue(SceneCommand::PauseWorld(true));
        Ok(true)
    })
}

pub fn pop_overlay(world: &mut ecs::World) -> Result<()> {
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::PopOverlay);
        manager.queue(SceneCommand::PauseWorld(false));
        Ok(())
    })
}

pub fn set_world_paused(world: &mut ecs::World, paused: bool) -> Result<()> {
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::PauseWorld(paused));
        Ok(())
    })
}

pub fn toggle_world_pause(world: &mut ecs::World) -> Result<()> {
    with_scene_manager_mut(world, |manager| {
        manager.queue(SceneCommand::PauseWorld(!manager.world.paused));
        Ok(())
    })
}

fn flush_lifecycle_status(manager: &mut SceneManager) {
    let lifecycle_events = std::mem::take(&mut manager.channels.lifecycle_events);
    for event in lifecycle_events {
        let line = format_lifecycle_event(event);
        manager.channels.overlay_console_lines.push(line.clone());
        manager.overlay_runtime.ui.editor.status = format!("editor: {line}");
    }
}

fn apply_overlay_messages(manager: &mut SceneManager) {
    let messages = std::mem::take(&mut manager.channels.world_to_overlay);
    for message in messages {
        manager
            .channels
            .overlay_console_lines
            .push(format_world_message(message));
    }

    let messages = std::mem::take(&mut manager.channels.overlay_console_lines);
    for message in messages {
        if manager.overlay_runtime.ui.logs_paused {
            manager.overlay_runtime.ui.log_paused_lines.push(message);
        } else {
            manager.overlay_runtime.ui.log_lines.push(message);
        }
    }
    clamp_scrollback_lines(
        &mut manager.overlay_runtime.ui.log_lines,
        manager.overlay_runtime.ui.max_lines,
    );
    manager.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    if let Ok(mut entity) = manager
        .overlay_runtime
        .world
        .entity_mut(manager.overlay_runtime.ui.scrollback)
        && let Some(mut dirty) = entity.get_mut::<UiDirty>()
    {
        dirty.text = true;
    }
}

fn clamp_scrollback_lines(lines: &mut Vec<String>, max_lines: usize) {
    if max_lines == 0 {
        lines.clear();
        return;
    }
    let overflow = lines.len().saturating_sub(max_lines);
    if overflow > 0 {
        lines.drain(..overflow);
    }
}

fn format_world_message(message: WorldToOverlayMessage) -> String {
    match message {
        WorldToOverlayMessage::Tick { tick, overlay } => {
            format!("[world] tick={} overlay={}", tick, overlay)
        }
        WorldToOverlayMessage::Combat {
            source,
            target,
            damage,
            critical,
        } => {
            if critical {
                format!("[combat] {source} crits {target} for {damage}")
            } else {
                format!("[combat] {source} hits {target} for {damage}")
            }
        }
        WorldToOverlayMessage::Loot {
            item,
            amount,
            rarity,
        } => {
            format!("[loot] +{amount} {item} ({rarity})")
        }
        WorldToOverlayMessage::Quest { quest, state } => match state {
            QuestState::Started => format!("[quest] started: {quest}"),
            QuestState::Progress { current, goal } => {
                format!("[quest] {quest}: {current}/{goal}")
            }
            QuestState::Completed => format!("[quest] completed: {quest}"),
        },
    }
}

fn format_lifecycle_event(event: SceneLifecycleEvent) -> String {
    let phase = match event.phase {
        SceneLifecyclePhase::Enter => "enter",
        SceneLifecyclePhase::Exit => "exit",
        SceneLifecyclePhase::Pause => "pause",
        SceneLifecyclePhase::Resume => "resume",
    };
    let layer = match event.layer {
        SceneLayer::World => "world",
        SceneLayer::OverlayUi => "overlay",
    };
    format!("[world] scene:{layer} {} {phase}", event.scene.label())
}

fn normalize_scene_label_alias(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "gameplay" => "gameplay_stub".to_string(),
        "hub" => "hub_stub".to_string(),
        "console" => "console_ui".to_string(),
        "hud" | "pause" => "hud_ui".to_string(),
        "inventory" | "inv" => "inventory_ui".to_string(),
        other => other.replace('-', "_"),
    }
}
