use super::*;
use crate::plugins::render::RenderPassId;

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

        let color_target = runtime_resources.resolve_color_target_from_plan(
            plan.pass_id,
            &plan.targets,
            frame_view,
            packet.surface_format,
        )?;

        let shader = resolve_shader_material(
            plan.shader.as_ref(),
            shader_registry,
            DEFAULT_FULLSCREEN_SHADER,
            "builtin:fullscreen",
        );

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

        let pipeline_layout = bind_group_layout.as_ref().map(|layout| {
            self.flow_pipeline_cache
                .get_or_create_pipeline_layout(pipeline_key.clone(), || {
                    device.create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some("engine_compiled_fullscreen_pipeline_layout"),
                        bind_group_layouts: &[layout],
                        push_constant_ranges: &[],
                    })
                })
        });

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
                                blend: Some(BlendState::ALPHA_BLENDING),
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
        let color_target = runtime_resources.resolve_color_target_from_plan(
            plan.pass_id,
            &plan.targets,
            frame_view,
            packet.surface_format,
        )?;
        let depth_target =
            runtime_resources.resolve_depth_target_from_plan(plan.pass_id, &plan.targets)?;

        let shader = resolve_shader_material(
            plan.shader.as_ref(),
            shader_registry,
            DEFAULT_GRAPHICS_SHADER,
            "builtin:graphics",
        );

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

        let pipeline_layout = bind_group_layout.as_ref().map(|layout| {
            self.flow_pipeline_cache
                .get_or_create_pipeline_layout(pipeline_key.clone(), || {
                    device.create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some("engine_compiled_graphics_pipeline_layout"),
                        bind_group_layouts: &[layout],
                        push_constant_ranges: &[],
                    })
                })
        });

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
                            buffers: &[],
                        },
                        fragment: Some(FragmentState {
                            module: &shader_module,
                            entry_point: Some("fs_main"),
                            compilation_options: PipelineCompilationOptions::default(),
                            targets: &[Some(ColorTargetState {
                                format: color_target.format,
                                blend: Some(BlendState::ALPHA_BLENDING),
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
        }

        let mut vertex_slot = 0u32;
        for resource in &plan.draw_buffers.vertex_buffers {
            let buffer = runtime_resources.resolve_storage_buffer_ref(plan.pass_id, resource)?;
            pass.set_vertex_buffer(vertex_slot, buffer.buffer.slice(..));
            vertex_slot = vertex_slot.saturating_add(1);
        }
        for resource in &plan.draw_buffers.instance_buffers {
            let buffer = runtime_resources.resolve_storage_buffer_ref(plan.pass_id, resource)?;
            pass.set_vertex_buffer(vertex_slot, buffer.buffer.slice(..));
            vertex_slot = vertex_slot.saturating_add(1);
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

        match (index_buffer.is_some(), indirect_buffer) {
            (true, Some(indirect)) => pass.draw_indexed_indirect(indirect.buffer, 0),
            (false, Some(indirect)) => pass.draw_indirect(indirect.buffer, 0),
            (true, None) => pass.draw_indexed(0..3, 0, 0..1),
            (false, None) => pass.draw(0..3, 0..1),
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
        if source.format != destination.format {
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
            .kind_of_resource(source_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "copy pass '{}' references unknown source resource '{}'",
                    pass.pass_id,
                    source_id
                )
            })?;
        let destination_kind = runtime_resources
            .kind_of_resource(destination_id)
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
                let source = runtime_resources.resolve_texture(
                    pass.pass_id,
                    source_id,
                    frame_texture,
                    packet.surface_size,
                    packet.surface_format,
                )?;
                let destination = runtime_resources.resolve_texture(
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
            .kind_of_resource(source_id)
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

        let source = runtime_resources.resolve_texture(
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
}
