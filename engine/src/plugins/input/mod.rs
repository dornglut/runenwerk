pub mod domain;

use crate::app::App;
use crate::plugin::Plugin;
use crate::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
use crate::runtime_v2::{CoreSet, RenderSubmit, ResMut, SystemConfigExt};
use anyhow::Result;
use domain::InputState;

pub struct InputFinalizePlugin;

impl Plugin for InputFinalizePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(RenderSubmit, typed_clear_input.in_set(CoreSet::FrameEnd));
    }
}

impl EnginePlugin for InputFinalizePlugin {
    fn name(&self) -> &'static str {
        "input_finalize"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges("clear_input", clear_input_system, &["frame_render_submit"]);
        Ok(())
    }
}

pub fn clear_input_system(data: &mut EngineData) -> anyhow::Result<()> {
    data.input.clear_frame();
    Ok(())
}

fn typed_clear_input(mut input: ResMut<InputState>) {
    input.clear_frame();
}
