use engine::SystemSetKey;
use engine::plugins::render::UiFrameSubmissionRegistryResource;
use engine::prelude::*;
use engine::runtime::{CoreSet, IntoSystemSetKey, SystemConfigExt};

use crate::runtime::resources::{
    EditorHostResource, EditorInputBridgeState, EditorViewportRenderState,
};
use crate::runtime::systems::{
    bootstrap_editor_demo_system, dispatch_editor_input_system, produce_editor_picking_system,
    seed_viewport_runtime_contracts_system, submit_editor_frame_system,
    sync_viewport_presentation_products_system,
};
use crate::runtime::viewport::{
    ViewportArtifactObservationResource, ViewportLayoutMapResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportProductRegistryResource, ViewportSurfaceSetResource,
};

pub struct EditorAppPlugin;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EditorRuntimeSet {
    Picking,
    InputBridge,
    FrameSubmit,
    ViewportPresentationSync,
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
            Self::ViewportPresentationSync => SystemSetKey::of::<EditorRuntimeSet>(
                "EditorRuntimeSet::ViewportPresentationSync",
            ),
        }
    }
}

impl Plugin for EditorAppPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorHostResource>();
        app.init_resource::<EditorInputBridgeState>();
        app.init_resource::<EditorViewportRenderState>();
        app.init_resource::<ViewportProductRegistryResource>();
        app.init_resource::<ViewportPresentationStateResource>();
        app.init_resource::<ViewportArtifactObservationResource>();
        app.init_resource::<ViewportLayoutMapResource>();
        app.init_resource::<ViewportSurfaceSetResource>();
        app.init_resource::<ViewportPickingResultsResource>();
        app.init_resource::<UiFrameSubmissionRegistryResource>();

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
    }
}
