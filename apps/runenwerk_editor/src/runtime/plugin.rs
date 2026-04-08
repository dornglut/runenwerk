use engine::plugins::render::UiFrameSubmissionRegistryResource;
use engine::prelude::*;
use engine::runtime::{CoreSet, SystemConfigExt};

use crate::runtime::resources::{EditorHostResource, EditorInputBridgeState};
use crate::runtime::systems::{
    bootstrap_editor_demo_system, dispatch_editor_input_system, submit_editor_frame_system,
};

pub struct EditorAppPlugin;

impl Plugin for EditorAppPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorHostResource>();
        app.init_resource::<EditorInputBridgeState>();
        app.init_resource::<UiFrameSubmissionRegistryResource>();

        app.add_systems(Startup, bootstrap_editor_demo_system);
        app.add_systems(
            Update,
            dispatch_editor_input_system
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
        app.add_systems(
            Update,
            submit_editor_frame_system
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
    }
}
