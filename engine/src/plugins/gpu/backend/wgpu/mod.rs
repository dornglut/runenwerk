//! Private WGPU containment for G4A context admission.
//!
//! The submodules own state, normalized adapter mapping, device requests, and
//! the bounded G7 bridge respectively. This root only composes those owners.

mod adapter_mapping;
mod current_host;
mod device_request;
mod resource_realization;
mod state;

pub(crate) use device_request::request_headless;
pub(crate) use resource_realization::{
    BufferRealizationRecord, QuerySetRealizationRecord, ResourceRealizationState,
    SamplerRealizationRecord, TextureRealizationRecord, TextureViewRealizationRecord,
};
pub(crate) use state::{CurrentRenderDeviceQueue, WgpuContextState};
