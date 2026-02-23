use crate::runtime::{EnginePlugin, EngineScheduleBuilder};
use crate::systems::{ui_editor_system, ui_hot_reload_system, ui_input_system};
use anyhow::Result;

pub struct UiInputPlugin;

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
