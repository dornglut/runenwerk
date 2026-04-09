use super::{SceneResource, lifecycle::install_scene_runtime_systems};
use crate::app::App;
use crate::plugin::Plugin;
use crate::state::{GameplayRuntimeConfig, SceneRuntimeState, UiOverlayState};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneResource>();
        app.init_resource::<super::runtime::SceneTemplateFlowResource>();
        app.init_resource::<SceneRuntimeState>();
        app.init_resource::<GameplayRuntimeConfig>();
        app.init_resource::<UiOverlayState>();
        install_scene_runtime_systems(app);
    }
}
