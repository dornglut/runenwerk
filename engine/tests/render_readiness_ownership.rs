use engine::plugins::DebugMetricsPlugin;
use engine::plugins::render::{RenderPlugin, RenderReadinessState};
use engine::prelude::App;

#[test]
fn bare_app_does_not_install_render_readiness_state() {
    let app = App::headless();

    assert!(app.world().resource::<RenderReadinessState>().is_err());
}

#[test]
fn render_plugin_installs_default_render_readiness_state() {
    let mut app = App::headless();
    app.add_plugin(RenderPlugin);

    let readiness = app.world().resource::<RenderReadinessState>().unwrap();
    assert!(readiness.is_loading());
}

#[test]
fn render_plugin_preserves_preinserted_render_readiness_state() {
    let mut app = App::headless();
    app.insert_resource(RenderReadinessState::ready());

    app.add_plugin(RenderPlugin);

    let readiness = app.world().resource::<RenderReadinessState>().unwrap();
    assert!(readiness.is_ready());
}

#[test]
fn debug_metrics_plugin_does_not_install_render_readiness_state() {
    let mut app = App::headless();
    app.add_plugin(DebugMetricsPlugin);

    assert!(app.world().resource::<RenderReadinessState>().is_err());
}
