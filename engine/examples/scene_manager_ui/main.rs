use anyhow::Result;
use engine::App;
use engine::plugins::{
    DebugMetricsPlugin, GridPlugin, RenderFlow, RenderPlugin, ScenePlugin, default_plugins,
};

const MAIN_MENU_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/main_menu.ron";
const SETTINGS_MENU_SCENE: &str =
    "engine/examples/scene_manager_ui/assets/scenes/settings_menu.ron";
const PAUSE_MENU_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/pause_menu.ron";
const GAME_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/game_scene.ron";
const LOADING_SCENE: &str = "engine/examples/scene_manager_ui/assets/scenes/loading_scene.ron";

fn build_scene_manager_ui_render_flow() -> RenderFlow {
    RenderFlow::new("scene_manager_ui")
        .with_surface_color()
        .with_builtin_ui()
        .fullscreen_pass("scene.background")
        .write_surface_color()
        .finish()
        .builtin_ui_composite_pass("scene.ui")
        .depends_on("scene.background")
        .finish()
        .validate()
        .expect("scene_manager_ui flow should validate")
}

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
    app.add_render_flow(build_scene_manager_ui_render_flow());
    app.run()
}
