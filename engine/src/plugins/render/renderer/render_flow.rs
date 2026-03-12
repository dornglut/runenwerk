use super::*;
use crate::plugins::render::backend::ensure_compiled_pass_is_supported;
use crate::plugins::render::graph::{CompiledPassDescriptor, CompiledRenderFlowPlan};
use anyhow::{Result, bail};

impl Renderer {
    pub(crate) fn render_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_view: &TextureView,
        _frame_data: &RenderFrameDataRegistry<'_>,
        packet: RendererPreparedPacket,
        compiled_flows: &[CompiledRenderFlowPlan],
        shader_registry: &ShaderRegistryResource,
    ) -> Result<RendererFrameTimings> {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_render_encoder"),
        });

        for flow in compiled_flows {
            for pass in &flow.pass_order {
                ensure_compiled_pass_is_supported(pass)?;
                self.encode_compiled_pass(
                    device,
                    &mut encoder,
                    frame_view,
                    &packet,
                    pass,
                    shader_registry,
                )?;
            }
        }

        let mut timings = packet.prepare_timings;
        let encode_submit_start = Instant::now();
        {
            let _span = tracing::info_span!("renderer.encode_submit").entered();
            queue.submit(std::iter::once(encoder.finish()));
        }
        timings.encode_submit_ms = encode_submit_start.elapsed().as_secs_f32() * 1000.0;
        Ok(timings)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_view: &TextureView,
        frame_data: &RenderFrameDataRegistry<'_>,
        draw_list: &UiDrawList,
        shader_registry: &mut ShaderRegistryResource,
        compiled_flows: &[CompiledRenderFlowPlan],
        ui_rect_shader: Option<ShaderHandle>,
        surface_format: TextureFormat,
        surface_width: f32,
        surface_height: f32,
    ) -> Result<RendererFrameTimings> {
        let packet = self.prepare_packet(
            device,
            queue,
            frame_data,
            draw_list,
            shader_registry,
            ui_rect_shader,
            surface_format,
            surface_width,
            surface_height,
        );
        self.render_packet(
            device,
            queue,
            frame_view,
            frame_data,
            packet,
            compiled_flows,
            shader_registry,
        )
    }

    fn encode_compiled_pass(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        pass: &CompiledPassDescriptor,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<()> {
        match pass {
            CompiledPassDescriptor::Compute(value) => {
                self.encode_compute_pass(device, encoder, packet, &value.node, shader_registry)
            }
            CompiledPassDescriptor::Fullscreen(value) => {
                self.encode_fullscreen_pass(
                    device,
                    encoder,
                    frame_view,
                    packet,
                    &value.node,
                    shader_registry,
                )
            }
            CompiledPassDescriptor::Copy(value) => self.encode_copy_pass(&value.node),
            CompiledPassDescriptor::Present(value) => self.encode_present_pass(&value.node),
            CompiledPassDescriptor::BuiltinUiComposite(_) => {
                self.encode_ui_pass(encoder, frame_view, &packet.prepared_ui);
                Ok(())
            }
            CompiledPassDescriptor::Graphics(value) => bail!(
                "graphics pass '{}' is declared but runtime graphics execution is not implemented",
                value.node.id.as_str()
            ),
        }
    }

    fn encode_compute_pass(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        packet: &RendererPreparedPacket,
        node: &crate::plugins::render::RenderPassNode,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<()> {
        if !node.uniform_bindings.is_empty()
            || !node.sampled_textures.is_empty()
            || !node.write_textures.is_empty()
            || !node.reads.is_empty()
            || !node.writes.is_empty()
            || !node.vertex_buffers.is_empty()
            || !node.index_buffers.is_empty()
            || !node.instance_buffers.is_empty()
            || !node.indirect_buffers.is_empty()
        {
            bail!(
                "compute pass '{}' requires runtime resource bindings not yet supported by core backend execution",
                node.id.as_str()
            );
        }

        let shader_source = node
            .shader
            .as_deref()
            .map(|path| shader_registry.source_or(path, DEFAULT_COMPUTE_SHADER))
            .unwrap_or(DEFAULT_COMPUTE_SHADER);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_compiled_compute_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("engine_compiled_compute_pipeline"),
            layout: None,
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let workgroup = node.workgroup_size.unwrap_or([1, 1, 1]);
        if workgroup[0] == 0 || workgroup[1] == 0 || workgroup[2] == 0 {
            bail!(
                "compute pass '{}' declared an invalid workgroup size",
                node.id.as_str()
            );
        }
        let dispatch_x = packet.surface_size.0.max(1).div_ceil(workgroup[0]);
        let dispatch_y = packet.surface_size.1.max(1).div_ceil(workgroup[1]);

        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("engine_compiled_compute_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.dispatch_workgroups(dispatch_x, dispatch_y, workgroup[2].max(1));
        Ok(())
    }

    fn encode_fullscreen_pass(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        node: &crate::plugins::render::RenderPassNode,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<()> {
        self.ensure_surface_color_write(node.id.as_str(), &node.writes)?;

        if !node.uniform_bindings.is_empty()
            || !node.sampled_textures.is_empty()
            || !node.write_textures.is_empty()
            || node
                .reads
                .iter()
                .any(|id| id.as_str() != "surface.color" && id.as_str() != "ui.draw_list")
        {
            bail!(
                "fullscreen pass '{}' requires runtime texture/uniform bindings not yet supported by core backend execution",
                node.id.as_str()
            );
        }

        let shader_source = node
            .shader
            .as_deref()
            .map(|path| shader_registry.source_or(path, DEFAULT_FULLSCREEN_SHADER))
            .unwrap_or(DEFAULT_FULLSCREEN_SHADER);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_compiled_fullscreen_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_compiled_fullscreen_pipeline"),
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: packet.surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let load = match node.clear_color {
            Some(color) => LoadOp::Clear(Color {
                r: color[0] as f64,
                g: color[1] as f64,
                b: color[2] as f64,
                a: color[3] as f64,
            }),
            None => LoadOp::Load,
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_compiled_fullscreen_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
        Ok(())
    }

    fn encode_copy_pass(
        &self,
        node: &crate::plugins::render::RenderPassNode,
    ) -> Result<()> {
        bail!(
            "copy pass '{}' is declared but runtime copy execution is not implemented",
            node.id.as_str()
        );
    }

    fn encode_present_pass(&self, node: &crate::plugins::render::RenderPassNode) -> Result<()> {
        bail!(
            "present pass '{}' is declared but runtime present execution is not implemented",
            node.id.as_str()
        );
    }
}
