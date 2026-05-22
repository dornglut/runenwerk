use super::backend::{BackendResourceAllocatorResource, RenderSurfaceRegistryResource};
use super::composition::{
    RenderFlowRegistryResource, RenderFragmentRegistryResource, sync_render_flow_registry_system,
};
use super::features::{
    EditorPickingResultResource, PreparedCaveFeatureResource, PreparedDeformationFeatureResource,
    PreparedDetailFeatureResource, PreparedDrawFeatureResource, PreparedMaterialFeatureResource,
    PreparedProceduralWorldFeatureResource, PreparedUiFrameResource,
    PreparedWindFieldFeatureResource, PreparedWorldFeatureResource, RenderFeatureRegistryResource,
    UiFontAtlasResource, UiFrameSubmissionRegistryResource, ViewportSurfaceBindingRegistryResource,
    prepare_ui_feature_resource_system, sync_render_feature_registry_system,
    world::{WorldLodPolicyResource, WorldLodSelectionResource, WorldRuntimeCacheResource},
};
use super::frame::{
    PreparedRenderFrameRequestResource, PreparedRenderFrameResource,
    PreparedRenderProductSelectionResource,
};
use super::inspect::{
    RenderCapturedTextureState, RenderDebugConfigResource, RenderDebugControlResource,
    RenderDebugFrameReportState, RenderDebugGraphDumpState, RenderDebugOverlayState,
    RenderDebugTimingsState, RenderFrameDiagnosticsPolicyResource, RenderPassProvenanceState,
    RenderRuntimeResourceInspectorState, RenderTextureInspectorState,
    WorldRuntimeInspectorSnapshot,
};
use super::pipelines::PipelineCacheResource;
use super::residency::{
    RenderGpuResidencyBudgetResource, RenderGpuResidencyResource,
    derive_render_gpu_residency_system,
};
use super::runtime::{
    RenderDynamicTextureTargetRequestRegistryResource, RenderDynamicTextureUploadRegistryResource,
    RenderRuntimeSet, collect_runtime_ui_frame_submissions_system, frame_render_prepare_system,
    frame_render_submit_system,
};
use super::shader::ShaderRegistryResource;
use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::scene::SceneResource;
use crate::runtime::{RenderPrepare, RenderSubmit, SystemConfigExt};
use crate::state::{DebugMetricsState, StartupState};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneResource>();
        app.init_resource::<ShaderRegistryResource>();
        app.init_resource::<RenderFlowRegistryResource>();
        app.init_resource::<RenderFragmentRegistryResource>();
        app.init_resource::<RenderFeatureRegistryResource>();
        app.init_resource::<PreparedUiFrameResource>();
        app.init_resource::<ViewportSurfaceBindingRegistryResource>();
        app.init_resource::<UiFontAtlasResource>();
        app.init_resource::<UiFrameSubmissionRegistryResource>();
        app.init_resource::<EditorPickingResultResource>();
        app.init_resource::<PreparedDrawFeatureResource>();
        app.init_resource::<PreparedWorldFeatureResource>();
        app.init_resource::<PreparedCaveFeatureResource>();
        app.init_resource::<PreparedDetailFeatureResource>();
        app.init_resource::<PreparedProceduralWorldFeatureResource>();
        app.init_resource::<PreparedMaterialFeatureResource>();
        app.init_resource::<PreparedDeformationFeatureResource>();
        app.init_resource::<PreparedWindFieldFeatureResource>();
        app.init_resource::<WorldRuntimeCacheResource>();
        app.init_resource::<WorldLodPolicyResource>();
        app.init_resource::<WorldLodSelectionResource>();
        app.init_resource::<PreparedRenderFrameResource>();
        app.init_resource::<PreparedRenderFrameRequestResource>();
        app.init_resource::<PreparedRenderProductSelectionResource>();
        app.init_resource::<RenderGpuResidencyResource>();
        app.init_resource::<RenderGpuResidencyBudgetResource>();
        app.init_resource::<RenderDynamicTextureTargetRequestRegistryResource>();
        app.init_resource::<RenderDynamicTextureUploadRegistryResource>();
        app.init_resource::<PipelineCacheResource>();
        app.init_resource::<BackendResourceAllocatorResource>();
        app.init_resource::<RenderSurfaceRegistryResource>();
        app.init_resource::<RenderDebugOverlayState>();
        app.init_resource::<RenderRuntimeResourceInspectorState>();
        app.init_resource::<RenderTextureInspectorState>();
        app.init_resource::<RenderDebugTimingsState>();
        app.init_resource::<RenderDebugGraphDumpState>();
        app.init_resource::<RenderDebugControlResource>();
        app.init_resource::<RenderDebugConfigResource>();
        app.init_resource::<RenderFrameDiagnosticsPolicyResource>();
        app.init_resource::<RenderCapturedTextureState>();
        app.init_resource::<RenderPassProvenanceState>();
        app.init_resource::<RenderDebugFrameReportState>();
        app.init_resource::<WorldRuntimeInspectorSnapshot>();
        app.init_resource::<StartupState>();
        app.init_resource::<DebugMetricsState>();

        app.add_systems(RenderPrepare, sync_render_flow_registry_system);
        app.add_systems(RenderPrepare, sync_render_feature_registry_system);
        app.add_systems(RenderPrepare, collect_runtime_ui_frame_submissions_system);
        app.add_systems(RenderPrepare, prepare_ui_feature_resource_system);
        app.add_systems(
            RenderPrepare,
            derive_render_gpu_residency_system
                .in_set(RenderRuntimeSet::GpuResidency)
                .before(RenderRuntimeSet::FramePrepare),
        );
        app.add_systems(
            RenderPrepare,
            frame_render_prepare_system.in_set(RenderRuntimeSet::FramePrepare),
        );
        app.add_systems(RenderSubmit, frame_render_submit_system);
    }
}
