use super::super::{
    GpuTextureAccess, GpuTextureAccessKind, GpuTextureAccessResource, GpuTextureAspect,
    GpuTextureSubresourceRange, GpuWorkOperationCause, GpuWorkOperationError,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuPresentOperation {
    source: GpuTextureAccessResource,
    subresource: GpuTextureSubresourceRange,
    source_access: GpuTextureAccess,
}

impl GpuPresentOperation {
    pub fn new(
        source: GpuTextureAccessResource,
        subresource: GpuTextureSubresourceRange,
    ) -> Result<Self, GpuWorkOperationError> {
        if subresource.mip_level_count() != 1
            || subresource.array_layer_count() != 1
            || !matches!(
                subresource.aspect(),
                GpuTextureAspect::All | GpuTextureAspect::Color
            )
            || source.parent_texture().descriptor().format().is_depth()
            || source.parent_texture().descriptor().sample_count() != 1
        {
            return Err(GpuWorkOperationError::invalid(
                "construct GPU present operation",
                source
                    .parent_texture()
                    .descriptor()
                    .common()
                    .label()
                    .as_str(),
                Some(source.parent_texture().diagnostic_identity()),
                GpuWorkOperationCause::InvalidAttachment,
                "select exactly one single-sampled color mip and array layer",
            ));
        }
        let source_access =
            GpuTextureAccess::new(source.clone(), subresource, GpuTextureAccessKind::Present)
                .map_err(|source| {
                    GpuWorkOperationError::from_access(
                        "construct GPU present operation",
                        "present",
                        GpuWorkOperationCause::InvalidAttachment,
                        "provide one checked color source subresource",
                        source,
                    )
                })?;
        Ok(Self {
            source,
            subresource,
            source_access,
        })
    }

    pub fn source(&self) -> &GpuTextureAccessResource {
        &self.source
    }
    pub const fn subresource(&self) -> GpuTextureSubresourceRange {
        self.subresource
    }
    pub fn source_access(&self) -> &GpuTextureAccess {
        &self.source_access
    }
}
