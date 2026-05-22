use crate::rendering::{BoidsRenderState, build_render_flow};
use anyhow::Result;
use engine::plugins::{DebugMetricsPlugin, RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{App, Res, ResMut, Time, Update};

pub(crate) fn run() -> Result<()> {
    let mut app = App::new();
    app.set_title("Boids Render Flow - Public API Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_plugin(DebugMetricsPlugin);
    app.insert_resource(BoidsRenderState::default());
    app.add_systems(Update, advance_boids_simulation_system);
    app.add_render_flow(build_render_flow());
    app.run()
}

fn advance_boids_simulation_system(time: Res<Time>, mut state: ResMut<BoidsRenderState>) {
    state.advance_by_frame_delta(time.delta_seconds);
}
