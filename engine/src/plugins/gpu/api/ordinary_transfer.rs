use super::{
    GpuBufferHandle, GpuBufferRange, GpuBufferRegion, GpuCopyExtent, GpuTextureAspect,
    GpuTextureCopyRegion, GpuTextureHandle, GpuTextureOrigin, GpuWorkOperationCause,
    GpuWorkOperationError,
};

impl GpuBufferRegion {
    /// Constructs one transfer region covering the complete validated buffer.
    ///
    /// Subranges remain available through [`GpuBufferRegion::new`].
    pub fn whole(buffer: &GpuBufferHandle) -> Result<Self, GpuWorkOperationError> {
        let range = GpuBufferRange::whole(buffer).map_err(|source| {
            GpuWorkOperationError::from_access(
                "construct whole GPU buffer region",
                buffer.descriptor().common().label().as_str(),
                GpuWorkOperationCause::InvalidCopyRegion,
                "use a validated nonempty buffer descriptor",
                source,
            )
        })?;
        Self::new(buffer, range)
    }
}

impl GpuTextureCopyRegion {
    /// Constructs one transfer region covering the complete base mip from zero origin.
    ///
    /// The descriptor supplies the full base-mip extent, including all D2 array layers or
    /// D3 depth slices. The canonical texture-copy constructor still validates single-sample
    /// eligibility and normalizes the texture aspect from the format. Partial coverage and
    /// nonzero mip selection remain available through [`GpuTextureCopyRegion::new`].
    pub fn whole_base_mip(texture: &GpuTextureHandle) -> Result<Self, GpuWorkOperationError> {
        let extent = texture.descriptor().extent();
        Self::new(
            texture,
            0,
            GpuTextureOrigin::new(0, 0, 0),
            GpuTextureAspect::All,
            GpuCopyExtent::new(
                extent.width(),
                extent.height(),
                extent.depth_or_layers(),
            )?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuBufferDescriptor, GpuBufferInitialization, GpuBufferUsage, GpuReconstruction,
        GpuResourceLifetime, GpuResourceScope, GpuTextureDescriptor, GpuTextureFormat,
        GpuTextureInitialization, GpuTextureUsage,
    };

    #[test]
    fn whole_buffer_region_preserves_complete_descriptor_range() {
        let mut resources = GpuResourceScope::new();
        let buffer = resources
            .buffer(
                GpuBufferDescriptor::ordinary_owned(
                    "whole transfer buffer",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    96,
                    [GpuBufferUsage::CopySource],
                    GpuBufferInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();

        let region = GpuBufferRegion::whole(&buffer).unwrap();
        assert_eq!(region.buffer(), &buffer);
        assert_eq!(region.range().offset(), 0);
        assert_eq!(region.range().size(), 96);
    }

    #[test]
    fn whole_base_mip_region_preserves_full_ordinary_2d_extent_and_color_aspect() {
        let mut resources = GpuResourceScope::new();
        let texture = resources
            .texture(
                GpuTextureDescriptor::ordinary_owned_2d(
                    "whole transfer texture",
                    GpuResourceLifetime::Transient,
                    GpuReconstruction::SourceBacked,
                    17,
                    11,
                    GpuTextureFormat::Rgba8Unorm,
                    [GpuTextureUsage::CopySource],
                    GpuTextureInitialization::Uninitialized,
                )
                .unwrap(),
            )
            .unwrap();

        let region = GpuTextureCopyRegion::whole_base_mip(&texture).unwrap();
        assert_eq!(region.texture(), &texture);
        assert_eq!(region.mip_level(), 0);
        assert_eq!(region.origin(), GpuTextureOrigin::new(0, 0, 0));
        assert_eq!(region.aspect(), GpuTextureAspect::Color);
        assert_eq!(region.extent(), GpuCopyExtent::new(17, 11, 1).unwrap());
        assert_eq!(region.subresources().base_mip_level(), 0);
        assert_eq!(region.subresources().mip_level_count(), 1);
        assert_eq!(region.subresources().base_array_layer(), 0);
        assert_eq!(region.subresources().array_layer_count(), 1);
    }
}
