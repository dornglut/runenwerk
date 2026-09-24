use super::{SceneResource, lifecycle::install_scene_runtime_systems};
use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::render::SurfaceFrameSubmissionRegistryResource;
use crate::state::{SceneCatalog, SceneOverlayViewportState, SceneRuntimeState};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneCatalog>();
        app.init_resource::<SceneResource>();
        app.init_resource::<super::runtime::SceneTemplateFlowResource>();
        app.init_resource::<SceneRuntimeState>();
        app.init_resource::<SceneOverlayViewportState>();
        app.init_resource::<SurfaceFrameSubmissionRegistryResource>();
        install_scene_runtime_systems(app);
    }
}
