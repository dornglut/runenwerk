// Owner: Game of Life SDF Example - Public RenderFlow API Demo
use crate::rendering::build_render_flow;
use crate::runtime::GameOfLifeSdfState;
use anyhow::Result;
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};
use engine::prelude::{App, Plugin, ResMut, Startup};
use engine::plugins::render::RenderFrameResourceBindings;

pub(crate) fn run() -> Result<()> {
    let flow = build_render_flow();
    flow.validate()?;

    let mut app = App::new();
    app.set_title("Game of Life SDF - Public RenderFlow API Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_render_flow(flow);
    app.add_plugin(GameOfLifeSdfExamplePlugin);
    app.run()
}

struct GameOfLifeSdfExamplePlugin;

impl Plugin for GameOfLifeSdfExamplePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameOfLifeSdfState>();
        app.add_systems(Startup, game_of_life_setup_system);
    }
}

fn game_of_life_setup_system(mut frame_bindings: ResMut<RenderFrameResourceBindings>) {
    if !frame_bindings.contains_resource::<GameOfLifeSdfState>() {
        frame_bindings.register_resource::<GameOfLifeSdfState>();
    }
}
