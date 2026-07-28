//! Runtime assembly for `runenwerk_draw`.

use anyhow::Result;
use engine::plugins::{
    RenderFlow, RenderPlugin, ScenePlugin, SchedulerDiagnosticsPlugin, default_plugins,
};
use engine::prelude::*;
use native_tablet_input::NativeTabletRuntimePlugin;

use crate::runtime::gpu_ink::register_drawing_ink_gpu_flow;
use crate::runtime::plugin::DrawingAppPlugin;

const WINDOW_TITLE: &str = "Runenwerk Draw";
const DRAW_MAIN_FLOW_ID: &str = "runenwerk.draw.main";
const DRAW_SURFACE_CLEAR_PASS_ID: &str = "runenwerk.draw.surface.clear";
const DRAW_MAIN_UI_PASS_ID: &str = "runenwerk.draw.main.ui";

fn configure_app(app: &mut App) -> Result<()> {
    app.set_title(WINDOW_TITLE);
    app.add_plugins(default_plugins());
    app.add_plugin(SchedulerDiagnosticsPlugin);
    app.add_plugin(ScenePlugin);
    app.add_plugin(RenderPlugin);
    app.add_plugin(NativeTabletRuntimePlugin);
    register_draw_render_flow(app)?;
    register_drawing_ink_gpu_flow(app)?;
    app.add_plugin(DrawingAppPlugin);
    Ok(())
}

fn register_draw_render_flow(app: &mut App) -> Result<()> {
    let flow = RenderFlow::new(DRAW_MAIN_FLOW_ID)
        .with_surface_color()?
        .fullscreen_pass(DRAW_SURFACE_CLEAR_PASS_ID)
        .main_surface_only()
        .write_surface_color()?
        .finish()
        .builtin_ui_composite_pass(DRAW_MAIN_UI_PASS_ID)?
        .main_surface_only()
        .finish()
        .validate()
        .expect("drawing render flow should validate");
    app.add_render_flow(flow);
    Ok(())
}

pub fn build_headless_app() -> Result<App> {
    let mut app = App::headless();
    configure_app(&mut app)?;
    Ok(app)
}

pub fn build_app() -> Result<App> {
    let mut app = App::new();
    configure_app(&mut app)?;
    Ok(app)
}

pub fn run() -> Result<()> {
    build_app()?.run()
}
