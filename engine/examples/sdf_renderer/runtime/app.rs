// Owner: SDF Renderer Example - App Entry and Systems
use crate::*;

pub(crate) fn run() -> Result<()> {
    let mut app = App::new();
    app.set_title("Grotto Quest - 3D SDF Compute Renderer");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(GridPlugin);
    app.add_plugin(DebugMetricsPlugin);
    app.add_plugin(RenderPlugin);
    app.add_plugin(SdfRendererExamplePlugin);
    app.run()
}

struct SdfRendererExamplePlugin;

impl Plugin for SdfRendererExamplePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SdfWorldState>();
        app.init_resource::<SdfRuntimeConfigState>();
        app.init_resource::<UiWorldHudStats>();
        app.add_systems(Startup, sdf_renderer_example_setup_system);
        app.add_systems(
            Update,
            sdf_renderer_example_update_system.after(CoreSet::Scene),
        );
    }
}

fn sdf_renderer_example_setup_system(
    mut frame_bindings: ResMut<RenderFrameResourceBindings>,
    mut render_graph_registry: ResMut<RenderGraphRegistryResource>,
    mut render_executor_registry: ResMut<RenderPassExecutorRegistryResource>,
    mut input: ResMut<InputState>,
    mut state: ResMut<SdfWorldState>,
    mut runtime_config: ResMut<SdfRuntimeConfigState>,
    mut hud: ResMut<UiWorldHudStats>,
) -> Result<()> {
    frame_bindings.register_resource::<SdfWorldState>();
    *hud = UiWorldHudStats::default();
    let params_config = load_config_with_default::<SdfParamsConfig>(PARAMS_CONFIG_FILE);
    apply_sdf_params(&mut state, &params_config);
    runtime_config.controls = params_config.controls;
    runtime_config.params_config_path = find_config_path(PARAMS_CONFIG_FILE);
    runtime_config.params_config_modified = file_modified(&runtime_config.params_config_path);

    let input_bindings = load_config_with_default::<SdfInputBindingsConfig>(INPUT_BINDINGS_CONFIG_FILE);
    let applied_bindings = apply_input_bindings(&mut input, &input_bindings);

    let render_graph_config = load_config_with_default::<SdfRenderGraphConfig>(RENDER_GRAPH_CONFIG_FILE);
    let (active_render_graph_config, spec) = match render_graph_config.to_spec() {
        Ok(spec) => (render_graph_config.clone(), spec),
        Err(err) => {
            tracing::error!(
                config = RENDER_GRAPH_CONFIG_FILE,
                ?err,
                "invalid sdf render graph config; using built-in defaults"
            );
            let fallback = SdfRenderGraphConfig::default();
            let spec = fallback.to_spec()?;
            (fallback, spec)
        }
    };
    render_graph_registry.register_feature_graph(spec);

    let bound_executors =
        match active_render_graph_config.register_custom_executors(&mut render_executor_registry) {
            Ok(count) => count,
            Err(err) => {
                tracing::error!(
                    config = RENDER_GRAPH_CONFIG_FILE,
                    ?err,
                    "invalid sdf executor bindings; using built-in defaults"
                );
                let fallback = SdfRenderGraphConfig::default();
                fallback.register_custom_executors(&mut render_executor_registry)?
            }
        };

    tracing::info!(
        config_path = find_config_path(PARAMS_CONFIG_FILE).display().to_string(),
        display_fit_mode = state.display_fit_mode,
        display_target_aspect = state.display_target_aspect,
        display_render_scale = state.display_render_scale,
        display_bar_color = ?state.display_bar_color,
        bindings = applied_bindings,
        graph_passes = active_render_graph_config.passes.len(),
        graph_compute_pipelines = active_render_graph_config.compute_pipelines.len(),
        graph_render_pipelines = active_render_graph_config.render_builtin_pipelines.len(),
        executor_bindings = bound_executors,
        "sdf renderer setup applied"
    );

    Ok(())
}

fn sdf_renderer_example_update_system(
    input: Res<InputState>,
    time: Res<Time>,
    scene: Res<SceneRuntimeState>,
    mut state: ResMut<SdfWorldState>,
    mut runtime_config: ResMut<SdfRuntimeConfigState>,
    mut hud: ResMut<UiWorldHudStats>,
) -> Result<()> {
    maybe_reload_sdf_params(&mut state, &mut runtime_config);
    state.agents.clear();
    let controls = runtime_config.controls;

    if input.toggle_pause_menu {
        state.world_paused = !state.world_paused;
    }

    if !state.world_paused {
        state.elapsed_time_seconds += time.delta_seconds.max(0.0);
    }

    if input.left_mouse_down() {
        let yaw_delta = input.mouse_delta.0 * controls.mouse_rotate_sensitivity;
        let pitch_delta = input.mouse_delta.1 * controls.mouse_rotate_sensitivity;
        state.camera_yaw -= yaw_delta;
        state.camera_pitch -= pitch_delta;
    }

    if input.scroll_delta.abs() > f32::EPSILON {
        let zoom_delta = input.scroll_delta * controls.scroll_zoom_sensitivity;
        state.camera_distance -= zoom_delta;
    }

    let min_pitch = state.camera_pitch_min.min(state.camera_pitch_max);
    let max_pitch = state.camera_pitch_min.max(state.camera_pitch_max);
    let min_distance = state
        .camera_distance_min
        .min(state.camera_distance_max)
        .max(0.1);
    let max_distance = state
        .camera_distance_min
        .max(state.camera_distance_max)
        .max(min_distance);
    state.camera_pitch = state.camera_pitch.clamp(min_pitch, max_pitch);
    state.camera_distance = state.camera_distance.clamp(min_distance, max_distance);

    let mut speed = controls.base_move_speed;
    if input.action_down(ACTION_SPEED_UP) {
        speed *= controls.speed_up_multiplier;
    }
    if input.action_down(ACTION_SPEED_DOWN) {
        speed *= controls.speed_down_multiplier;
    }

    let move_dt = speed * time.delta_seconds;
    let yaw = state.camera_yaw;
    let forward = [yaw.sin(), yaw.cos()];
    let right = [forward[1], -forward[0]];

    let forward_axis = (if input.action_down(action::WORLD_MOVE_UP) {
        1.0
    } else {
        0.0
    }) - (if input.action_down(action::WORLD_MOVE_DOWN) {
        1.0
    } else {
        0.0
    });
    let strafe_axis = (if input.action_down(action::WORLD_MOVE_RIGHT) {
        1.0
    } else {
        0.0
    }) - (if input.action_down(action::WORLD_MOVE_LEFT) {
        1.0
    } else {
        0.0
    });
    let vertical_axis = (if input.action_down(ACTION_UP) { 1.0 } else { 0.0 })
        - (if input.action_down(ACTION_DOWN) { 1.0 } else { 0.0 });

    state.camera_target[0] += (forward[0] * forward_axis + right[0] * strafe_axis) * move_dt;
    state.camera_target[2] += (forward[1] * forward_axis + right[1] * strafe_axis) * move_dt;
    state.camera_target[1] += vertical_axis * move_dt;
    state.camera_target[1] =
        state.camera_target[1].clamp(controls.camera_target_y_min, controls.camera_target_y_max);

    if input.action_pressed(ACTION_DEBUG_NEXT) {
        state.debug_view_mode = (state.debug_view_mode + 1) % 4;
    }
    if input.action_pressed(ACTION_DEBUG_PREV) {
        state.debug_view_mode = if state.debug_view_mode == 0 {
            3
        } else {
            state.debug_view_mode.saturating_sub(1)
        };
    }

    let player = state.agents.iter().find(|agent| agent.team == 0);
    let enemies_alive = state
        .agents
        .iter()
        .filter(|agent| agent.team != 0 && agent.health_ratio > 0.0)
        .count();
    *hud = if let Some(player) = player {
        UiWorldHudStats {
            visible: true,
            player_x: player.x,
            player_y: player.y,
            enemies_alive,
            enemy_kills: scene.enemy_kills,
            ..UiWorldHudStats::default()
        }
    } else {
        UiWorldHudStats::default()
    };

    Ok(())
}
