use crate::plugins::gpu::{
    GpuAddressMode, GpuBufferDescriptor, GpuBufferInitialization, GpuBufferUsage, GpuBufferUsages,
    GpuFilterMode, GpuMemoryIntent, GpuReconstruction, GpuResourceCommon, GpuResourceLabel,
    GpuResourceLifetime, GpuResourceProvenance, GpuSamplerDescriptor, GpuTextureAspect,
    GpuTextureDescriptor, GpuTextureDimension, GpuTextureExtent, GpuTextureFormat,
    GpuTextureHandle, GpuTextureInitialization, GpuTextureSubresourceRange, GpuTextureUsage,
    GpuTextureUsages, GpuTextureViewDescriptor,
};
use anyhow::{Result, bail};
use wgpu::TextureFormat;

pub(super) fn owned_common(
    label: impl Into<String>,
    lifetime: GpuResourceLifetime,
    memory_intent: GpuMemoryIntent,
) -> Result<GpuResourceCommon> {
    let label = GpuResourceLabel::new(label)?;
    let provenance = GpuResourceProvenance::new(label.clone(), None, None);
    Ok(GpuResourceCommon::owned(
        label,
        lifetime,
        memory_intent,
        GpuReconstruction::SourceBacked,
        provenance,
    )?)
}

pub(super) fn buffer_descriptor(
    label: impl Into<String>,
    size: u64,
    usages: impl IntoIterator<Item = GpuBufferUsage>,
    lifetime: GpuResourceLifetime,
    memory_intent: GpuMemoryIntent,
) -> Result<GpuBufferDescriptor> {
    let common = owned_common(label, lifetime, memory_intent)?;
    let usages = GpuBufferUsages::new(common.label(), usages)?;
    Ok(GpuBufferDescriptor::new(
        common,
        size.max(1),
        usages,
        GpuBufferInitialization::Uninitialized,
    )?)
}

pub(super) fn texture_descriptor(
    label: impl Into<String>,
    size: (u32, u32),
    format: GpuTextureFormat,
    usages: impl IntoIterator<Item = GpuTextureUsage>,
    lifetime: GpuResourceLifetime,
) -> Result<GpuTextureDescriptor> {
    texture_descriptor_with_extent(
        label,
        GpuTextureDimension::D2,
        (size.0, size.1, 1),
        format,
        usages,
        lifetime,
    )
}

pub(super) fn texture_descriptor_with_extent(
    label: impl Into<String>,
    dimension: GpuTextureDimension,
    extent: (u32, u32, u32),
    format: GpuTextureFormat,
    usages: impl IntoIterator<Item = GpuTextureUsage>,
    lifetime: GpuResourceLifetime,
) -> Result<GpuTextureDescriptor> {
    let common = owned_common(label, lifetime, GpuMemoryIntent::Device)?;
    let extent = GpuTextureExtent::new(
        common.label(),
        dimension,
        extent.0.max(1),
        extent.1.max(1),
        extent.2.max(1),
    )?;
    let usages = GpuTextureUsages::new(common.label(), usages)?;
    Ok(GpuTextureDescriptor::new(
        common,
        dimension,
        extent,
        1,
        1,
        format,
        usages,
        GpuTextureInitialization::Uninitialized,
    )?)
}

pub(super) fn whole_texture_view_descriptor(
    label: impl Into<String>,
    texture: &GpuTextureHandle,
) -> Result<GpuTextureViewDescriptor> {
    let descriptor = texture.descriptor();
    let common = owned_common(
        label,
        descriptor.common().lifetime(),
        GpuMemoryIntent::Device,
    )?;
    let aspect = if descriptor.format().is_depth() {
        GpuTextureAspect::DepthOnly
    } else {
        GpuTextureAspect::Color
    };
    let array_layer_count = match descriptor.dimension() {
        GpuTextureDimension::D2 => descriptor.extent().depth_or_layers(),
        GpuTextureDimension::D1 | GpuTextureDimension::D3 => 1,
    };
    let subresources = GpuTextureSubresourceRange::new(
        common.label(),
        0,
        descriptor.mip_level_count(),
        0,
        array_layer_count,
        aspect,
    )?;
    Ok(GpuTextureViewDescriptor::new(
        common,
        texture,
        None,
        descriptor.dimension(),
        subresources,
    )?)
}

pub(super) fn linear_sampler_descriptor(
    label: impl Into<String>,
    lifetime: GpuResourceLifetime,
) -> Result<GpuSamplerDescriptor> {
    linear_sampler_descriptor_with_address_mode(label, lifetime, GpuAddressMode::ClampToEdge)
}

pub(super) fn repeat_linear_sampler_descriptor(
    label: impl Into<String>,
    lifetime: GpuResourceLifetime,
) -> Result<GpuSamplerDescriptor> {
    linear_sampler_descriptor_with_address_mode(label, lifetime, GpuAddressMode::Repeat)
}

fn linear_sampler_descriptor_with_address_mode(
    label: impl Into<String>,
    lifetime: GpuResourceLifetime,
    address_mode: GpuAddressMode,
) -> Result<GpuSamplerDescriptor> {
    Ok(GpuSamplerDescriptor::new(
        owned_common(label, lifetime, GpuMemoryIntent::Device)?,
        address_mode,
        address_mode,
        address_mode,
        GpuFilterMode::Linear,
        GpuFilterMode::Linear,
        GpuFilterMode::Nearest,
        0.0,
        32.0,
        None,
    )?)
}

pub(super) fn gpu_texture_format(format: TextureFormat) -> Result<GpuTextureFormat> {
    match format {
        TextureFormat::R8Unorm => Ok(GpuTextureFormat::R8Unorm),
        TextureFormat::Rgba8Unorm => Ok(GpuTextureFormat::Rgba8Unorm),
        TextureFormat::Rgba8UnormSrgb => Ok(GpuTextureFormat::Rgba8UnormSrgb),
        TextureFormat::Bgra8Unorm => Ok(GpuTextureFormat::Bgra8Unorm),
        TextureFormat::Bgra8UnormSrgb => Ok(GpuTextureFormat::Bgra8UnormSrgb),
        TextureFormat::R32Uint => Ok(GpuTextureFormat::R32Uint),
        TextureFormat::Depth32Float => Ok(GpuTextureFormat::Depth32Float),
        other => bail!("current renderer format {other:?} has no admitted RunenGPU mapping"),
    }
}

pub(super) const fn wgpu_texture_format(format: GpuTextureFormat) -> TextureFormat {
    match format {
        GpuTextureFormat::R8Unorm => TextureFormat::R8Unorm,
        GpuTextureFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
        GpuTextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        GpuTextureFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
        GpuTextureFormat::Bgra8UnormSrgb => TextureFormat::Bgra8UnormSrgb,
        GpuTextureFormat::R32Uint => TextureFormat::R32Uint,
        GpuTextureFormat::Depth32Float => TextureFormat::Depth32Float,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::GpuWorkResourceIdAllocator;
    use std::num::NonZeroU64;

    #[test]
    fn whole_volume_view_uses_one_array_layer() {
        let descriptor = texture_descriptor_with_extent(
            "whole volume view test texture",
            GpuTextureDimension::D3,
            (4, 8, 16),
            GpuTextureFormat::Rgba8Unorm,
            [GpuTextureUsage::Sampled],
            GpuResourceLifetime::Retained,
        )
        .unwrap();
        let mut allocator =
            GpuWorkResourceIdAllocator::for_owner_scope(NonZeroU64::new(1).unwrap());
        let texture = allocator.allocate_texture_handle(descriptor).unwrap();

        let view = whole_texture_view_descriptor("whole volume view test", &texture).unwrap();

        assert_eq!(view.dimension(), GpuTextureDimension::D3);
        assert_eq!(view.subresources().array_layer_count(), 1);
    }
}
