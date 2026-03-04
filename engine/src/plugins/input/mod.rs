pub mod domain;

use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CoreSet, FrameEnd, ResMut, SystemConfigExt};
use domain::InputState;

pub struct InputFinalizePlugin;

impl Plugin for InputFinalizePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FrameEnd, clear_input_system.in_set(CoreSet::FrameEnd));
    }
}

fn clear_input_system(mut input: ResMut<InputState>) {
    input.clear_frame();
}
