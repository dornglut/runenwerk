pub mod adapters;
pub mod api;
mod app_ext;
pub mod backend;
pub mod composition;
pub mod features;
pub mod frame;
mod gpu_context_policy;
pub mod gpu_primitives;
pub mod graph;
pub mod inspect;
pub mod material_compiler;
pub(crate) mod native_host;
pub mod params;
pub mod pipelines;
pub mod procedural;
pub mod readiness;
pub mod renderer;
pub mod residency;
pub mod resource;
pub mod shader;
mod texture_upload;

mod plugin;
pub mod runtime;

pub use adapters::*;
pub use api::*;
pub use app_ext::AppRenderExt;
pub use bytemuck;
pub use composition::*;
pub use engine_render_macros::{GpuStorage, GpuUniform};
pub use features::*;
pub use frame::*;
pub use gpu_context_policy::{
    apply_runenwerk_gpu_context_policy, runenwerk_gpu_backend_preference,
};
pub use gpu_primitives::*;
pub use graph::*;
pub use material_compiler::*;
pub use params::*;
pub use plugin::RenderPlugin;
pub(crate) use plugin::render_integration_is_active;
pub use procedural::*;
pub use readiness::{RenderReadinessPhase, RenderReadinessState};
pub use renderer::{Gfx, GfxFrameTimings, Renderer, RendererFrameTimings};
pub use residency::*;
pub use resource::*;
pub use runtime::*;
pub use shader::{
    ShaderHandle, ShaderRegistryEvent, ShaderRegistryEventKind, ShaderRegistryResource,
    ShaderReloadPollReport, ShaderReloadPollStatus,
};
