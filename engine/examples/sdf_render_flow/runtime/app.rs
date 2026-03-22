use crate::rendering::{Sdf3dRenderState, build_render_flow};
use anyhow::Result;
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{App, Res, ResMut, Time, Update};

pub(crate) fn run() -> Result<()> {
    let mut app = App::new();
    app.set_title("3D SDF Render Flow - Public API Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.insert_resource(Sdf3dRenderState::default());
    app.add_systems(Update, advance_sdf_animation_system);
    app.add_render_flow(build_render_flow());
    app.run()
}

fn advance_sdf_animation_system(time: Res<Time>, mut state: ResMut<Sdf3dRenderState>) {
    state.advance_by_frame_delta(time.delta_seconds);
}
