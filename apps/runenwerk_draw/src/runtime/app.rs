//! Runtime assembly for `runenwerk_draw`.

use anyhow::Result;
use engine::plugins::{
    RenderFlow, RenderPlugin, ScenePlugin, SchedulerDiagnosticsPlugin, default_plugins,
};
use engine::prelude::*;
use native_tablet_input::NativeTabletRuntimePlugin;

use crate::runtime::plugin::DrawingAppPlugin;

const WINDOW_TITLE: &str = "Runenwerk Draw";
const DRAW_MAIN_FLOW_ID: &str = "runenwerk.draw.main";
const DRAW_SURFACE_CLEAR_PASS_ID: &str = "runenwerk.draw.surface.clear";
const DRAW_MAIN_UI_PASS_ID: &str = "runenwerk.draw.main.ui";

fn configure_app(app: &mut App) {
    app.set_title(WINDOW_TITLE);
    app.add_plugins(default_plugins());
    app.add_plugin(SchedulerDiagnosticsPlugin);
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_plugin(NativeTabletRuntimePlugin);
    register_draw_render_flow(app);
    app.add_plugin(DrawingAppPlugin);
}

fn register_draw_render_flow(app: &mut App) {
    let flow = RenderFlow::new(DRAW_MAIN_FLOW_ID)
        .with_surface_color()
        .fullscreen_pass(DRAW_SURFACE_CLEAR_PASS_ID)
        .main_surface_only()
        .write_surface_color()
        .finish()
        .builtin_ui_composite_pass(DRAW_MAIN_UI_PASS_ID)
        .main_surface_only()
        .depends_on(DRAW_SURFACE_CLEAR_PASS_ID)
        .finish()
        .validate()
        .expect("drawing render flow should validate");
    app.add_render_flow(flow);
}

pub fn build_headless_app() -> App {
    let mut app = App::headless();
    configure_app(&mut app);
    app
}

pub fn build_app() -> App {
    let mut app = App::new();
    configure_app(&mut app);
    app
}

pub fn run() -> Result<()> {
    build_app().run()
}
