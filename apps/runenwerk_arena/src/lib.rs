pub mod arena;
pub mod command;
pub mod input;
pub mod player;
pub mod plugin;
pub mod presentation;

use engine::plugins::render::RenderPlugin;
use engine::plugins::{
    FixedStepPlugin, InputFinalizePlugin, SimulationPlugin, TimePlugin, WorldPlugin,
};
use engine::prelude::App;
use engine::prelude::{AppSimulationExt, AuthorityRole, SimulationProfile};

pub use arena::*;
pub use command::*;
pub use input::*;
pub use player::*;
pub use plugin::*;
pub use presentation::*;

pub fn build_game_app(headless: bool) -> App {
    let mut app = if headless {
        App::headless()
    } else {
        App::new()
    };
    app.add_plugins((
        TimePlugin,
        FixedStepPlugin,
        SimulationPlugin,
        InputFinalizePlugin,
        WorldPlugin,
        ArenaWorldPlugin,
        ArenaGamePlugin,
    ));
    if !headless {
        app.add_plugins((RenderPlugin, ArenaPresentationPlugin));
    }
    app.set_simulation_profile(SimulationProfile::LocalSinglePlayer);
    app.set_authority_role(AuthorityRole::Local);
    app
}

pub fn build_headless_game_app() -> App {
    build_game_app(true)
}
