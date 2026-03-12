use super::*;
use crate::plugins::render::backend::ensure_compiled_pass_is_supported;
use crate::plugins::render::graph::{CompiledPassDescriptor, CompiledRenderFlowPlan};
use crate::plugins::render::{RenderResourceDescriptor, RenderResourceId};
use anyhow::{Result, bail};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeResourceKind {
    TextureLike,
    BufferLike,
}

#[derive(Debug)]
struct RuntimeTextureResource {
    texture: Texture,
    format: TextureFormat,
    size: (u32, u32),
    is_depth: bool,
}

#[derive(Debug)]
struct RuntimeBufferResource {
    buffer: Buffer,
    size: u64,
}

#[derive(Debug, Default)]
struct FlowRuntimeResources {
    textures: BTreeMap<String, RuntimeTextureResource>,
    buffers: BTreeMap<String, RuntimeBufferResource>,
    kinds: BTreeMap<String, RuntimeResourceKind>,
}

#[derive(Debug)]
struct ResolvedTextureRef<'a> {
    id: &'a str,
    texture: &'a Texture,
    format: TextureFormat,
    size: (u32, u32),
    is_depth: bool,
}

#[derive(Debug)]
struct ResolvedBufferRef<'a> {
    id: &'a str,
    buffer: &'a Buffer,
    size: u64,
}

#[derive(Debug)]
enum RuntimeTextureView<'a> {
    Borrowed(&'a TextureView),
    Owned(TextureView),
}

impl<'a> RuntimeTextureView<'a> {
    fn as_ref(&self) -> &TextureView {
        match self {
            Self::Borrowed(value) => value,
            Self::Owned(value) => value,
        }
    }
}

#[derive(Debug)]
struct ResolvedColorTargetView<'a> {
    view: RuntimeTextureView<'a>,
    format: TextureFormat,
}

#[derive(Debug)]
struct ResolvedDepthTargetView {
    view: TextureView,
    format: TextureFormat,
}

impl FlowRuntimeResources {
    fn realize(
        device: &Device,
        flow: &CompiledRenderFlowPlan,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
    ) -> Self {
        let mut runtime = Self::default();

        for descriptor in &flow.resources.resources {
            let id = descriptor.id().as_str().to_string();
            let kind = match descriptor {
                RenderResourceDescriptor::UniformBuffer(_)
                | RenderResourceDescriptor::StorageBuffer(_)
                | RenderResourceDescriptor::ImportedBuffer(_) => RuntimeResourceKind::BufferLike,
                _ => RuntimeResourceKind::TextureLike,
            };
            runtime.kinds.insert(id.clone(), kind);

            let texture_allocation = match descriptor {
                RenderResourceDescriptor::SampledTexture(_)
                | RenderResourceDescriptor::ColorTarget(_)
                | RenderResourceDescriptor::HistoryTexture(_) => Some((
                    id.clone(),
                    surface_format,
                    TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT,
                    false,
                )),
                RenderResourceDescriptor::StorageTexture(_) => Some((
                    id.clone(),
                    TextureFormat::Rgba8Unorm,
                    TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST
                        | TextureUsages::STORAGE_BINDING,
                    false,
                )),
                RenderResourceDescriptor::DepthTarget(_) => Some((
                    id.clone(),
                    TextureFormat::Depth32Float,
                    TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT,
                    true,
                )),
                RenderResourceDescriptor::ImportedTexture(_)
                | RenderResourceDescriptor::UniformBuffer(_)
                | RenderResourceDescriptor::StorageBuffer(_)
                | RenderResourceDescriptor::ImportedBuffer(_) => None,
            };

            if let Some((resource_id, format, usage, is_depth)) = texture_allocation {
                let label = format!("engine_render_resource_{}", resource_id);
                let texture = device.create_texture(&TextureDescriptor {
                    label: Some(label.as_str()),
                    size: Extent3d {
                        width: surface_size.0.max(1),
                        height: surface_size.1.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                });
                runtime.textures.insert(
                    resource_id,
                    RuntimeTextureResource {
                        texture,
                        format,
                        size: (surface_size.0.max(1), surface_size.1.max(1)),
                        is_depth,
                    },
                );
            }

            let buffer_allocation = match descriptor {
                RenderResourceDescriptor::UniformBuffer(value) => Some((
                    id.clone(),
                    value.size_bytes,
                    BufferUsages::UNIFORM | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
                )),
                RenderResourceDescriptor::StorageBuffer(value) => Some((
                    id.clone(),
                    value.size_bytes,
                    BufferUsages::STORAGE
                        | BufferUsages::COPY_SRC
                        | BufferUsages::COPY_DST
                        | BufferUsages::VERTEX
                        | BufferUsages::INDEX
                        | BufferUsages::INDIRECT,
                )),
                RenderResourceDescriptor::SampledTexture(_)
                | RenderResourceDescriptor::StorageTexture(_)
                | RenderResourceDescriptor::ColorTarget(_)
                | RenderResourceDescriptor::DepthTarget(_)
                | RenderResourceDescriptor::HistoryTexture(_)
                | RenderResourceDescriptor::ImportedTexture(_)
                | RenderResourceDescriptor::ImportedBuffer(_) => None,
            };
            if let Some((resource_id, size, usage)) = buffer_allocation {
                let label = format!("engine_render_resource_{}", resource_id);
                let buffer = device.create_buffer(&BufferDescriptor {
                    label: Some(label.as_str()),
                    size: size.max(1),
                    usage,
                    mapped_at_creation: false,
                });
                runtime.buffers.insert(
                    resource_id,
                    RuntimeBufferResource {
                        buffer,
                        size: size.max(1),
                    },
                );
            }
        }

        runtime
    }

    fn kind_of(&self, id: &str) -> Option<RuntimeResourceKind> {
        self.kinds.get(id).copied().or_else(|| {
            if id == "surface.color" {
                Some(RuntimeResourceKind::TextureLike)
            } else {
                None
            }
        })
    }

    fn resolve_color_target_view<'a>(
        &'a self,
        pass_id: &str,
        writes: &[RenderResourceId],
        frame_view: &'a TextureView,
        frame_format: TextureFormat,
    ) -> Result<ResolvedColorTargetView<'a>> {
        if writes.len() != 1 {
            bail!(
                "pass '{}' declares {} writes(...) resources but core runtime execution requires exactly one color output target",
                pass_id,
                writes.len()
            );
        }

        let resource_id = writes[0].as_str();
        if resource_id == "surface.color" {
            return Ok(ResolvedColorTargetView {
                view: RuntimeTextureView::Borrowed(frame_view),
                format: frame_format,
            });
        }

        let kind = self.kind_of(resource_id).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' writes unknown resource '{}' during runtime encoding",
                pass_id,
                resource_id
            )
        })?;
        if matches!(kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "pass '{}' writes '{}' as a color target, but it is buffer-like",
                pass_id,
                resource_id
            );
        }

        let Some(texture) = self.textures.get(resource_id) else {
            bail!(
                "pass '{}' writes imported texture '{}' but only imported 'surface.color' is supported in core runtime execution",
                pass_id,
                resource_id
            );
        };
        if texture.is_depth {
            bail!(
                "pass '{}' writes '{}' as a color target, but it is depth-only",
                pass_id,
                resource_id
            );
        }

        Ok(ResolvedColorTargetView {
            view: RuntimeTextureView::Owned(
                texture
                    .texture
                    .create_view(&TextureViewDescriptor::default()),
            ),
            format: texture.format,
        })
    }

    fn resolve_depth_target_view(
        &self,
        pass_id: &str,
        depth_target: &RenderResourceId,
    ) -> Result<ResolvedDepthTargetView> {
        let resource_id = depth_target.as_str();
        if resource_id == "surface.color" {
            bail!(
                "graphics pass '{}' uses 'surface.color' as depth_target, which is not supported",
                pass_id
            );
        }

        let kind = self.kind_of(resource_id).ok_or_else(|| {
            anyhow::anyhow!(
                "graphics pass '{}' uses unknown depth target '{}'",
                pass_id,
                resource_id
            )
        })?;
        if matches!(kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "graphics pass '{}' uses '{}' as depth_target, but it is buffer-like",
                pass_id,
                resource_id
            );
        }

        let Some(texture) = self.textures.get(resource_id) else {
            bail!(
                "graphics pass '{}' uses imported depth target '{}' but core runtime execution only supports flow-owned depth targets",
                pass_id,
                resource_id
            );
        };
        if !texture.is_depth {
            bail!(
                "graphics pass '{}' uses '{}' as depth_target, but it is not a depth resource",
                pass_id,
                resource_id
            );
        }

        Ok(ResolvedDepthTargetView {
            view: texture
                .texture
                .create_view(&TextureViewDescriptor::default()),
            format: texture.format,
        })
    }

    fn resolve_texture<'a>(
        &'a self,
        pass_id: &str,
        resource_id: &'a str,
        frame_texture: &'a Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<ResolvedTextureRef<'a>> {
        if resource_id == "surface.color" {
            return Ok(ResolvedTextureRef {
                id: resource_id,
                texture: frame_texture,
                format: frame_format,
                size: frame_size,
                is_depth: false,
            });
        }

        let kind = self.kind_of(resource_id).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' references unknown resource '{}' during runtime encoding",
                pass_id,
                resource_id
            )
        })?;
        if matches!(kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "pass '{}' references '{}' as a texture, but it is buffer-like",
                pass_id,
                resource_id
            );
        }

        let Some(texture) = self.textures.get(resource_id) else {
            bail!(
                "pass '{}' references imported texture '{}' but only imported 'surface.color' is supported in core runtime execution",
                pass_id,
                resource_id
            );
        };

        Ok(ResolvedTextureRef {
            id: resource_id,
            texture: &texture.texture,
            format: texture.format,
            size: texture.size,
            is_depth: texture.is_depth,
        })
    }

    fn resolve_buffer<'a>(
        &'a self,
        pass_id: &str,
        resource_id: &'a str,
    ) -> Result<ResolvedBufferRef<'a>> {
        let kind = self.kind_of(resource_id).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' references unknown resource '{}' during runtime encoding",
                pass_id,
                resource_id
            )
        })?;
        if matches!(kind, RuntimeResourceKind::TextureLike) {
            bail!(
                "pass '{}' references '{}' as a buffer, but it is texture-like",
                pass_id,
                resource_id
            );
        }

        let Some(buffer) = self.buffers.get(resource_id) else {
            bail!(
                "pass '{}' references imported buffer '{}' but core runtime execution only supports flow-owned buffers",
                pass_id,
                resource_id
            );
        };

        Ok(ResolvedBufferRef {
            id: resource_id,
            buffer: &buffer.buffer,
            size: buffer.size,
        })
    }
}

impl Renderer {
    pub(crate) fn render_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_texture: &Texture,
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
            let runtime_resources = FlowRuntimeResources::realize(
                device,
                flow,
                packet.surface_size,
                packet.surface_format,
            );
            for pass in &flow.pass_order {
                ensure_compiled_pass_is_supported(pass)?;
                self.encode_compiled_pass(
                    device,
                    &mut encoder,
                    frame_texture,
                    frame_view,
                    &packet,
                    pass,
                    shader_registry,
                    &runtime_resources,
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
        frame_texture: &Texture,
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
            frame_texture,
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
        frame_texture: &Texture,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        pass: &CompiledPassDescriptor,
        shader_registry: &ShaderRegistryResource,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<()> {
        match pass {
            CompiledPassDescriptor::Compute(value) => {
                self.encode_compute_pass(device, encoder, packet, &value.node, shader_registry)
            }
            CompiledPassDescriptor::Fullscreen(value) => self.encode_fullscreen_pass(
                device,
                encoder,
                frame_view,
                packet,
                runtime_resources,
                &value.node,
                shader_registry,
            ),
            CompiledPassDescriptor::Graphics(value) => self.encode_graphics_pass(
                device,
                encoder,
                frame_view,
                packet,
                runtime_resources,
                &value.node,
                shader_registry,
            ),
            CompiledPassDescriptor::Copy(value) => self.encode_copy_pass(
                encoder,
                frame_texture,
                packet,
                runtime_resources,
                &value.node,
            ),
            CompiledPassDescriptor::Present(value) => self.encode_present_pass(
                encoder,
                frame_texture,
                packet,
                runtime_resources,
                &value.node,
            ),
            CompiledPassDescriptor::BuiltinUiComposite(_) => {
                self.encode_ui_pass(encoder, frame_view, &packet.prepared_ui);
                Ok(())
            }
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
        if !node.sampled_textures.is_empty()
            || !node.write_textures.is_empty()
            || !node.vertex_buffers.is_empty()
            || !node.index_buffers.is_empty()
            || !node.instance_buffers.is_empty()
            || !node.indirect_buffers.is_empty()
        {
            bail!(
                "compute pass '{}' declares sampled/write texture or graphics buffer bindings that are not yet supported by core backend execution",
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
        runtime_resources: &FlowRuntimeResources,
        node: &crate::plugins::render::RenderPassNode,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<()> {
        if !node.sampled_textures.is_empty() || !node.write_textures.is_empty() {
            bail!(
                "fullscreen pass '{}' declares sampled/write texture bindings that are not yet supported by core backend execution",
                node.id.as_str()
            );
        }
        if !node.vertex_buffers.is_empty()
            || !node.index_buffers.is_empty()
            || !node.instance_buffers.is_empty()
            || !node.indirect_buffers.is_empty()
        {
            bail!(
                "fullscreen pass '{}' declares graphics buffer bindings, which are not supported",
                node.id.as_str()
            );
        }

        let color_target = runtime_resources.resolve_color_target_view(
            node.id.as_str(),
            &node.writes,
            frame_view,
            packet.surface_format,
        )?;

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
        pass.draw(0..3, 0..1);
        Ok(())
    }

    fn encode_graphics_pass(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        runtime_resources: &FlowRuntimeResources,
        node: &crate::plugins::render::RenderPassNode,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<()> {
        if !node.sampled_textures.is_empty() || !node.write_textures.is_empty() {
            bail!(
                "graphics pass '{}' declares sampled/write textures, which are not yet supported by core graphics execution",
                node.id.as_str()
            );
        }
        if !node.vertex_buffers.is_empty()
            || !node.index_buffers.is_empty()
            || !node.instance_buffers.is_empty()
            || !node.indirect_buffers.is_empty()
        {
            bail!(
                "graphics pass '{}' declares vertex/index/instance/indirect buffers, but buffer binding execution is not implemented yet",
                node.id.as_str()
            );
        }

        let color_target = runtime_resources.resolve_color_target_view(
            node.id.as_str(),
            &node.writes,
            frame_view,
            packet.surface_format,
        )?;
        let depth_target = node
            .depth_target
            .as_ref()
            .map(|target| runtime_resources.resolve_depth_target_view(node.id.as_str(), target))
            .transpose()?;

        let shader_source = node
            .shader
            .as_deref()
            .map(|path| shader_registry.source_or(path, DEFAULT_GRAPHICS_SHADER))
            .unwrap_or(DEFAULT_GRAPHICS_SHADER);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_compiled_graphics_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_compiled_graphics_pipeline"),
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
        pass.draw(0..3, 0..1);
        Ok(())
    }

    fn encode_texture_copy(
        &self,
        encoder: &mut CommandEncoder,
        pass_id: &str,
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
        pass_id: &str,
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
        node: &crate::plugins::render::RenderPassNode,
    ) -> Result<()> {
        if node.reads.len() != 1 || node.writes.len() != 1 {
            bail!(
                "copy pass '{}' must declare exactly one reads(...) and one writes(...) resource",
                node.id.as_str()
            );
        }

        let source_id = node.reads[0].as_str();
        let destination_id = node.writes[0].as_str();
        if source_id == destination_id {
            return Ok(());
        }

        let source_kind = runtime_resources.kind_of(source_id).ok_or_else(|| {
            anyhow::anyhow!(
                "copy pass '{}' references unknown source resource '{}'",
                node.id.as_str(),
                source_id
            )
        })?;
        let destination_kind = runtime_resources.kind_of(destination_id).ok_or_else(|| {
            anyhow::anyhow!(
                "copy pass '{}' references unknown destination resource '{}'",
                node.id.as_str(),
                destination_id
            )
        })?;

        match (source_kind, destination_kind) {
            (RuntimeResourceKind::BufferLike, RuntimeResourceKind::BufferLike) => {
                let source = runtime_resources.resolve_buffer(node.id.as_str(), source_id)?;
                let destination =
                    runtime_resources.resolve_buffer(node.id.as_str(), destination_id)?;
                self.encode_buffer_copy(encoder, node.id.as_str(), source, destination)
            }
            (RuntimeResourceKind::BufferLike, RuntimeResourceKind::TextureLike)
            | (RuntimeResourceKind::TextureLike, RuntimeResourceKind::BufferLike) => {
                bail!(
                    "copy pass '{}' mixes incompatible resource classes '{}' -> '{}'",
                    node.id.as_str(),
                    source_id,
                    destination_id
                );
            }
            (RuntimeResourceKind::TextureLike, RuntimeResourceKind::TextureLike) => {
                let source = runtime_resources.resolve_texture(
                    node.id.as_str(),
                    source_id,
                    frame_texture,
                    packet.surface_size,
                    packet.surface_format,
                )?;
                let destination = runtime_resources.resolve_texture(
                    node.id.as_str(),
                    destination_id,
                    frame_texture,
                    packet.surface_size,
                    packet.surface_format,
                )?;
                self.encode_texture_copy(encoder, node.id.as_str(), source, destination)
            }
        }
    }

    fn encode_present_pass(
        &self,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        packet: &RendererPreparedPacket,
        runtime_resources: &FlowRuntimeResources,
        node: &crate::plugins::render::RenderPassNode,
    ) -> Result<()> {
        if node.reads.len() != 1 {
            bail!(
                "present pass '{}' must declare exactly one reads(...) resource",
                node.id.as_str()
            );
        }

        let source_id = node.reads[0].as_str();
        if source_id == "surface.color" {
            return Ok(());
        }

        let source_kind = runtime_resources.kind_of(source_id).ok_or_else(|| {
            anyhow::anyhow!(
                "present pass '{}' references unknown source resource '{}'",
                node.id.as_str(),
                source_id
            )
        })?;
        if matches!(source_kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "present pass '{}' reads buffer-like resource '{}' but present requires a texture-like source",
                node.id.as_str(),
                source_id
            );
        }

        let source = runtime_resources.resolve_texture(
            node.id.as_str(),
            source_id,
            frame_texture,
            packet.surface_size,
            packet.surface_format,
        )?;
        let destination = ResolvedTextureRef {
            id: "surface.color",
            texture: frame_texture,
            format: packet.surface_format,
            size: packet.surface_size,
            is_depth: false,
        };
        self.encode_texture_copy(encoder, node.id.as_str(), source, destination)
    }
}
