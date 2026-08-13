mod pipeline;

use super::*;
use crate::plugins::gpu::{
    CurrentRenderBufferCopyTerminal, CurrentRenderTextureCopyTerminal,
    CurrentSurfaceTextureCopyTerminal,
};
use crate::plugins::render::RenderPassId;

impl Renderer {
    fn encode_texture_copy(
        &self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        pass_id: RenderPassId,
        source: ResolvedTextureRef<'_>,
        destination: ResolvedTextureRef<'_>,
    ) -> Result<()> {
        if source.is_depth || destination.is_depth {
            bail!(
                "pass '{}' requested unsupported depth copy '{}' -> '{}'; only color-like texture copies are supported",
                pass_id,
                source.id,
                destination.id
            );
        }
        if !copy_formats_are_raw_compatible(source.format, destination.format) {
            bail!(
                "pass '{}' requested copy with incompatible formats '{}' ({:?}) -> '{}' ({:?})",
                pass_id,
                source.id,
                source.format,
                destination.id,
                destination.format
            );
        }

        let width = source.size.0.min(destination.size.0);
        let height = source.size.1.min(destination.size.1);
        if width == 0 || height == 0 {
            bail!(
                "pass '{}' resolved copy extent to zero for '{}' -> '{}'",
                pass_id,
                source.id,
                destination.id
            );
        }

        let extent = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        match (source.texture, destination.texture) {
            (RuntimeTextureRef::Realized(source), RuntimeTextureRef::Realized(destination)) => {
                context.current_render_execution_bridge().for_texture_copy(
                    source,
                    destination,
                    CopyTextures { encoder, extent },
                )?;
            }
            (RuntimeTextureRef::Realized(source), RuntimeTextureRef::Surface(destination)) => {
                context
                    .current_render_execution_bridge()
                    .for_surface_texture_copy(
                        source,
                        CopySurfaceTexture {
                            encoder,
                            surface: destination,
                            extent,
                            realized_is_source: true,
                        },
                    )?;
            }
            (RuntimeTextureRef::Surface(source), RuntimeTextureRef::Realized(destination)) => {
                context
                    .current_render_execution_bridge()
                    .for_surface_texture_copy(
                        destination,
                        CopySurfaceTexture {
                            encoder,
                            surface: source,
                            extent,
                            realized_is_source: false,
                        },
                    )?;
            }
            (RuntimeTextureRef::Surface(source), RuntimeTextureRef::Surface(destination)) => {
                encoder.copy_texture_to_texture(
                    TexelCopyTextureInfo {
                        texture: source,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    TexelCopyTextureInfo {
                        texture: destination,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    extent,
                );
            }
        }
        Ok(())
    }

    fn encode_buffer_copy(
        &self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        pass_id: RenderPassId,
        source: ResolvedBufferRef<'_>,
        destination: ResolvedBufferRef<'_>,
    ) -> Result<()> {
        let size = source.size.min(destination.size);
        if size == 0 {
            bail!(
                "pass '{}' resolved buffer copy extent to zero for '{}' -> '{}'",
                pass_id,
                source.id,
                destination.id
            );
        }
        context.current_render_execution_bridge().for_buffer_copy(
            source.buffer,
            destination.buffer,
            CopyBuffers { encoder, size },
        )?;
        Ok(())
    }

    fn encode_copy_pass(
        &self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        packet: &RendererPreparedPacket,
        runtime_resources: &FlowRuntimeResources,
        pass: &CompiledCopyExecutionPlan,
    ) -> Result<()> {
        let source = pass.source.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "copy pass '{}' is missing source resource in execution plan",
                pass.pass_id
            )
        })?;
        let destination = pass.destination.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "copy pass '{}' is missing destination resource in execution plan",
                pass.pass_id
            )
        })?;

        let source_id =
            runtime_resources.resolve_resource_key(pass.pass_id, source, "copy_source")?;
        let destination_id = runtime_resources.resolve_resource_key(
            pass.pass_id,
            destination,
            "copy_destination",
        )?;
        if source_id == destination_id {
            return Ok(());
        }

        let source_kind = runtime_resources
            .kind_of_resource(source_id.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "copy pass '{}' references unknown source resource '{}'",
                    pass.pass_id,
                    source_id
                )
            })?;
        let destination_kind = runtime_resources
            .kind_of_resource(destination_id.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "copy pass '{}' references unknown destination resource '{}'",
                    pass.pass_id,
                    destination_id
                )
            })?;

        match (source_kind, destination_kind) {
            (RuntimeResourceKind::BufferLike, RuntimeResourceKind::BufferLike) => {
                let source = runtime_resources.resolve_buffer_key(pass.pass_id, source_id)?;
                let destination =
                    runtime_resources.resolve_buffer_key(pass.pass_id, destination_id)?;
                self.encode_buffer_copy(context, encoder, pass.pass_id, source, destination)
            }
            (RuntimeResourceKind::BufferLike, RuntimeResourceKind::TextureLike)
            | (RuntimeResourceKind::TextureLike, RuntimeResourceKind::BufferLike) => {
                bail!(
                    "copy pass '{}' mixes incompatible resource classes '{}' -> '{}'",
                    pass.pass_id,
                    source_id,
                    destination_id
                );
            }
            (RuntimeResourceKind::TextureLike, RuntimeResourceKind::TextureLike) => {
                let source = self.resolve_texture_by_key(
                    runtime_resources,
                    pass.pass_id,
                    source_id,
                    frame_texture,
                    packet.surface_size,
                    packet.surface_format,
                )?;
                let destination = self.resolve_texture_by_key(
                    runtime_resources,
                    pass.pass_id,
                    destination_id,
                    frame_texture,
                    packet.surface_size,
                    packet.surface_format,
                )?;
                self.encode_texture_copy(context, encoder, pass.pass_id, source, destination)
            }
        }
    }

    fn encode_present_pass(
        &self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        packet: &RendererPreparedPacket,
        runtime_resources: &FlowRuntimeResources,
        pass: &CompiledPresentExecutionPlan,
    ) -> Result<()> {
        let source = pass.source.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "present pass '{}' is missing source resource in execution plan",
                pass.pass_id
            )
        })?;
        let source_id =
            runtime_resources.resolve_resource_key(pass.pass_id, source, "present_source")?;
        if source_id == RuntimeResourceKey::SurfaceColor {
            return Ok(());
        }

        let source_kind = runtime_resources
            .kind_of_resource(source_id.clone())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "present pass '{}' references unknown source resource '{}'",
                    pass.pass_id,
                    source_id
                )
            })?;
        if matches!(source_kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "present pass '{}' reads buffer-like resource '{}' but present requires a texture-like source",
                pass.pass_id,
                source_id
            );
        }

        let source = self.resolve_texture_by_key(
            runtime_resources,
            pass.pass_id,
            source_id,
            frame_texture,
            packet.surface_size,
            packet.surface_format,
        )?;
        let destination = ResolvedTextureRef {
            id: RuntimeResourceKey::SurfaceColor,
            texture: RuntimeTextureRef::Surface(frame_texture),
            view_handle: None,
            format: packet.surface_format,
            size: packet.surface_size,
            is_depth: false,
        };
        self.encode_texture_copy(context, encoder, pass.pass_id, source, destination)
    }

    fn resolve_color_target_from_plan<'a>(
        &self,
        runtime_resources: &'a FlowRuntimeResources,
        pass_id: RenderPassId,
        targets: &CompiledTargetPlan,
        frame_view: &'a TextureView,
        frame_format: TextureFormat,
    ) -> Result<ResolvedColorTargetView<'a>> {
        if targets.color_outputs.len() != 1 {
            bail!(
                "pass '{}' declares {} color outputs, but runtime execution currently requires exactly one color output",
                pass_id,
                targets.color_outputs.len()
            );
        }
        let output = targets.color_outputs.first().ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' is missing a color output target in execution plan",
                pass_id
            )
        })?;
        let output_key = runtime_resources.resolve_resource_key(pass_id, output, "color_output")?;
        match output_key {
            RuntimeResourceKey::DynamicTexture(key) => self
                .dynamic_texture_targets
                .color_target_view(pass_id, &key),
            _ => runtime_resources.resolve_color_target_from_plan(
                pass_id,
                targets,
                frame_view,
                frame_format,
            ),
        }
    }

    fn resolve_depth_target_from_plan(
        &self,
        runtime_resources: &FlowRuntimeResources,
        pass_id: RenderPassId,
        targets: &CompiledTargetPlan,
    ) -> Result<Option<ResolvedDepthTargetView>> {
        let Some(depth_target) = targets.depth_output.as_ref() else {
            return Ok(None);
        };
        let resource_key =
            runtime_resources.resolve_resource_key(pass_id, depth_target, "depth_output")?;
        match resource_key {
            RuntimeResourceKey::DynamicTexture(key) => self
                .dynamic_texture_targets
                .depth_target_view(pass_id, &key)
                .map(Some),
            _ => runtime_resources.resolve_depth_target_from_plan(pass_id, targets),
        }
    }

    fn resolve_texture_by_key<'a>(
        &'a self,
        runtime_resources: &'a FlowRuntimeResources,
        pass_id: RenderPassId,
        resource_key: RuntimeResourceKey,
        frame_texture: &'a Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<ResolvedTextureRef<'a>> {
        match resource_key {
            RuntimeResourceKey::DynamicTexture(key) => {
                self.dynamic_texture_targets.texture_ref(pass_id, &key)
            }
            other => runtime_resources.resolve_texture(
                pass_id,
                other,
                frame_texture,
                frame_size,
                frame_format,
            ),
        }
    }
}

struct CopyBuffers<'a> {
    encoder: &'a mut CommandEncoder,
    size: u64,
}

impl CurrentRenderBufferCopyTerminal for CopyBuffers<'_> {
    fn copy_buffers(self, source: &Buffer, destination: &Buffer) {
        self.encoder
            .copy_buffer_to_buffer(source, 0, destination, 0, self.size);
    }
}

struct CopyTextures<'a> {
    encoder: &'a mut CommandEncoder,
    extent: Extent3d,
}

struct CopySurfaceTexture<'a> {
    encoder: &'a mut CommandEncoder,
    surface: &'a Texture,
    extent: Extent3d,
    realized_is_source: bool,
}

impl CurrentSurfaceTextureCopyTerminal for CopySurfaceTexture<'_> {
    fn copy_with_surface(self, realized: &Texture) {
        let (source, destination) = if self.realized_is_source {
            (realized, self.surface)
        } else {
            (self.surface, realized)
        };
        self.encoder.copy_texture_to_texture(
            TexelCopyTextureInfo {
                texture: source,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyTextureInfo {
                texture: destination,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            self.extent,
        );
    }
}

impl CurrentRenderTextureCopyTerminal for CopyTextures<'_> {
    fn copy_textures(self, source: &Texture, destination: &Texture) {
        self.encoder.copy_texture_to_texture(
            TexelCopyTextureInfo {
                texture: source,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyTextureInfo {
                texture: destination,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            self.extent,
        );
    }
}

fn copy_formats_are_raw_compatible(source: TextureFormat, destination: TextureFormat) -> bool {
    if texture_format_is_depth_or_stencil(source) || texture_format_is_depth_or_stencil(destination)
    {
        return false;
    }
    source.remove_srgb_suffix() == destination.remove_srgb_suffix()
}

fn texture_format_is_depth_or_stencil(format: TextureFormat) -> bool {
    format.is_depth_stencil_format()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_copy_formats_accept_srgb_suffix_pairs() {
        assert!(copy_formats_are_raw_compatible(
            TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb
        ));
        assert!(copy_formats_are_raw_compatible(
            TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Rgba8Unorm
        ));
        assert!(copy_formats_are_raw_compatible(
            TextureFormat::Bgra8Unorm,
            TextureFormat::Bgra8UnormSrgb
        ));
        assert!(copy_formats_are_raw_compatible(
            TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Bgra8Unorm
        ));
    }

    #[test]
    fn raw_copy_formats_reject_unrelated_color_formats() {
        assert!(!copy_formats_are_raw_compatible(
            TextureFormat::Rgba8Unorm,
            TextureFormat::Bgra8Unorm
        ));
        assert!(!copy_formats_are_raw_compatible(
            TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba16Float
        ));
    }

    #[test]
    fn raw_copy_formats_reject_depth_stencil_formats() {
        assert!(!copy_formats_are_raw_compatible(
            TextureFormat::Depth32Float,
            TextureFormat::Depth32Float
        ));
        assert!(!copy_formats_are_raw_compatible(
            TextureFormat::Rgba8Unorm,
            TextureFormat::Depth32Float
        ));
    }
}
