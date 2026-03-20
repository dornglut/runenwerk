pub mod api;
pub mod backend;
pub mod composition;
pub mod graph;
pub mod inspect;
pub mod params;
pub mod pipelines;
pub mod renderer;
pub mod resource;
pub mod shader;

mod plugin;

pub use api::{
    BuiltinUiCompositePassBuilder, ComputeDispatchBinding, ComputeDispatchDescriptor,
    ComputePassBuilder, DoubleBufferHandle, FullscreenPassBuilder, ParamProjectionError,
    ParamProjectionErrorKind, PassHandle, PassParamBinding, PassUniformProjection,
    ProjectedUniformBuffer, ProjectedUniformSet, RenderFlow, RenderFlowId, RenderPassId,
    RenderResourceId, StorageArrayHandle, UniformHandle,
};
pub use bytemuck;
pub use composition::RenderFlowRegistryResource;
pub use engine_render_macros::{GpuStorage, GpuUniform};
pub use graph::{
    CompiledComputePass, CompiledCopyPass, CompiledFullscreenPass, CompiledGraphicsPass,
    CompiledPassDescriptor, CompiledPresentPass, CompiledRenderFlowPlan, CompiledUiCompositePass,
    FlowValidationReport, RenderFlowGraph, RenderFlowValidationError, RenderPassKind,
    RenderPassNode, RenderShaderReference, compile_flow_plan, validate_flow_graph,
};
pub use params::{
    GpuBoolU32, GpuParams, GpuStorage, GpuUniform, GpuUniformField, ToGpuValue, align_up_const,
    write_uniform_field,
};
pub use plugin::RenderPlugin;
pub use renderer::frame_bindings::RenderFrameDataRegistry;
pub use renderer::{Gfx, GfxFrameTimings, Renderer, RendererFrameTimings};
pub use resource::{
    RenderResourceDescriptor, ResourceLifetime, TransientAliasAssignment, TransientAliasCandidate,
    TransientAliasSlot, TransientResourceWindow,
};
pub use shader::{ShaderHandle, ShaderRegistryResource};
