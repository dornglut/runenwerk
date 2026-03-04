pub mod domain;

use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CoreSet, PreUpdate, ResMut, SystemConfigExt};
use domain::Time;

pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, time_system.in_set(CoreSet::Time));
    }
}

fn time_system(mut time: ResMut<Time>) {
    time.tick();
}
