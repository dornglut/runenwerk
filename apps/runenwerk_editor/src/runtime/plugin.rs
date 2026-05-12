use ecs::World;
use engine::plugins::render::{
    PreparedRenderProductSelectionResource, RenderDynamicTextureTargetRequestRegistryResource,
    RenderGpuResidencyBudgetResource, RenderGpuResidencyResource, RenderRuntimeSet,
    UiFrameSubmissionRegistryResource,
};
use engine::prelude::*;
use engine::runtime::ProductPublicationRuntimeResource;
use engine::runtime::{CoreSet, IntoSystemSetKey, SystemConfigExt};
use engine::{BarrierKind, ExecutionBarrier, SystemSetKey};

use crate::asset_pipeline::publish_pending_field_product_publications;
use crate::runtime::resources::{
    EditorHostResource, EditorInputBridgeState, EditorViewportRenderState,
    RuntimePreviewProcessResource,
};
use crate::runtime::systems::{
    bootstrap_editor_demo_system, dispatch_editor_input_system, produce_editor_picking_system,
    seed_viewport_runtime_contracts_system, submit_editor_frame_system,
    sync_viewport_instances_system,
};
use crate::runtime::viewport::{
    MountedSurfaceRegistryResource, SurfaceDefinitionRegistryResource,
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource, ViewportLayoutMapResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportProductRegistryResource,
    ViewportProductTargetRegistryResource, ViewportRenderJobResource,
    ViewportRenderStateCommandQueueResource, ViewportRenderStateResource,
    ViewportSurfaceSetResource, apply_viewport_render_state_commands_system,
    prepare_viewport_render_product_selections_system, publish_viewport_query_snapshots_at_barrier,
    summarize_viewport_gpu_residency_system, sync_viewport_presentation_products_system,
    sync_viewport_product_targets_system, sync_viewport_render_jobs_system,
};

pub struct EditorAppPlugin;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EditorRuntimeSet {
    Picking,
    InputBridge,
    ViewportLifecycle,
    FrameSubmit,
    ViewportRenderStateCommands,
    ViewportPresentationSync,
    ViewportProductTargets,
    ViewportRenderJobs,
    ViewportRenderProductSelection,
    ViewportGpuResidencySummary,
}

impl IntoSystemSetKey for EditorRuntimeSet {
    fn system_set_key(&self) -> SystemSetKey {
        match self {
            Self::Picking => SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::Picking"),
            Self::InputBridge => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::InputBridge")
            }
            Self::ViewportLifecycle => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::ViewportLifecycle")
            }
            Self::FrameSubmit => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::FrameSubmit")
            }
            Self::ViewportRenderStateCommands => SystemSetKey::of::<EditorRuntimeSet>(
                "EditorRuntimeSet::ViewportRenderStateCommands",
            ),
            Self::ViewportPresentationSync => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::ViewportPresentationSync")
            }
            Self::ViewportProductTargets => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::ViewportProductTargets")
            }
            Self::ViewportRenderJobs => {
                SystemSetKey::of::<EditorRuntimeSet>("EditorRuntimeSet::ViewportRenderJobs")
            }
            Self::ViewportRenderProductSelection => SystemSetKey::of::<EditorRuntimeSet>(
                "EditorRuntimeSet::ViewportRenderProductSelection",
            ),
            Self::ViewportGpuResidencySummary => SystemSetKey::of::<EditorRuntimeSet>(
                "EditorRuntimeSet::ViewportGpuResidencySummary",
            ),
        }
    }
}

impl Plugin for EditorAppPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorHostResource>();
        app.init_resource::<RuntimePreviewProcessResource>();
        app.init_resource::<EditorInputBridgeState>();
        app.init_resource::<EditorViewportRenderState>();
        app.init_resource::<ViewportProductRegistryResource>();
        app.init_resource::<ViewportProductTargetRegistryResource>();
        app.init_resource::<ViewportPresentationStateResource>();
        app.init_resource::<ViewportArtifactObservationResource>();
        app.init_resource::<ViewportRenderJobResource>();
        app.init_resource::<ViewportRenderStateResource>();
        app.init_resource::<ViewportRenderStateCommandQueueResource>();
        app.init_resource::<ViewportInstanceRegistryResource>();
        app.init_resource::<ViewportLayoutMapResource>();
        app.init_resource::<ToolSurfaceRuntimeBindingRegistryResource>();
        app.init_resource::<SurfaceDefinitionRegistryResource>();
        app.init_resource::<MountedSurfaceRegistryResource>();
        app.init_resource::<ViewportSurfaceSetResource>();
        app.init_resource::<ViewportPickingResultsResource>();
        app.init_resource::<UiFrameSubmissionRegistryResource>();
        app.init_resource::<RenderDynamicTextureTargetRequestRegistryResource>();
        app.init_resource::<PreparedRenderProductSelectionResource>();
        app.init_resource::<RenderGpuResidencyResource>();
        app.init_resource::<RenderGpuResidencyBudgetResource>();
        app.add_barrier_handler(
            BarrierKind::ProductPublication,
            publish_editor_field_products_at_barrier,
        );
        app.add_barrier_handler(
            BarrierKind::QuerySnapshotPublication,
            publish_viewport_query_snapshots_at_barrier,
        );

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
            apply_viewport_render_state_commands_system
                .in_set(EditorRuntimeSet::ViewportRenderStateCommands)
                .after(EditorRuntimeSet::InputBridge)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
        app.add_systems(
            Update,
            sync_viewport_instances_system
                .in_set(EditorRuntimeSet::ViewportLifecycle)
                .after(EditorRuntimeSet::ViewportRenderStateCommands)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
        app.add_systems(
            Update,
            submit_editor_frame_system
                .in_set(EditorRuntimeSet::FrameSubmit)
                .after(EditorRuntimeSet::ViewportLifecycle)
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
        app.add_systems(
            RenderPrepare,
            prepare_viewport_render_product_selections_system
                .in_set(EditorRuntimeSet::ViewportRenderProductSelection)
                .before(RenderRuntimeSet::GpuResidency),
        );
        app.add_systems(
            RenderPrepare,
            summarize_viewport_gpu_residency_system
                .in_set(EditorRuntimeSet::ViewportGpuResidencySummary)
                .after(RenderRuntimeSet::GpuResidency)
                .before(RenderRuntimeSet::FramePrepare),
        );
    }
}

fn publish_editor_field_products_at_barrier(
    barrier: &ExecutionBarrier,
    world: &mut World,
) -> anyhow::Result<()> {
    let Some(mut host) = world.remove_resource::<EditorHostResource>() else {
        return Ok(());
    };
    let Some(mut publications) = world.remove_resource::<ProductPublicationRuntimeResource>()
    else {
        world.insert_resource(host);
        return Ok(());
    };

    publish_pending_field_product_publications(&mut host.app, &mut publications, barrier);

    world.insert_resource(publications);
    world.insert_resource(host);
    Ok(())
}
