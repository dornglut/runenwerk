use anyhow::Result;
use engine::App;
use engine::plugins::{
    DebugMetricsPlugin, GridPlugin, RenderPlugin, ScenePlugin, default_plugins,
};

const MAIN_MENU_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/main_menu.ron";
const SETTINGS_MENU_SCENE: &str =
    "engine/examples/scene_manager_ui/assets/scenes/settings_menu.ron";
const PAUSE_MENU_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/pause_menu.ron";
const GAME_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/game_scene.ron";
const LOADING_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/loading_scene.ron";

fn main() -> Result<()> {
    let mut app = App::new();
    app.set_title("Grotto Quest - Scene Manager UI Example");
    app.add_plugins(default_plugins());
    app.add_plugin(ScenePlugin);
    app.add_plugin(GridPlugin);
    app.add_plugin(DebugMetricsPlugin);
    app.add_plugin(RenderPlugin);
    app.add_scene(LOADING_SCENE);
    app.add_scene(MAIN_MENU_SCENE);
    app.add_scene(SETTINGS_MENU_SCENE);
    app.add_scene(PAUSE_MENU_SCENE);
    app.add_scene(GAME_SCENE);
    app.run()
}
