use super::logical_timing::LogicalGpuPassTiming;
use super::*;
use crate::plugins::gpu::{
    GpuAttachmentStore, GpuBlendConstant, GpuBufferHandle, GpuBufferRange, GpuBufferRegion,
    GpuColorAttachmentLoad, GpuColorClearValue, GpuComputeOperation, GpuCopyOperation,
    GpuDepthAttachmentLoad, GpuDepthClearValue, GpuDepthStencilAccess, GpuDispatchIntent,
    GpuDispatchSize, GpuDrawIntent, GpuDrawRange, GpuIndexBufferBinding, GpuIndexFormat,
    GpuQueryAccess, GpuQueryAccessKind, GpuQueryRange, GpuQueryResolveOperation,
    GpuRenderColorAttachment, GpuRenderDepthStencilAttachment, GpuRenderDraw, GpuRenderOperation,
    GpuScissorRect, GpuTextureAccessResource, GpuTextureSubresourceRange, GpuTextureViewHandle,
    GpuUploadOperation, GpuVertexBufferBinding, GpuViewport, GpuWorkOperation, PreparedGpuData,
    TransferData,
};
use crate::plugins::render::graph::CompiledDrawSource;
use crate::plugins::render::{RenderDepthPolicy, RenderIndirectDrawArgsKind};

/// Projects one already-realized compute pass into its execution-complete RunenGPU operation.
///
/// This function does not inspect compiled binding declarations or reconstruct accesses. The
/// exact pipeline descriptor and complete logical runtime binding set are consumed from G4C2/C3
/// realization, and all access/capability meaning is subsequently derived by `GpuWorkOperation`.
pub(super) fn project_compute_operation(
    context: &GpuContext,
    pass: &CompiledComputeExecutionPlan,
    flow_inputs: &PreparedFlowInputs,
    pipeline: &PreparedPipelinePass,
    timing: Option<(&LogicalGpuPassTiming, usize)>,
) -> Result<GpuWorkOperation> {
    let PreparedFlowPipeline::Compute(realized_pipeline) = &pipeline.pipeline else {
        bail!(
            "compute pass '{}' realized a non-compute GPU pipeline before G5A operation projection",
            pass.pass_id
        );
    };
    let dispatch = flow_inputs
        .projected_dispatch_workgroups
        .get(&pass.pass_id)
        .copied()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "compute pass '{}' is missing its prepared dispatch projection",
                pass.pass_id
            )
        })?;
    let dispatch = GpuDispatchIntent::direct(
        GpuDispatchSize::new(dispatch[0], dispatch[1], dispatch[2])?,
        context.device_facts().device_limits().values(),
    )?;
    let mut operation = GpuComputeOperation::new(
        realized_pipeline.descriptor().clone(),
        pipeline.bindings.runtime_bindings.clone(),
        dispatch,
    )?;
    if let Some((timing, ordinal)) = timing {
        operation = operation.with_timestamp_writes([timestamp_access(timing, ordinal)?])?;
    }
    Ok(GpuWorkOperation::Compute(operation))
}

/// Projects one already-realized, surface-independent raster pass into canonical RunenGPU work.
///
/// `None` is returned only when the pass still depends on a target whose logical GPU identity is
/// intentionally unavailable before G7A (the current host surface or renderer dynamic-target
/// compatibility path). G5A must not invent a placeholder surface identity to erase that boundary.
pub(super) fn project_render_operation(
    context: &GpuContext,
    runtime_resources: &FlowRuntimeResources,
    pass: &CompiledPassExecutionPlan,
    pipeline: &PreparedPipelinePass,
    timing: Option<(&LogicalGpuPassTiming, usize)>,
) -> Result<Option<GpuWorkOperation>> {
    let raster = match pass {
        CompiledPassExecutionPlan::Fullscreen(value) | CompiledPassExecutionPlan::Graphics(value) => {
            value
        }
        _ => bail!(
            "pass '{}' is not a raster pass for G5A render-operation projection",
            execution_pass_id(pass)
        ),
    };
    let PreparedFlowPipeline::Render(realized_pipeline) = &pipeline.pipeline else {
        bail!(
            "raster pass '{}' realized a non-render GPU pipeline before G5A operation projection",
            raster.pass_id
        );
    };

    let Some(color_target_ref) = raster.targets.color_outputs.first() else {
        bail!("raster pass '{}' has no color output", raster.pass_id);
    };
    if raster.targets.color_outputs.len() != 1 {
        bail!(
            "raster pass '{}' has {} color outputs; current renderer execution supports exactly one",
            raster.pass_id,
            raster.targets.color_outputs.len()
        );
    }
    let color_key = runtime_resources.resolve_resource_key(
        raster.pass_id,
        color_target_ref,
        "color_output",
    )?;
    let Some(color_target) = logical_texture_target(runtime_resources, &color_key) else {
        return Ok(None);
    };
    if color_target.is_depth {
        bail!(
            "raster pass '{}' resolved a depth texture as its color output",
            raster.pass_id
        );
    }

    let color_load = match raster.clear_color {
        Some(color) => GpuColorAttachmentLoad::Clear(GpuColorClearValue::from_array(
            color.map(f64::from),
        )?),
        None => GpuColorAttachmentLoad::Load,
    };
    let color_attachment = GpuRenderColorAttachment::new(
        GpuTextureAccessResource::TextureView(color_target.view.clone()),
        color_target.view.descriptor().subresources(),
        color_load,
        GpuAttachmentStore::Store,
        None,
    )?;

    let depth_attachment = if matches!(raster.raster_state.state.depth_policy, RenderDepthPolicy::Disabled)
    {
        None
    } else if let Some(depth_ref) = raster.targets.depth_output.as_ref() {
        let depth_key =
            runtime_resources.resolve_resource_key(raster.pass_id, depth_ref, "depth_output")?;
        let Some(depth_target) = logical_texture_target(runtime_resources, &depth_key) else {
            return Ok(None);
        };
        if !depth_target.is_depth {
            bail!(
                "raster pass '{}' resolved a color texture as its depth output",
                raster.pass_id
            );
        }
        let read_only = matches!(
            raster.raster_state.state.depth_policy,
            RenderDepthPolicy::ReadOnly
        );
        Some(GpuRenderDepthStencilAttachment::new(
            GpuTextureAccessResource::TextureView(depth_target.view.clone()),
            depth_target.view.descriptor().subresources(),
            if read_only {
                GpuDepthStencilAccess::ReadOnly
            } else {
                GpuDepthStencilAccess::ReadWrite
            },
            if read_only {
                GpuDepthAttachmentLoad::Load
            } else {
                GpuDepthAttachmentLoad::Clear(GpuDepthClearValue::new(1.0)?)
            },
            GpuAttachmentStore::Store,
        )?)
    } else {
        None
    };

    let limits = context.device_facts().device_limits().values();
    let draw = project_render_draw(
        runtime_resources,
        pass,
        realized_pipeline.descriptor().clone(),
        pipeline.bindings.runtime_bindings.clone(),
        color_target.size,
        limits,
    )?;
    let timestamp_writes = match timing {
        Some((timing, ordinal)) => vec![timestamp_access(timing, ordinal)?],
        None => Vec::new(),
    };
    Ok(Some(GpuWorkOperation::Render(GpuRenderOperation::new(
        [color_attachment],
        depth_attachment,
        [draw],
        timestamp_writes,
    )?)))
}

fn project_render_draw(
    runtime_resources: &FlowRuntimeResources,
    pass: &CompiledPassExecutionPlan,
    pipeline: crate::plugins::gpu::GpuRenderPipelineDescriptor,
    bindings: crate::plugins::gpu::GpuRuntimeBindingSet,
    target_size: (u32, u32),
    limits: crate::plugins::gpu::GpuLimits,
) -> Result<GpuRenderDraw> {
    let pass_id = execution_pass_id(pass);
    let (vertex_buffers, index_buffer, draw) = match pass {
        CompiledPassExecutionPlan::Fullscreen(_) => (
            Vec::new(),
            None,
            GpuDrawIntent::direct(GpuDrawRange::new(0, 3)?, GpuDrawRange::new(0, 1)?),
        ),
        CompiledPassExecutionPlan::Graphics(plan) => {
            let mut vertex_buffers = Vec::new();
            for binding in &plan.draw_buffers.vertex_buffers {
                let buffer =
                    runtime_resources.resolve_storage_buffer_ref(pass_id, &binding.resource)?;
                vertex_buffers.push(GpuVertexBufferBinding::new(
                    binding.layout.slot,
                    buffer.handle,
                    GpuBufferRange::whole(buffer.handle)?,
                )?);
            }
            for (resource, layout) in plan
                .draw_buffers
                .instance_buffers
                .iter()
                .zip(plan.draw_buffers.instance_buffer_layouts.iter())
            {
                let buffer = runtime_resources.resolve_storage_buffer_ref(pass_id, resource)?;
                vertex_buffers.push(GpuVertexBufferBinding::new(
                    layout.slot,
                    buffer.handle,
                    GpuBufferRange::whole(buffer.handle)?,
                )?);
            }
            let index_buffer = match plan.draw_buffers.index_buffers.as_slice() {
                [] => None,
                [only] => {
                    let buffer = runtime_resources.resolve_storage_buffer_ref(pass_id, only)?;
                    Some(GpuIndexBufferBinding::new(
                        buffer.handle,
                        GpuBufferRange::whole(buffer.handle)?,
                        GpuIndexFormat::Uint32,
                    )?)
                }
                _ => bail!(
                    "graphics pass '{}' declares multiple index buffers; current runtime supports exactly one",
                    pass_id
                ),
            };
            let authored_draw = plan.draw.ok_or_else(|| {
                anyhow::anyhow!(
                    "graphics pass '{}' is missing draw parameters in execution plan",
                    pass_id
                )
            })?;
            let instances =
                GpuDrawRange::new(authored_draw.first_instance, authored_draw.instance_count)?;
            let draw = match authored_draw.source {
                CompiledDrawSource::Direct if index_buffer.is_some() => GpuDrawIntent::indexed(
                    GpuDrawRange::new(authored_draw.first_vertex, authored_draw.vertex_count)?,
                    0,
                    instances,
                ),
                CompiledDrawSource::Direct => GpuDrawIntent::direct(
                    GpuDrawRange::new(authored_draw.first_vertex, authored_draw.vertex_count)?,
                    instances,
                ),
                CompiledDrawSource::Indirect {
                    args_buffer,
                    args_kind,
                    byte_offset,
                    ..
                } => {
                    let resource = plan
                        .draw_buffers
                        .indirect_buffers
                        .iter()
                        .find(|resource| compiled_resource_ref_matches_id(resource, args_buffer))
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "graphics pass '{}' indirect draw references an argument buffer absent from its compiled indirect set",
                                pass_id
                            )
                        })?;
                    let buffer = runtime_resources.resolve_storage_buffer_ref(pass_id, resource)?;
                    let indexed = matches!(args_kind, RenderIndirectDrawArgsKind::DrawIndexed);
                    let size = if indexed { 20 } else { 16 };
                    GpuDrawIntent::indirect(
                        buffer.handle,
                        GpuBufferRange::new(buffer.handle, byte_offset, size)?,
                        indexed,
                    )?
                }
            };
            (vertex_buffers, index_buffer, draw)
        }
        _ => bail!("pass '{}' is not a raster draw pass", pass_id),
    };

    let width = target_size.0;
    let height = target_size.1;
    Ok(GpuRenderDraw::new(
        pipeline,
        bindings,
        vertex_buffers,
        index_buffer,
        draw,
        GpuViewport::new(
            0.0,
            0.0,
            width as f32,
            height as f32,
            0.0,
            1.0,
            limits,
        )?,
        GpuScissorRect::new(0, 0, width, height)?,
        GpuBlendConstant::new(0.0, 0.0, 0.0, 0.0)?,
        0,
        limits,
    )?)
}

struct LogicalTextureTarget {
    view: GpuTextureViewHandle,
    size: (u32, u32),
    is_depth: bool,
}

fn logical_texture_target(
    runtime_resources: &FlowRuntimeResources,
    key: &RuntimeResourceKey,
) -> Option<LogicalTextureTarget> {
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
    Some(LogicalTextureTarget {
        view: texture.view_handle.clone(),
        size: texture.size,
        is_depth: texture.is_depth,
    })
}

fn compiled_resource_ref_matches_id(
    resource: &CompiledResourceRef,
    expected: crate::plugins::gpu::GpuWorkResourceId,
) -> bool {
    match resource {
        CompiledResourceRef::FlowOwned(id) | CompiledResourceRef::Imported(id) => *id == expected,
        CompiledResourceRef::TargetAlias(alias) => alias.resource_id == expected,
        CompiledResourceRef::ImportedBuiltin(_) => false,
    }
}

/// Projects an immutable renderer-prepared byte sequence into exact logical CPU→GPU buffer work.
/// Physical `queue.write_buffer` remains a temporary G5B/G5C realization detail.
pub(super) fn project_buffer_upload(
    buffer: &GpuBufferHandle,
    bytes: &[u8],
) -> Result<GpuUploadOperation> {
    let size = u64::try_from(bytes.len())
        .map_err(|_| anyhow::anyhow!("render GPU upload length exceeds u64"))?;
    let range = GpuBufferRange::new(buffer, 0, size)?;
    let region = GpuBufferRegion::new(buffer, range)?;
    let payload = PreparedGpuData::<TransferData>::from_pod_transfer(
        format!(
            "{}.render-upload",
            buffer.descriptor().common().label().as_str()
        ),
        bytes,
        buffer.descriptor().common().provenance().clone(),
    )?;
    Ok(GpuUploadOperation::new(region.into(), payload)?)
}

pub(super) fn project_timing_tail(
    timing: &LogicalGpuPassTiming,
) -> Result<(GpuQueryResolveOperation, GpuCopyOperation)> {
    let resolve = GpuQueryResolveOperation::new(
        timing.query_set(),
        timing.query_range()?,
        timing.resolve_buffer(),
        0,
    )?;
    let byte_len = u64::from(timing.query_capacity())
        .checked_mul(8)
        .ok_or_else(|| anyhow::anyhow!("render GPU timestamp byte coverage overflow"))?;
    let copy = GpuCopyOperation::buffer_to_buffer(
        GpuBufferRegion::new(
            timing.resolve_buffer(),
            GpuBufferRange::new(timing.resolve_buffer(), 0, byte_len)?,
        )?,
        GpuBufferRegion::new(
            timing.readback_buffer(),
            GpuBufferRange::new(timing.readback_buffer(), 0, byte_len)?,
        )?,
    )?;
    Ok((resolve, copy))
}

pub(super) fn timestamp_access(
    timing: &LogicalGpuPassTiming,
    ordinal: usize,
) -> Result<GpuQueryAccess> {
    let indices = timing.range_for_occurrence(ordinal)?;
    let range = GpuQueryRange::new(timing.query_set(), indices.begin, 2)?;
    Ok(GpuQueryAccess::new(
        timing.query_set(),
        range,
        GpuQueryAccessKind::WriteTimestamp,
    )?)
}
