use crate::rendering::{Sdf3dRenderState, build_render_flow};
use anyhow::Result;
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{App, CoreSet, InputState, Res, ResMut, Startup, Time, Update, WindowState};
use engine::SystemConfigExt;
use winit::keyboard::KeyCode;

const ACTION_CYCLE_VIEW_MODE: &str = "sdf.view.cycle";

pub(crate) fn run() -> Result<()> {
    let mut app = App::new();
    app.set_title("3D SDF Render Flow - Public API Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.insert_resource(Sdf3dRenderState::default());
    app.add_systems(Startup, setup_sdf_input_bindings);
    app.add_systems(
        Update,
        update_sdf_view_and_animation_system
            .after(CoreSet::Input)
            .after(CoreSet::Time),
    );
    app.add_render_flow(build_render_flow());
    app.run()
}

fn setup_sdf_input_bindings(mut input: ResMut<InputState>) {
    input.map_key(ACTION_CYCLE_VIEW_MODE, KeyCode::Tab);
}

fn update_sdf_view_and_animation_system(
    input: Res<InputState>,
    time: Res<Time>,
    mut state: ResMut<Sdf3dRenderState>,
    mut window: ResMut<WindowState>,
) {
    state.advance_by_frame_delta(time.delta_seconds);
    if input.action_pressed(ACTION_CYCLE_VIEW_MODE) {
        state.cycle_view_mode();
    }
    window.set_title(format!(
        "3D SDF Render Flow - Public API Example | View: {} (Tab)",
        state.view_mode_label()
    ));
}
