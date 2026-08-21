//! Private WGPU containment for G4A context admission.
//!
//! The submodules own state, normalized adapter mapping, device requests, execution, surfaces, and
//! bounded migration bridges. This root only composes those owners.

mod adapter_mapping;
mod current_host;
mod device_request;
mod execution;
#[cfg(test)]
mod execution_tests;
mod health;
mod pipeline_realization;
mod program_binding_realization;
mod resource_realization;
mod state;
mod surface;

pub(crate) use device_request::request_headless;
pub(crate) use execution::WgpuExecutionState;
pub(crate) use health::{WgpuDeviceHealth, WgpuErrorAttributionGate};
pub(crate) use pipeline_realization::{
    ComputePipelineRealizationRecord, PipelineRealizationState, RenderPipelineRealizationRecord,
};
pub(crate) use program_binding_realization::{
    BindGroupLayoutRealizationRecord, BindGroupRealizationRecord, CurrentRenderAttachmentsTerminal,
    CurrentRenderBufferCopyTerminal, CurrentRenderBufferUploadTerminal,
    CurrentRenderComputePipelineTerminal, CurrentRenderIndexBufferTerminal,
    CurrentRenderIndirectBufferTerminal, CurrentRenderPipelineBindGroupsTerminal,
    CurrentRenderReadbackBufferTerminal, CurrentRenderRenderPipelineTerminal,
    CurrentRenderRenderPipelinesTerminal, CurrentRenderTextureCopyTerminal,
    CurrentRenderTextureReadbackCopyTerminal, CurrentRenderTextureUploadTerminal,
    CurrentRenderTimestampResourcesTerminal, CurrentRenderTimestampWritesTerminal,
    CurrentRenderVertexBufferTerminal, CurrentSurfaceReadbackCopyTerminal,
    CurrentSurfaceTextureCopyTerminal, PipelineLayoutRealizationRecord,
    ProgramBindingRealizationState, ProgramRealizationRecord,
};
pub(crate) use resource_realization::{
    BufferRealizationRecord, QuerySetRealizationRecord, ResourceRealizationState,
    SamplerRealizationRecord, TextureRealizationRecord, TextureViewRealizationRecord,
};
pub(crate) use state::{CurrentRenderDeviceQueue, WgpuContextState};
pub(crate) use surface::WgpuSurfaceState;
