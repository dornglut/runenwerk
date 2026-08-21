use super::canonical_execution::{
    realized_buffer_for_handle, realized_texture_view_for_handle,
    validate_pre_g5b_dynamic_offset_boundary, validate_realized_binding_groups,
    validate_renderer_timestamp_projection,
};
use super::*;
use crate::plugins::gpu::{
    CurrentRenderAttachmentsTerminal, CurrentRenderIndexBufferTerminal,
    CurrentRenderIndirectBufferTerminal, CurrentRenderPipelineBindGroupsTerminal,
    CurrentRenderRenderPipelineTerminal, CurrentRenderTimestampWritesTerminal,
    CurrentRenderVertexBufferTerminal, GpuBufferRange, GpuColorAttachmentLoad,
    GpuDepthAttachmentLoad, GpuDepthStencilAccess, GpuDrawIntent, GpuIndexFormat,
    GpuRealizedBindGroup, GpuRealizedBuffer, GpuRealizedTextureView, GpuRenderColorAttachment,
    GpuRenderDraw, GpuRenderOperation, GpuTextureViewHandle,
};

impl Renderer {
    /// Temporary pre-G5B physical realization of one execution-complete canonical render
    /// operation. The canonical operation owns attachment, pipeline, binding, draw, dynamic-state,
    /// and timestamp meaning; renderer state contributes only already-realized opaque G4 resources
    /// plus legacy diagnostic evidence. No resource is lazily realized during G5 encoding.
    pub(super) fn encode_canonical_render_operation(
        &mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        operation: &GpuRenderOperation,
        prepared: &PreparedPipelinePass,
        gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<EncodedPassEvidence> {
        let [draw] = operation.draws() else {
            bail!(
                "canonical render operation has {} draws; the temporary pre-G5B renderer bridge accepts exactly the one-draw shape authored by the current renderer projection",
                operation.draws().len()
            );
        };
        let [color_attachment] = operation.color_attachments() else {
            bail!(
                "canonical render operation has {} color attachments; the temporary pre-G5B renderer bridge accepts exactly the one-color-attachment shape authored by the current renderer projection",
                operation.color_attachments().len()
            );
        };
        if color_attachment.resolve_target().is_some() {
            bail!(
                "canonical render operation requires multisample resolve execution owned by G5B; the current renderer projection does not author a resolve target"
            );
        }

        let pipeline_key = prepared.bindings.pipeline_key.clone();
        let realized_pipeline = match &prepared.pipeline {
            PreparedFlowPipeline::Render(pipeline) => pipeline,
            PreparedFlowPipeline::Compute(_) => {
                bail!("canonical render operation retained a realized compute pipeline")
            }
        };
        if realized_pipeline.descriptor() != draw.pipeline() {
            bail!("canonical render operation pipeline disagrees with its G4C3 realized pipeline");
        }
        if &prepared.bindings.runtime_bindings != draw.bindings() {
            bail!(
                "canonical render operation bindings disagree with its G4C2 realized binding set"
            );
        }
        validate_pre_g5b_dynamic_offset_boundary("render", draw.bindings())?;
        validate_realized_binding_groups(
            "render",
            draw.bindings(),
            &prepared.bindings.bind_groups,
        )?;
        validate_renderer_timestamp_projection(
            "render",
            operation.timestamp_writes(),
            gpu_timestamp_writes.as_ref(),
        )?;

        let color_view = realized_attachment_view(runtime_resources, color_attachment.source())?;
        let depth_view = operation
            .depth_stencil_attachment()
            .map(|attachment| realized_attachment_view(runtime_resources, attachment.source()))
            .transpose()?;
        let mut attachment_views = vec![color_view];
        if let Some(depth_view) = depth_view {
            attachment_views.push(depth_view);
        }
        let vertex_buffers = realized_vertex_buffers(runtime_resources, draw)?;
        let index_buffer = match draw.index_buffer() {
            Some(binding) => Some(CanonicalIndexBuffer {
                buffer: realized_buffer_for_handle("render", runtime_resources, binding.buffer())?,
                range: binding.range(),
                format: binding.format(),
            }),
            None => None,
        };
        let indirect_buffer = match draw.draw() {
            GpuDrawIntent::Indirect {
                arguments,
                range,
                indexed,
            } => Some(CanonicalIndirectBuffer {
                buffer: realized_buffer_for_handle("render", runtime_resources, arguments)?,
                byte_offset: range.offset(),
                indexed: *indexed,
            }),
            GpuDrawIntent::Direct { .. } | GpuDrawIntent::Indexed { .. } => None,
        };

        let mut encode_result = Ok(());
        context
            .current_render_execution_bridge()
            .for_render_pipeline(
                realized_pipeline,
                EncodeCanonicalRenderPipeline {
                    context,
                    encoder,
                    attachment_views: &attachment_views,
                    bind_groups: &prepared.bindings.bind_groups,
                    operation,
                    draw,
                    color_attachment,
                    vertex_buffers: &vertex_buffers,
                    index_buffer: index_buffer.as_ref(),
                    indirect_buffer: indirect_buffer.as_ref(),
                    gpu_timestamp_writes,
                    result: &mut encode_result,
                },
            )?;
        encode_result?;

        Ok(EncodedPassEvidence {
            dispatch_workgroups: None,
            shader_id: prepared.shader_id.clone(),
            shader_revision: prepared.shader_revision,
            fallback_used: prepared.fallback_used,
            pipeline_key: Some(pipeline_key),
        })
    }
}

fn realized_attachment_view<'a>(
    runtime_resources: &'a FlowRuntimeResources,
    view: &GpuTextureViewHandle,
) -> Result<&'a GpuRealizedTextureView> {
    realized_texture_view_for_handle("render", runtime_resources, view)
}

fn realized_vertex_buffers<'a>(
    runtime_resources: &'a FlowRuntimeResources,
    draw: &GpuRenderDraw,
) -> Result<Vec<CanonicalVertexBuffer<'a>>> {
    draw.vertex_buffers()
        .iter()
        .map(|binding| {
            Ok(CanonicalVertexBuffer {
                slot: binding.slot(),
                buffer: realized_buffer_for_handle("render", runtime_resources, binding.buffer())?,
                range: binding.range(),
            })
        })
        .collect()
}

struct CanonicalVertexBuffer<'a> {
    slot: u32,
    buffer: &'a GpuRealizedBuffer,
    range: GpuBufferRange,
}

struct CanonicalIndexBuffer<'a> {
    buffer: &'a GpuRealizedBuffer,
    range: GpuBufferRange,
    format: GpuIndexFormat,
}

struct CanonicalIndirectBuffer<'a> {
    buffer: &'a GpuRealizedBuffer,
    byte_offset: u64,
    indexed: bool,
}

struct EncodeCanonicalRenderPipeline<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    attachment_views: &'a [&'a GpuRealizedTextureView],
    bind_groups: &'a [GpuRealizedBindGroup],
    operation: &'a GpuRenderOperation,
    draw: &'a GpuRenderDraw,
    color_attachment: &'a GpuRenderColorAttachment,
    vertex_buffers: &'a [CanonicalVertexBuffer<'a>],
    index_buffer: Option<&'a CanonicalIndexBuffer<'a>>,
    indirect_buffer: Option<&'a CanonicalIndirectBuffer<'a>>,
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    result: &'a mut Result<()>,
}

impl CurrentRenderRenderPipelineTerminal for EncodeCanonicalRenderPipeline<'_> {
    fn use_render_pipeline(self, pipeline: &RenderPipeline) {
        let bridge_result = self
            .context
            .current_render_execution_bridge()
            .for_pass_attachments(
                self.attachment_views,
                EncodeCanonicalRenderAttachments {
                    context: self.context,
                    encoder: self.encoder,
                    pipeline,
                    bind_groups: self.bind_groups,
                    operation: self.operation,
                    draw: self.draw,
                    color_attachment: self.color_attachment,
                    vertex_buffers: self.vertex_buffers,
                    index_buffer: self.index_buffer,
                    indirect_buffer: self.indirect_buffer,
                    gpu_timestamp_writes: self.gpu_timestamp_writes,
                    result: self.result,
                },
            );
        if let Err(error) = bridge_result {
            *self.result = Err(error.into());
        }
    }
}

struct EncodeCanonicalRenderAttachments<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    pipeline: &'a RenderPipeline,
    bind_groups: &'a [GpuRealizedBindGroup],
    operation: &'a GpuRenderOperation,
    draw: &'a GpuRenderDraw,
    color_attachment: &'a GpuRenderColorAttachment,
    vertex_buffers: &'a [CanonicalVertexBuffer<'a>],
    index_buffer: Option<&'a CanonicalIndexBuffer<'a>>,
    indirect_buffer: Option<&'a CanonicalIndirectBuffer<'a>>,
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    result: &'a mut Result<()>,
}

impl CurrentRenderAttachmentsTerminal for EncodeCanonicalRenderAttachments<'_> {
    fn encode_with_attachments(self, views: &[&TextureView]) {
        let expected_views = if self.operation.depth_stencil_attachment().is_some() {
            2
        } else {
            1
        };
        if views.len() != expected_views {
            *self.result = Err(anyhow::anyhow!(
                "canonical render operation expected {expected_views} realized attachment views but execution bridge supplied {}",
                views.len()
            ));
            return;
        }
        let color_view = views[0];
        let depth_view = self.operation.depth_stencil_attachment().map(|_| views[1]);
        let operation = CanonicalRenderPassOperation {
            encoder: self.encoder,
            pipeline: self.pipeline,
            bind_groups: self.bind_groups,
            logical_operation: self.operation,
            draw: self.draw,
            color_attachment: self.color_attachment,
            color_view,
            depth_view,
            vertex_buffers: self.vertex_buffers,
            index_buffer: self.index_buffer,
            indirect_buffer: self.indirect_buffer,
        };
        if let Some(writes) = self.gpu_timestamp_writes {
            let mut nested_result = Ok(());
            let bridge_result = self
                .context
                .current_render_execution_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedCanonicalRenderPass {
                        context: self.context,
                        operation,
                        indices: writes.indices,
                        result: &mut nested_result,
                    },
                );
            *self.result = match bridge_result {
                Ok(()) => nested_result,
                Err(error) => Err(error.into()),
            };
        } else {
            *self.result = operation.encode(self.context, None);
        }
    }
}

struct CanonicalRenderPassOperation<'a> {
    encoder: &'a mut CommandEncoder,
    pipeline: &'a RenderPipeline,
    bind_groups: &'a [GpuRealizedBindGroup],
    logical_operation: &'a GpuRenderOperation,
    draw: &'a GpuRenderDraw,
    color_attachment: &'a GpuRenderColorAttachment,
    color_view: &'a TextureView,
    depth_view: Option<&'a TextureView>,
    vertex_buffers: &'a [CanonicalVertexBuffer<'a>],
    index_buffer: Option<&'a CanonicalIndexBuffer<'a>>,
    indirect_buffer: Option<&'a CanonicalIndirectBuffer<'a>>,
}

impl CanonicalRenderPassOperation<'_> {
    fn encode(
        self,
        context: &GpuContext,
        timestamp: Option<(&QuerySet, GpuPassTimestampIndices)>,
    ) -> Result<()> {
        let color_attachment = Some(RenderPassColorAttachment {
            view: self.color_view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: color_load(self.color_attachment.load()),
                store: attachment_store(self.color_attachment.store()),
            },
        });
        let depth_stencil_attachment = match (
            self.logical_operation.depth_stencil_attachment(),
            self.depth_view,
        ) {
            (Some(logical), Some(view)) => Some(RenderPassDepthStencilAttachment {
                view,
                depth_ops: match logical.access() {
                    GpuDepthStencilAccess::ReadOnly => None,
                    GpuDepthStencilAccess::ReadWrite => Some(Operations {
                        load: depth_load(logical.load()),
                        store: attachment_store(logical.store()),
                    }),
                },
                stencil_ops: None,
            }),
            (None, None) => None,
            _ => bail!(
                "canonical render operation depth attachment realization disagrees with logical attachment presence"
            ),
        };
        let timestamp_writes = timestamp.map(|(query_set, indices)| RenderPassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(indices.begin),
            end_of_pass_write_index: Some(indices.end),
        });
        let mut pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("runengpu_canonical_render_pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment,
            timestamp_writes,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(self.pipeline);
        for bind_group in self.bind_groups {
            let index = bind_group.layout_descriptor().group();
            context
                .current_render_execution_bridge()
                .for_pipeline_bind_groups(
                    &[bind_group],
                    SetCanonicalRenderBindGroup {
                        pass: &mut pass,
                        index,
                    },
                )?;
        }
        for vertex in self.vertex_buffers {
            context
                .current_render_execution_bridge()
                .for_vertex_buffer(
                    vertex.buffer,
                    SetCanonicalVertexBuffer {
                        pass: &mut pass,
                        slot: vertex.slot,
                        range: vertex.range,
                    },
                )?;
        }
        if let Some(index) = self.index_buffer {
            context.current_render_execution_bridge().for_index_buffer(
                index.buffer,
                SetCanonicalIndexBuffer {
                    pass: &mut pass,
                    range: index.range,
                    format: index.format,
                },
            )?;
        }

        let [x, y, width, height, min_depth, max_depth] = self.draw.viewport().values();
        pass.set_viewport(x, y, width, height, min_depth, max_depth);
        let scissor = self.draw.scissor();
        pass.set_scissor_rect(scissor.x(), scissor.y(), scissor.width(), scissor.height());
        let [r, g, b, a] = self.draw.blend_constant().components();
        pass.set_blend_constant(Color { r, g, b, a });
        pass.set_stencil_reference(self.draw.stencil_reference());

        match self.draw.draw() {
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
                let indirect = self.indirect_buffer.ok_or_else(|| {
                    anyhow::anyhow!(
                        "canonical indirect render draw has no already-realized argument buffer"
                    )
                })?;
                if indirect.indexed != *indexed {
                    bail!(
                        "canonical indirect render realization disagrees with logical indexedness"
                    );
                }
                context
                    .current_render_execution_bridge()
                    .for_indirect_buffer(
                        indirect.buffer,
                        DrawCanonicalRenderIndirect {
                            pass: &mut pass,
                            byte_offset: indirect.byte_offset,
                            indexed: indirect.indexed,
                        },
                    )?;
            }
        }
        Ok(())
    }
}

struct EncodeTimestampedCanonicalRenderPass<'a> {
    context: &'a GpuContext,
    operation: CanonicalRenderPassOperation<'a>,
    indices: GpuPassTimestampIndices,
    result: &'a mut Result<()>,
}

impl CurrentRenderTimestampWritesTerminal for EncodeTimestampedCanonicalRenderPass<'_> {
    fn write_timestamps(self, query_set: &QuerySet) {
        *self.result = self
            .operation
            .encode(self.context, Some((query_set, self.indices)));
    }
}

struct SetCanonicalRenderBindGroup<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    index: u32,
}

impl CurrentRenderPipelineBindGroupsTerminal for SetCanonicalRenderBindGroup<'_, '_> {
    fn bind_groups(self, groups: &[&BindGroup]) {
        debug_assert_eq!(groups.len(), 1, "one terminal binds one canonical group");
        self.pass.set_bind_group(self.index, groups[0], &[]);
    }
}

struct SetCanonicalVertexBuffer<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    slot: u32,
    range: GpuBufferRange,
}

impl CurrentRenderVertexBufferTerminal for SetCanonicalVertexBuffer<'_, '_> {
    fn use_vertex_buffer(self, buffer: &Buffer) {
        self.pass.set_vertex_buffer(
            self.slot,
            buffer.slice(self.range.offset()..self.range.end()),
        );
    }
}

struct SetCanonicalIndexBuffer<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    range: GpuBufferRange,
    format: GpuIndexFormat,
}

impl CurrentRenderIndexBufferTerminal for SetCanonicalIndexBuffer<'_, '_> {
    fn use_index_buffer(self, buffer: &Buffer) {
        self.pass.set_index_buffer(
            buffer.slice(self.range.offset()..self.range.end()),
            match self.format {
                GpuIndexFormat::Uint16 => IndexFormat::Uint16,
                GpuIndexFormat::Uint32 => IndexFormat::Uint32,
            },
        );
    }
}

struct DrawCanonicalRenderIndirect<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    byte_offset: u64,
    indexed: bool,
}

impl CurrentRenderIndirectBufferTerminal for DrawCanonicalRenderIndirect<'_, '_> {
    fn use_indirect_buffer(self, buffer: &Buffer) {
        if self.indexed {
            self.pass.draw_indexed_indirect(buffer, self.byte_offset);
        } else {
            self.pass.draw_indirect(buffer, self.byte_offset);
        }
    }
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

const fn attachment_store(store: crate::plugins::gpu::GpuAttachmentStore) -> StoreOp {
    match store {
        crate::plugins::gpu::GpuAttachmentStore::Store => StoreOp::Store,
        crate::plugins::gpu::GpuAttachmentStore::Discard => StoreOp::Discard,
    }
}
