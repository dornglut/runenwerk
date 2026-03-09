use super::domain::{
    RenderFrameResourceBindings, RenderGraphRegistryResource, RenderPassExecutorRegistryResource,
    ShaderRegistryResource,
};
use super::submit::{frame_render_prepare_system, ui_render_submit_system};
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
        app.init_resource::<StartupState>();
        app.init_resource::<DebugMetricsState>();
        app.add_systems(RenderPrepare, frame_render_prepare_system);
        app.add_systems(RenderSubmit, ui_render_submit_system);
    }
}
