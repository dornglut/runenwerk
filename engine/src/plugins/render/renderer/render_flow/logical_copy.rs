use super::*;
use crate::plugins::gpu::{
    GpuBufferRange, GpuBufferRegion, GpuCopyExtent, GpuCopyOperation, GpuTextureAspect,
    GpuTextureCopyRegion, GpuTextureHandle, GpuTextureOrigin, GpuWorkOperation,
};

/// Projects one resolved renderer copy into canonical RunenGPU work.
///
/// `None` has exactly two meanings:
/// - source and destination resolve to the same runtime resource, so no GPU work exists; or
/// - at least one texture endpoint still lacks durable logical GPU identity before G7A
///   (surface/dynamic texture compatibility paths).
///
/// Buffer/texture class mismatches remain invalid. The current renderer does not implement
/// buffer-texture copies, so G5A does not invent that execution meaning here.
pub(super) fn project_copy_operation(
    runtime_resources: &FlowRuntimeResources,
    pass: &CompiledCopyExecutionPlan,
) -> Result<Option<GpuWorkOperation>> {
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
    let source_key = runtime_resources.resolve_resource_key(pass.pass_id, source, "copy_source")?;
    let destination_key =
        runtime_resources.resolve_resource_key(pass.pass_id, destination, "copy_destination")?;
    if source_key == destination_key {
        return Ok(None);
    }

    let source_kind = runtime_resources
        .kind_of_resource(source_key.clone())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "copy pass '{}' references unknown source resource '{}'",
                pass.pass_id,
                source_key
            )
        })?;
    let destination_kind = runtime_resources
        .kind_of_resource(destination_key.clone())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "copy pass '{}' references unknown destination resource '{}'",
                pass.pass_id,
                destination_key
            )
        })?;

    let operation = match (source_kind, destination_kind) {
        (RuntimeResourceKind::BufferLike, RuntimeResourceKind::BufferLike) => {
            let source = runtime_resources.resolve_buffer_key(pass.pass_id, source_key)?;
            let destination =
                runtime_resources.resolve_buffer_key(pass.pass_id, destination_key)?;
            let size = source.size.min(destination.size);
            if size == 0 {
                anyhow::bail!(
                    "copy pass '{}' resolved buffer copy extent to zero for '{}' -> '{}'",
                    pass.pass_id,
                    source.id,
                    destination.id
                );
            }
            GpuCopyOperation::buffer_to_buffer(
                GpuBufferRegion::new(source.handle, GpuBufferRange::new(source.handle, 0, size)?)?,
                GpuBufferRegion::new(
                    destination.handle,
                    GpuBufferRange::new(destination.handle, 0, size)?,
                )?,
            )?
        }
        (RuntimeResourceKind::BufferLike, RuntimeResourceKind::TextureLike)
        | (RuntimeResourceKind::TextureLike, RuntimeResourceKind::BufferLike) => {
            anyhow::bail!(
                "copy pass '{}' mixes incompatible resource classes '{}' -> '{}'",
                pass.pass_id,
                source_key,
                destination_key
            );
        }
        (RuntimeResourceKind::TextureLike, RuntimeResourceKind::TextureLike) => {
            let Some(source) = logical_texture(runtime_resources, &source_key) else {
                return Ok(None);
            };
            let Some(destination) = logical_texture(runtime_resources, &destination_key) else {
                return Ok(None);
            };
            if source.is_depth || destination.is_depth {
                anyhow::bail!(
                    "copy pass '{}' requested unsupported depth copy '{}' -> '{}'; only color-like texture copies are supported",
                    pass.pass_id,
                    source_key,
                    destination_key
                );
            }
            let width = source.size.0.min(destination.size.0);
            let height = source.size.1.min(destination.size.1);
            if width == 0 || height == 0 {
                anyhow::bail!(
                    "copy pass '{}' resolved texture copy extent to zero for '{}' -> '{}'",
                    pass.pass_id,
                    source_key,
                    destination_key
                );
            }
            let extent = GpuCopyExtent::new(width, height, 1)?;
            GpuCopyOperation::texture_to_texture(
                GpuTextureCopyRegion::new(
                    source.handle,
                    0,
                    GpuTextureOrigin::new(0, 0, 0),
                    GpuTextureAspect::Color,
                    extent,
                )?,
                GpuTextureCopyRegion::new(
                    destination.handle,
                    0,
                    GpuTextureOrigin::new(0, 0, 0),
                    GpuTextureAspect::Color,
                    extent,
                )?,
            )?
        }
    };
    Ok(Some(GpuWorkOperation::Copy(operation)))
}

struct LogicalCopyTexture<'a> {
    handle: &'a GpuTextureHandle,
    size: (u32, u32),
    is_depth: bool,
}

fn logical_texture<'a>(
    runtime_resources: &'a FlowRuntimeResources,
    key: &RuntimeResourceKey,
) -> Option<LogicalCopyTexture<'a>> {
    let texture = match key {
        RuntimeResourceKey::FlowOwned(resource_id) => runtime_resources.textures.get(resource_id),
        RuntimeResourceKey::InvocationHistory {
            invocation_id,
            resource_id,
        } => runtime_resources
            .invocation_history_textures
            .get(&(invocation_id.clone(), *resource_id)),
        RuntimeResourceKey::InvocationUniform { .. }
        | RuntimeResourceKey::DynamicTexture(_)
        | RuntimeResourceKey::SurfaceColor
        | RuntimeResourceKey::SurfaceDepth => None,
    }?;
    Some(LogicalCopyTexture {
        handle: &texture.handle,
        size: texture.size,
        is_depth: texture.is_depth,
    })
}
