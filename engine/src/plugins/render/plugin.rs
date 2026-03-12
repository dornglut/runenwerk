use super::debug::{
    RenderDebugGraphDumpState, RenderDebugOverlayState, RenderDebugTimingsState,
    RenderTextureInspectorState,
};
use super::domain::{
    BuiltinRenderPassExecutor, RenderFrameResourceBindings, RenderGraphRegistryResource,
    RenderPassExecutorRegistryResource, ShaderRegistryResource,
};
use super::pipelines::PipelineCacheResource;
use super::renderer::submit::{frame_render_prepare_system, ui_render_submit_system};
use super::resources::{BufferResourceRegistry, TextureResourceRegistry, TransientResourceTracker};
use super::sdf::SdfRenderFeatureState;
use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::scene::SceneResource;
use crate::runtime::{RenderPrepare, RenderSubmit};
use crate::state::{DebugMetricsState, StartupState};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneResource>();
        app.init_resource::<RenderFrameResourceBindings>();
        app.init_resource::<ShaderRegistryResource>();
        app.init_resource::<RenderGraphRegistryResource>();
        app.init_resource::<RenderPassExecutorRegistryResource>();
        app.init_resource::<PipelineCacheResource>();
        app.init_resource::<TextureResourceRegistry>();
        app.init_resource::<BufferResourceRegistry>();
        app.init_resource::<TransientResourceTracker>();
        app.init_resource::<SdfRenderFeatureState>();
        app.init_resource::<RenderDebugOverlayState>();
        app.init_resource::<RenderTextureInspectorState>();
        app.init_resource::<RenderDebugTimingsState>();
        app.init_resource::<RenderDebugGraphDumpState>();
        app.init_resource::<StartupState>();
        app.init_resource::<DebugMetricsState>();
        register_builtin_executor_ids(app);
        app.add_systems(RenderPrepare, frame_render_prepare_system);
        app.add_systems(RenderSubmit, ui_render_submit_system);
    }
}

fn register_builtin_executor_ids(app: &mut App) {
    let Ok(registry) = app
        .world_mut()
        .resource_mut::<RenderPassExecutorRegistryResource>()
    else {
        return;
    };

    registry.register_builtin("builtin_compute", BuiltinRenderPassExecutor::Compute);
    registry.register_builtin("builtin_compose", BuiltinRenderPassExecutor::Compose);
    registry.register_builtin(
        "builtin_mesh_overlay",
        BuiltinRenderPassExecutor::MeshOverlay,
    );
    registry.register_builtin(
        "builtin_ui_composite",
        BuiltinRenderPassExecutor::UiComposite,
    );
}
