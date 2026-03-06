// Owner: Engine Scene Plugin - Runtime Helpers
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
    runtime.ctx.player_move_x =
        (if input.world_move_right { 1.0 } else { 0.0 }) - (if input.world_move_left { 1.0 } else { 0.0 });
    runtime.ctx.player_move_y =
        (if input.world_move_up { 1.0 } else { 0.0 }) - (if input.world_move_down { 1.0 } else { 0.0 });
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
    let distance_min = camera_cfg.distance_min.min(camera_cfg.distance_max).max(0.1);
    let distance_max = camera_cfg.distance_min.max(camera_cfg.distance_max).max(distance_min);
    runtime.ctx.camera_pitch = runtime.ctx.camera_pitch.clamp(pitch_min, pitch_max);
    runtime.ctx.camera_distance = runtime.ctx.camera_distance.clamp(distance_min, distance_max);

    let latest_modified = gameplay_config_modified();
    if should_reload(true, false, runtime.ctx.gameplay_config_modified, latest_modified) {
        let (config, modified, error) = load_gameplay_config_with_modified_and_error();
        runtime.ctx.gameplay_config = config;
        runtime.ctx.gameplay_config_modified = modified;
        runtime.ctx.gameplay_config_revision = runtime.ctx.gameplay_config_revision.saturating_add(1);
        let payload = ReloadStatusPayload::new(
            "gameplay_config",
            manager.world.active.label(),
            if error.is_some() { "fallback" } else { "reloaded" },
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

fn sync_world_scene_context_from_session(manager: &mut SceneManager, session: &SessionRuntimeState) {
    let runtime = &mut manager.world_runtime;
    runtime.ctx.session_admitted = session.admitted;
    runtime.ctx.session_lobby_id = session.lobby_id.clone();
    runtime.ctx.session_roster_player_codes = session.roster_player_codes.clone();
    runtime.ctx.session_max_players = session.max_players.max(1);
    runtime.ctx.session_ai_fill_target = session.ai_fill_target.clamp(1, runtime.ctx.session_max_players);
    runtime.ctx.session_settings_json = session.settings_json.clone();
}

fn publish_scene_state(
    manager: &SceneManager,
    scene_state: &mut SceneRuntimeState,
    gameplay: &mut GameplayRuntimeConfig,
    overlay: &mut UiOverlayState,
) {
    let (scene_state_value, gameplay_value, overlay_value, _) = snapshot_public_scene_state(manager);
    *scene_state = scene_state_value;
    *gameplay = gameplay_value;
    *overlay = overlay_value;
}
