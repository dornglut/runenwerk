use super::super::contract_diagnostics::{GpuProgramContractCause, GpuProgramContractError};
use super::super::interface::{GpuBindingKey, GpuTextureViewDimension};
use crate::plugins::gpu::{GpuBufferHandle, GpuSamplerHandle, GpuTextureViewHandle};
use core::num::NonZeroU64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuRuntimeBufferBinding {
    handle: GpuBufferHandle,
    offset: u64,
    size: NonZeroU64,
    dynamic_offset: Option<u64>,
}

impl GpuRuntimeBufferBinding {
    pub fn new(
        handle: GpuBufferHandle,
        offset: u64,
        size: NonZeroU64,
        dynamic_offset: Option<u64>,
    ) -> Self {
        Self {
            handle,
            offset,
            size,
            dynamic_offset,
        }
    }

    pub fn handle(&self) -> &GpuBufferHandle {
        &self.handle
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub const fn size(&self) -> NonZeroU64 {
        self.size
    }

    pub const fn dynamic_offset(&self) -> Option<u64> {
        self.dynamic_offset
    }

    pub fn checked_effective_offset(&self) -> Option<u64> {
        self.offset.checked_add(self.dynamic_offset.unwrap_or(0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuRuntimeTextureViewBinding {
    handle: GpuTextureViewHandle,
    dimension: GpuTextureViewDimension,
}

impl GpuRuntimeTextureViewBinding {
    pub fn new(handle: GpuTextureViewHandle, dimension: GpuTextureViewDimension) -> Self {
        Self { handle, dimension }
    }

    pub fn handle(&self) -> &GpuTextureViewHandle {
        &self.handle
    }

    pub const fn dimension(&self) -> GpuTextureViewDimension {
        self.dimension
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuRuntimeBindingResource {
    Buffer(GpuRuntimeBufferBinding),
    TextureView(GpuRuntimeTextureViewBinding),
    Sampler(GpuSamplerHandle),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuRuntimeBindingValue {
    key: GpuBindingKey,
    resources: Vec<GpuRuntimeBindingResource>,
}

impl GpuRuntimeBindingValue {
    pub fn new(
        key: GpuBindingKey,
        resources: impl IntoIterator<Item = GpuRuntimeBindingResource>,
    ) -> Result<Self, GpuProgramContractError> {
        let resources = resources.into_iter().collect::<Vec<_>>();
        if resources.is_empty() {
            return Err(GpuProgramContractError::invalid(
                "construct runtime GPU binding value",
                key.to_string(),
                GpuProgramContractCause::RuntimeBindingIncompatible,
                "provide at least one typed runtime resource",
            ));
        }
        Ok(Self { key, resources })
    }

    pub const fn key(&self) -> GpuBindingKey {
        self.key
    }

    pub fn resources(&self) -> impl ExactSizeIterator<Item = &GpuRuntimeBindingResource> {
        self.resources.iter()
    }
}
