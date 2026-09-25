use super::{
    SceneCatalog, SceneOverlayViewportState, SceneResource, SceneRuntimeState,
    lifecycle::install_scene_runtime_systems,
};
use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::render::SurfaceFrameSubmissionRegistryResource;
#[derive(Debug, Default, runen_ecs::Resource)]
pub(crate) struct SceneIntegrationActivation;

pub(crate) fn scene_integration_is_active(world: &runen_ecs::World) -> bool {
    world.has_resource::<SceneIntegrationActivation>()
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneIntegrationActivation>();
        app.init_resource::<SceneCatalog>();
        app.init_resource::<SceneResource>();
        app.init_resource::<super::runtime::SceneTemplateFlowResource>();
        app.init_resource::<SceneRuntimeState>();
        app.init_resource::<SceneOverlayViewportState>();
        app.init_resource::<SurfaceFrameSubmissionRegistryResource>();
        install_scene_runtime_systems(app);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_selection_is_explicit_and_not_inferred_from_shared_resource_presence() {
        let mut app = App::headless();
        app.init_resource::<SceneResource>();

        assert!(!scene_integration_is_active(app.world()));

        app.add_plugin(ScenePlugin);

        assert!(scene_integration_is_active(app.world()));
    }
}
