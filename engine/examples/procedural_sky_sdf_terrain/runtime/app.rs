use crate::rendering::{ProceduralSkyTerrainState, build_render_flow};
use anyhow::Result;
use engine::SystemConfigExt;
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{App, CoreSet, InputState, Res, ResMut, Startup, Time, Update, WindowState};
use winit::keyboard::KeyCode;

const ACTION_CYCLE_VIEW_MODE: &str = "terrain.view.cycle";

pub(crate) fn run() -> Result<()> {
    let mut app = App::new();
    app.set_title("Procedural Sky + SDF Terrain - Public API Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.insert_resource(ProceduralSkyTerrainState::default());
    app.add_systems(Startup, setup_terrain_input_bindings);
    app.add_systems(
        Update,
        update_terrain_view_and_animation_system
            .after(CoreSet::Input)
            .after(CoreSet::Time),
    );
    app.add_render_flow(build_render_flow());
    app.run()
}

fn setup_terrain_input_bindings(mut input: ResMut<InputState>) {
    input.map_key(ACTION_CYCLE_VIEW_MODE, KeyCode::Tab);
}

fn update_terrain_view_and_animation_system(
    input: Res<InputState>,
    time: Res<Time>,
    mut state: ResMut<ProceduralSkyTerrainState>,
    mut window: ResMut<WindowState>,
) {
    state.advance_by_frame_delta(time.delta_seconds);
    if input.action_pressed(ACTION_CYCLE_VIEW_MODE) {
        state.cycle_view_mode();
    }
    window.set_title(format!(
        "Procedural Sky + SDF Terrain - Public API Example | View: {} (Tab)",
        state.view_mode_label()
    ));
}
