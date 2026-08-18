use crate::plugins::gpu::{GpuTextureFormat, GpuTextureFormatCapabilities};
use core::num::NonZeroU64;
use std::collections::BTreeMap;

/// Device-dependent facts required by backend-neutral runtime binding validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRuntimeBindingDeviceFacts {
    uniform_buffer_offset_alignment: NonZeroU64,
    storage_buffer_offset_alignment: NonZeroU64,
    max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    max_dynamic_storage_buffers_per_pipeline_layout: u32,
    format_capabilities: BTreeMap<GpuTextureFormat, GpuTextureFormatCapabilities>,
}

impl GpuRuntimeBindingDeviceFacts {
    pub fn new(
        uniform_buffer_offset_alignment: NonZeroU64,
        storage_buffer_offset_alignment: NonZeroU64,
        max_dynamic_uniform_buffers_per_pipeline_layout: u32,
        max_dynamic_storage_buffers_per_pipeline_layout: u32,
        format_capabilities: impl IntoIterator<Item = (GpuTextureFormat, GpuTextureFormatCapabilities)>,
    ) -> Self {
        Self {
            uniform_buffer_offset_alignment,
            storage_buffer_offset_alignment,
            max_dynamic_uniform_buffers_per_pipeline_layout,
            max_dynamic_storage_buffers_per_pipeline_layout,
            format_capabilities: format_capabilities.into_iter().collect(),
        }
    }

    pub const fn uniform_buffer_offset_alignment(&self) -> NonZeroU64 {
        self.uniform_buffer_offset_alignment
    }

    pub const fn storage_buffer_offset_alignment(&self) -> NonZeroU64 {
        self.storage_buffer_offset_alignment
    }

    pub const fn max_dynamic_uniform_buffers_per_pipeline_layout(&self) -> u32 {
        self.max_dynamic_uniform_buffers_per_pipeline_layout
    }

    pub const fn max_dynamic_storage_buffers_per_pipeline_layout(&self) -> u32 {
        self.max_dynamic_storage_buffers_per_pipeline_layout
    }

    pub fn format_capabilities(
        &self,
        format: GpuTextureFormat,
    ) -> Option<GpuTextureFormatCapabilities> {
        self.format_capabilities.get(&format).copied()
    }
}
