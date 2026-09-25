pub mod command;
pub mod input;
pub mod player;
pub mod plugin;

use engine::prelude::App;
use engine::plugins::{FixedStepPlugin, InputFinalizePlugin, SimulationPlugin, TimePlugin};
use engine::prelude::{AppSimulationExt, AuthorityRole, SimulationProfile};

pub use command::*;
pub use input::*;
pub use player::*;
pub use plugin::*;

pub fn build_game_app(headless: bool) -> App {
    let mut app = if headless { App::headless() } else { App::new() };
    app.add_plugins((
        TimePlugin,
        FixedStepPlugin,
        SimulationPlugin,
        InputFinalizePlugin,
        ArenaGamePlugin,
    ));
    app.set_simulation_profile(SimulationProfile::LocalSinglePlayer);
    app.set_authority_role(AuthorityRole::Local);
    app
}

pub fn build_headless_game_app() -> App {
    build_game_app(true)
}
