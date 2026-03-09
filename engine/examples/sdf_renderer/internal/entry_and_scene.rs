// Owner: SDF Renderer Example - Entry, Plugin, and Scene Update
const ACTION_UP: &str = "sdf.move_up";
const ACTION_DOWN: &str = "sdf.move_down";
const ACTION_DEBUG_NEXT: &str = "sdf.debug_next";
const ACTION_DEBUG_PREV: &str = "sdf.debug_prev";
const ACTION_SPEED_UP: &str = "sdf.speed_up";
const ACTION_SPEED_DOWN: &str = "sdf.speed_down";

const SDF_ASSETS_DIR_PRIMARY: &str = "engine/examples/sdf_renderer/assets";
const SDF_ASSETS_DIR_FALLBACK: &str = "examples/sdf_renderer/assets";

const PARAMS_CONFIG_FILE: &str = "sdf_params.ron";
const INPUT_BINDINGS_CONFIG_FILE: &str = "input_bindings.ron";
const RENDER_GRAPH_CONFIG_FILE: &str = "render_graph.ron";
const SDF_COMPUTE_SHADER: &str =
    include_str!("../../../../assets/shaders/sdf_compute_3d_example.wgsl");
const SDF_COMPOSE_SHADER: &str =
    include_str!("../../../../assets/shaders/world_compose_fullscreen.wgsl");
const SDF_MAX_AGENTS: usize = 512;
const SDF_MAX_MODELS: usize = 1;

static SDF_CONTROLS: OnceLock<SdfControlsConfig> = OnceLock::new();

fn main() -> Result<()> {
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

#[derive(Debug, Clone)]
struct SdfWorldAgent {
    x: f32,
    y: f32,
    radius: f32,
    health_ratio: f32,
    team: u32,
}

#[derive(Debug, Clone)]
struct SdfWorldState {
    world_bounds: [f32; 4],
    world_paused: bool,
    camera_yaw: f32,
    camera_pitch: f32,
    camera_distance: f32,
    camera_pitch_min: f32,
    camera_pitch_max: f32,
    camera_distance_min: f32,
    camera_distance_max: f32,
    camera_target: [f32; 3],
    camera_fov_y: f32,
    debug_view_mode: u32,
    elapsed_time_seconds: f32,
    agents: Vec<SdfWorldAgent>,
}

impl Default for SdfWorldState {
    fn default() -> Self {
        Self {
            world_bounds: [-18.0, -18.0, 18.0, 18.0],
            world_paused: false,
            camera_yaw: std::f32::consts::PI,
            camera_pitch: 0.58,
            camera_distance: 14.0,
            camera_pitch_min: -1.10,
            camera_pitch_max: 1.10,
            camera_distance_min: 2.0,
            camera_distance_max: 80.0,
            camera_target: [0.0, 1.8, 0.0],
            camera_fov_y: 55.0f32.to_radians(),
            debug_view_mode: 0,
            elapsed_time_seconds: 0.0,
            agents: Vec::new(),
        }
    }
}

impl Plugin for SdfRendererExamplePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SdfWorldState>();
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
    mut hud: ResMut<UiWorldHudStats>,
) -> Result<()> {
    frame_bindings.register_resource::<SdfWorldState>();
    *hud = UiWorldHudStats::default();
    let params_config = load_config_with_default::<SdfParamsConfig>(PARAMS_CONFIG_FILE);
    apply_sdf_params(&mut state, &params_config);
    let _ = SDF_CONTROLS.set(params_config.controls);

    let input_bindings =
        load_config_with_default::<SdfInputBindingsConfig>(INPUT_BINDINGS_CONFIG_FILE);
    let applied_bindings = apply_input_bindings(&mut input, &input_bindings);

    let render_graph_config =
        load_config_with_default::<SdfRenderGraphConfig>(RENDER_GRAPH_CONFIG_FILE);
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
    mut hud: ResMut<UiWorldHudStats>,
) -> Result<()> {
    state.agents.clear();
    let controls = SDF_CONTROLS.get().copied().unwrap_or_default();

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
        state.debug_view_mode = (state.debug_view_mode + 3) % 4;
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
