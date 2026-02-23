use anyhow::Result;
use engine::runtime::{EnginePlugin, EngineScheduleBuilder};
use engine::systems::{
    clear_input_system, time_system, ui_build_batches_system, ui_editor_system,
    ui_hot_reload_system, ui_input_system, ui_layout_system, ui_render_extract_system,
    ui_render_submit_system,
};

pub struct TimePlugin;
pub struct UiInputPlugin;
pub struct UiRenderPlugin;
pub struct RenderPlugin;
pub struct InputFinalizePlugin;

impl EnginePlugin for TimePlugin {
    fn name(&self) -> &'static str {
        "time"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node("time", time_system);
        Ok(())
    }
}

impl EnginePlugin for UiInputPlugin {
    fn name(&self) -> &'static str {
        "ui_input"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges("overlay_ui_hot_reload", ui_hot_reload_system, &["time"]);
        builder.add_node_with_edges(
            "overlay_ui_input",
            ui_input_system,
            &["overlay_ui_hot_reload"],
        );
        builder.add_node_with_edges("overlay_ui_editor", ui_editor_system, &["overlay_ui_input"]);
        Ok(())
    }
}

impl EnginePlugin for UiRenderPlugin {
    fn name(&self) -> &'static str {
        "ui_render"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges("overlay_ui_layout", ui_layout_system, &["overlay_ui_editor"]);
        builder.add_node_with_edges(
            "overlay_ui_build_batches",
            ui_build_batches_system,
            &["overlay_ui_layout"],
        );
        builder.add_node_with_edges(
            "overlay_ui_render_extract",
            ui_render_extract_system,
            &["overlay_ui_build_batches"],
        );
        Ok(())
    }
}

impl EnginePlugin for RenderPlugin {
    fn name(&self) -> &'static str {
        "render"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "frame_render_submit",
            ui_render_submit_system,
            &["overlay_ui_render_extract"],
        );
        Ok(())
    }
}

impl EnginePlugin for InputFinalizePlugin {
    fn name(&self) -> &'static str {
        "input_finalize"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges("clear_input", clear_input_system, &["frame_render_submit"]);
        Ok(())
    }
}

pub fn default_engine_plugins() -> Vec<Box<dyn EnginePlugin>> {
    vec![
        Box::new(TimePlugin),
        Box::new(UiInputPlugin),
        Box::new(UiRenderPlugin),
        Box::new(RenderPlugin),
        Box::new(InputFinalizePlugin),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        InputFinalizePlugin, RenderPlugin, TimePlugin, UiInputPlugin, UiRenderPlugin,
        default_engine_plugins,
    };
    use engine::runtime::{EnginePlugin, EngineScheduleBuilder};

    #[test]
    fn default_plugins_have_stable_order() {
        let plugins = default_engine_plugins();
        let names: Vec<_> = plugins.iter().map(|plugin| plugin.name()).collect();
        assert_eq!(
            names,
            vec![
                "time",
                "ui_input",
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
