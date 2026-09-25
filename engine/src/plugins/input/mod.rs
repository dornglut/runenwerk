mod actions_and_bindings;
mod state;
#[cfg(test)]
mod tests;

pub mod domain;

use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CoreSet, FrameEnd, PreUpdate, Res, ResMut, SystemConfigExt};

pub use actions_and_bindings::*;
pub use state::*;

pub struct InputFinalizePlugin;

impl Plugin for InputFinalizePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<state::InputState>();
        app.init_resource::<ActionState>();
        app.add_systems(PreUpdate, project_actions_system.in_set(CoreSet::Input));
        app.add_systems(FrameEnd, clear_input_system.in_set(CoreSet::FrameEnd));
    }
}

fn project_actions_system(input: Res<state::InputState>, mut actions: ResMut<ActionState>) {
    actions.project(&input);
}

fn clear_input_system(mut input: ResMut<state::InputState>, mut actions: ResMut<ActionState>) {
    actions.clear_frame(&input);
    input.clear_frame();
}
