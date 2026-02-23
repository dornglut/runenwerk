use anyhow::Result;
use engine::platform::App;
use game::plugins::{GameCommandPlugin, GameSimulationPlugin};

fn main() -> Result<()> {
    App::new()
        .set_title("Grotto Quest - Game")
        .add_plugin(GameSimulationPlugin)
        .add_plugin(GameCommandPlugin)
        .run()
}
