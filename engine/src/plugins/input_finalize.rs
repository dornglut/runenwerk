use crate::runtime::{EnginePlugin, EngineScheduleBuilder};
use crate::systems::clear_input_system;
use anyhow::Result;

pub struct InputFinalizePlugin;

impl EnginePlugin for InputFinalizePlugin {
    fn name(&self) -> &'static str {
        "input_finalize"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges("clear_input", clear_input_system, &["frame_render_submit"]);
        Ok(())
    }
}
