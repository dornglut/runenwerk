use super::*;
use crate::plugins::gpu::{
    CurrentRenderAttachmentsTerminal, CurrentRenderBufferCopyTerminal,
    CurrentRenderIndexBufferTerminal, CurrentRenderIndirectBufferTerminal,
    CurrentRenderPipelineBindGroupsTerminal, CurrentRenderPipelineCreationTerminal,
    CurrentRenderTextureCopyTerminal, CurrentRenderTimestampWritesTerminal,
    CurrentRenderVertexBufferTerminal, CurrentSurfaceTextureCopyTerminal, GpuAdmittedProgramSource,
    GpuCapabilityRequirements, GpuComputePipelineDescriptor, GpuProgramSourceKey,
    GpuProgramSourceProvenance, GpuRealizedBindGroup, GpuRealizedBuffer,
    GpuRenderPipelineDescriptor, GpuSpecializationDeclaration, GpuSpecializationEntry,
    GpuSpecializationKey, GpuSpecializationSchema, GpuSpecializationValue,
    GpuSpecializationValueSet, GpuWorkResourceId,
};
use crate::plugins::render::RenderPassId;
use crate::plugins::render::graph::{
    CompiledDrawBufferPlan, CompiledDrawSource, CompiledResourceRef,
};
use crate::plugins::render::pipelines::FlowPassPipelineDescriptor;
use crate::plugins::render::{
    RenderBlendMode, RenderCullMode, RenderDepthPolicy, RenderIndirectDrawArgsKind,
    RenderPrimitiveTopology, RenderRasterState, RenderShaderConstant, RenderVertexFormat,
    RenderVertexStepMode,
};

impl Renderer {
    /// First half of the renderer's two-phase integration: realize every G4C2 program, layout,
    /// bind group (and dependent G4C1 sampler) while no raw device/queue loan is live.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn realize_compiled_pass(
        &mut self,
        context: &GpuContext,
        frame_texture: &Texture,
        frame_view: &TextureView,
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
                if dispatch[0] == 0 || dispatch[1] == 0 || dispatch[2] == 0 {
                    bail!(
                        "compute pass '{}' resolved invalid dispatch dimensions ({}, {}, {})",
                        value.pass_id,
                        dispatch[0],
                        dispatch[1],
                        dispatch[2]
                    );
                }
                let admitted_source = admit_resolved_program_source(
                    &mut self.flow_pipeline_cache,
                    &shader,
                    format!("compute pass {}", value.pass_id),
                )?;
                let bindings = self.resolve_compiled_bind_group(
                    context,
                    frame_texture,
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
                Ok(Some(PreparedPipelinePass {
                    bindings,
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
                let color_target = self.resolve_color_target_from_plan(
                    runtime_resources,
                    value.pass_id,
                    &value.targets,
                    frame_view,
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
                    frame_texture,
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
                    vec![color_target.format],
                    None,
                    runtime_resources,
                )?;
                Ok(Some(PreparedPipelinePass {
                    bindings,
                    shader_id: shader.shader_id,
                    shader_revision: shader.revision,
                    fallback_used: shader.fallback_used,
                }))
            }
            CompiledPassExecutionPlan::Graphics(value) => {
                let color_target = self.resolve_color_target_from_plan(
                    runtime_resources,
                    value.pass_id,
                    &value.targets,
                    frame_view,
                    packet.surface_format,
                )?;
                let depth_target = self.resolve_depth_target_from_plan(
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
                    frame_texture,
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
                    vec![color_target.format],
                    depth_target.as_ref().map(|target| target.format),
                    runtime_resources,
                )?;
                Ok(Some(PreparedPipelinePass {
                    bindings,
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
    pub(super) fn encode_compiled_pass(
        &mut self,
        context: &GpuContext,
        device: &Device,
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
            CompiledPassExecutionPlan::Compute(value) => self
                .encode_compute_pass(
                    context,
                    device,
                    encoder,
                    frame_texture,
                    packet,
                    flow,
                    flow_inputs,
                    runtime_resources,
                    value,
                    prepared_pipeline.ok_or_else(|| {
                        anyhow::anyhow!(
                            "compute pass '{}' reached G4C3/G5 without G4C2 realization",
                            value.pass_id
                        )
                    })?,
                    gpu_timestamp_writes,
                )
                .map(|value| EncodedPassEvidence {
                    dispatch_workgroups: value.dispatch_workgroups,
                    shader_id: value.shader_id,
                    shader_revision: value.shader_revision,
                    fallback_used: value.fallback_used,
                    pipeline_key: Some(value.pipeline_key),
                }),
            CompiledPassExecutionPlan::Fullscreen(value) => self
                .encode_fullscreen_pass(
                    context,
                    device,
                    encoder,
                    frame_texture,
                    frame_view,
                    packet,
                    flow,
                    runtime_resources,
                    value,
                    prepared_pipeline.ok_or_else(|| {
                        anyhow::anyhow!(
                            "fullscreen pass '{}' reached G4C3/G5 without G4C2 realization",
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
                    device,
                    encoder,
                    frame_texture,
                    frame_view,
                    packet,
                    flow,
                    runtime_resources,
                    value,
                    prepared_pipeline.ok_or_else(|| {
                        anyhow::anyhow!(
                            "graphics pass '{}' reached G4C3/G5 without G4C2 realization",
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
            CompiledPassExecutionPlan::BuiltinUiComposite(_value) => {
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

    #[allow(clippy::too_many_arguments)]
    fn encode_compute_pass(
        &mut self,
        context: &GpuContext,
        device: &Device,
        encoder: &mut CommandEncoder,
        _frame_texture: &Texture,
        _packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        flow_inputs: &PreparedFlowInputs,
        _runtime_resources: &FlowRuntimeResources,
        pass: &CompiledComputeExecutionPlan,
        prepared: &PreparedPipelinePass,
        gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    ) -> Result<EncodedPipelinePass> {
        let dispatch = flow_inputs
            .projected_dispatch_workgroups
            .get(&pass.pass_id)
            .copied()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "missing prepared dispatch for pass '{}' in flow '{}'",
                    pass.pass_id,
                    flow.flow_id
                )
            })?;
        if dispatch[0] == 0 || dispatch[1] == 0 || dispatch[2] == 0 {
            bail!(
                "compute pass '{}' resolved invalid dispatch dimensions ({}, {}, {})",
                pass.pass_id,
                dispatch[0],
                dispatch[1],
                dispatch[2]
            );
        }

        let pipeline_key = prepared.bindings.pipeline_key.clone();
        let compute_descriptor = match &pipeline_key.pipeline_descriptor {
            FlowPassPipelineDescriptor::Compute(descriptor) => descriptor,
            FlowPassPipelineDescriptor::Render(_) => {
                bail!(
                    "compute pass '{}' resolved a render pipeline descriptor",
                    pass.pass_id
                )
            }
        };

        let pipeline = match self.flow_pipeline_cache.compute_pipeline(&pipeline_key) {
            Some(pipeline) => pipeline,
            None => {
                let mut created = None;
                context
                    .current_render_pipeline_bridge()
                    .for_pipeline_creation(
                        &prepared.bindings.program,
                        &prepared.bindings.pipeline_layout,
                        CreateFlowComputePipeline {
                            device,
                            descriptor: compute_descriptor,
                            output: &mut created,
                        },
                    )?;
                let created = created.ok_or_else(|| {
                    anyhow::anyhow!(
                        "current render pipeline bridge did not create compute pipeline for pass '{}'",
                        pass.pass_id
                    )
                })?;
                self.flow_pipeline_cache
                    .insert_compute_pipeline(pipeline_key.clone(), created)
            }
        };

        let operation = EncodeComputePass {
            encoder,
            pipeline: &pipeline,
            bind_group: prepared.bindings.bind_group.as_ref(),
            context,
            dispatch,
        };
        if let Some(writes) = gpu_timestamp_writes {
            let mut encode_result = Ok(());
            context
                .current_render_pipeline_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedComputePass {
                        operation,
                        indices: writes.indices,
                        result: &mut encode_result,
                    },
                )?;
            encode_result?;
        } else {
            operation.encode(None)?;
        }
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
        device: &Device,
        encoder: &mut CommandEncoder,
        _frame_texture: &Texture,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        _flow: &CompiledRenderFlowPlan,
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
        let render_descriptor = match &pipeline_key.pipeline_descriptor {
            FlowPassPipelineDescriptor::Render(descriptor) => descriptor,
            FlowPassPipelineDescriptor::Compute(_) => {
                bail!(
                    "fullscreen pass '{}' resolved a compute pipeline descriptor",
                    plan.pass_id
                )
            }
        };

        let material_resources =
            material_resources_for_pass(packet, plan.feature_id, plan.shader.as_ref());
        let pipeline = match self.flow_pipeline_cache.render_pipeline(&pipeline_key) {
            Some(pipeline) => pipeline,
            None => {
                let mut created = None;
                context
                    .current_render_pipeline_bridge()
                    .for_pipeline_creation(
                        &prepared.bindings.program,
                        &prepared.bindings.pipeline_layout,
                        CreateFlowFullscreenPipeline {
                            device,
                            descriptor: render_descriptor,
                            color_format: color_target.format,
                            output: &mut created,
                        },
                    )?;
                let created = created.ok_or_else(|| {
                    anyhow::anyhow!(
                        "current render pipeline bridge did not create fullscreen pipeline for pass '{}'",
                        plan.pass_id
                    )
                })?;
                self.flow_pipeline_cache
                    .insert_render_pipeline(pipeline_key.clone(), created)
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
            .current_render_pipeline_bridge()
            .for_pass_attachments(
                &realized_views,
                EncodeFullscreenPass {
                    context,
                    encoder,
                    surface_view,
                    pipeline: &pipeline,
                    bind_group: prepared.bindings.bind_group.as_ref(),
                    material_resources,
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
        device: &Device,
        encoder: &mut CommandEncoder,
        _frame_texture: &Texture,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        _flow: &CompiledRenderFlowPlan,
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
        let render_descriptor = match &pipeline_key.pipeline_descriptor {
            FlowPassPipelineDescriptor::Render(descriptor) => descriptor,
            FlowPassPipelineDescriptor::Compute(_) => {
                bail!(
                    "graphics pass '{}' resolved a compute pipeline descriptor",
                    plan.pass_id
                )
            }
        };

        let material_resources =
            material_resources_for_pass(packet, plan.feature_id, plan.shader.as_ref());
        let vertex_attribute_sets = build_vertex_attribute_sets(&plan.draw_buffers);
        let vertex_buffer_layouts =
            build_vertex_buffer_layouts(&plan.draw_buffers, &vertex_attribute_sets);
        let pipeline = match self.flow_pipeline_cache.render_pipeline(&pipeline_key) {
            Some(pipeline) => pipeline,
            None => {
                let mut created = None;
                context
                    .current_render_pipeline_bridge()
                    .for_pipeline_creation(
                        &prepared.bindings.program,
                        &prepared.bindings.pipeline_layout,
                        CreateFlowGraphicsPipeline {
                            device,
                            descriptor: render_descriptor,
                            color_format: color_target.format,
                            depth_format: depth_target.as_ref().map(|target| target.format),
                            raster_state: plan.raster_state.state,
                            vertex_buffer_layouts: &vertex_buffer_layouts,
                            output: &mut created,
                        },
                    )?;
                let created = created.ok_or_else(|| {
                    anyhow::anyhow!(
                        "current render pipeline bridge did not create graphics pipeline for pass '{}'",
                        plan.pass_id
                    )
                })?;
                self.flow_pipeline_cache
                    .insert_render_pipeline(pipeline_key.clone(), created)
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
            _ => {
                bail!(
                    "graphics pass '{}' declares multiple index_buffer(...) resources; runtime currently supports exactly one",
                    plan.pass_id
                );
            }
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
            (Some(_), Some((_indirect, RenderIndirectDrawArgsKind::Draw, _byte_offset))) => {
                bail!(
                    "graphics pass '{}' indexed indirect draw uses non-indexed indirect args",
                    plan.pass_id
                );
            }
            (None, Some((_indirect, RenderIndirectDrawArgsKind::DrawIndexed, _byte_offset))) => {
                bail!(
                    "graphics pass '{}' non-indexed indirect draw uses indexed indirect args",
                    plan.pass_id
                );
            }
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
            .current_render_pipeline_bridge()
            .for_pass_attachments(
                &attachment_views,
                EncodeGraphicsPass {
                    context,
                    encoder,
                    surface_color_view,
                    color_is_realized: surface_color_view.is_none(),
                    has_depth: depth_target.is_some(),
                    pipeline: &pipeline,
                    bind_group: prepared.bindings.bind_group.as_ref(),
                    material_resources,
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
                context.current_render_pipeline_bridge().for_texture_copy(
                    source,
                    destination,
                    CopyTextures { encoder, extent },
                )?;
            }
            (RuntimeTextureRef::Realized(source), RuntimeTextureRef::Surface(destination)) => {
                context
                    .current_render_pipeline_bridge()
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
                    .current_render_pipeline_bridge()
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
        context.current_render_pipeline_bridge().for_buffer_copy(
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

/// G4C3's temporary compute-pipeline creation terminal. The G4C2 program and layout are lent
/// only for this one lexical WGPU call; this terminal retains no backend reference.
struct CreateFlowComputePipeline<'a> {
    device: &'a Device,
    descriptor: &'a GpuComputePipelineDescriptor,
    output: &'a mut Option<ComputePipeline>,
}

impl CurrentRenderPipelineCreationTerminal for CreateFlowComputePipeline<'_> {
    fn create_pipeline(self, program: &ShaderModule, layout: &PipelineLayout) {
        let constants = wgpu_specialization_constants(self.descriptor.specialization());
        *self.output = Some(
            self.device
                .create_compute_pipeline(&ComputePipelineDescriptor {
                    label: Some("engine_compiled_compute_pipeline"),
                    layout: Some(layout),
                    module: program,
                    entry_point: Some(self.descriptor.entry_point().as_str()),
                    compilation_options: PipelineCompilationOptions {
                        constants: constants.as_slice(),
                        ..PipelineCompilationOptions::default()
                    },
                    cache: None,
                }),
        );
    }
}

/// G4C3's temporary fullscreen render-pipeline creation terminal.
struct CreateFlowFullscreenPipeline<'a> {
    device: &'a Device,
    descriptor: &'a GpuRenderPipelineDescriptor,
    color_format: TextureFormat,
    output: &'a mut Option<RenderPipeline>,
}

impl CurrentRenderPipelineCreationTerminal for CreateFlowFullscreenPipeline<'_> {
    fn create_pipeline(self, program: &ShaderModule, layout: &PipelineLayout) {
        let constants = wgpu_specialization_constants(self.descriptor.specialization());
        let fragment = self
            .descriptor
            .entry_points()
            .fragment()
            .expect("G4B fullscreen descriptor always names a fragment entry point");
        *self.output = Some(
            self.device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("engine_compiled_fullscreen_pipeline"),
                    layout: Some(layout),
                    vertex: VertexState {
                        module: program,
                        entry_point: Some(self.descriptor.entry_points().vertex().as_str()),
                        compilation_options: PipelineCompilationOptions {
                            constants: constants.as_slice(),
                            ..PipelineCompilationOptions::default()
                        },
                        buffers: &[],
                    },
                    fragment: Some(FragmentState {
                        module: program,
                        entry_point: Some(fragment.as_str()),
                        compilation_options: PipelineCompilationOptions {
                            constants: constants.as_slice(),
                            ..PipelineCompilationOptions::default()
                        },
                        targets: &[Some(ColorTargetState {
                            format: self.color_format,
                            blend: blend_state_for_color_format(self.color_format),
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    multiview: None,
                    cache: None,
                }),
        );
    }
}

/// G4C3's temporary graphics render-pipeline creation terminal.
struct CreateFlowGraphicsPipeline<'a, 'layouts> {
    device: &'a Device,
    descriptor: &'a GpuRenderPipelineDescriptor,
    color_format: TextureFormat,
    depth_format: Option<TextureFormat>,
    raster_state: RenderRasterState,
    vertex_buffer_layouts: &'layouts [VertexBufferLayout<'layouts>],
    output: &'a mut Option<RenderPipeline>,
}

impl CurrentRenderPipelineCreationTerminal for CreateFlowGraphicsPipeline<'_, '_> {
    fn create_pipeline(self, program: &ShaderModule, layout: &PipelineLayout) {
        let constants = wgpu_specialization_constants(self.descriptor.specialization());
        let fragment = self
            .descriptor
            .entry_points()
            .fragment()
            .expect("G4B graphics descriptor always names a fragment entry point");
        *self.output = Some(
            self.device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("engine_compiled_graphics_pipeline"),
                    layout: Some(layout),
                    vertex: VertexState {
                        module: program,
                        entry_point: Some(self.descriptor.entry_points().vertex().as_str()),
                        compilation_options: PipelineCompilationOptions {
                            constants: constants.as_slice(),
                            ..PipelineCompilationOptions::default()
                        },
                        buffers: self.vertex_buffer_layouts,
                    },
                    fragment: Some(FragmentState {
                        module: program,
                        entry_point: Some(fragment.as_str()),
                        compilation_options: PipelineCompilationOptions {
                            constants: constants.as_slice(),
                            ..PipelineCompilationOptions::default()
                        },
                        targets: &[Some(ColorTargetState {
                            format: self.color_format,
                            blend: blend_state_for_policy(
                                self.color_format,
                                self.raster_state.blend_mode,
                            ),
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: primitive_state_from_raster_state(self.raster_state),
                    depth_stencil: depth_stencil_state_for_policy(
                        self.depth_format,
                        self.raster_state.depth_policy,
                    ),
                    multisample: MultisampleState::default(),
                    multiview: None,
                    cache: None,
                }),
        );
    }
}

struct EncodeComputePass<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    pipeline: &'a ComputePipeline,
    bind_group: Option<&'a GpuRealizedBindGroup>,
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
        if let Some(bind_group) = self.bind_group {
            self.context
                .current_render_pipeline_bridge()
                .for_pipeline_bind_groups(
                    &[bind_group],
                    SetComputeBindGroup {
                        pass: &mut pass,
                        index: 0,
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

struct EncodeFullscreenPass<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    surface_view: Option<&'a TextureView>,
    pipeline: &'a RenderPipeline,
    bind_group: Option<&'a GpuRealizedBindGroup>,
    material_resources: Option<&'a PreparedMaterialGpuResources>,
    load: LoadOp<Color>,
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    result: &'a mut Result<()>,
}

impl CurrentRenderAttachmentsTerminal for EncodeFullscreenPass<'_> {
    fn encode_with_attachments(self, views: &[&TextureView]) {
        let EncodeFullscreenPass {
            context,
            encoder,
            surface_view,
            pipeline,
            bind_group,
            material_resources,
            load,
            gpu_timestamp_writes,
            result,
        } = self;
        let view = surface_view.unwrap_or_else(|| views[0]);
        let operation = FullscreenPassOperation {
            context,
            encoder,
            view,
            pipeline,
            bind_group,
            material_resources,
            load,
        };
        if let Some(writes) = gpu_timestamp_writes {
            let mut nested_result = Ok(());
            let bridge_result = context
                .current_render_pipeline_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedFullscreenPass {
                        operation,
                        indices: writes.indices,
                        result: &mut nested_result,
                    },
                );
            if let Err(error) = bridge_result {
                *result = Err(error.into());
            } else {
                *result = nested_result;
            }
        } else {
            *result = operation.encode(None);
        }
    }
}

struct FullscreenPassOperation<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    view: &'a TextureView,
    pipeline: &'a RenderPipeline,
    bind_group: Option<&'a GpuRealizedBindGroup>,
    material_resources: Option<&'a PreparedMaterialGpuResources>,
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
        });
        pass.set_pipeline(self.pipeline);
        if let Some(bind_group) = self.bind_group {
            self.context
                .current_render_pipeline_bridge()
                .for_pipeline_bind_groups(
                    &[bind_group],
                    SetRenderBindGroup {
                        pass: &mut pass,
                        index: 0,
                    },
                )?;
        }
        if let Some(resources) = self.material_resources {
            self.context
                .current_render_pipeline_bridge()
                .for_pipeline_bind_groups(
                    &[resources.bind_group()],
                    SetRenderBindGroup {
                        pass: &mut pass,
                        index: 1,
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

struct EncodeGraphicsPass<'a> {
    context: &'a GpuContext,
    encoder: &'a mut CommandEncoder,
    surface_color_view: Option<&'a TextureView>,
    color_is_realized: bool,
    has_depth: bool,
    pipeline: &'a RenderPipeline,
    bind_group: Option<&'a GpuRealizedBindGroup>,
    material_resources: Option<&'a PreparedMaterialGpuResources>,
    load: LoadOp<Color>,
    gpu_timestamp_writes: Option<GpuPassTimestampWrites>,
    vertex_buffers: &'a [(u32, &'a GpuRealizedBuffer)],
    index_buffer: Option<&'a GpuRealizedBuffer>,
    draw: GraphicsDraw<'a>,
    result: &'a mut Result<()>,
}

impl CurrentRenderAttachmentsTerminal for EncodeGraphicsPass<'_> {
    fn encode_with_attachments(self, views: &[&TextureView]) {
        let EncodeGraphicsPass {
            context,
            encoder,
            surface_color_view,
            color_is_realized,
            has_depth,
            pipeline,
            bind_group,
            material_resources,
            load,
            gpu_timestamp_writes,
            vertex_buffers,
            index_buffer,
            draw,
            result,
        } = self;
        let mut realized_index = 0;
        let color_view = if color_is_realized {
            let view = views[realized_index];
            realized_index += 1;
            view
        } else {
            surface_color_view.expect("surface color marker retains its lexical view")
        };
        let depth_view = has_depth.then(|| views[realized_index]);
        let operation = GraphicsPassOperation {
            encoder,
            color_view,
            depth_view,
            pipeline,
            bind_group,
            material_resources,
            load,
            vertex_buffers,
            index_buffer,
            draw,
        };
        if let Some(writes) = gpu_timestamp_writes {
            let mut nested_result = Ok(());
            let bridge_result = context
                .current_render_pipeline_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedGraphicsPass {
                        context,
                        operation,
                        indices: writes.indices,
                        result: &mut nested_result,
                    },
                );
            if let Err(error) = bridge_result {
                *result = Err(error.into());
            } else {
                *result = nested_result;
            }
        } else {
            operation.encode(context, None, result);
        }
    }
}

struct GraphicsPassOperation<'a> {
    encoder: &'a mut CommandEncoder,
    color_view: &'a TextureView,
    depth_view: Option<&'a TextureView>,
    pipeline: &'a RenderPipeline,
    bind_group: Option<&'a GpuRealizedBindGroup>,
    material_resources: Option<&'a PreparedMaterialGpuResources>,
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
        });
        pass.set_pipeline(self.pipeline);
        if let Some(bind_group) = self.bind_group
            && let Err(error) = context
                .current_render_pipeline_bridge()
                .for_pipeline_bind_groups(
                    &[bind_group],
                    SetRenderBindGroup {
                        pass: &mut pass,
                        index: 0,
                    },
                )
        {
            *result = Err(error.into());
            return;
        }
        if let Some(resources) = self.material_resources
            && let Err(error) = context
                .current_render_pipeline_bridge()
                .for_pipeline_bind_groups(
                    &[resources.bind_group()],
                    SetRenderBindGroup {
                        pass: &mut pass,
                        index: 1,
                    },
                )
        {
            *result = Err(error.into());
            return;
        }
        for &(slot, buffer) in self.vertex_buffers {
            if let Err(error) = context.current_render_pipeline_bridge().for_vertex_buffer(
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
                .current_render_pipeline_bridge()
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
                    .current_render_pipeline_bridge()
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

fn admit_resolved_program_source(
    cache: &mut FlowPipelineArtifactCache,
    shader: &super::provenance::ResolvedShaderMaterial<'_>,
    provenance_detail: impl Into<String>,
) -> Result<GpuAdmittedProgramSource> {
    let admitted_source = cache.admit_program_source(
        GpuProgramSourceKey::new(shader.pipeline_identity.as_str())?,
        shader.revision,
        shader.source,
        GpuProgramSourceProvenance::new(
            "render-flow-resolved-program",
            Some(provenance_detail.into()),
        )?,
    )?;
    Ok(admitted_source)
}

fn reject_material_shader_fallback(
    feature_id: Option<crate::plugins::render::RenderFeatureId>,
    shader_reference: Option<&RenderShaderReference>,
    pass_id: RenderPassId,
    shader: &super::provenance::ResolvedShaderMaterial<'_>,
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
    pass_id: RenderPassId,
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
    if texture_count == 0 {
        return Ok(());
    }
    if packet.prepared_material_gpu_resources.is_some() {
        return Ok(());
    }
    bail!(
        "material feature pass '{}' requires {} GPU-resident material texture bindings, but render-flow material resource bind groups are not prepared; refusing shader execution instead of using pseudo texture sampling",
        pass_id,
        texture_count
    );
}

fn material_resources_for_pass<'a>(
    packet: &'a RendererPreparedPacket,
    feature_id: Option<crate::plugins::render::RenderFeatureId>,
    shader: Option<&RenderShaderReference>,
) -> Option<&'a PreparedMaterialGpuResources> {
    if pass_consumes_material_resources(feature_id, shader) {
        packet.prepared_material_gpu_resources.as_ref()
    } else {
        None
    }
}

fn build_vertex_attribute_sets(draw_buffers: &CompiledDrawBufferPlan) -> Vec<Vec<VertexAttribute>> {
    draw_buffers
        .vertex_buffers
        .iter()
        .map(|binding| {
            binding
                .layout
                .attributes
                .iter()
                .map(|attribute| VertexAttribute {
                    format: render_vertex_format_to_wgpu(attribute.format),
                    offset: attribute.offset,
                    shader_location: attribute.shader_location,
                })
                .collect::<Vec<_>>()
        })
        .chain(draw_buffers.instance_buffer_layouts.iter().map(|layout| {
            layout
                .attributes
                .iter()
                .map(|attribute| VertexAttribute {
                    format: render_vertex_format_to_wgpu(attribute.format),
                    offset: attribute.offset,
                    shader_location: attribute.shader_location,
                })
                .collect::<Vec<_>>()
        }))
        .collect()
}

fn blend_state_for_color_format(format: TextureFormat) -> Option<BlendState> {
    blend_state_for_policy(format, RenderBlendMode::Alpha)
}

fn blend_state_for_policy(format: TextureFormat, policy: RenderBlendMode) -> Option<BlendState> {
    if matches!(policy, RenderBlendMode::Replace) {
        return None;
    }
    match format {
        TextureFormat::R8Uint
        | TextureFormat::R8Sint
        | TextureFormat::R16Uint
        | TextureFormat::R16Sint
        | TextureFormat::Rg8Uint
        | TextureFormat::Rg8Sint
        | TextureFormat::R32Uint
        | TextureFormat::R32Sint
        | TextureFormat::Rg16Uint
        | TextureFormat::Rg16Sint
        | TextureFormat::Rgba8Uint
        | TextureFormat::Rgba8Sint
        | TextureFormat::Rg32Uint
        | TextureFormat::Rg32Sint
        | TextureFormat::Rgba16Uint
        | TextureFormat::Rgba16Sint
        | TextureFormat::Rgba32Uint
        | TextureFormat::Rgba32Sint => None,
        _ => Some(BlendState::ALPHA_BLENDING),
    }
}

fn primitive_state_from_raster_state(state: RenderRasterState) -> PrimitiveState {
    PrimitiveState {
        topology: render_primitive_topology_to_wgpu(state.primitive_topology),
        strip_index_format: match state.primitive_topology {
            RenderPrimitiveTopology::TriangleStrip | RenderPrimitiveTopology::LineStrip => {
                Some(IndexFormat::Uint32)
            }
            RenderPrimitiveTopology::TriangleList
            | RenderPrimitiveTopology::LineList
            | RenderPrimitiveTopology::PointList => None,
        },
        front_face: FrontFace::Ccw,
        cull_mode: render_cull_mode_to_wgpu(state.cull_mode),
        unclipped_depth: false,
        polygon_mode: PolygonMode::Fill,
        conservative: false,
    }
}

fn render_primitive_topology_to_wgpu(value: RenderPrimitiveTopology) -> PrimitiveTopology {
    match value {
        RenderPrimitiveTopology::TriangleList => PrimitiveTopology::TriangleList,
        RenderPrimitiveTopology::TriangleStrip => PrimitiveTopology::TriangleStrip,
        RenderPrimitiveTopology::LineList => PrimitiveTopology::LineList,
        RenderPrimitiveTopology::LineStrip => PrimitiveTopology::LineStrip,
        RenderPrimitiveTopology::PointList => PrimitiveTopology::PointList,
    }
}

fn render_cull_mode_to_wgpu(value: RenderCullMode) -> Option<Face> {
    match value {
        RenderCullMode::None => None,
        RenderCullMode::Front => Some(Face::Front),
        RenderCullMode::Back => Some(Face::Back),
    }
}

fn depth_stencil_state_for_policy(
    depth_format: Option<TextureFormat>,
    policy: RenderDepthPolicy,
) -> Option<DepthStencilState> {
    let format = depth_format?;
    if matches!(policy, RenderDepthPolicy::Disabled) {
        return None;
    }
    Some(DepthStencilState {
        format,
        depth_write_enabled: !matches!(policy, RenderDepthPolicy::ReadOnly),
        depth_compare: CompareFunction::LessEqual,
        stencil: StencilState::default(),
        bias: DepthBiasState::default(),
    })
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

fn build_vertex_buffer_layouts<'a>(
    draw_buffers: &'a CompiledDrawBufferPlan,
    attribute_sets: &'a [Vec<VertexAttribute>],
) -> Vec<VertexBufferLayout<'a>> {
    let mut layouts = Vec::<(u32, VertexBufferLayout<'a>)>::new();
    let mut attribute_index = 0usize;

    for binding in &draw_buffers.vertex_buffers {
        layouts.push((
            binding.layout.slot,
            VertexBufferLayout {
                array_stride: binding.layout.array_stride,
                step_mode: render_vertex_step_mode_to_wgpu(binding.layout.step_mode),
                attributes: &attribute_sets[attribute_index],
            },
        ));
        attribute_index = attribute_index.saturating_add(1);
    }

    for layout in &draw_buffers.instance_buffer_layouts {
        layouts.push((
            layout.slot,
            VertexBufferLayout {
                array_stride: layout.array_stride,
                step_mode: render_vertex_step_mode_to_wgpu(layout.step_mode),
                attributes: &attribute_sets[attribute_index],
            },
        ));
        attribute_index = attribute_index.saturating_add(1);
    }

    layouts.sort_by_key(|(slot, _)| *slot);
    layouts.into_iter().map(|(_, layout)| layout).collect()
}

fn render_vertex_step_mode_to_wgpu(value: RenderVertexStepMode) -> VertexStepMode {
    match value {
        RenderVertexStepMode::Vertex => VertexStepMode::Vertex,
        RenderVertexStepMode::Instance => VertexStepMode::Instance,
    }
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

    let schema = GpuSpecializationSchema::new(declarations)?;
    Ok(GpuSpecializationValueSet::new(schema, entries)?)
}

fn wgpu_specialization_constants(values: &GpuSpecializationValueSet) -> Vec<(&str, f64)> {
    values
        .entries()
        .map(|entry| {
            let value = match entry.value() {
                GpuSpecializationValue::Bool(value) => {
                    if value {
                        1.0
                    } else {
                        0.0
                    }
                }
                GpuSpecializationValue::U32(value) => f64::from(value),
                GpuSpecializationValue::I32(value) => f64::from(value),
                GpuSpecializationValue::F32(value) => f64::from(value.get()),
            };
            (entry.key().as_str(), value)
        })
        .collect()
}

fn render_vertex_format_to_wgpu(value: RenderVertexFormat) -> VertexFormat {
    match value {
        RenderVertexFormat::Float32 => VertexFormat::Float32,
        RenderVertexFormat::Float32x2 => VertexFormat::Float32x2,
        RenderVertexFormat::Float32x3 => VertexFormat::Float32x3,
        RenderVertexFormat::Float32x4 => VertexFormat::Float32x4,
        RenderVertexFormat::Uint32 => VertexFormat::Uint32,
        RenderVertexFormat::Uint32x2 => VertexFormat::Uint32x2,
        RenderVertexFormat::Uint32x3 => VertexFormat::Uint32x3,
        RenderVertexFormat::Uint32x4 => VertexFormat::Uint32x4,
        RenderVertexFormat::Sint32 => VertexFormat::Sint32,
        RenderVertexFormat::Sint32x2 => VertexFormat::Sint32x2,
        RenderVertexFormat::Sint32x3 => VertexFormat::Sint32x3,
        RenderVertexFormat::Sint32x4 => VertexFormat::Sint32x4,
    }
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
        assert_eq!(
            wgpu_specialization_constants(&first),
            [("COUNT", 4.0), ("OFFSET", -1.0)]
        );
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
