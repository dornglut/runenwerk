pub mod api;
pub mod backend;
pub mod composition;
pub mod features;
pub mod frame;
pub mod frame_packet;
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
pub use features::{
    DEFORMATION_RENDER_FEATURE_ID, FeatureContributionStatus, FeatureFallbackPolicy,
    MATERIAL_RENDER_FEATURE_ID, PreparedDeformationFeatureResource, PreparedDrawFeatureResource,
    PreparedMaterialFeatureResource, RenderFeatureDescriptor, RenderFeatureId,
    RenderFeatureRegistryResource, SCENE_ROUTE_RENDER_FEATURE_ID, UI_RENDER_FEATURE_ID,
    WORLD_DRAW_RENDER_FEATURE_ID,
};
pub use frame::{
    PreparedDeformationFeatureContribution, PreparedDeformationStream, PreparedDrawBatch,
    PreparedDrawFeatureContribution, PreparedFeatureContribution, PreparedFeatureGate,
    PreparedFeaturePayload, PreparedFlowInputs, PreparedFrameContext, PreparedFrameContributions,
    PreparedMaterialFeatureContribution, PreparedMaterialInstanceInput, PreparedRenderFrame,
    PreparedRenderFrameResource, PreparedSceneRouteContribution, PreparedShaderSnapshot,
    PreparedStateTypeInfo, PreparedSurfaceInfo, PreparedUiFeatureContribution, PreparedViewFrame,
};
pub use graph::{
    CompiledBindGroupPlan, CompiledBindingEntry, CompiledBuiltinImport,
    CompiledComputeExecutionPlan, CompiledComputePass, CompiledCopyExecutionPlan, CompiledCopyPass,
    CompiledDispatchPlan, CompiledDrawBufferPlan, CompiledFlowExecutionPlan,
    CompiledFullscreenPass, CompiledGraphicsPass, CompiledPassBindings, CompiledPassDescriptor,
    CompiledPassExecutionPlan, CompiledPresentExecutionPlan, CompiledPresentPass,
    CompiledRasterExecutionPlan, CompiledRenderFlowPlan, CompiledResourceRef,
    CompiledStateRequirement, CompiledStorageAccess, CompiledStorageBinding, CompiledTargetPlan,
    CompiledUiCompositeExecutionPlan, CompiledUiCompositePass, CompiledViewMask,
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
    ImportedBufferSemantic, ImportedTextureSemantic, RenderResourceDescriptor, ResourceLifetime,
    TransientAliasAssignment, TransientAliasCandidate, TransientAliasSlot, TransientResourceWindow,
};
pub use shader::{ShaderHandle, ShaderRegistryResource};
