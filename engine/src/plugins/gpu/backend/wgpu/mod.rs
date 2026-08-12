//! Private WGPU containment for G4A context admission.
//!
//! The submodules own state, normalized adapter mapping, device requests, and
//! the bounded G7 bridge respectively. This root only composes those owners.

mod adapter_mapping;
mod current_host;
mod device_request;
mod health;
mod pipeline_realization;
mod program_binding_realization;
mod resource_realization;
mod state;

pub(crate) use device_request::request_headless;
pub(crate) use health::{WgpuDeviceHealth, WgpuErrorAttributionGate};
pub(crate) use pipeline_realization::{ComputePipelineRealizationRecord, PipelineRealizationState};
pub(crate) use program_binding_realization::{
    BindGroupLayoutRealizationRecord, BindGroupRealizationRecord, CurrentRenderAttachmentsTerminal,
    CurrentRenderBufferCopyTerminal, CurrentRenderBufferUploadTerminal,
    CurrentRenderIndexBufferTerminal, CurrentRenderIndirectBufferTerminal,
    CurrentRenderPipelineBindGroupsTerminal, CurrentRenderPipelineCreationTerminal,
    CurrentRenderReadbackBufferTerminal, CurrentRenderTextureCopyTerminal,
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
