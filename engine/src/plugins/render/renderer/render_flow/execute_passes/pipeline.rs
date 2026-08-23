use super::super::*;
use crate::plugins::gpu::{
    CurrentRenderAttachmentsTerminal, CurrentRenderComputePipelineTerminal,
    CurrentRenderIndexBufferTerminal, CurrentRenderIndirectBufferTerminal,
    CurrentRenderPipelineBindGroupsTerminal, CurrentRenderRenderPipelineTerminal,
    CurrentRenderTimestampWritesTerminal, CurrentRenderVertexBufferTerminal,
    GpuAdmittedProgramSource, GpuCapabilityRequirements, GpuProgramSourceKey,
    GpuProgramSourceProvenance, GpuRealizedBindGroup, GpuRealizedBuffer,
    GpuSpecializationDeclaration, GpuSpecializationEntry, GpuSpecializationKey,
    GpuSpecializationSchema, GpuSpecializationValueSet, GpuWorkResourceId,
};
use crate::plugins::render::graph::{CompiledDrawSource, CompiledResourceRef};
use crate::plugins::render::pipelines::FlowPassPipelineDescriptor;
use crate::plugins::render::{RenderDepthPolicy, RenderIndirectDrawArgsKind, RenderShaderConstant};

impl Renderer {
    /// First half of the renderer's two-phase integration: realize every G4C1/G4C2/G4C3
    /// dependency while no raw device/queue or physical surface object is required.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::plugins::render::renderer::render_flow) fn realize_compiled_pass(
        &mut self,
        context: &GpuContext,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        flow_inputs: &PreparedFlowInputs,
        pass: &CompiledPassExecutionPlan,
        shader_registry: &ShaderRegistryResource,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<Option<PreparedPipelinePass>> {
        match pass {
            CompiledPassExecutionPlan::Compute(value) => {
                let shader = resolve_shader_material(
                    value.shader.as_ref(),
                    shader_registry,
                    DEFAULT_COMPUTE_SHADER,
                    "builtin:compute",
                );
                let specialization =
                    compute_specialization_from_constants(&value.shader_constants)?;
                flow_inputs
                    .projected_dispatch_workgroups
                    .get(&value.pass_id)
                    .copied()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "missing prepared dispatch for pass '{}' in flow '{}'",
                            value.pass_id,
                            flow.flow_id
                        )
                    })?;
                let admitted_source = admit_resolved_program_source(
                    &mut self.flow_pipeline_cache,
                    &shader,
                    format!("compute pass {}", value.pass_id),
                )?;
                let bindings = self.resolve_compiled_bind_group(
                    context,
                    packet,
                    flow,
                    value.pass_id,
                    FlowPassKind::Compute,
                    value.feature_id,
                    &admitted_source,
                    specialization,
                    &value.bindings,
                    ShaderStages::COMPUTE,
                    true,
                    Vec::new(),
                    None,
                    runtime_resources,
                )?;
                let pipeline = match &bindings.pipeline_key.pipeline_descriptor {
                    FlowPassPipelineDescriptor::Compute(descriptor) => {
                        PreparedFlowPipeline::Compute(pollster::block_on(
                            context.realize_compute_pipeline(
                                descriptor,
                                &bindings.program,
                                &bindings.pipeline_layout,
                            ),
                        )?)
                    }
                    FlowPassPipelineDescriptor::Render(_) => {
                        bail!(
                            "compute pass '{}' resolved a render pipeline descriptor",
                            value.pass_id
                        )
                    }
                };
                Ok(Some(PreparedPipelinePass {
                    bindings,
                    pipeline,
                    shader_id: shader.shader_id,
                    shader_revision: shader.revision,
                    fallback_used: shader.fallback_used,
                }))
            }
            CompiledPassExecutionPlan::Fullscreen(value) => {
                if !value.draw_buffers.vertex_buffers.is_empty()
                    || !value.draw_buffers.index_buffers.is_empty()
                    || !value.draw_buffers.instance_buffers.is_empty()
                    || !value.draw_buffers.indirect_buffers.is_empty()
                {
                    bail!(
                        "fullscreen pass '{}' cannot bind graphics vertex/index/instance/indirect buffers",
                        value.pass_id
                    );
                }
                let color_format = self.resolve_color_target_format_from_plan(
                    runtime_resources,
                    value.pass_id,
                    &value.targets,
                    packet.surface_format,
                )?;
                let shader = resolve_shader_material_for_packet(
                    value.shader.as_ref(),
                    packet,
                    shader_registry,
                    DEFAULT_FULLSCREEN_SHADER,
                    "builtin:fullscreen",
                );
                reject_material_shader_fallback(
                    value.feature_id,
                    value.shader.as_ref(),
                    value.pass_id,
                    &shader,
                )?;
                reject_unresident_material_textures(
                    packet,
                    value.feature_id,
                    value.shader.as_ref(),
                    value.pass_id,
                )?;
                let admitted_source = admit_resolved_program_source(
                    &mut self.flow_pipeline_cache,
                    &shader,
                    format!("fullscreen pass {}", value.pass_id),
                )?;
                let bindings = self.resolve_compiled_bind_group(
                    context,
                    packet,
                    flow,
                    value.pass_id,
                    FlowPassKind::Fullscreen,
                    value.feature_id,
                    &admitted_source,
                    empty_specialization_value_set()?,
                    &value.bindings,
                    ShaderStages::VERTEX_FRAGMENT,
                    true,
                    vec![color_format],
                    None,
                    runtime_resources,
                )?;
                let pipeline = match &bindings.pipeline_key.pipeline_descriptor {
                    FlowPassPipelineDescriptor::Render(descriptor) => PreparedFlowPipeline::Render(
                        pollster::block_on(context.realize_render_pipeline(
                            descriptor,
                            &bindings.program,
                            &bindings.pipeline_layout,
                        ))?,
                    ),
                    FlowPassPipelineDescriptor::Compute(_) => {
                        bail!(
                            "fullscreen pass '{}' resolved a compute pipeline descriptor",
                            value.pass_id
                        )
                    }
                };
                Ok(Some(PreparedPipelinePass {
                    bindings,
                    pipeline,
                    shader_id: shader.shader_id,
                    shader_revision: shader.revision,
                    fallback_used: shader.fallback_used,
                }))
            }
            CompiledPassExecutionPlan::Graphics(value) => {
                let color_format = self.resolve_color_target_format_from_plan(
                    runtime_resources,
                    value.pass_id,
                    &value.targets,
                    packet.surface_format,
                )?;
                let depth_format = self.resolve_depth_target_format_from_plan(
                    runtime_resources,
                    value.pass_id,
                    &value.targets,
                )?;
                let shader = resolve_shader_material_for_packet(
                    value.shader.as_ref(),
                    packet,
                    shader_registry,
                    DEFAULT_GRAPHICS_SHADER,
                    "builtin:graphics",
                );
                reject_material_shader_fallback(
                    value.feature_id,
                    value.shader.as_ref(),
                    value.pass_id,
                    &shader,
                )?;
                reject_unresident_material_textures(
                    packet,
                    value.feature_id,
                    value.shader.as_ref(),
                    value.pass_id,
                )?;
                let admitted_source = admit_resolved_program_source(
                    &mut self.flow_pipeline_cache,
                    &shader,
                    format!("graphics pass {}", value.pass_id),
                )?;
                let bindings = self.resolve_compiled_bind_group(
                    context,
                    packet,
                    flow,
                    value.pass_id,
                    FlowPassKind::Graphics,
                    value.feature_id,
                    &admitted_source,
                    empty_specialization_value_set()?,
                    &value.bindings,
                    ShaderStages::VERTEX_FRAGMENT,
                    true,
                    vec![color_format],
                    depth_format,
                    runtime_resources,
                )?;
                let pipeline = match &bindings.pipeline_key.pipeline_descriptor {
                    FlowPassPipelineDescriptor::Render(descriptor) => PreparedFlowPipeline::Render(
                        pollster::block_on(context.realize_render_pipeline(
                            descriptor,
                            &bindings.program,
                            &bindings.pipeline_layout,
                        ))?,
                    ),
                    FlowPassPipelineDescriptor::Compute(_) => {
                        bail!(
                            "graphics pass '{}' resolved a compute pipeline descriptor",
                            value.pass_id
                        )
                    }
                };
                Ok(Some(PreparedPipelinePass {
                    bindings,
                    pipeline,
                    shader_id: shader.shader_id,
                    shader_revision: shader.revision,
                    fallback_used: shader.fallback_used,
                }))
            }
            CompiledPassExecutionPlan::Copy(_)
            | CompiledPassExecutionPlan::Present(_)
            | CompiledPassExecutionPlan::BuiltinUiComposite(_) => Ok(None),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::plugins::render::renderer::render_flow) fn encode_compiled_pass(
        &mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        flow_inputs: &PreparedFlowInputs,
        pass: &CompiledPassExecutionPlan,
        runtime_resources: &FlowRuntimeResources,
        prepared_pipeline: Option<&PreparedPipelinePass>,
        gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    ) -> Result<EncodedPassEvidence> {
        match pass {
            CompiledPassExecutionPlan::Compute(value) => {
                let dispatch = flow_inputs
                    .projected_dispatch_workgroups
                    .get(&value.pass_id)
                    .copied()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "missing prepared dispatch for pass '{}' in flow '{}'",
                            value.pass_id,
                            flow.flow_id
                        )
                    })?;
                self.encode_compute_pass(
                    context,
                    encoder,
                    value,
                    prepared_pipeline.ok_or_else(|| {
                        anyhow::anyhow!(
                            "compute pass '{}' reached G5 without G4C3 realization",
                            value.pass_id
                        )
                    })?,
                    dispatch,
                    gpu_timestamp_writes,
                )
                .map(|value| EncodedPassEvidence {
                    dispatch_workgroups: value.dispatch_workgroups,
                    shader_id: value.shader_id,
                    shader_revision: value.shader_revision,
                    fallback_used: value.fallback_used,
                    pipeline_key: Some(value.pipeline_key),
                })
            }
            CompiledPassExecutionPlan::Fullscreen(value) => self
                .encode_fullscreen_pass(
                    context,
                    encoder,
                    frame_view,
                    packet,
                    runtime_resources,
                    value,
                    prepared_pipeline.ok_or_else(|| {
                        anyhow::anyhow!(
                            "fullscreen pass '{}' reached G5 without G4C3 realization",
                            value.pass_id
                        )
                    })?,
                    gpu_timestamp_writes,
                )
                .map(|value| EncodedPassEvidence {
                    dispatch_workgroups: None,
                    shader_id: value.shader_id,
                    shader_revision: value.shader_revision,
                    fallback_used: value.fallback_used,
                    pipeline_key: Some(value.pipeline_key),
                }),
            CompiledPassExecutionPlan::Graphics(value) => self
                .encode_graphics_pass(
                    context,
                    encoder,
                    frame_view,
                    packet,
                    runtime_resources,
                    value,
                    prepared_pipeline.ok_or_else(|| {
                        anyhow::anyhow!(
                            "graphics pass '{}' reached G5 without G4C3 realization",
                            value.pass_id
                        )
                    })?,
                    gpu_timestamp_writes,
                )
                .map(|value| EncodedPassEvidence {
                    dispatch_workgroups: None,
                    shader_id: value.shader_id,
                    shader_revision: value.shader_revision,
                    fallback_used: value.fallback_used,
                    pipeline_key: Some(value.pipeline_key),
                }),
            CompiledPassExecutionPlan::Copy(value) => self
                .encode_copy_pass(
                    context,
                    encoder,
                    frame_texture,
                    packet,
                    runtime_resources,
                    value,
                )
                .map(|()| EncodedPassEvidence {
                    dispatch_workgroups: None,
                    shader_id: "builtin:copy".to_string(),
                    shader_revision: 0,
                    fallback_used: false,
                    pipeline_key: None,
                }),
            CompiledPassExecutionPlan::Present(value) => self
                .encode_present_pass(
                    context,
                    encoder,
                    frame_texture,
                    packet,
                    runtime_resources,
                    value,
                )
                .map(|()| EncodedPassEvidence {
                    dispatch_workgroups: None,
                    shader_id: "builtin:present".to_string(),
                    shader_revision: 0,
                    fallback_used: false,
                    pipeline_key: None,
                }),
            CompiledPassExecutionPlan::BuiltinUiComposite(_) => {
                self.encode_ui_pass(
                    context,
                    encoder,
                    frame_view,
                    &packet.prepared_ui,
                    &packet.viewport_surface_bindings,
                    &packet.ui_dynamic_bind_groups.viewport,
                    &packet.ui_dynamic_bind_groups.product_surface,
                    gpu_timestamp_writes,
                )?;
                Ok(EncodedPassEvidence {
                    dispatch_workgroups: None,
                    shader_id: "builtin:ui_composite".to_string(),
                    shader_revision: 0,
                    fallback_used: false,
                    pipeline_key: None,
                })
            }
        }
    }

    fn encode_compute_pass(
        &mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        pass: &CompiledComputeExecutionPlan,
        prepared: &PreparedPipelinePass,
        dispatch: [u32; 3],
        gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    ) -> Result<EncodedPipelinePass> {
        let pipeline_key = prepared.bindings.pipeline_key.clone();
        let pipeline = match &prepared.pipeline {
            PreparedFlowPipeline::Compute(pipeline) => pipeline,
            PreparedFlowPipeline::Render(_) => {
                bail!("compute pass '{}' retained a render pipeline", pass.pass_id)
            }
        };
        let mut encode_result = Ok(());
        context
            .current_render_execution_bridge()
            .for_compute_pipeline(
                pipeline,
                EncodeComputePipeline {
                    context,
                    encoder,
                    bind_groups: &prepared.bindings.bind_groups,
                    dispatch,
                    gpu_timestamp_writes,
                    result: &mut encode_result,
                },
            )?;
        encode_result?;
        Ok(EncodedPipelinePass {
            dispatch_workgroups: Some(dispatch),
            shader_id: prepared.shader_id.clone(),
            shader_revision: prepared.shader_revision,
            fallback_used: prepared.fallback_used,
            pipeline_key,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_fullscreen_pass(
        &mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        runtime_resources: &FlowRuntimeResources,
        plan: &CompiledRasterExecutionPlan,
        prepared: &PreparedPipelinePass,
        gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    ) -> Result<EncodedPipelinePass> {
        let color_target = self.resolve_color_target_from_plan(
            runtime_resources,
            plan.pass_id,
            &plan.targets,
            frame_view,
            packet.surface_format,
        )?;
        let pipeline_key = prepared.bindings.pipeline_key.clone();
        let pipeline = match &prepared.pipeline {
            PreparedFlowPipeline::Render(pipeline) => pipeline,
            PreparedFlowPipeline::Compute(_) => {
                bail!(
                    "fullscreen pass '{}' retained a compute pipeline",
                    plan.pass_id
                )
            }
        };
        let load = match plan.clear_color {
            Some(color) => LoadOp::Clear(Color {
                r: color[0] as f64,
                g: color[1] as f64,
                b: color[2] as f64,
                a: color[3] as f64,
            }),
            None => LoadOp::Load,
        };
        let (surface_view, realized_views) = match &color_target.view {
            RuntimeTextureView::Surface(view) => (Some(*view), Vec::new()),
            RuntimeTextureView::Realized(view) => (None, vec![view]),
        };
        let mut encode_result = Ok(());
        context
            .current_render_execution_bridge()
            .for_render_pipeline(
                pipeline,
                EncodeFullscreenPipeline {
                    context,
                    encoder,
                    surface_view,
                    realized_views: &realized_views,
                    bind_groups: &prepared.bindings.bind_groups,
                    load,
                    gpu_timestamp_writes,
                    result: &mut encode_result,
                },
            )?;
        encode_result?;
        Ok(EncodedPipelinePass {
            dispatch_workgroups: None,
            shader_id: prepared.shader_id.clone(),
            shader_revision: prepared.shader_revision,
            fallback_used: prepared.fallback_used,
            pipeline_key,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_graphics_pass(
        &mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        runtime_resources: &FlowRuntimeResources,
        plan: &CompiledRasterExecutionPlan,
        prepared: &PreparedPipelinePass,
        gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    ) -> Result<EncodedPipelinePass> {
        let color_target = self.resolve_color_target_from_plan(
            runtime_resources,
            plan.pass_id,
            &plan.targets,
            frame_view,
            packet.surface_format,
        )?;
        let depth_target =
            self.resolve_depth_target_from_plan(runtime_resources, plan.pass_id, &plan.targets)?;
        let pipeline_key = prepared.bindings.pipeline_key.clone();
        let pipeline = match &prepared.pipeline {
            PreparedFlowPipeline::Render(pipeline) => pipeline,
            PreparedFlowPipeline::Compute(_) => {
                bail!(
                    "graphics pass '{}' retained a compute pipeline",
                    plan.pass_id
                )
            }
        };
        let load = match plan.clear_color {
            Some(color) => LoadOp::Clear(Color {
                r: color[0] as f64,
                g: color[1] as f64,
                b: color[2] as f64,
                a: color[3] as f64,
            }),
            None => LoadOp::Load,
        };
        let mut vertex_buffers = Vec::new();
        for binding in &plan.draw_buffers.vertex_buffers {
            let buffer =
                runtime_resources.resolve_storage_buffer_ref(plan.pass_id, &binding.resource)?;
            vertex_buffers.push((binding.layout.slot, buffer.buffer));
        }
        for (resource, layout) in plan
            .draw_buffers
            .instance_buffers
            .iter()
            .zip(plan.draw_buffers.instance_buffer_layouts.iter())
        {
            let buffer = runtime_resources.resolve_storage_buffer_ref(plan.pass_id, resource)?;
            vertex_buffers.push((layout.slot, buffer.buffer));
        }
        let index_buffer = match plan.draw_buffers.index_buffers.as_slice() {
            [] => None,
            [only] => Some(runtime_resources.resolve_storage_buffer_ref(plan.pass_id, only)?),
            _ => bail!(
                "graphics pass '{}' declares multiple index_buffer(...) resources; runtime currently supports exactly one",
                plan.pass_id
            ),
        };
        let draw = plan.draw.ok_or_else(|| {
            anyhow::anyhow!(
                "graphics pass '{}' is missing draw parameters in execution plan",
                plan.pass_id
            )
        })?;
        let indirect_buffer = match draw.source {
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
                            "graphics pass '{}' indirect draw references args buffer '{:?}' that is not in the compiled indirect buffer set",
                            plan.pass_id,
                            args_buffer
                        )
                    })?;
                Some((
                    runtime_resources.resolve_storage_buffer_ref(plan.pass_id, resource)?,
                    args_kind,
                    byte_offset,
                ))
            }
            CompiledDrawSource::Direct => None,
        };
        let vertex_range = draw.first_vertex..draw.first_vertex + draw.vertex_count;
        let instance_range = draw.first_instance..draw.first_instance + draw.instance_count;
        let draw = match (index_buffer.as_ref(), indirect_buffer) {
            (Some(_), Some((indirect, RenderIndirectDrawArgsKind::DrawIndexed, byte_offset))) => {
                GraphicsDraw::Indirect {
                    buffer: indirect.buffer,
                    byte_offset,
                    indexed: true,
                }
            }
            (None, Some((indirect, RenderIndirectDrawArgsKind::Draw, byte_offset))) => {
                GraphicsDraw::Indirect {
                    buffer: indirect.buffer,
                    byte_offset,
                    indexed: false,
                }
            }
            (Some(_), Some((_indirect, RenderIndirectDrawArgsKind::Draw, _))) => bail!(
                "graphics pass '{}' indexed indirect draw uses non-indexed indirect args",
                plan.pass_id
            ),
            (None, Some((_indirect, RenderIndirectDrawArgsKind::DrawIndexed, _))) => bail!(
                "graphics pass '{}' non-indexed indirect draw uses indexed indirect args",
                plan.pass_id
            ),
            (Some(_), None) => GraphicsDraw::Direct {
                indexed: true,
                vertex_range,
                instance_range,
            },
            (None, None) => GraphicsDraw::Direct {
                indexed: false,
                vertex_range,
                instance_range,
            },
        };
        let index_buffer = index_buffer.as_ref().map(|buffer| buffer.buffer);
        let depth_target = depth_target.as_ref().filter(|_| {
            !matches!(
                plan.raster_state.state.depth_policy,
                RenderDepthPolicy::Disabled
            )
        });
        let surface_color_view = match &color_target.view {
            RuntimeTextureView::Surface(view) => Some(*view),
            RuntimeTextureView::Realized(_) => None,
        };
        let mut attachment_views = Vec::new();
        if let RuntimeTextureView::Realized(view) = &color_target.view {
            attachment_views.push(view);
        }
        if let Some(depth) = depth_target {
            attachment_views.push(&depth.view);
        }
        let mut encode_result = Ok(());
        context
            .current_render_execution_bridge()
            .for_render_pipeline(
                pipeline,
                EncodeGraphicsPipeline {
                    context,
                    encoder,
                    surface_color_view,
                    color_is_realized: surface_color_view.is_none(),
                    has_depth: depth_target.is_some(),
                    attachment_views: &attachment_views,
                    bind_groups: &prepared.bindings.bind_groups,
                    load,
                    gpu_timestamp_writes,
                    vertex_buffers: &vertex_buffers,
                    index_buffer,
                    draw,
                    result: &mut encode_result,
                },
            )?;
        encode_result?;
        Ok(EncodedPipelinePass {
            dispatch_workgroups: None,
            shader_id: prepared.shader_id.clone(),
            shader_revision: prepared.shader_revision,
            fallback_used: prepared.fallback_used,
            pipeline_key,
        })
    }
}

struct EncodeComputePipeline<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    bind_groups: &'a [GpuRealizedBindGroup],
    dispatch: [u32; 3],
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    result: &'a mut Result<()>,
}

impl CurrentRenderComputePipelineTerminal for EncodeComputePipeline<'_> {
    fn use_compute_pipeline(self, pipeline: &ComputePipeline) {
        let operation = EncodeComputePass {
            context: self.context,
            encoder: self.encoder,
            pipeline,
            bind_groups: self.bind_groups,
            dispatch: self.dispatch,
        };
        if let Some(writes) = self.gpu_timestamp_writes {
            let mut nested_result = Ok(());
            let bridge_result = self
                .context
                .current_render_execution_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedComputePass {
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
            *self.result = operation.encode(None);
        }
    }
}

struct EncodeComputePass<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    pipeline: &'a ComputePipeline,
    bind_groups: &'a [GpuRealizedBindGroup],
    dispatch: [u32; 3],
}

impl EncodeComputePass<'_> {
    fn encode(self, timestamp: Option<(&QuerySet, GpuPassTimestampIndices)>) -> Result<()> {
        let timestamp_writes = timestamp.map(|(query_set, indices)| ComputePassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(indices.begin),
            end_of_pass_write_index: Some(indices.end),
        });
        let mut pass = self.encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("engine_compiled_compute_pass"),
            timestamp_writes,
        });
        pass.set_pipeline(self.pipeline);
        for bind_group in self.bind_groups {
            let index = bind_group.layout_descriptor().group();
            self.context
                .current_render_execution_bridge()
                .for_pipeline_bind_groups(
                    &[bind_group],
                    SetComputeBindGroup {
                        pass: &mut pass,
                        index,
                    },
                )?;
        }
        pass.dispatch_workgroups(self.dispatch[0], self.dispatch[1], self.dispatch[2]);
        Ok(())
    }
}

struct EncodeTimestampedComputePass<'a> {
    operation: EncodeComputePass<'a>,
    indices: GpuPassTimestampIndices,
    result: &'a mut Result<()>,
}
impl CurrentRenderTimestampWritesTerminal for EncodeTimestampedComputePass<'_> {
    fn write_timestamps(self, query_set: &QuerySet) {
        *self.result = self.operation.encode(Some((query_set, self.indices)));
    }
}

struct SetComputeBindGroup<'a, 'pass> {
    pass: &'a mut ComputePass<'pass>,
    index: u32,
}
impl CurrentRenderPipelineBindGroupsTerminal for SetComputeBindGroup<'_, '_> {
    fn bind_groups(self, groups: &[&BindGroup]) {
        debug_assert_eq!(
            groups.len(),
            1,
            "each current render terminal binds one group"
        );
        self.pass.set_bind_group(self.index, groups[0], &[]);
    }
}
struct SetRenderBindGroup<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    index: u32,
}
impl CurrentRenderPipelineBindGroupsTerminal for SetRenderBindGroup<'_, '_> {
    fn bind_groups(self, groups: &[&BindGroup]) {
        debug_assert_eq!(
            groups.len(),
            1,
            "each current render terminal binds one group"
        );
        self.pass.set_bind_group(self.index, groups[0], &[]);
    }
}

struct EncodeFullscreenPipeline<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    surface_view: Option<&'a TextureView>,
    realized_views: &'a [&'a GpuRealizedTextureView],
    bind_groups: &'a [GpuRealizedBindGroup],
    load: LoadOp<Color>,
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    result: &'a mut Result<()>,
}
impl CurrentRenderRenderPipelineTerminal for EncodeFullscreenPipeline<'_> {
    fn use_render_pipeline(self, pipeline: &RenderPipeline) {
        let bridge_result = self
            .context
            .current_render_execution_bridge()
            .for_pass_attachments(
                self.realized_views,
                EncodeFullscreenPass {
                    context: self.context,
                    encoder: self.encoder,
                    surface_view: self.surface_view,
                    pipeline,
                    bind_groups: self.bind_groups,
                    load: self.load,
                    gpu_timestamp_writes: self.gpu_timestamp_writes,
                    result: self.result,
                },
            );
        if let Err(error) = bridge_result {
            *self.result = Err(error.into());
        }
    }
}

struct EncodeFullscreenPass<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    surface_view: Option<&'a TextureView>,
    pipeline: &'a RenderPipeline,
    bind_groups: &'a [GpuRealizedBindGroup],
    load: LoadOp<Color>,
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    result: &'a mut Result<()>,
}
impl CurrentRenderAttachmentsTerminal for EncodeFullscreenPass<'_> {
    fn encode_with_attachments(self, views: &[&TextureView]) {
        let view = self.surface_view.unwrap_or_else(|| views[0]);
        let operation = FullscreenPassOperation {
            context: self.context,
            encoder: self.encoder,
            view,
            pipeline: self.pipeline,
            bind_groups: self.bind_groups,
            load: self.load,
        };
        if let Some(writes) = self.gpu_timestamp_writes {
            let mut nested_result = Ok(());
            let bridge_result = self
                .context
                .current_render_execution_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedFullscreenPass {
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
            *self.result = operation.encode(None);
        }
    }
}

struct FullscreenPassOperation<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    view: &'a TextureView,
    pipeline: &'a RenderPipeline,
    bind_groups: &'a [GpuRealizedBindGroup],
    load: LoadOp<Color>,
}
impl FullscreenPassOperation<'_> {
    fn encode(self, timestamp: Option<(&QuerySet, GpuPassTimestampIndices)>) -> Result<()> {
        let color_attachment = Some(RenderPassColorAttachment {
            view: self.view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: self.load,
                store: StoreOp::Store,
            },
        });
        let timestamp_writes = timestamp.map(|(query_set, indices)| RenderPassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(indices.begin),
            end_of_pass_write_index: Some(indices.end),
        });
        let mut pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_compiled_fullscreen_pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment: None,
            timestamp_writes,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(self.pipeline);
        for bind_group in self.bind_groups {
            let index = bind_group.layout_descriptor().group();
            self.context
                .current_render_execution_bridge()
                .for_pipeline_bind_groups(
                    &[bind_group],
                    SetRenderBindGroup {
                        pass: &mut pass,
                        index,
                    },
                )?;
        }
        pass.draw(0..3, 0..1);
        Ok(())
    }
}

struct EncodeTimestampedFullscreenPass<'a> {
    operation: FullscreenPassOperation<'a>,
    indices: GpuPassTimestampIndices,
    result: &'a mut Result<()>,
}
impl CurrentRenderTimestampWritesTerminal for EncodeTimestampedFullscreenPass<'_> {
    fn write_timestamps(self, query_set: &QuerySet) {
        *self.result = self.operation.encode(Some((query_set, self.indices)));
    }
}

enum GraphicsDraw<'a> {
    Direct {
        indexed: bool,
        vertex_range: std::ops::Range<u32>,
        instance_range: std::ops::Range<u32>,
    },
    Indirect {
        buffer: &'a GpuRealizedBuffer,
        byte_offset: u64,
        indexed: bool,
    },
}

struct EncodeGraphicsPipeline<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    surface_color_view: Option<&'a TextureView>,
    color_is_realized: bool,
    has_depth: bool,
    attachment_views: &'a [&'a GpuRealizedTextureView],
    bind_groups: &'a [GpuRealizedBindGroup],
    load: LoadOp<Color>,
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    vertex_buffers: &'a [(u32, &'a GpuRealizedBuffer)],
    index_buffer: Option<&'a GpuRealizedBuffer>,
    draw: GraphicsDraw<'a>,
    result: &'a mut Result<()>,
}
impl CurrentRenderRenderPipelineTerminal for EncodeGraphicsPipeline<'_> {
    fn use_render_pipeline(self, pipeline: &RenderPipeline) {
        let bridge_result = self
            .context
            .current_render_execution_bridge()
            .for_pass_attachments(
                self.attachment_views,
                EncodeGraphicsPass {
                    context: self.context,
                    encoder: self.encoder,
                    surface_color_view: self.surface_color_view,
                    color_is_realized: self.color_is_realized,
                    has_depth: self.has_depth,
                    pipeline,
                    bind_groups: self.bind_groups,
                    load: self.load,
                    gpu_timestamp_writes: self.gpu_timestamp_writes,
                    vertex_buffers: self.vertex_buffers,
                    index_buffer: self.index_buffer,
                    draw: self.draw,
                    result: self.result,
                },
            );
        if let Err(error) = bridge_result {
            *self.result = Err(error.into());
        }
    }
}

struct EncodeGraphicsPass<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    surface_color_view: Option<&'a TextureView>,
    color_is_realized: bool,
    has_depth: bool,
    pipeline: &'a RenderPipeline,
    bind_groups: &'a [GpuRealizedBindGroup],
    load: LoadOp<Color>,
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    vertex_buffers: &'a [(u32, &'a GpuRealizedBuffer)],
    index_buffer: Option<&'a GpuRealizedBuffer>,
    draw: GraphicsDraw<'a>,
    result: &'a mut Result<()>,
}
impl CurrentRenderAttachmentsTerminal for EncodeGraphicsPass<'_> {
    fn encode_with_attachments(self, views: &[&TextureView]) {
        let mut realized_index = 0;
        let color_view = if self.color_is_realized {
            let view = views[realized_index];
            realized_index += 1;
            view
        } else {
            self.surface_color_view
                .expect("surface color marker retains its lexical view")
        };
        let depth_view = self.has_depth.then(|| views[realized_index]);
        let operation = GraphicsPassOperation {
            encoder: self.encoder,
            color_view,
            depth_view,
            pipeline: self.pipeline,
            bind_groups: self.bind_groups,
            load: self.load,
            vertex_buffers: self.vertex_buffers,
            index_buffer: self.index_buffer,
            draw: self.draw,
        };
        if let Some(writes) = self.gpu_timestamp_writes {
            let mut nested_result = Ok(());
            let bridge_result = self
                .context
                .current_render_execution_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedGraphicsPass {
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
            operation.encode(self.context, None, self.result);
        }
    }
}

struct GraphicsPassOperation<'a> {
    encoder: &'a mut CommandEncoder,
    color_view: &'a TextureView,
    depth_view: Option<&'a TextureView>,
    pipeline: &'a RenderPipeline,
    bind_groups: &'a [GpuRealizedBindGroup],
    load: LoadOp<Color>,
    vertex_buffers: &'a [(u32, &'a GpuRealizedBuffer)],
    index_buffer: Option<&'a GpuRealizedBuffer>,
    draw: GraphicsDraw<'a>,
}
impl GraphicsPassOperation<'_> {
    fn encode(
        self,
        context: &GpuContext,
        timestamp: Option<(&QuerySet, GpuPassTimestampIndices)>,
        result: &mut Result<()>,
    ) {
        let depth_attachment = self
            .depth_view
            .map(|view| RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            });
        let color_attachment = Some(RenderPassColorAttachment {
            view: self.color_view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: self.load,
                store: StoreOp::Store,
            },
        });
        let timestamp_writes = timestamp.map(|(query_set, indices)| RenderPassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(indices.begin),
            end_of_pass_write_index: Some(indices.end),
        });
        let mut pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_compiled_graphics_pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment: depth_attachment,
            timestamp_writes,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(self.pipeline);
        for bind_group in self.bind_groups {
            let index = bind_group.layout_descriptor().group();
            if let Err(error) = context
                .current_render_execution_bridge()
                .for_pipeline_bind_groups(
                    &[bind_group],
                    SetRenderBindGroup {
                        pass: &mut pass,
                        index,
                    },
                )
            {
                *result = Err(error.into());
                return;
            }
        }
        for &(slot, buffer) in self.vertex_buffers {
            if let Err(error) = context.current_render_execution_bridge().for_vertex_buffer(
                buffer,
                SetVertexBuffer {
                    pass: &mut pass,
                    slot,
                },
            ) {
                *result = Err(error.into());
                return;
            }
        }
        if let Some(index) = self.index_buffer
            && let Err(error) = context
                .current_render_execution_bridge()
                .for_index_buffer(index, SetIndexBuffer { pass: &mut pass })
        {
            *result = Err(error.into());
            return;
        }
        match self.draw {
            GraphicsDraw::Direct {
                indexed,
                vertex_range,
                instance_range,
            } => {
                if indexed {
                    pass.draw_indexed(vertex_range, 0, instance_range);
                } else {
                    pass.draw(vertex_range, instance_range);
                }
            }
            GraphicsDraw::Indirect {
                buffer,
                byte_offset,
                indexed,
            } => {
                if let Err(error) = context
                    .current_render_execution_bridge()
                    .for_indirect_buffer(
                        buffer,
                        DrawIndirect {
                            pass: &mut pass,
                            byte_offset,
                            indexed,
                        },
                    )
                {
                    *result = Err(error.into());
                }
            }
        }
    }
}

struct EncodeTimestampedGraphicsPass<'a> {
    context: &'a GpuContext,
    operation: GraphicsPassOperation<'a>,
    indices: GpuPassTimestampIndices,
    result: &'a mut Result<()>,
}
impl CurrentRenderTimestampWritesTerminal for EncodeTimestampedGraphicsPass<'_> {
    fn write_timestamps(self, query_set: &QuerySet) {
        self.operation
            .encode(self.context, Some((query_set, self.indices)), self.result);
    }
}

struct SetVertexBuffer<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    slot: u32,
}
impl CurrentRenderVertexBufferTerminal for SetVertexBuffer<'_, '_> {
    fn use_vertex_buffer(self, buffer: &Buffer) {
        self.pass.set_vertex_buffer(self.slot, buffer.slice(..));
    }
}
struct SetIndexBuffer<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
}
impl CurrentRenderIndexBufferTerminal for SetIndexBuffer<'_, '_> {
    fn use_index_buffer(self, buffer: &Buffer) {
        self.pass
            .set_index_buffer(buffer.slice(..), IndexFormat::Uint32);
    }
}
struct DrawIndirect<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    byte_offset: u64,
    indexed: bool,
}
impl CurrentRenderIndirectBufferTerminal for DrawIndirect<'_, '_> {
    fn use_indirect_buffer(self, buffer: &Buffer) {
        if self.indexed {
            self.pass.draw_indexed_indirect(buffer, self.byte_offset);
        } else {
            self.pass.draw_indirect(buffer, self.byte_offset);
        }
    }
}

fn admit_resolved_program_source(
    cache: &mut FlowPipelineArtifactCache,
    shader: &super::super::provenance::ResolvedShaderMaterial<'_>,
    provenance_detail: impl Into<String>,
) -> Result<GpuAdmittedProgramSource> {
    Ok(cache.admit_program_source(
        GpuProgramSourceKey::new(shader.pipeline_identity.as_str())?,
        shader.revision,
        shader.source,
        GpuProgramSourceProvenance::new(
            "render-flow-resolved-program",
            Some(provenance_detail.into()),
        )?,
    )?)
}

fn reject_material_shader_fallback(
    feature_id: Option<crate::plugins::render::RenderFeatureId>,
    shader_reference: Option<&RenderShaderReference>,
    pass_id: crate::plugins::render::RenderPassId,
    shader: &super::super::provenance::ResolvedShaderMaterial<'_>,
) -> Result<()> {
    if pass_consumes_material_resources(feature_id, shader_reference) && shader.fallback_used {
        bail!(
            "material feature pass '{}' requires the exact generated shader '{}' to be loaded; builtin or scene-bundle fallback is forbidden",
            pass_id,
            shader.shader_id
        );
    }
    Ok(())
}

fn reject_unresident_material_textures(
    packet: &RendererPreparedPacket,
    feature_id: Option<crate::plugins::render::RenderFeatureId>,
    shader: Option<&RenderShaderReference>,
    pass_id: crate::plugins::render::RenderPassId,
) -> Result<()> {
    if !pass_consumes_material_resources(feature_id, shader) {
        return Ok(());
    }
    let Some(material) = &packet.prepared_material else {
        return Ok(());
    };
    let texture_count = material
        .instances
        .iter()
        .map(|instance| instance.texture_bindings.len())
        .sum::<usize>();
    if texture_count == 0 || packet.prepared_material_gpu_resources.is_some() {
        return Ok(());
    }
    bail!(
        "material feature pass '{}' requires {} GPU-resident material texture bindings, but render-flow material resource bind groups are not prepared; refusing shader execution instead of using pseudo texture sampling",
        pass_id,
        texture_count
    );
}

fn empty_specialization_value_set() -> Result<GpuSpecializationValueSet> {
    Ok(GpuSpecializationValueSet::new(
        GpuSpecializationSchema::new([])?,
        [],
    )?)
}

fn compute_specialization_from_constants(
    constants: &[RenderShaderConstant],
) -> Result<GpuSpecializationValueSet> {
    if constants.is_empty() {
        return empty_specialization_value_set();
    }
    let mut declarations = Vec::with_capacity(constants.len());
    let mut entries = Vec::with_capacity(constants.len());
    for constant in constants {
        let key = GpuSpecializationKey::new(constant.name.clone())?;
        declarations.push(GpuSpecializationDeclaration::new(
            key.clone(),
            constant.value.value_type(),
            None,
            GpuCapabilityRequirements::new(),
        )?);
        entries.push(GpuSpecializationEntry::new(key, constant.value));
    }
    Ok(GpuSpecializationValueSet::new(
        GpuSpecializationSchema::new(declarations)?,
        entries,
    )?)
}

fn compiled_resource_ref_matches_id(
    resource: &CompiledResourceRef,
    expected: GpuWorkResourceId,
) -> bool {
    match resource {
        CompiledResourceRef::FlowOwned(id) | CompiledResourceRef::Imported(id) => *id == expected,
        CompiledResourceRef::TargetAlias(alias) => alias.resource_id == expected,
        CompiledResourceRef::ImportedBuiltin(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_compute_specialization_normalizes_order_and_preserves_types() {
        let first = compute_specialization_from_constants(&[
            RenderShaderConstant::u32("COUNT", 4),
            RenderShaderConstant::i32("OFFSET", -1),
        ])
        .unwrap();
        let reordered = compute_specialization_from_constants(&[
            RenderShaderConstant::i32("OFFSET", -1),
            RenderShaderConstant::u32("COUNT", 4),
        ])
        .unwrap();
        let signed_count = compute_specialization_from_constants(&[
            RenderShaderConstant::i32("COUNT", 4),
            RenderShaderConstant::i32("OFFSET", -1),
        ])
        .unwrap();

        assert_eq!(first, reordered);
        assert_ne!(first, signed_count);
    }

    #[test]
    fn typed_compute_specialization_rejects_invalid_or_duplicate_keys() {
        assert!(
            compute_specialization_from_constants(&[RenderShaderConstant::u32("a=1,b", 2)])
                .is_err()
        );
        assert!(
            compute_specialization_from_constants(&[
                RenderShaderConstant::u32("COUNT", 1),
                RenderShaderConstant::u32("COUNT", 2),
            ])
            .is_err()
        );
    }
}
