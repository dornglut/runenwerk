use super::WgpuContextState;
use super::resource_realization::map_texture_aspect;
use crate::plugins::gpu::{
    GpuAttachmentStore, GpuColorAttachmentLoad, GpuContext, GpuDepthAttachmentLoad,
    GpuDepthStencilAccess, GpuDrawIntent, GpuIndexFormat, GpuPipelineRealizationError,
    GpuPipelineRealizationErrorCategory, GpuProgramBindingRealizationError,
    GpuProgramBindingRealizationErrorCategory, GpuRealizedBindGroup, GpuRealizedBuffer,
    GpuRealizedQuerySet, GpuRealizedRenderPipeline, GpuRealizedTexture, GpuRealizedTextureView,
    GpuRenderDraw, GpuRenderOperation, GpuRuntimeBindingResource, GpuSubmissionFailure,
    GpuSubmissionFailureKind, GpuSubmissionPreparationError, GpuSubmissionPreparationErrorKind,
    GpuTextureAccessResource, GpuTextureAspect, GpuTextureDimension, GpuTextureFormat,
    GpuTextureSubresourceRange, GpuValidatedBindGroupBindings, GpuWorkResourceId,
};
use std::collections::BTreeMap;
use wgpu::{
    Color, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPassTimestampWrites, StoreOp,
    TextureFormat, TextureFormatFeatureFlags, TextureView, TextureViewDescriptor,
    TextureViewDimension,
};

#[derive(Debug, Clone)]
pub(super) struct PreparedRenderOperation {
    color_attachments: Vec<PreparedRenderColorAttachment>,
    depth_stencil_attachment: Option<PreparedRenderDepthStencilAttachment>,
    draws: Vec<PreparedRenderDraw>,
    timestamp_writes: Option<PreparedRenderTimestampWrites>,
}

#[derive(Debug, Clone)]
struct PreparedRenderAttachment {
    texture: GpuRealizedTexture,
    _logical_view: Option<GpuRealizedTextureView>,
    format: GpuTextureFormat,
    subresources: GpuTextureSubresourceRange,
}

#[derive(Debug, Clone)]
struct PreparedRenderColorAttachment {
    source: PreparedRenderAttachment,
    load: GpuColorAttachmentLoad,
    store: GpuAttachmentStore,
    resolve_target: Option<PreparedRenderAttachment>,
}

#[derive(Debug, Clone)]
struct PreparedRenderDepthStencilAttachment {
    source: PreparedRenderAttachment,
    access: GpuDepthStencilAccess,
    load: GpuDepthAttachmentLoad,
    store: GpuAttachmentStore,
}

#[derive(Debug, Clone)]
struct PreparedRenderBindGroup {
    index: u32,
    realization: GpuRealizedBindGroup,
    dynamic_offsets: Vec<u32>,
}

#[derive(Debug, Clone)]
struct PreparedRenderVertexBuffer {
    slot: u32,
    buffer: GpuRealizedBuffer,
    offset: u64,
    end: u64,
}

#[derive(Debug, Clone)]
struct PreparedRenderIndexBuffer {
    buffer: GpuRealizedBuffer,
    offset: u64,
    end: u64,
    format: GpuIndexFormat,
}

#[derive(Debug, Clone)]
struct PreparedRenderIndirectBuffer {
    buffer: GpuRealizedBuffer,
    offset: u64,
    indexed: bool,
}

#[derive(Debug, Clone)]
struct PreparedRenderDraw {
    logical: GpuRenderDraw,
    pipeline: GpuRealizedRenderPipeline,
    bind_groups: Vec<PreparedRenderBindGroup>,
    vertex_buffers: Vec<PreparedRenderVertexBuffer>,
    index_buffer: Option<PreparedRenderIndexBuffer>,
    indirect_buffer: Option<PreparedRenderIndirectBuffer>,
}

#[derive(Debug, Clone)]
struct PreparedRenderTimestampWrites {
    query_set: GpuRealizedQuerySet,
    beginning_of_pass: Option<u32>,
    end_of_pass: Option<u32>,
}

pub(super) async fn prepare_render_operation(
    context: &GpuContext,
    render: &GpuRenderOperation,
) -> Result<PreparedRenderOperation, GpuSubmissionPreparationError> {
    if render.color_attachments().is_empty() && render.depth_stencil_attachment().is_none() {
        return render_not_encodable(
            "the current WGPU backend requires at least one color or depth attachment for every render pass",
        );
    }
    if render.color_attachments().len() > context.backend.device.limits().max_color_attachments as usize
    {
        return render_not_encodable(format!(
            "render pass declares {} color attachments but the admitted WGPU device supports at most {}",
            render.color_attachments().len(),
            context.backend.device.limits().max_color_attachments,
        ));
    }

    let mut buffers = BTreeMap::<GpuWorkResourceId, GpuRealizedBuffer>::new();
    let mut textures = BTreeMap::<GpuWorkResourceId, GpuRealizedTexture>::new();
    let mut texture_views = BTreeMap::<GpuWorkResourceId, GpuRealizedTextureView>::new();
    let mut query_sets = BTreeMap::<GpuWorkResourceId, GpuRealizedQuerySet>::new();

    let mut color_attachments = Vec::with_capacity(render.color_attachments().len());
    for attachment in render.color_attachments() {
        let source = prepare_attachment(
            context,
            &mut textures,
            &mut texture_views,
            attachment.source(),
            attachment.source_access().normalized_subresources(),
        )?;
        let resolve_target = attachment
            .resolve_target()
            .map(|resolve| {
                prepare_attachment(
                    context,
                    &mut textures,
                    &mut texture_views,
                    resolve.destination(),
                    resolve.access().normalized_subresources(),
                )
            })
            .transpose()?;
        if let Some(resolve_target) = &resolve_target {
            let native_format = map_texture_format(resolve_target.format);
            let features = context
                .backend
                .adapter
                .get_texture_format_features(native_format);
            if !features
                .flags
                .contains(TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
            {
                return render_not_encodable(format!(
                    "render resolve target format {:?} lacks WGPU MULTISAMPLE_RESOLVE support on the admitted adapter",
                    resolve_target.format
                ));
            }
        }
        color_attachments.push(PreparedRenderColorAttachment {
            source,
            load: attachment.load(),
            store: attachment.store(),
            resolve_target,
        });
    }

    let depth_stencil_attachment = render
        .depth_stencil_attachment()
        .map(|attachment| {
            if attachment.access() == GpuDepthStencilAccess::ReadOnly
                && attachment.store() != GpuAttachmentStore::Store
            {
                return render_not_encodable(
                    "read-only depth with Discard cannot be represented by the current WGPU render-pass API",
                );
            }
            Ok(PreparedRenderDepthStencilAttachment {
                source: prepare_attachment(
                    context,
                    &mut textures,
                    &mut texture_views,
                    attachment.source(),
                    attachment.source_access().normalized_subresources(),
                )?,
                access: attachment.access(),
                load: attachment.load(),
                store: attachment.store(),
            })
        })
        .transpose()?;

    let mut draws = Vec::with_capacity(render.draws().len());
    for draw in render.draws() {
        draws.push(
            prepare_render_draw(context, &mut buffers, draw)
                .await?,
        );
    }

    let timestamp_writes = render
        .timestamp_writes()
        .map(|writes| {
            Ok(PreparedRenderTimestampWrites {
                query_set: realized_query_set(context, &mut query_sets, writes.query_set())?,
                beginning_of_pass: writes.beginning_of_pass(),
                end_of_pass: writes.end_of_pass(),
            })
        })
        .transpose()?;

    Ok(PreparedRenderOperation {
        color_attachments,
        depth_stencil_attachment,
        draws,
        timestamp_writes,
    })
}

async fn prepare_render_draw(
    context: &GpuContext,
    buffers: &mut BTreeMap<GpuWorkResourceId, GpuRealizedBuffer>,
    draw: &GpuRenderDraw,
) -> Result<PreparedRenderDraw, GpuSubmissionPreparationError> {
    let descriptor = draw.pipeline();
    let program = context
        .realize_program(descriptor.program())
        .await
        .map_err(preparation_program_binding_failure)?;
    let layout = context
        .realize_pipeline_layout(descriptor.layout())
        .await
        .map_err(preparation_program_binding_failure)?;
    let pipeline = context
        .realize_render_pipeline(descriptor, &program, &layout)
        .await
        .map_err(preparation_pipeline_failure)?;

    let mut bind_groups = Vec::with_capacity(draw.bindings().groups().len());
    for group in draw.bindings().groups() {
        let layout = context
            .realize_bind_group_layout(group.layout())
            .await
            .map_err(preparation_program_binding_failure)?;
        let realization = context
            .realize_bind_group(&layout, group.values().cloned())
            .await
            .map_err(preparation_program_binding_failure)?;
        bind_groups.push(PreparedRenderBindGroup {
            index: group.layout().group(),
            realization,
            dynamic_offsets: checked_dynamic_offsets(group)?,
        });
    }

    let mut vertex_buffers = Vec::with_capacity(draw.vertex_buffers().len());
    for binding in draw.vertex_buffers() {
        vertex_buffers.push(PreparedRenderVertexBuffer {
            slot: binding.slot(),
            buffer: realized_buffer(context, buffers, binding.buffer())?,
            offset: binding.range().offset(),
            end: binding.range().end(),
        });
    }
    let index_buffer = draw
        .index_buffer()
        .map(|binding| {
            Ok(PreparedRenderIndexBuffer {
                buffer: realized_buffer(context, buffers, binding.buffer())?,
                offset: binding.range().offset(),
                end: binding.range().end(),
                format: binding.format(),
            })
        })
        .transpose()?;
    let indirect_buffer = match draw.draw() {
        GpuDrawIntent::Indirect {
            arguments,
            range,
            indexed,
        } => Some(PreparedRenderIndirectBuffer {
            buffer: realized_buffer(context, buffers, arguments)?,
            offset: range.offset(),
            indexed: *indexed,
        }),
        GpuDrawIntent::Direct { .. } | GpuDrawIntent::Indexed { .. } => None,
    };

    Ok(PreparedRenderDraw {
        logical: draw.clone(),
        pipeline,
        bind_groups,
        vertex_buffers,
        index_buffer,
        indirect_buffer,
    })
}

fn prepare_attachment(
    context: &GpuContext,
    textures: &mut BTreeMap<GpuWorkResourceId, GpuRealizedTexture>,
    texture_views: &mut BTreeMap<GpuWorkResourceId, GpuRealizedTextureView>,
    resource: &GpuTextureAccessResource,
    subresources: GpuTextureSubresourceRange,
) -> Result<PreparedRenderAttachment, GpuSubmissionPreparationError> {
    let parent = resource.parent_texture();
    if parent.descriptor().dimension() != GpuTextureDimension::D2
        || subresources.mip_level_count() != 1
        || subresources.array_layer_count() != 1
    {
        return render_not_encodable(format!(
            "WGPU render attachments require a 2D view selecting exactly one mip and one array layer; got dimension={:?}, mip_count={}, layer_count={}",
            parent.descriptor().dimension(),
            subresources.mip_level_count(),
            subresources.array_layer_count(),
        ));
    }
    if !matches!(
        subresources.aspect(),
        GpuTextureAspect::All | GpuTextureAspect::Color | GpuTextureAspect::DepthOnly
    ) {
        return render_not_encodable(format!(
            "WGPU render attachment aspect {:?} is not representable for the current RunenGPU formats",
            subresources.aspect()
        ));
    }

    let texture = realized_texture(context, textures, parent)?;
    let logical_view = match resource {
        GpuTextureAccessResource::Texture(_) => None,
        GpuTextureAccessResource::TextureView(view) => {
            let identity = view.diagnostic_identity();
            if let Some(realized) = texture_views.get(&identity) {
                Some(realized.clone())
            } else {
                let realized = context.realize_texture_view(view, &texture).map_err(|error| {
                    GpuSubmissionPreparationError::new(
                        GpuSubmissionPreparationErrorKind::ResourceRealizationFailed,
                        error.to_string(),
                    )
                })?;
                texture_views.insert(identity, realized.clone());
                Some(realized)
            }
        }
    };
    let format = match resource {
        GpuTextureAccessResource::Texture(texture) => texture.descriptor().format(),
        GpuTextureAccessResource::TextureView(view) => view
            .descriptor()
            .format()
            .unwrap_or_else(|| view.descriptor().texture().descriptor().format()),
    };

    Ok(PreparedRenderAttachment {
        texture,
        _logical_view: logical_view,
        format,
        subresources,
    })
}

fn checked_dynamic_offsets(
    group: &GpuValidatedBindGroupBindings,
) -> Result<Vec<u32>, GpuSubmissionPreparationError> {
    let mut offsets = Vec::new();
    for declaration in group.layout().bindings() {
        if !declaration.kind().uses_dynamic_offset() {
            continue;
        }
        let value = group.value(declaration.key().binding()).ok_or_else(|| {
            GpuSubmissionPreparationError::new(
                GpuSubmissionPreparationErrorKind::InternalInvariant,
                format!(
                    "validated dynamic binding {} disappeared before render execution preparation",
                    declaration.key()
                ),
            )
        })?;
        for resource in value.resources() {
            let GpuRuntimeBindingResource::Buffer(binding) = resource else {
                return Err(GpuSubmissionPreparationError::new(
                    GpuSubmissionPreparationErrorKind::InternalInvariant,
                    format!(
                        "validated dynamic binding {} no longer contains a buffer",
                        declaration.key()
                    ),
                ));
            };
            let offset = binding.dynamic_offset().ok_or_else(|| {
                GpuSubmissionPreparationError::new(
                    GpuSubmissionPreparationErrorKind::InternalInvariant,
                    format!(
                        "validated dynamic binding {} lost its per-use offset",
                        declaration.key()
                    ),
                )
            })?;
            offsets.push(u32::try_from(offset).map_err(|_| {
                GpuSubmissionPreparationError::new(
                    GpuSubmissionPreparationErrorKind::DynamicOffsetNotEncodable,
                    format!(
                        "logical dynamic offset {offset} for {} exceeds the private WGPU u32 domain",
                        declaration.key()
                    ),
                )
            })?);
        }
    }
    Ok(offsets)
}

fn realized_buffer(
    context: &GpuContext,
    cache: &mut BTreeMap<GpuWorkResourceId, GpuRealizedBuffer>,
    handle: &crate::plugins::gpu::GpuBufferHandle,
) -> Result<GpuRealizedBuffer, GpuSubmissionPreparationError> {
    let identity = handle.diagnostic_identity();
    if let Some(realized) = cache.get(&identity) {
        return Ok(realized.clone());
    }
    let realized = context.realize_buffer(handle).map_err(|error| {
        GpuSubmissionPreparationError::new(
            GpuSubmissionPreparationErrorKind::ResourceRealizationFailed,
            error.to_string(),
        )
    })?;
    cache.insert(identity, realized.clone());
    Ok(realized)
}

fn realized_texture(
    context: &GpuContext,
    cache: &mut BTreeMap<GpuWorkResourceId, GpuRealizedTexture>,
    handle: &crate::plugins::gpu::GpuTextureHandle,
) -> Result<GpuRealizedTexture, GpuSubmissionPreparationError> {
    let identity = handle.diagnostic_identity();
    if let Some(realized) = cache.get(&identity) {
        return Ok(realized.clone());
    }
    let realized = context.realize_texture(handle).map_err(|error| {
        GpuSubmissionPreparationError::new(
            GpuSubmissionPreparationErrorKind::ResourceRealizationFailed,
            error.to_string(),
        )
    })?;
    cache.insert(identity, realized.clone());
    Ok(realized)
}

fn realized_query_set(
    context: &GpuContext,
    cache: &mut BTreeMap<GpuWorkResourceId, GpuRealizedQuerySet>,
    handle: &crate::plugins::gpu::GpuQuerySetHandle,
) -> Result<GpuRealizedQuerySet, GpuSubmissionPreparationError> {
    let identity = handle.diagnostic_identity();
    if let Some(realized) = cache.get(&identity) {
        return Ok(realized.clone());
    }
    let realized = context.realize_query_set(handle).map_err(|error| {
        GpuSubmissionPreparationError::new(
            GpuSubmissionPreparationErrorKind::ResourceRealizationFailed,
            error.to_string(),
        )
    })?;
    cache.insert(identity, realized.clone());
    Ok(realized)
}

fn preparation_program_binding_failure(
    error: GpuProgramBindingRealizationError,
) -> GpuSubmissionPreparationError {
    let kind = if error.category()
        == GpuProgramBindingRealizationErrorCategory::ContextOrDeviceUnavailableOrLost
    {
        GpuSubmissionPreparationErrorKind::ContextOrDeviceUnavailableOrLost
    } else {
        GpuSubmissionPreparationErrorKind::ProgramBindingRealizationFailed
    };
    GpuSubmissionPreparationError::new(kind, error.to_string())
}

fn preparation_pipeline_failure(
    error: GpuPipelineRealizationError,
) -> GpuSubmissionPreparationError {
    let kind = if error.category()
        == GpuPipelineRealizationErrorCategory::ContextOrDeviceUnavailableOrLost
    {
        GpuSubmissionPreparationErrorKind::ContextOrDeviceUnavailableOrLost
    } else {
        GpuSubmissionPreparationErrorKind::PipelineRealizationFailed
    };
    GpuSubmissionPreparationError::new(kind, error.to_string())
}

fn render_not_encodable<T>(
    detail: impl Into<String>,
) -> Result<T, GpuSubmissionPreparationError> {
    Err(GpuSubmissionPreparationError::new(
        GpuSubmissionPreparationErrorKind::UnsupportedOperation,
        detail,
    ))
}

pub(super) fn encode_render_operation(
    backend: &WgpuContextState,
    encoder: &mut wgpu::CommandEncoder,
    render: &PreparedRenderOperation,
) -> Result<(), GpuSubmissionFailure> {
    let color_views = render
        .color_attachments
        .iter()
        .map(|attachment| attachment.source.create_view())
        .collect::<Vec<_>>();
    let resolve_views = render
        .color_attachments
        .iter()
        .map(|attachment| {
            attachment
                .resolve_target
                .as_ref()
                .map(PreparedRenderAttachment::create_view)
        })
        .collect::<Vec<_>>();
    let depth_view = render
        .depth_stencil_attachment
        .as_ref()
        .map(|attachment| attachment.source.create_view());
    let pipeline_refs = render
        .draws
        .iter()
        .map(|draw| &draw.pipeline)
        .collect::<Vec<_>>();

    if pipeline_refs.is_empty() {
        return encode_render_pass(
            backend,
            encoder,
            render,
            &color_views,
            &resolve_views,
            depth_view.as_ref(),
            &[],
        );
    }

    backend
        .pipeline_realization
        .with_execution_render_pipelines(
            &pipeline_refs,
            &backend.program_binding_realization,
            |pipelines| {
                encode_render_pass(
                    backend,
                    encoder,
                    render,
                    &color_views,
                    &resolve_views,
                    depth_view.as_ref(),
                    pipelines,
                )
            },
        )
        .map_err(submission_pipeline_failure)?
}

impl PreparedRenderAttachment {
    fn create_view(&self) -> TextureView {
        self.texture.record.object.create_view(&TextureViewDescriptor {
            label: Some("RunenGPU G5B render attachment"),
            format: Some(map_texture_format(self.format)),
            dimension: Some(TextureViewDimension::D2),
            usage: None,
            aspect: map_texture_aspect(self.subresources.aspect()),
            base_mip_level: self.subresources.base_mip_level(),
            mip_level_count: Some(1),
            base_array_layer: self.subresources.base_array_layer(),
            array_layer_count: Some(1),
        })
    }
}

fn encode_render_pass(
    backend: &WgpuContextState,
    encoder: &mut wgpu::CommandEncoder,
    render: &PreparedRenderOperation,
    color_views: &[TextureView],
    resolve_views: &[Option<TextureView>],
    depth_view: Option<&TextureView>,
    pipelines: &[&wgpu::RenderPipeline],
) -> Result<(), GpuSubmissionFailure> {
    if pipelines.len() != render.draws.len() {
        return Err(GpuSubmissionFailure::new(
            GpuSubmissionFailureKind::InternalInvariant,
            "prepared render pipeline count no longer matches the prepared draw count",
        ));
    }
    if color_views.len() != render.color_attachments.len()
        || resolve_views.len() != render.color_attachments.len()
    {
        return Err(GpuSubmissionFailure::new(
            GpuSubmissionFailureKind::InternalInvariant,
            "prepared render attachment views no longer match logical attachment count",
        ));
    }

    let color_attachments = render
        .color_attachments
        .iter()
        .enumerate()
        .map(|(index, attachment)| {
            Some(RenderPassColorAttachment {
                view: &color_views[index],
                depth_slice: None,
                resolve_target: resolve_views[index].as_ref(),
                ops: Operations {
                    load: color_load(attachment.load),
                    store: attachment_store(attachment.store),
                },
            })
        })
        .collect::<Vec<_>>();
    let depth_stencil_attachment = match (&render.depth_stencil_attachment, depth_view) {
        (Some(logical), Some(view)) => Some(RenderPassDepthStencilAttachment {
            view,
            depth_ops: match logical.access {
                GpuDepthStencilAccess::ReadOnly => None,
                GpuDepthStencilAccess::ReadWrite => Some(Operations {
                    load: depth_load(logical.load),
                    store: attachment_store(logical.store),
                }),
            },
            stencil_ops: None,
        }),
        (None, None) => None,
        _ => {
            return Err(GpuSubmissionFailure::new(
                GpuSubmissionFailureKind::InternalInvariant,
                "prepared render depth view no longer matches logical depth attachment presence",
            ));
        }
    };
    let timestamp_writes = render
        .timestamp_writes
        .as_ref()
        .map(|writes| RenderPassTimestampWrites {
            query_set: &writes.query_set.record.object,
            beginning_of_pass_write_index: writes.beginning_of_pass,
            end_of_pass_write_index: writes.end_of_pass,
        });

    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("RunenGPU G5B render"),
        color_attachments: &color_attachments,
        depth_stencil_attachment,
        timestamp_writes,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    for (draw, pipeline) in render.draws.iter().zip(pipelines) {
        pass.set_pipeline(pipeline);
        let group_refs = draw
            .bind_groups
            .iter()
            .map(|group| &group.realization)
            .collect::<Vec<_>>();
        backend
            .program_binding_realization
            .with_execution_bind_groups(&group_refs, |objects| {
                for (prepared, object) in draw.bind_groups.iter().zip(objects) {
                    pass.set_bind_group(prepared.index, *object, &prepared.dynamic_offsets);
                }
            })
            .map_err(submission_program_binding_failure)?;

        for vertex in &draw.vertex_buffers {
            pass.set_vertex_buffer(
                vertex.slot,
                vertex.buffer.record.object.slice(vertex.offset..vertex.end),
            );
        }
        if let Some(index) = &draw.index_buffer {
            pass.set_index_buffer(
                index.buffer.record.object.slice(index.offset..index.end),
                map_index_format(index.format),
            );
        }

        let [x, y, width, height, min_depth, max_depth] = draw.logical.viewport().values();
        pass.set_viewport(x, y, width, height, min_depth, max_depth);
        let scissor = draw.logical.scissor();
        pass.set_scissor_rect(scissor.x(), scissor.y(), scissor.width(), scissor.height());
        let [r, g, b, a] = draw.logical.blend_constant().components();
        pass.set_blend_constant(Color { r, g, b, a });
        pass.set_stencil_reference(draw.logical.stencil_reference());

        match draw.logical.draw() {
            GpuDrawIntent::Direct {
                vertices,
                instances,
            } => pass.draw(
                vertices.first()..vertices.end(),
                instances.first()..instances.end(),
            ),
            GpuDrawIntent::Indexed {
                indices,
                base_vertex,
                instances,
            } => pass.draw_indexed(
                indices.first()..indices.end(),
                *base_vertex,
                instances.first()..instances.end(),
            ),
            GpuDrawIntent::Indirect { indexed, .. } => {
                let indirect = draw.indirect_buffer.as_ref().ok_or_else(|| {
                    GpuSubmissionFailure::new(
                        GpuSubmissionFailureKind::InternalInvariant,
                        "prepared indirect render draw lost its realized argument buffer",
                    )
                })?;
                if indirect.indexed != *indexed {
                    return Err(GpuSubmissionFailure::new(
                        GpuSubmissionFailureKind::InternalInvariant,
                        "prepared indirect render draw indexedness no longer matches logical intent",
                    ));
                }
                if indirect.indexed {
                    pass.draw_indexed_indirect(&indirect.buffer.record.object, indirect.offset);
                } else {
                    pass.draw_indirect(&indirect.buffer.record.object, indirect.offset);
                }
            }
        }
    }
    Ok(())
}

fn submission_program_binding_failure(
    error: GpuProgramBindingRealizationError,
) -> GpuSubmissionFailure {
    let kind = match error.category() {
        GpuProgramBindingRealizationErrorCategory::BackendResourceExhaustion => {
            GpuSubmissionFailureKind::BackendResourceExhaustion
        }
        GpuProgramBindingRealizationErrorCategory::ContextOrDeviceUnavailableOrLost => {
            GpuSubmissionFailureKind::ContextOrDeviceUnavailableOrLost
        }
        GpuProgramBindingRealizationErrorCategory::ForeignContext
        | GpuProgramBindingRealizationErrorCategory::StaleDeviceGeneration
        | GpuProgramBindingRealizationErrorCategory::CurrentRenderExecutionBridgeViolation => {
            GpuSubmissionFailureKind::InternalInvariant
        }
        _ => GpuSubmissionFailureKind::BackendValidation,
    };
    GpuSubmissionFailure::new(kind, error.to_string())
}

fn submission_pipeline_failure(error: GpuPipelineRealizationError) -> GpuSubmissionFailure {
    let kind = match error.category() {
        GpuPipelineRealizationErrorCategory::BackendResourceExhaustion => {
            GpuSubmissionFailureKind::BackendResourceExhaustion
        }
        GpuPipelineRealizationErrorCategory::ContextOrDeviceUnavailableOrLost => {
            GpuSubmissionFailureKind::ContextOrDeviceUnavailableOrLost
        }
        GpuPipelineRealizationErrorCategory::ForeignContext
        | GpuPipelineRealizationErrorCategory::StaleDeviceGeneration
        | GpuPipelineRealizationErrorCategory::CurrentRenderExecutionBridgeViolation => {
            GpuSubmissionFailureKind::InternalInvariant
        }
        _ => GpuSubmissionFailureKind::BackendValidation,
    };
    GpuSubmissionFailure::new(kind, error.to_string())
}

fn color_load(load: GpuColorAttachmentLoad) -> LoadOp<Color> {
    match load {
        GpuColorAttachmentLoad::Load => LoadOp::Load,
        GpuColorAttachmentLoad::Clear(value) => {
            let [r, g, b, a] = value.components();
            LoadOp::Clear(Color { r, g, b, a })
        }
    }
}

fn depth_load(load: GpuDepthAttachmentLoad) -> LoadOp<f32> {
    match load {
        GpuDepthAttachmentLoad::Load => LoadOp::Load,
        GpuDepthAttachmentLoad::Clear(value) => LoadOp::Clear(value.value()),
    }
}

const fn attachment_store(store: GpuAttachmentStore) -> StoreOp {
    match store {
        GpuAttachmentStore::Store => StoreOp::Store,
        GpuAttachmentStore::Discard => StoreOp::Discard,
    }
}

const fn map_index_format(value: GpuIndexFormat) -> IndexFormat {
    match value {
        GpuIndexFormat::Uint16 => IndexFormat::Uint16,
        GpuIndexFormat::Uint32 => IndexFormat::Uint32,
    }
}

const fn map_texture_format(value: GpuTextureFormat) -> TextureFormat {
    match value {
        GpuTextureFormat::R8Unorm => TextureFormat::R8Unorm,
        GpuTextureFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
        GpuTextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        GpuTextureFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
        GpuTextureFormat::Bgra8UnormSrgb => TextureFormat::Bgra8UnormSrgb,
        GpuTextureFormat::R32Uint => TextureFormat::R32Uint,
        GpuTextureFormat::Depth32Float => TextureFormat::Depth32Float,
    }
}
