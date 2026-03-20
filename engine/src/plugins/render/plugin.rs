use super::backend::{BackendPipelineCacheResource, BackendResourceAllocatorResource};
use super::composition::{RenderFlowRegistryResource, sync_render_flow_registry_system};
use super::inspect::{
    RenderDebugGraphDumpState, RenderDebugOverlayState, RenderDebugTimingsState,
    RenderRuntimeResourceInspectorState,
    RenderTextureInspectorState,
};
use super::renderer::submit::{frame_render_prepare_system, ui_render_submit_system};
use super::shader::ShaderRegistryResource;
use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::scene::SceneResource;
use crate::runtime::{RenderPrepare, RenderSubmit};
use crate::state::{DebugMetricsState, StartupState};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneResource>();
        app.init_resource::<ShaderRegistryResource>();
        app.init_resource::<RenderFlowRegistryResource>();
        app.init_resource::<BackendPipelineCacheResource>();
        app.init_resource::<BackendResourceAllocatorResource>();
        app.init_resource::<RenderDebugOverlayState>();
        app.init_resource::<RenderRuntimeResourceInspectorState>();
        app.init_resource::<RenderTextureInspectorState>();
        app.init_resource::<RenderDebugTimingsState>();
        app.init_resource::<RenderDebugGraphDumpState>();
        app.init_resource::<StartupState>();
        app.init_resource::<DebugMetricsState>();
        app.add_systems(RenderPrepare, sync_render_flow_registry_system);
        app.add_systems(RenderPrepare, frame_render_prepare_system);
        app.add_systems(RenderSubmit, ui_render_submit_system);
    }
}
