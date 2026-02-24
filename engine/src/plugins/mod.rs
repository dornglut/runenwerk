pub mod debug_metrics;
pub mod grid;
pub mod input;
pub mod render;
pub mod scene;
pub mod scheduler_diagnostics;
pub(crate) mod shared;
pub mod time;
pub mod ui;

pub use debug_metrics::DebugMetricsPlugin;
pub use grid::GridPlugin;
pub use input::InputFinalizePlugin;
pub use render::RenderPlugin;
pub use scene::ScenePlugin;
pub use scheduler_diagnostics::SchedulerDiagnosticsPlugin;
pub use time::TimePlugin;
pub use ui::UiInputPlugin;
pub use ui::UiRenderPlugin;

use crate::runtime::EnginePlugin;

pub fn default_engine_plugins() -> Vec<Box<dyn EnginePlugin>> {
    vec![
        Box::new(TimePlugin),
        Box::new(UiInputPlugin),
        Box::new(ScenePlugin),
        Box::new(GridPlugin),
        Box::new(UiRenderPlugin),
        Box::new(DebugMetricsPlugin),
        Box::new(RenderPlugin),
        Box::new(InputFinalizePlugin),
    ]
}

pub fn default_engine_plugins_with_diagnostics() -> Vec<Box<dyn EnginePlugin>> {
    let mut plugins = default_engine_plugins();
    plugins.push(Box::new(SchedulerDiagnosticsPlugin));
    plugins
}

#[cfg(test)]
mod tests {
    use super::{
        DebugMetricsPlugin, GridPlugin, InputFinalizePlugin, RenderPlugin, ScenePlugin, TimePlugin,
        UiInputPlugin, UiRenderPlugin, default_engine_plugins,
    };
    use crate::runtime::{EnginePlugin, EngineScheduleBuilder};

    #[test]
    fn default_plugins_have_stable_order() {
        let plugins = default_engine_plugins();
        let names: Vec<_> = plugins.iter().map(|plugin| plugin.name()).collect();
        assert_eq!(
            names,
            vec![
                "time",
                "ui_input",
                "scene",
                "grid",
                "ui_render",
                "debug_metrics",
                "render",
                "input_finalize"
            ]
        );
    }

    #[test]
    fn default_plugins_build_scheduler_successfully() {
        let plugins = default_engine_plugins();
        let mut builder = EngineScheduleBuilder::new();
        for plugin in &plugins {
            plugin
                .configure(&mut builder)
                .expect("plugin configure should succeed");
        }
        assert!(builder.build_scheduler().is_ok());
    }

    #[test]
    fn dependent_plugins_fail_without_prerequisites() {
        assert!(build_with_plugin(UiInputPlugin).is_err());
        assert!(build_with_plugin(ScenePlugin).is_err());
        assert!(build_with_plugin(GridPlugin).is_err());
        assert!(build_with_plugin(UiRenderPlugin).is_err());
        assert!(build_with_plugin(DebugMetricsPlugin).is_err());
        assert!(build_with_plugin(RenderPlugin).is_err());
        assert!(build_with_plugin(InputFinalizePlugin).is_err());
        assert!(build_with_plugin(TimePlugin).is_ok());
    }

    fn build_with_plugin(plugin: impl EnginePlugin) -> anyhow::Result<()> {
        let mut builder = EngineScheduleBuilder::new();
        plugin.configure(&mut builder)?;
        builder.build_scheduler().map(|_| ())
    }
}
