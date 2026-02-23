pub mod domain;

use crate::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
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

pub fn time_system(data: &mut EngineData) -> anyhow::Result<()> {
    data.time.tick();
    Ok(())
}
