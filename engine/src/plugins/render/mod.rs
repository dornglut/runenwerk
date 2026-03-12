pub mod api;
pub mod backend;
pub mod composition;
pub mod debug;
pub mod domain;
pub mod frame_graph;
pub mod graph;
pub mod inspect;
pub mod params;
pub mod pipelines;
pub mod renderer;
pub mod resource;
pub mod resources;
pub mod sdf;
pub mod shader;

mod plugin;

pub use api::{
    BuiltinUiCompositePassBuilder, ComputePassBuilder, CopyPassBuilder, FullscreenPassBuilder,
    GraphicsPassBuilder, NamespaceIdError, ParamProjectionError, ParamProjectionErrorKind,
    PassParamBinding, PassUniformProjection, PresentPassBuilder, ProjectedUniformBuffer,
    RenderFlow, RenderFlowId, RenderPassId, RenderResourceId, is_namespaced_id, namespace_of,
    validate_namespaced_id,
};
pub use bytemuck;
pub use composition::{
    FragmentHotReloadEntry, FragmentPassKind, FragmentPassSpec, FragmentReloadOutcome,
    FragmentResourceSpec, FragmentSpecError, RenderFlowContribution,
    RenderFlowFragmentHotReloadState, RenderFlowFragmentSpec, RenderFlowRegistryResource,
    RenderFlowVariant, parse_fragment_ron,
};
pub use engine_render_macros::{GpuStorage, GpuUniform};
pub use graph::{
    FlowValidationReport, RenderFlowGraph, RenderFlowValidationError, RenderPassKind,
    RenderPassNode, validate_flow_graph,
};
pub use params::{GpuBoolU32, GpuParams, GpuStorage, GpuUniform, ToGpuValue};
pub use plugin::RenderPlugin;
pub use resource::{
    RenderResourceDescriptor, ResourceLifetime, TransientAliasAssignment, TransientAliasCandidate,
    TransientAliasSlot, TransientResourceWindow,
};
