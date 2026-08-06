use crate::plugins::gpu::{GpuTextureFormat, GpuTextureFormatCapabilities};
use core::num::NonZeroU64;
use std::collections::BTreeMap;

/// Device-dependent facts required by backend-neutral runtime binding validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRuntimeBindingDeviceFacts {
    uniform_buffer_offset_alignment: NonZeroU64,
    storage_buffer_offset_alignment: NonZeroU64,
    format_capabilities: BTreeMap<GpuTextureFormat, GpuTextureFormatCapabilities>,
}

impl GpuRuntimeBindingDeviceFacts {
    pub fn new(
        uniform_buffer_offset_alignment: NonZeroU64,
        storage_buffer_offset_alignment: NonZeroU64,
        format_capabilities: impl IntoIterator<Item = (GpuTextureFormat, GpuTextureFormatCapabilities)>,
    ) -> Self {
        Self {
            uniform_buffer_offset_alignment,
            storage_buffer_offset_alignment,
            format_capabilities: format_capabilities.into_iter().collect(),
        }
    }

    pub const fn uniform_buffer_offset_alignment(&self) -> NonZeroU64 {
        self.uniform_buffer_offset_alignment
    }

    pub const fn storage_buffer_offset_alignment(&self) -> NonZeroU64 {
        self.storage_buffer_offset_alignment
    }

    pub fn format_capabilities(
        &self,
        format: GpuTextureFormat,
    ) -> Option<GpuTextureFormatCapabilities> {
        self.format_capabilities.get(&format).copied()
    }
}
