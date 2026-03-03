pub mod domain;

use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
use crate::runtime_v2::{CoreSet, ResMut, SystemConfigExt, Update};
use anyhow::Result;
use domain::Time;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, typed_time_system.in_set(CoreSet::Time));
    }
}

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

fn typed_time_system(mut time: ResMut<Time>) {
    time.tick();
}
