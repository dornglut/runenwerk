use anyhow::Result;
use engine::platform::App;

const MAIN_MENU_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/main_menu.ron";
const SETTINGS_MENU_SCENE: &str =
    "engine/examples/scene_manager_ui/assets/scenes/settings_menu.ron";
const PAUSE_MENU_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/pause_menu.ron";
const GAME_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/game_scene.ron";

fn main() -> Result<()> {
    App::new()
        .set_title("Grotto Quest - Scene Manager UI Example")
        .add_scene(MAIN_MENU_SCENE)
        .add_scene(SETTINGS_MENU_SCENE)
        .add_scene(PAUSE_MENU_SCENE)
        .add_scene(GAME_SCENE)
        .run()
}
