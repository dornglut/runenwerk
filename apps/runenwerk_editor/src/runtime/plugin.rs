use engine::SystemSetKey;
use engine::plugins::render::{
    RenderDynamicTextureTargetRequestRegistryResource, UiFrameSubmissionRegistryResource,
};
use engine::prelude::*;
use engine::runtime::{CoreSet, IntoSystemSetKey, SystemConfigExt};

use crate::runtime::resources::{
    EditorHostResource, EditorInputBridgeState, EditorViewportRenderState,
};
use crate::runtime::systems::{
    bootstrap_editor_demo_system, dispatch_editor_input_system, produce_editor_picking_system,
    seed_viewport_runtime_contracts_system, submit_editor_frame_system,
};
use crate::runtime::viewport::{
    MountedSurfaceRegistryResource, SurfaceDefinitionRegistryResource,
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportLayoutMapResource, ViewportPickingResultsResource, ViewportPresentationStateResource,
    ViewportProductRegistryResource, ViewportProductTargetRegistryResource,
    ViewportRenderJobResource, ViewportRenderStateResource, ViewportSurfaceSetResource,
    sync_viewport_presentation_products_system, sync_viewport_product_targets_system,
    sync_viewport_render_jobs_system,
};

pub struct EditorAppPlugin;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EditorRuntimeSet {
    Picking,
    InputBridge,
    FrameSubmit,
    ViewportPresentationSync,
    ViewportProductTargets,
    ViewportRenderJobs,
}

impl IntoSystemSetKey for EditorRuntimeSet {
    fn system_set_key(&self) -> SystemSetKey {
        match self {
            Self::Picking => SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::Picking"),
            Self::InputBridge => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::InputBridge")
            }
            Self::FrameSubmit => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::FrameSubmit")
            }
            Self::ViewportPresentationSync => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::ViewportPresentationSync")
            }
            Self::ViewportProductTargets => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::ViewportProductTargets")
            }
            Self::ViewportRenderJobs => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::ViewportRenderJobs")
            }
        }
    }
}

impl Plugin for EditorAppPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorHostResource>();
        app.init_resource::<EditorInputBridgeState>();
        app.init_resource::<EditorViewportRenderState>();
        app.init_resource::<ViewportProductRegistryResource>();
        app.init_resource::<ViewportProductTargetRegistryResource>();
        app.init_resource::<ViewportPresentationStateResource>();
        app.init_resource::<ViewportArtifactObservationResource>();
        app.init_resource::<ViewportRenderJobResource>();
        app.init_resource::<ViewportRenderStateResource>();
        app.init_resource::<ViewportLayoutMapResource>();
        app.init_resource::<ToolSurfaceRuntimeBindingRegistryResource>();
        app.init_resource::<SurfaceDefinitionRegistryResource>();
        app.init_resource::<MountedSurfaceRegistryResource>();
        app.init_resource::<ViewportSurfaceSetResource>();
        app.init_resource::<ViewportPickingResultsResource>();
        app.init_resource::<UiFrameSubmissionRegistryResource>();
        app.init_resource::<RenderDynamicTextureTargetRequestRegistryResource>();

        app.add_systems(Startup, bootstrap_editor_demo_system);
        app.add_systems(Startup, seed_viewport_runtime_contracts_system);
        app.add_systems(
            Update,
            produce_editor_picking_system
                .in_set(EditorRuntimeSet::Picking)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
        app.add_systems(
            Update,
            dispatch_editor_input_system
                .in_set(EditorRuntimeSet::InputBridge)
                .after(EditorRuntimeSet::Picking)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
        app.add_systems(
            Update,
            submit_editor_frame_system
                .in_set(EditorRuntimeSet::FrameSubmit)
                .after(EditorRuntimeSet::InputBridge)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
        app.add_systems(
            Update,
            sync_viewport_presentation_products_system
                .in_set(EditorRuntimeSet::ViewportPresentationSync)
                .after(EditorRuntimeSet::FrameSubmit)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
        app.add_systems(
            Update,
            sync_viewport_product_targets_system
                .in_set(EditorRuntimeSet::ViewportProductTargets)
                .after(EditorRuntimeSet::ViewportPresentationSync)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
        app.add_systems(
            Update,
            sync_viewport_render_jobs_system
                .in_set(EditorRuntimeSet::ViewportRenderJobs)
                .after(EditorRuntimeSet::ViewportProductTargets)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
    }
}
