use crate::rendering::{GameOfLifeRenderState, build_render_flow};
use anyhow::Result;
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{App, Res, ResMut, Time, Update};

pub(crate) fn run() -> Result<()> {
    let mut app = App::new();
    app.set_title("Game of Life SDF - Public RenderFlow API Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.insert_resource(GameOfLifeRenderState::default());
    app.add_systems(Update, game_of_life_update_system);
    app.add_render_flow(build_render_flow());
    app.run()
}

fn game_of_life_update_system(time: Res<Time>, mut state: ResMut<GameOfLifeRenderState>) {
    state.advance_by_frame_delta(time.delta_seconds);
}
