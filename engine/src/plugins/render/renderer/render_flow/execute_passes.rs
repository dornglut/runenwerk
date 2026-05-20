use super::*;
use crate::plugins::render::RenderPassId;
use crate::plugins::render::graph::CompiledDrawBufferPlan;
use crate::plugins::render::{RenderVertexFormat, RenderVertexStepMode};

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_compiled_pass(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        flow_inputs: &PreparedFlowInputs,
        pass: &CompiledPassExecutionPlan,
        shader_registry: &ShaderRegistryResource,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<EncodedPassEvidence> {
        match pass {
            CompiledPassExecutionPlan::Compute(value) => self
                .encode_compute_pass(
                    device,
                    encoder,
                    frame_texture,
                    packet,
                    flow,
                    flow_inputs,
                    runtime_resources,
                    value,
                    shader_registry,
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
                    device,
                    encoder,
                    frame_texture,
                    frame_view,
                    packet,
                    flow,
                    runtime_resources,
                    value,
                    shader_registry,
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
                    device,
                    encoder,
                    frame_texture,
                    frame_view,
                    packet,
                    flow,
                    runtime_resources,
                    value,
                    shader_registry,
                )
                .map(|value| EncodedPassEvidence {
                    dispatch_workgroups: None,
                    shader_id: value.shader_id,
                    shader_revision: value.shader_revision,
                    fallback_used: value.fallback_used,
                    pipeline_key: Some(value.pipeline_key),
                }),
            CompiledPassExecutionPlan::Copy(value) => self
                .encode_copy_pass(encoder, frame_texture, packet, runtime_resources, value)
                .map(|()| EncodedPassEvidence {
                    dispatch_workgroups: None,
                    shader_id: "builtin:copy".to_string(),
                    shader_revision: 0,
                    fallback_used: false,
                    pipeline_key: None,
                }),
            CompiledPassExecutionPlan::Present(value) => self
                .encode_present_pass(encoder, frame_texture, packet, runtime_resources, value)
                .map(|()| EncodedPassEvidence {
                    dispatch_workgroups: None,
                    shader_id: "builtin:present".to_string(),
                    shader_revision: 0,
                    fallback_used: false,
                    pipeline_key: None,
                }),
            CompiledPassExecutionPlan::BuiltinUiComposite(_value) => {
                self.encode_ui_pass(
                    device,
                    encoder,
                    frame_texture,
                    frame_view,
                    &packet.prepared_ui,
                    flow.flow_label.as_str(),
                    runtime_resources,
                    &packet.viewport_surface_bindings,
                    packet.surface_size,
                    packet.surface_format,
                );
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
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        flow_inputs: &PreparedFlowInputs,
        runtime_resources: &FlowRuntimeResources,
        pass: &CompiledComputeExecutionPlan,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<EncodedPipelinePass> {
        let shader = resolve_shader_material(
            pass.shader.as_ref(),
            shader_registry,
            DEFAULT_COMPUTE_SHADER,
            "builtin:compute",
        );
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

        let (pipeline_key, bind_group_layout, bind_group) = self.resolve_compiled_bind_group(
            device,
            frame_texture,
            packet,
            flow,
            pass.pass_id,
            FlowPassKind::Compute,
            pass.feature_id,
            shader.pipeline_identity.as_str(),
            shader.revision,
            &pass.bindings,
            ShaderStages::COMPUTE,
            true,
            Vec::new(),
            None,
            0,
            FlowPrimitiveTopologyClass::None,
            runtime_resources,
        )?;

        let shader_module =
            self.flow_pipeline_cache
                .get_or_create_shader_module(pipeline_key.clone(), || {
                    device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("engine_compiled_compute_shader"),
                        source: ShaderSource::Wgsl(shader.source.into()),
                    })
                });

        let pipeline_layout = bind_group_layout.as_ref().map(|layout| {
            self.flow_pipeline_cache
                .get_or_create_pipeline_layout(pipeline_key.clone(), || {
                    device.create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some("engine_compiled_compute_pipeline_layout"),
                        bind_group_layouts: &[layout],
                        push_constant_ranges: &[],
                    })
                })
        });

        let pipeline =
            self.flow_pipeline_cache
                .get_or_create_compute_pipeline(pipeline_key.clone(), || {
                    device.create_compute_pipeline(&ComputePipelineDescriptor {
                        label: Some("engine_compiled_compute_pipeline"),
                        layout: pipeline_layout.as_ref(),
                        module: &shader_module,
                        entry_point: Some("cs_main"),
                        compilation_options: PipelineCompilationOptions::default(),
                        cache: None,
                    })
                });

        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("engine_compiled_compute_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        if let Some(bind_group) = bind_group.as_ref() {
            pass.set_bind_group(0, bind_group, &[]);
        }
        pass.dispatch_workgroups(dispatch[0], dispatch[1], dispatch[2]);
        Ok(EncodedPipelinePass {
            dispatch_workgroups: Some(dispatch),
            shader_id: shader.shader_id,
            shader_revision: shader.revision,
            fallback_used: shader.fallback_used,
            pipeline_key,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_fullscreen_pass(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        runtime_resources: &FlowRuntimeResources,
        plan: &CompiledRasterExecutionPlan,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<EncodedPipelinePass> {
        if !plan.draw_buffers.vertex_buffers.is_empty()
            || !plan.draw_buffers.index_buffers.is_empty()
            || !plan.draw_buffers.instance_buffers.is_empty()
            || !plan.draw_buffers.indirect_buffers.is_empty()
        {
            bail!(
                "fullscreen pass '{}' cannot bind graphics vertex/index/instance/indirect buffers",
                plan.pass_id
            );
        }

        let color_target = self.resolve_color_target_from_plan(
            runtime_resources,
            plan.pass_id,
            &plan.targets,
            frame_view,
            packet.surface_format,
        )?;

        let shader = resolve_shader_material_for_packet(
            plan.shader.as_ref(),
            packet,
            shader_registry,
            DEFAULT_FULLSCREEN_SHADER,
            "builtin:fullscreen",
        );
        reject_material_shader_fallback(
            plan.feature_id,
            plan.shader.as_ref(),
            plan.pass_id,
            &shader,
        )?;
        reject_unresident_material_textures(
            packet,
            plan.feature_id,
            plan.shader.as_ref(),
            plan.pass_id,
        )?;

        let (pipeline_key, bind_group_layout, bind_group) = self.resolve_compiled_bind_group(
            device,
            frame_texture,
            packet,
            flow,
            plan.pass_id,
            FlowPassKind::Fullscreen,
            plan.feature_id,
            shader.pipeline_identity.as_str(),
            shader.revision,
            &plan.bindings,
            ShaderStages::VERTEX_FRAGMENT,
            true,
            vec![color_target.format],
            None,
            0,
            FlowPrimitiveTopologyClass::TriangleList,
            runtime_resources,
        )?;

        let shader_module =
            self.flow_pipeline_cache
                .get_or_create_shader_module(pipeline_key.clone(), || {
                    device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("engine_compiled_fullscreen_shader"),
                        source: ShaderSource::Wgsl(shader.source.into()),
                    })
                });

        let material_resources =
            material_resources_for_pass(packet, plan.feature_id, plan.shader.as_ref());
        let empty_group0 = if material_resources.is_some() && bind_group_layout.is_none() {
            let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_compiled_fullscreen_empty_group0_layout"),
                entries: &[],
            });
            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("engine_compiled_fullscreen_empty_group0_bind_group"),
                layout: &layout,
                entries: &[],
            });
            Some((layout, bind_group))
        } else {
            None
        };
        let mut pipeline_layout_entries = Vec::<&BindGroupLayout>::new();
        if let Some(layout) = bind_group_layout.as_ref() {
            pipeline_layout_entries.push(layout);
        } else if let Some((layout, _)) = empty_group0.as_ref() {
            pipeline_layout_entries.push(layout);
        }
        if let Some(resources) = material_resources {
            pipeline_layout_entries.push(resources.layout());
        }
        let pipeline_layout = if pipeline_layout_entries.is_empty() {
            None
        } else {
            Some(self.flow_pipeline_cache.get_or_create_pipeline_layout(
                pipeline_key.clone(),
                || {
                    device.create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some("engine_compiled_fullscreen_pipeline_layout"),
                        bind_group_layouts: &pipeline_layout_entries,
                        push_constant_ranges: &[],
                    })
                },
            ))
        };

        let pipeline =
            self.flow_pipeline_cache
                .get_or_create_render_pipeline(pipeline_key.clone(), || {
                    device.create_render_pipeline(&RenderPipelineDescriptor {
                        label: Some("engine_compiled_fullscreen_pipeline"),
                        layout: pipeline_layout.as_ref(),
                        vertex: VertexState {
                            module: &shader_module,
                            entry_point: Some("vs_main"),
                            compilation_options: PipelineCompilationOptions::default(),
                            buffers: &[],
                        },
                        fragment: Some(FragmentState {
                            module: &shader_module,
                            entry_point: Some("fs_main"),
                            compilation_options: PipelineCompilationOptions::default(),
                            targets: &[Some(ColorTargetState {
                                format: color_target.format,
                                blend: blend_state_for_color_format(color_target.format),
                                write_mask: ColorWrites::ALL,
                            })],
                        }),
                        primitive: PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: MultisampleState::default(),
                        multiview: None,
                        cache: None,
                    })
                });

        let load = match plan.clear_color {
            Some(color) => LoadOp::Clear(Color {
                r: color[0] as f64,
                g: color[1] as f64,
                b: color[2] as f64,
                a: color[3] as f64,
            }),
            None => LoadOp::Load,
        };

        let color_attachment = Some(RenderPassColorAttachment {
            view: color_target.view.as_ref(),
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load,
                store: StoreOp::Store,
            },
        });
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_compiled_fullscreen_pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        if let Some(bind_group) = bind_group.as_ref() {
            pass.set_bind_group(0, bind_group, &[]);
        } else if let Some((_, bind_group)) = empty_group0.as_ref() {
            pass.set_bind_group(0, bind_group, &[]);
        }
        if let Some(resources) = material_resources {
            pass.set_bind_group(1, resources.bind_group(), &[]);
        }
        pass.draw(0..3, 0..1);
        Ok(EncodedPipelinePass {
            dispatch_workgroups: None,
            shader_id: shader.shader_id,
            shader_revision: shader.revision,
            fallback_used: shader.fallback_used,
            pipeline_key,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_graphics_pass(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        runtime_resources: &FlowRuntimeResources,
        plan: &CompiledRasterExecutionPlan,
        shader_registry: &ShaderRegistryResource,
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

        let shader = resolve_shader_material_for_packet(
            plan.shader.as_ref(),
            packet,
            shader_registry,
            DEFAULT_GRAPHICS_SHADER,
            "builtin:graphics",
        );
        reject_material_shader_fallback(
            plan.feature_id,
            plan.shader.as_ref(),
            plan.pass_id,
            &shader,
        )?;
        reject_unresident_material_textures(
            packet,
            plan.feature_id,
            plan.shader.as_ref(),
            plan.pass_id,
        )?;

        let vertex_layout_signature_hash = plan.draw_buffers.vertex_layout_signature_hash();
        let (pipeline_key, bind_group_layout, bind_group) = self.resolve_compiled_bind_group(
            device,
            frame_texture,
            packet,
            flow,
            plan.pass_id,
            FlowPassKind::Graphics,
            plan.feature_id,
            shader.pipeline_identity.as_str(),
            shader.revision,
            &plan.bindings,
            ShaderStages::VERTEX_FRAGMENT,
            true,
            vec![color_target.format],
            depth_target.as_ref().map(|value| value.format),
            vertex_layout_signature_hash,
            FlowPrimitiveTopologyClass::TriangleList,
            runtime_resources,
        )?;

        let shader_module =
            self.flow_pipeline_cache
                .get_or_create_shader_module(pipeline_key.clone(), || {
                    device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("engine_compiled_graphics_shader"),
                        source: ShaderSource::Wgsl(shader.source.into()),
                    })
                });

        let material_resources =
            material_resources_for_pass(packet, plan.feature_id, plan.shader.as_ref());
        let empty_group0 = if material_resources.is_some() && bind_group_layout.is_none() {
            let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_compiled_graphics_empty_group0_layout"),
                entries: &[],
            });
            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("engine_compiled_graphics_empty_group0_bind_group"),
                layout: &layout,
                entries: &[],
            });
            Some((layout, bind_group))
        } else {
            None
        };
        let mut pipeline_layout_entries = Vec::<&BindGroupLayout>::new();
        if let Some(layout) = bind_group_layout.as_ref() {
            pipeline_layout_entries.push(layout);
        } else if let Some((layout, _)) = empty_group0.as_ref() {
            pipeline_layout_entries.push(layout);
        }
        if let Some(resources) = material_resources {
            pipeline_layout_entries.push(resources.layout());
        }
        let pipeline_layout = if pipeline_layout_entries.is_empty() {
            None
        } else {
            Some(self.flow_pipeline_cache.get_or_create_pipeline_layout(
                pipeline_key.clone(),
                || {
                    device.create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some("engine_compiled_graphics_pipeline_layout"),
                        bind_group_layouts: &pipeline_layout_entries,
                        push_constant_ranges: &[],
                    })
                },
            ))
        };

        let vertex_attribute_sets = build_vertex_attribute_sets(&plan.draw_buffers);
        let vertex_buffer_layouts =
            build_vertex_buffer_layouts(&plan.draw_buffers, &vertex_attribute_sets);

        let pipeline =
            self.flow_pipeline_cache
                .get_or_create_render_pipeline(pipeline_key.clone(), || {
                    device.create_render_pipeline(&RenderPipelineDescriptor {
                        label: Some("engine_compiled_graphics_pipeline"),
                        layout: pipeline_layout.as_ref(),
                        vertex: VertexState {
                            module: &shader_module,
                            entry_point: Some("vs_main"),
                            compilation_options: PipelineCompilationOptions::default(),
                            buffers: &vertex_buffer_layouts,
                        },
                        fragment: Some(FragmentState {
                            module: &shader_module,
                            entry_point: Some("fs_main"),
                            compilation_options: PipelineCompilationOptions::default(),
                            targets: &[Some(ColorTargetState {
                                format: color_target.format,
                                blend: blend_state_for_color_format(color_target.format),
                                write_mask: ColorWrites::ALL,
                            })],
                        }),
                        primitive: PrimitiveState::default(),
                        depth_stencil: depth_target.as_ref().map(|target| DepthStencilState {
                            format: target.format,
                            depth_write_enabled: true,
                            depth_compare: CompareFunction::LessEqual,
                            stencil: StencilState::default(),
                            bias: DepthBiasState::default(),
                        }),
                        multisample: MultisampleState::default(),
                        multiview: None,
                        cache: None,
                    })
                });

        let load = match plan.clear_color {
            Some(color) => LoadOp::Clear(Color {
                r: color[0] as f64,
                g: color[1] as f64,
                b: color[2] as f64,
                a: color[3] as f64,
            }),
            None => LoadOp::Load,
        };
        let color_attachment = Some(RenderPassColorAttachment {
            view: color_target.view.as_ref(),
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load,
                store: StoreOp::Store,
            },
        });
        let depth_attachment =
            depth_target
                .as_ref()
                .map(|target| RenderPassDepthStencilAttachment {
                    view: &target.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                });

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_compiled_graphics_pass"),
            color_attachments: &[color_attachment],
            depth_stencil_attachment: depth_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        if let Some(bind_group) = bind_group.as_ref() {
            pass.set_bind_group(0, bind_group, &[]);
        } else if let Some((_, bind_group)) = empty_group0.as_ref() {
            pass.set_bind_group(0, bind_group, &[]);
        }
        if let Some(resources) = material_resources {
            pass.set_bind_group(1, resources.bind_group(), &[]);
        }

        for binding in &plan.draw_buffers.vertex_buffers {
            let buffer =
                runtime_resources.resolve_storage_buffer_ref(plan.pass_id, &binding.resource)?;
            pass.set_vertex_buffer(binding.layout.slot, buffer.buffer.slice(..));
        }
        for (resource, layout) in plan
            .draw_buffers
            .instance_buffers
            .iter()
            .zip(plan.draw_buffers.instance_buffer_layouts.iter())
        {
            let buffer = runtime_resources.resolve_storage_buffer_ref(plan.pass_id, resource)?;
            pass.set_vertex_buffer(layout.slot, buffer.buffer.slice(..));
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
        if let Some(ref index) = index_buffer {
            pass.set_index_buffer(index.buffer.slice(..), IndexFormat::Uint32);
        }

        let indirect_buffer = match plan.draw_buffers.indirect_buffers.as_slice() {
            [] => None,
            [only] => Some(runtime_resources.resolve_storage_buffer_ref(plan.pass_id, only)?),
            _ => {
                bail!(
                    "graphics pass '{}' declares multiple indirect_buffer(...) resources; runtime currently supports exactly one",
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

        let vertex_range = draw.first_vertex..draw.first_vertex + draw.vertex_count;
        let instance_range = draw.first_instance..draw.first_instance + draw.instance_count;

        match (index_buffer.is_some(), indirect_buffer) {
            (true, Some(indirect)) => pass.draw_indexed_indirect(indirect.buffer, 0),
            (false, Some(indirect)) => pass.draw_indirect(indirect.buffer, 0),
            (true, None) => pass.draw_indexed(vertex_range, 0, instance_range),
            (false, None) => pass.draw(vertex_range, instance_range),
        }
        Ok(EncodedPipelinePass {
            dispatch_workgroups: None,
            shader_id: shader.shader_id,
            shader_revision: shader.revision,
            fallback_used: shader.fallback_used,
            pipeline_key,
        })
    }

    fn encode_texture_copy(
        &self,
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

        encoder.copy_texture_to_texture(
            TexelCopyTextureInfo {
                texture: source.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyTextureInfo {
                texture: destination.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        Ok(())
    }

    fn encode_buffer_copy(
        &self,
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
        encoder.copy_buffer_to_buffer(source.buffer, 0, destination.buffer, 0, size);
        Ok(())
    }

    fn encode_copy_pass(
        &self,
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
                self.encode_buffer_copy(encoder, pass.pass_id, source, destination)
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
                self.encode_texture_copy(encoder, pass.pass_id, source, destination)
            }
        }
    }

    fn encode_present_pass(
        &self,
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
            texture: frame_texture,
            format: packet.surface_format,
            size: packet.surface_size,
            is_depth: false,
            generation: None,
        };
        self.encode_texture_copy(encoder, pass.pass_id, source, destination)
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
