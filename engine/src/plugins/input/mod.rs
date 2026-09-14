mod actions_and_bindings;
mod neutral;
mod state;
#[cfg(test)]
mod tests;

pub mod domain;

use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{CoreSet, FrameEnd, ResMut, SystemConfigExt};

pub use actions_and_bindings::*;
pub use neutral::{
    AnalogMeasurement, ContactInput, ContactPhase, CoordinateSpace, DigitalState, InputContext,
    InputDeviceId, InputSourceId, KeyLocation, KeyboardInput, LogicalKey, MeasurementDomain,
    NativeLogicalKey, NativePhysicalKeyCode, ObservationOrigin, PhysicalKeyIdentity, Point2,
    PointerButton, PointerButtonInput, ScrollDelta, ScrollDomain, ScrollInput, ScrollPhase,
};
pub use state::*;

pub struct InputFinalizePlugin;

impl Plugin for InputFinalizePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FrameEnd, clear_input_system.in_set(CoreSet::FrameEnd));
    }
}

fn clear_input_system(mut input: ResMut<state::InputState>) {
    input.clear_frame();
}
