use crate::runtime::{EnginePlugin, EngineScheduleBuilder};
use crate::systems::ui_render_submit_system;
use anyhow::Result;

pub struct RenderPlugin;

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
