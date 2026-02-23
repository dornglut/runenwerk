mod input_finalize;
mod render;
mod scene;
mod time;
mod ui_input;
mod ui_render;

pub use input_finalize::InputFinalizePlugin;
pub use render::RenderPlugin;
pub use scene::ScenePlugin;
pub use time::TimePlugin;
pub use ui_input::UiInputPlugin;
pub use ui_render::UiRenderPlugin;

use crate::runtime::EnginePlugin;

pub fn default_engine_plugins() -> Vec<Box<dyn EnginePlugin>> {
    vec![
        Box::new(TimePlugin),
        Box::new(UiInputPlugin),
        Box::new(ScenePlugin),
        Box::new(UiRenderPlugin),
        Box::new(RenderPlugin),
        Box::new(InputFinalizePlugin),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        InputFinalizePlugin, RenderPlugin, ScenePlugin, TimePlugin, UiInputPlugin, UiRenderPlugin,
        default_engine_plugins,
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
                "ui_render",
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
        assert!(build_with_plugin(UiRenderPlugin).is_err());
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
