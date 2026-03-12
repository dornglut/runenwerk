// Owner: Game of Life SDF Example - App Setup and Runtime Systems
use crate::rendering::{
    COMPOSE_EXECUTOR_ID, COMPUTE_EXECUTOR_ID, GameOfLifeComposeExecutor, GameOfLifeComputeExecutor,
    GameOfLifeGpuSharedState, build_feature_graph_spec,
};
use crate::runtime::{
    ACTION_SINGLE_STEP, ACTION_SPEED_DOWN, ACTION_SPEED_UP, ACTION_TOGGLE_PAUSE, GameOfLifeSdfState,
};
use anyhow::Result;
use engine::plugins::render::domain::{
    RenderFrameResourceBindings, RenderGraphRegistryResource, RenderPassExecutorRegistryResource,
};
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{
    App, CoreSet, InputState, Plugin, Res, ResMut, Startup, SystemConfigExt, Time, Update,
    WindowState,
};
use std::sync::{Arc, Mutex};
use winit::keyboard::KeyCode;

pub(crate) fn run() -> Result<()> {
    let mut app = App::new();
    app.set_title("Game of Life SDF - Render Plugin Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_plugin(GameOfLifeSdfExamplePlugin);
    app.run()
}

struct GameOfLifeSdfExamplePlugin;

impl Plugin for GameOfLifeSdfExamplePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameOfLifeSdfState>();
        app.add_systems(Startup, game_of_life_setup_system);
        app.add_systems(
            Update,
            game_of_life_update_system
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
    }
}

fn game_of_life_setup_system(
    mut frame_bindings: ResMut<RenderFrameResourceBindings>,
    mut render_graph_registry: ResMut<RenderGraphRegistryResource>,
    mut render_executor_registry: ResMut<RenderPassExecutorRegistryResource>,
    mut input: ResMut<InputState>,
) -> Result<()> {
    if !frame_bindings.contains_resource::<GameOfLifeSdfState>() {
        frame_bindings.register_resource::<GameOfLifeSdfState>();
    }

    render_graph_registry.register_feature_graph(build_feature_graph_spec()?);

    let shared = Arc::new(Mutex::new(GameOfLifeGpuSharedState::default()));
    render_executor_registry.register_custom(
        COMPUTE_EXECUTOR_ID,
        Arc::new(GameOfLifeComputeExecutor::new(Arc::clone(&shared))),
    );
    render_executor_registry.register_custom(
        COMPOSE_EXECUTOR_ID,
        Arc::new(GameOfLifeComposeExecutor::new(shared)),
    );

    input.map_key(ACTION_TOGGLE_PAUSE.to_string(), KeyCode::Space);
    input.map_key(ACTION_SINGLE_STEP.to_string(), KeyCode::Enter);
    input.map_key(ACTION_SPEED_UP.to_string(), KeyCode::PageUp);
    input.map_key(ACTION_SPEED_DOWN.to_string(), KeyCode::PageDown);

    tracing::info!(
        feature = "game_of_life_sdf_example",
        controls = "Space pause | Enter step | PageUp/PageDown speed",
        "game of life sdf render setup complete"
    );
    Ok(())
}

fn game_of_life_update_system(
    input: Res<InputState>,
    time: Res<Time>,
    mut state: ResMut<GameOfLifeSdfState>,
    mut window: ResMut<WindowState>,
) {
    let state = &mut *state;

    if input.action_pressed(ACTION_TOGGLE_PAUSE) {
        state.paused = !state.paused;
    }
    if input.action_pressed(ACTION_SINGLE_STEP) {
        state.single_step_requested = true;
    }
    if input.action_pressed(ACTION_SPEED_UP) {
        state.steps_per_second = (state.steps_per_second * 1.2).clamp(1.0, 120.0);
    }
    if input.action_pressed(ACTION_SPEED_DOWN) {
        state.steps_per_second = (state.steps_per_second * 0.8).clamp(1.0, 120.0);
    }

    let dt = time.delta_seconds.max(0.0);
    state.time_seconds += dt;

    let mut step_now = false;
    if state.single_step_requested {
        step_now = true;
        state.single_step_requested = false;
    }
    if !state.paused {
        state.accumulator_seconds += dt;
        let interval = state.step_interval_seconds();
        if state.accumulator_seconds >= interval {
            state.accumulator_seconds %= interval;
            step_now = true;
        }
    }

    state.step_simulation = step_now;
    if step_now {
        state.generation = state.generation.saturating_add(1);
    }

    let mode = if state.paused { "paused" } else { "running" };
    window.set_title(format!(
        "Game of Life SDF | {}x{} | gen={} | {:.1} Hz | {}",
        state.grid_size[0], state.grid_size[1], state.generation, state.steps_per_second, mode
    ));
}
