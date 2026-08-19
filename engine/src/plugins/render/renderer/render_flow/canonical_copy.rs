use super::*;
use crate::plugins::gpu::{
    CurrentRenderBufferCopyTerminal, CurrentRenderTextureCopyTerminal, GpuBufferHandle,
    GpuCopyExtent, GpuCopyOperation, GpuRealizedBuffer, GpuRealizedTexture,
    GpuTextureAspect as LogicalTextureAspect, GpuTextureHandle, GpuTextureOrigin,
};

impl Renderer {
    /// Temporary pre-G5B physical realization for canonical, surface-independent copy work.
    ///
    /// All transfer semantics are already owned by `GpuCopyOperation`. This method may only map
    /// those facts onto already-realized opaque G4 resources and the current lexical execution
    /// bridge. It must not reconstruct copy ranges, origins, extents, or compatibility from a
    /// renderer execution plan, and it never realizes resources lazily during G5 encoding.
    pub(super) fn encode_canonical_copy_operation(
        &self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        operation: &GpuCopyOperation,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<EncodedPassEvidence> {
        match operation {
            GpuCopyOperation::BufferToBuffer {
                source,
                destination,
            } => {
                let source_realized =
                    realized_buffer_for_handle(runtime_resources, source.buffer())?;
                let destination_realized =
                    realized_buffer_for_handle(runtime_resources, destination.buffer())?;
                context.current_render_execution_bridge().for_buffer_copy(
                    source_realized,
                    destination_realized,
                    CanonicalBufferCopy {
                        encoder,
                        source_offset: source.range().offset(),
                        destination_offset: destination.range().offset(),
                        size: source.range().size(),
                    },
                )?;
            }
            GpuCopyOperation::TextureToTexture {
                source,
                destination,
            } => {
                let source_realized =
                    realized_texture_for_handle(runtime_resources, source.texture())?;
                let destination_realized =
                    realized_texture_for_handle(runtime_resources, destination.texture())?;
                context.current_render_execution_bridge().for_texture_copy(
                    source_realized,
                    destination_realized,
                    CanonicalTextureCopy {
                        encoder,
                        source_mip_level: source.mip_level(),
                        source_origin: source.origin(),
                        source_aspect: source.aspect(),
                        destination_mip_level: destination.mip_level(),
                        destination_origin: destination.origin(),
                        destination_aspect: destination.aspect(),
                        extent: source.extent(),
                    },
                )?;
            }
            GpuCopyOperation::BufferToTexture { .. } | GpuCopyOperation::TextureToBuffer { .. } => {
                bail!(
                    "current renderer G5A physical bridge cannot encode canonical buffer-texture copies; renderer lowering does not produce this operation class and G5B owns the generic backend executor"
                );
            }
        }

        Ok(EncodedPassEvidence {
            dispatch_workgroups: None,
            shader_id: "builtin:copy".to_string(),
            shader_revision: 0,
            fallback_used: false,
            pipeline_key: None,
        })
    }
}

fn realized_buffer_for_handle<'a>(
    runtime_resources: &'a FlowRuntimeResources,
    handle: &GpuBufferHandle,
) -> Result<&'a GpuRealizedBuffer> {
    runtime_resources
        .buffers
        .values()
        .find(|resource| &resource.handle == handle)
        .map(|resource| &resource.realized)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "canonical copy references buffer '{}' without an already-realized G4 resource",
                handle.descriptor().common().label().as_str()
            )
        })
}

fn realized_texture_for_handle<'a>(
    runtime_resources: &'a FlowRuntimeResources,
    handle: &GpuTextureHandle,
) -> Result<&'a GpuRealizedTexture> {
    runtime_resources
        .textures
        .values()
        .chain(runtime_resources.invocation_history_textures.values())
        .find(|resource| &resource.handle == handle)
        .map(|resource| &resource.realized)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "canonical copy references texture '{}' without an already-realized G4 resource",
                handle.descriptor().common().label().as_str()
            )
        })
}

struct CanonicalBufferCopy<'a> {
    encoder: &'a mut CommandEncoder,
    source_offset: u64,
    destination_offset: u64,
    size: u64,
}

impl CurrentRenderBufferCopyTerminal for CanonicalBufferCopy<'_> {
    fn copy_buffers(self, source: &Buffer, destination: &Buffer) {
        self.encoder.copy_buffer_to_buffer(
            source,
            self.source_offset,
            destination,
            self.destination_offset,
            self.size,
        );
    }
}

struct CanonicalTextureCopy<'a> {
    encoder: &'a mut CommandEncoder,
    source_mip_level: u32,
    source_origin: GpuTextureOrigin,
    source_aspect: LogicalTextureAspect,
    destination_mip_level: u32,
    destination_origin: GpuTextureOrigin,
    destination_aspect: LogicalTextureAspect,
    extent: GpuCopyExtent,
}

impl CurrentRenderTextureCopyTerminal for CanonicalTextureCopy<'_> {
    fn copy_textures(self, source: &Texture, destination: &Texture) {
        self.encoder.copy_texture_to_texture(
            TexelCopyTextureInfo {
                texture: source,
                mip_level: self.source_mip_level,
                origin: wgpu_origin(self.source_origin),
                aspect: wgpu_texture_aspect(self.source_aspect),
            },
            TexelCopyTextureInfo {
                texture: destination,
                mip_level: self.destination_mip_level,
                origin: wgpu_origin(self.destination_origin),
                aspect: wgpu_texture_aspect(self.destination_aspect),
            },
            wgpu_copy_extent(self.extent),
        );
    }
}

fn wgpu_origin(origin: GpuTextureOrigin) -> Origin3d {
    Origin3d {
        x: origin.x(),
        y: origin.y(),
        z: origin.z(),
    }
}

fn wgpu_copy_extent(extent: GpuCopyExtent) -> Extent3d {
    Extent3d {
        width: extent.width(),
        height: extent.height(),
        depth_or_array_layers: extent.depth_or_layers(),
    }
}

fn wgpu_texture_aspect(aspect: LogicalTextureAspect) -> TextureAspect {
    match aspect {
        LogicalTextureAspect::All | LogicalTextureAspect::Color => TextureAspect::All,
        LogicalTextureAspect::DepthOnly => TextureAspect::DepthOnly,
        LogicalTextureAspect::StencilOnly => TextureAspect::StencilOnly,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_texture_copy_lowering_preserves_origin_extent_and_aspect() {
        assert_eq!(
            wgpu_origin(GpuTextureOrigin::new(3, 5, 7)),
            Origin3d { x: 3, y: 5, z: 7 }
        );
        assert_eq!(
            wgpu_copy_extent(GpuCopyExtent::new(11, 13, 17).unwrap()),
            Extent3d {
                width: 11,
                height: 13,
                depth_or_array_layers: 17,
            }
        );
        assert_eq!(
            wgpu_texture_aspect(LogicalTextureAspect::Color),
            TextureAspect::All
        );
        assert_eq!(
            wgpu_texture_aspect(LogicalTextureAspect::DepthOnly),
            TextureAspect::DepthOnly
        );
        assert_eq!(
            wgpu_texture_aspect(LogicalTextureAspect::StencilOnly),
            TextureAspect::StencilOnly
        );
    }
}
