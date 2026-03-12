// Owner: SDF Renderer Example - App Entry and Systems
use crate::*;

pub(crate) fn run() -> Result<()> {
    let input_config = load_config_with_default::<SdfInputBindingsConfig>(INPUT_BINDINGS_CONFIG_FILE);
    let input_bindings = app_input_bindings(&input_config);
    let binding_count = input_bindings.len();

    let mut app = App::new();
    app.set_title("Grotto Quest - 3D SDF Compute Renderer");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(GridPlugin);
    app.add_plugin(DebugMetricsPlugin);
    app.add_plugin(RenderPlugin);
    app.add_input_bindings(input_bindings);
    app.add_render_flow(build_render_flow());
    app.add_plugin(SdfRendererExamplePlugin);

    tracing::info!(
        bindings = binding_count,
        "sdf renderer input bindings loaded into app input layer"
    );

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
    mut render_executor_registry: ResMut<RenderPassExecutorRegistryResource>,
    mut state: ResMut<SdfWorldState>,
    mut runtime_config: ResMut<SdfRuntimeConfigState>,
    mut hud: ResMut<UiWorldHudStats>,
) -> Result<()> {
    let frame_bindings = &mut *frame_bindings;
    let mut render_executor_registry = &mut *render_executor_registry;
    let state = &mut *state;
    let runtime_config = &mut *runtime_config;
    let hud = &mut *hud;

    frame_bindings.register_resource::<SdfWorldState>();
    *hud = UiWorldHudStats::default();
    let params_config = load_config_with_default::<SdfParamsConfig>(PARAMS_CONFIG_FILE);
    apply_sdf_params(state, &params_config);
    runtime_config.controls = params_config.controls;
    runtime_config.params_config_path = find_config_path(PARAMS_CONFIG_FILE);
    runtime_config.params_config_modified = file_modified(&runtime_config.params_config_path);

    let shared = Arc::new(Mutex::new(SdfGpuSharedState::default()));
    render_executor_registry.register_custom(
        COMPUTE_EXECUTOR_ID,
        Arc::new(SdfComputeExecutor::new(Arc::clone(&shared))),
    );
    render_executor_registry.register_custom(
        COMPOSE_EXECUTOR_ID,
        Arc::new(SdfComposeExecutor::new(shared)),
    );

    tracing::info!(
        config_path = find_config_path(PARAMS_CONFIG_FILE).display().to_string(),
        display_fit_mode = state.display_fit_mode,
        display_target_aspect = state.display_target_aspect,
        display_render_scale = state.display_render_scale,
        display_bar_color = ?state.display_bar_color,
        flow = "sdf_renderer_example",
        compute_pass = COMPUTE_EXECUTOR_ID,
        compose_pass = COMPOSE_EXECUTOR_ID,
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
    let state = &mut *state;
    let runtime_config = &mut *runtime_config;
    let hud = &mut *hud;

    maybe_reload_sdf_params(state, runtime_config);
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
    let vertical_axis = (if input.action_down(ACTION_UP) {
        1.0
    } else {
        0.0
    }) - (if input.action_down(ACTION_DOWN) {
        1.0
    } else {
        0.0
    });

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
