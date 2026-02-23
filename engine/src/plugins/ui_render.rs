use crate::runtime::{EnginePlugin, EngineScheduleBuilder};
use crate::systems::{ui_build_batches_system, ui_layout_system, ui_render_extract_system};
use anyhow::Result;

pub struct UiRenderPlugin;

impl EnginePlugin for UiRenderPlugin {
    fn name(&self) -> &'static str {
        "ui_render"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "overlay_ui_layout",
            ui_layout_system,
            &["overlay_ui_editor"],
        );
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
