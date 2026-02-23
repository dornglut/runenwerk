use crate::runtime::{EnginePlugin, EngineScheduleBuilder};
use crate::systems::time_system;
use anyhow::Result;

pub struct TimePlugin;

impl EnginePlugin for TimePlugin {
    fn name(&self) -> &'static str {
        "time"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node("time", time_system);
        Ok(())
    }
}
