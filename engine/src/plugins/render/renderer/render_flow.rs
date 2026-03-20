use super::*;
use crate::plugins::render::api::project_uniform_bindings_for_pass;
use crate::plugins::render::backend::ensure_compiled_pass_is_supported;
use crate::plugins::render::graph::{
    CompiledPassDescriptor, CompiledRenderFlowPlan, RenderPassNode, RenderShaderReference,
    ResourceGraph,
};
use crate::plugins::render::inspect::{
    PassTimingSample, RuntimeResourceInspectionEntry, RuntimeResourceReuse, resource_kind_name,
};
use crate::plugins::render::{RenderResourceDescriptor, RenderResourceId};
use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeResourceKind {
    TextureLike,
    BufferLike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeBufferKind {
    Uniform,
    Storage,
}

#[derive(Debug)]
struct RuntimeTextureResource {
    texture: Texture,
    format: TextureFormat,
    size: (u32, u32),
    is_depth: bool,
    generation: u64,
    reused_last_frame: bool,
}

#[derive(Debug)]
struct RuntimeBufferResource {
    buffer: Buffer,
    size: u64,
    kind: RuntimeBufferKind,
    generation: u64,
    reused_last_frame: bool,
}

#[derive(Debug, Clone, Copy)]
struct TextureAllocationSpec {
    format: TextureFormat,
    usage: TextureUsages,
    is_depth: bool,
}

#[derive(Debug, Clone, Copy)]
struct BufferAllocationSpec {
    size: u64,
    usage: BufferUsages,
    kind: RuntimeBufferKind,
}

#[derive(Debug, Default)]
pub(super) struct FlowRuntimeResources {
    textures: BTreeMap<String, RuntimeTextureResource>,
    buffers: BTreeMap<String, RuntimeBufferResource>,
    kinds: BTreeMap<String, RuntimeResourceKind>,
    descriptors: BTreeMap<String, RenderResourceDescriptor>,
}

#[derive(Debug)]
struct ResolvedTextureRef<'a> {
    id: String,
    texture: &'a Texture,
    format: TextureFormat,
    size: (u32, u32),
    is_depth: bool,
}

#[derive(Debug)]
struct ResolvedBufferRef<'a> {
    id: String,
    buffer: &'a Buffer,
    size: u64,
    kind: RuntimeBufferKind,
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
    fn realize_for_frame(
        &mut self,
        device: &Device,
        flow: &CompiledRenderFlowPlan,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
    ) {
        let frame_size = (surface_size.0.max(1), surface_size.1.max(1));
        let mut declared_ids = BTreeSet::<String>::new();

        self.kinds.clear();
        self.descriptors.clear();

        for descriptor in &flow.resources.resources {
            let id = descriptor.id().as_str().to_string();
            declared_ids.insert(id.clone());
            self.descriptors.insert(id.clone(), descriptor.clone());

            let kind = match descriptor {
                RenderResourceDescriptor::UniformBuffer(_)
                | RenderResourceDescriptor::StorageBuffer(_)
                | RenderResourceDescriptor::ImportedBuffer(_) => RuntimeResourceKind::BufferLike,
                _ => RuntimeResourceKind::TextureLike,
            };
            self.kinds.insert(id.clone(), kind);

            if let Some(texture_spec) = Self::texture_allocation_spec(descriptor, surface_format) {
                let previous_generation = self
                    .textures
                    .get(&id)
                    .map(|existing| existing.generation)
                    .unwrap_or(0);
                let should_recreate = match self.textures.get(&id) {
                    Some(existing) => {
                        descriptor.lifetime().is_transient()
                            || existing.format != texture_spec.format
                            || existing.size != frame_size
                            || existing.is_depth != texture_spec.is_depth
                    }
                    None => true,
                };

                if should_recreate {
                    let label = format!("engine_render_resource_{}", id);
                    let texture = device.create_texture(&TextureDescriptor {
                        label: Some(label.as_str()),
                        size: Extent3d {
                            width: frame_size.0,
                            height: frame_size.1,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: TextureDimension::D2,
                        format: texture_spec.format,
                        usage: texture_spec.usage,
                        view_formats: &[],
                    });
                    self.textures.insert(
                        id.clone(),
                        RuntimeTextureResource {
                            texture,
                            format: texture_spec.format,
                            size: frame_size,
                            is_depth: texture_spec.is_depth,
                            generation: previous_generation.saturating_add(1),
                            reused_last_frame: false,
                        },
                    );
                } else if let Some(existing) = self.textures.get_mut(&id) {
                    existing.reused_last_frame = true;
                }
            } else {
                self.textures.remove(&id);
            }

            if let Some(buffer_spec) = Self::buffer_allocation_spec(descriptor) {
                let previous_generation = self
                    .buffers
                    .get(&id)
                    .map(|existing| existing.generation)
                    .unwrap_or(0);
                let should_recreate = match self.buffers.get(&id) {
                    Some(existing) => {
                        descriptor.lifetime().is_transient()
                            || existing.size != buffer_spec.size.max(1)
                            || existing.kind != buffer_spec.kind
                    }
                    None => true,
                };

                if should_recreate {
                    let label = format!("engine_render_resource_{}", id);
                    let buffer = device.create_buffer(&BufferDescriptor {
                        label: Some(label.as_str()),
                        size: buffer_spec.size.max(1),
                        usage: buffer_spec.usage,
                        mapped_at_creation: false,
                    });
                    self.buffers.insert(
                        id.clone(),
                        RuntimeBufferResource {
                            buffer,
                            size: buffer_spec.size.max(1),
                            kind: buffer_spec.kind,
                            generation: previous_generation.saturating_add(1),
                            reused_last_frame: false,
                        },
                    );
                } else if let Some(existing) = self.buffers.get_mut(&id) {
                    existing.reused_last_frame = true;
                }
            } else {
                self.buffers.remove(&id);
            }
        }

        self.textures.retain(|id, _| declared_ids.contains(id));
        self.buffers.retain(|id, _| declared_ids.contains(id));
    }

    fn texture_allocation_spec(
        descriptor: &RenderResourceDescriptor,
        surface_format: TextureFormat,
    ) -> Option<TextureAllocationSpec> {
        match descriptor {
            RenderResourceDescriptor::SampledTexture(_)
            | RenderResourceDescriptor::ColorTarget(_)
            | RenderResourceDescriptor::HistoryTexture(_) => Some(TextureAllocationSpec {
                format: surface_format,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                is_depth: false,
            }),
            RenderResourceDescriptor::StorageTexture(_) => Some(TextureAllocationSpec {
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST
                    | TextureUsages::STORAGE_BINDING,
                is_depth: false,
            }),
            RenderResourceDescriptor::DepthTarget(_) => Some(TextureAllocationSpec {
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                is_depth: true,
            }),
            RenderResourceDescriptor::ImportedTexture(_)
            | RenderResourceDescriptor::UniformBuffer(_)
            | RenderResourceDescriptor::StorageBuffer(_)
            | RenderResourceDescriptor::ImportedBuffer(_) => None,
        }
    }

    fn buffer_allocation_spec(descriptor: &RenderResourceDescriptor) -> Option<BufferAllocationSpec> {
        match descriptor {
            RenderResourceDescriptor::UniformBuffer(value) => Some(BufferAllocationSpec {
                size: value.size_bytes,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
                kind: RuntimeBufferKind::Uniform,
            }),
            RenderResourceDescriptor::StorageBuffer(value) => Some(BufferAllocationSpec {
                size: value.size_bytes,
                usage: BufferUsages::STORAGE
                    | BufferUsages::COPY_SRC
                    | BufferUsages::COPY_DST
                    | BufferUsages::VERTEX
                    | BufferUsages::INDEX
                    | BufferUsages::INDIRECT,
                kind: RuntimeBufferKind::Storage,
            }),
            RenderResourceDescriptor::SampledTexture(_)
            | RenderResourceDescriptor::StorageTexture(_)
            | RenderResourceDescriptor::ColorTarget(_)
            | RenderResourceDescriptor::DepthTarget(_)
            | RenderResourceDescriptor::HistoryTexture(_)
            | RenderResourceDescriptor::ImportedTexture(_)
            | RenderResourceDescriptor::ImportedBuffer(_) => None,
        }
    }

    fn descriptor_of(&self, id: &str) -> Option<&RenderResourceDescriptor> {
        self.descriptors.get(id)
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
        resource_id: &str,
        frame_texture: &'a Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<ResolvedTextureRef<'a>> {
        if resource_id == "surface.color" {
            return Ok(ResolvedTextureRef {
                id: resource_id.to_string(),
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
            id: resource_id.to_string(),
            texture: &texture.texture,
            format: texture.format,
            size: texture.size,
            is_depth: texture.is_depth,
        })
    }

    fn resolve_buffer<'a>(&'a self, pass_id: &str, resource_id: &str) -> Result<ResolvedBufferRef<'a>> {
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
            id: resource_id.to_string(),
            buffer: &buffer.buffer,
            size: buffer.size,
            kind: buffer.kind,
        })
    }

    fn resolve_uniform_buffer<'a>(
        &'a self,
        pass_id: &str,
        resource_id: &str,
    ) -> Result<ResolvedBufferRef<'a>> {
        let resolved = self.resolve_buffer(pass_id, resource_id)?;
        if !matches!(resolved.kind, RuntimeBufferKind::Uniform) {
            bail!(
                "pass '{}' binds '{}' as a uniform buffer but the resource is not uniform",
                pass_id,
                resource_id
            );
        }
        Ok(resolved)
    }

    fn resolve_storage_buffer<'a>(
        &'a self,
        pass_id: &str,
        resource_id: &str,
    ) -> Result<ResolvedBufferRef<'a>> {
        let resolved = self.resolve_buffer(pass_id, resource_id)?;
        if !matches!(resolved.kind, RuntimeBufferKind::Storage) {
            bail!(
                "pass '{}' binds '{}' as a storage buffer but the resource is not storage",
                pass_id,
                resource_id
            );
        }
        Ok(resolved)
    }

    fn inspect_entries(&self, flow_id: &str) -> Vec<RuntimeResourceInspectionEntry> {
        let mut entries = Vec::<RuntimeResourceInspectionEntry>::new();
        for (id, descriptor) in &self.descriptors {
            let lifetime = descriptor.lifetime();
            let imported = lifetime.is_imported();
            let kind = resource_kind_name(descriptor).to_string();

            let mut realized = false;
            let mut reuse = RuntimeResourceReuse::NotRealized;
            let mut size_bytes = None::<u64>;
            let mut texture_size = None::<(u32, u32)>;
            let mut element_count = None::<u64>;
            let mut generation = None::<u64>;

            match descriptor {
                RenderResourceDescriptor::UniformBuffer(_) => {
                    if let Some(buffer) = self.buffers.get(id) {
                        realized = true;
                        reuse = if buffer.reused_last_frame {
                            RuntimeResourceReuse::Reused
                        } else {
                            RuntimeResourceReuse::Created
                        };
                        size_bytes = Some(buffer.size);
                        element_count = Some(1);
                        generation = Some(buffer.generation);
                    }
                }
                RenderResourceDescriptor::StorageBuffer(value) => {
                    if let Some(buffer) = self.buffers.get(id) {
                        realized = true;
                        reuse = if buffer.reused_last_frame {
                            RuntimeResourceReuse::Reused
                        } else {
                            RuntimeResourceReuse::Created
                        };
                        size_bytes = Some(buffer.size);
                        element_count = Some(value.element_count);
                        generation = Some(buffer.generation);
                    } else {
                        element_count = Some(value.element_count);
                    }
                }
                RenderResourceDescriptor::SampledTexture(_)
                | RenderResourceDescriptor::StorageTexture(_)
                | RenderResourceDescriptor::ColorTarget(_)
                | RenderResourceDescriptor::DepthTarget(_)
                | RenderResourceDescriptor::HistoryTexture(_) => {
                    if let Some(texture) = self.textures.get(id) {
                        realized = true;
                        reuse = if texture.reused_last_frame {
                            RuntimeResourceReuse::Reused
                        } else {
                            RuntimeResourceReuse::Created
                        };
                        texture_size = Some(texture.size);
                        generation = Some(texture.generation);
                    }
                }
                RenderResourceDescriptor::ImportedTexture(_)
                | RenderResourceDescriptor::ImportedBuffer(_) => {}
            }

            entries.push(RuntimeResourceInspectionEntry {
                flow_id: flow_id.to_string(),
                id: id.clone(),
                kind,
                lifetime,
                imported,
                realized,
                reuse,
                size_bytes,
                texture_size,
                element_count,
                generation,
            });
        }
        entries
    }
}

impl Renderer {
    pub(crate) fn render_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_texture: &Texture,
        frame_view: &TextureView,
        frame_data: &RenderFrameDataRegistry<'_>,
        packet: RendererPreparedPacket,
        compiled_flows: &[CompiledRenderFlowPlan],
        shader_registry: &ShaderRegistryResource,
    ) -> Result<RendererFrameTimings> {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_render_encoder"),
        });
        self.last_pass_timings.clear();
        self.last_runtime_resources.clear();

        let mut flow_runtime_cache = std::mem::take(&mut self.flow_runtime_cache);
        let render_result = (|| -> Result<()> {
            let active_flow_ids = compiled_flows
                .iter()
                .map(|flow| flow.flow_id.as_str())
                .collect::<BTreeSet<_>>();
            flow_runtime_cache.retain(|flow_id, _| active_flow_ids.contains(flow_id.as_str()));

            for flow in compiled_flows {
                let runtime_resources = flow_runtime_cache.entry(flow.flow_id.clone()).or_default();
                runtime_resources.realize_for_frame(
                    device,
                    flow,
                    packet.surface_size,
                    packet.surface_format,
                );
                self.last_runtime_resources
                    .extend(runtime_resources.inspect_entries(flow.flow_id.as_str()));

                self.upload_projected_uniform_buffers(
                    queue,
                    flow,
                    frame_data,
                    packet.surface_size,
                    runtime_resources,
                )?;

                for pass in &flow.pass_order {
                    ensure_compiled_pass_is_supported(pass)?;
                    let pass_encode_start = Instant::now();
                    let dispatch_workgroups = self.encode_compiled_pass(
                        device,
                        &mut encoder,
                        frame_texture,
                        frame_view,
                        frame_data,
                        &packet,
                        pass,
                        &flow.resources,
                        shader_registry,
                        runtime_resources,
                    )?;
                    self.last_pass_timings.push(PassTimingSample {
                        flow_id: flow.flow_id.clone(),
                        pass_id: pass.pass_id().to_string(),
                        pass_kind: pass_kind_name(pass.node()).to_string(),
                        millis: pass_encode_start.elapsed().as_secs_f32() * 1000.0,
                        dispatch_workgroups,
                    });
                }
            }
            Ok(())
        })();
        self.flow_runtime_cache = flow_runtime_cache;
        render_result?;

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

    fn upload_projected_uniform_buffers(
        &self,
        queue: &Queue,
        flow: &CompiledRenderFlowPlan,
        frame_data: &RenderFrameDataRegistry<'_>,
        surface_size: (u32, u32),
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<()> {
        let mut projected_by_buffer = BTreeMap::<String, (Vec<u8>, String)>::new();

        for pass in &flow.pass_order {
            let pass_id = pass.pass_id().to_string();
            let pass_buffers = project_uniform_bindings_for_pass(
                pass.node(),
                &flow.resources,
                frame_data,
                surface_size,
            )
            .map_err(|errors| {
                anyhow::anyhow!(
                    "uniform projection failed for flow '{}': {}",
                    flow.flow_id,
                    errors
                        .into_iter()
                        .map(|err| err.to_string())
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            })?;

            for projected in pass_buffers {
                let key = projected.buffer_id.as_str().to_string();
                if let Some((existing, first_pass)) = projected_by_buffer.get(&key) {
                    if existing != &projected.bytes {
                        bail!(
                            "uniform projection conflict for buffer '{}' in flow '{}': pass '{}' and pass '{}' wrote different bytes",
                            projected.buffer_id.as_str(),
                            flow.flow_id,
                            first_pass,
                            pass_id
                        );
                    }
                    continue;
                }

                projected_by_buffer.insert(key, (projected.bytes, pass_id.clone()));
            }
        }

        for (buffer_id, (bytes, _pass_id)) in projected_by_buffer {
            let runtime_buffer =
                runtime_resources.resolve_uniform_buffer("uniform.upload", buffer_id.as_str())?;
            if bytes.len() as u64 > runtime_buffer.size {
                bail!(
                    "uniform upload for '{}' writes {} bytes but runtime buffer size is {}",
                    buffer_id,
                    bytes.len(),
                    runtime_buffer.size
                );
            }
            queue.write_buffer(runtime_buffer.buffer, 0, &bytes);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_compiled_pass(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_view: &TextureView,
        frame_data: &RenderFrameDataRegistry<'_>,
        packet: &RendererPreparedPacket,
        pass: &CompiledPassDescriptor,
        flow_resources: &ResourceGraph,
        shader_registry: &ShaderRegistryResource,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<Option<[u32; 3]>> {
        match pass {
            CompiledPassDescriptor::Compute(value) => self.encode_compute_pass(
                device,
                encoder,
                frame_texture,
                frame_data,
                packet,
                runtime_resources,
                flow_resources,
                &value.node,
                shader_registry,
            )
            .map(Some),
            CompiledPassDescriptor::Fullscreen(value) => self.encode_fullscreen_pass(
                device,
                encoder,
                frame_texture,
                frame_view,
                packet,
                runtime_resources,
                flow_resources,
                &value.node,
                shader_registry,
            )
            .map(|()| None),
            CompiledPassDescriptor::Graphics(value) => self.encode_graphics_pass(
                device,
                encoder,
                frame_texture,
                frame_view,
                packet,
                runtime_resources,
                flow_resources,
                &value.node,
                shader_registry,
            )
            .map(|()| None),
            CompiledPassDescriptor::Copy(value) => self.encode_copy_pass(
                encoder,
                frame_texture,
                packet,
                runtime_resources,
                &value.node,
            )
            .map(|()| None),
            CompiledPassDescriptor::Present(value) => self.encode_present_pass(
                encoder,
                frame_texture,
                packet,
                runtime_resources,
                &value.node,
            )
            .map(|()| None),
            CompiledPassDescriptor::BuiltinUiComposite(_) => {
                self.encode_ui_pass(encoder, frame_view, &packet.prepared_ui);
                Ok(None)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_compute_pass(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_data: &RenderFrameDataRegistry<'_>,
        packet: &RendererPreparedPacket,
        runtime_resources: &FlowRuntimeResources,
        flow_resources: &ResourceGraph,
        node: &RenderPassNode,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<[u32; 3]> {
        if !node.vertex_buffers.is_empty()
            || !node.index_buffers.is_empty()
            || !node.instance_buffers.is_empty()
            || !node.indirect_buffers.is_empty()
        {
            bail!(
                "compute pass '{}' cannot bind graphics vertex/index/instance/indirect buffers",
                node.id.as_str()
            );
        }

        let shader_source = node
            .shader
            .as_ref()
            .map(|reference| resolve_shader_source(reference, shader_registry, DEFAULT_COMPUTE_SHADER))
            .unwrap_or(DEFAULT_COMPUTE_SHADER);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_compiled_compute_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });

        let workgroup = node.workgroup_size.unwrap_or([1, 1, 1]);
        if workgroup[0] == 0 || workgroup[1] == 0 || workgroup[2] == 0 {
            bail!(
                "compute pass '{}' declared an invalid workgroup size",
                node.id.as_str()
            );
        }

        let dispatch = match &node.compute_dispatch {
            Some(crate::plugins::render::api::ComputeDispatchDescriptor::Fixed(value)) => *value,
            Some(crate::plugins::render::api::ComputeDispatchDescriptor::State(binding)) => {
                let state = frame_data
                    .get_by_type_id(binding.state_type_id())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "compute pass '{}' dispatch_state requires missing ECS resource '{}'",
                            node.id.as_str(),
                            binding.state_type_name()
                        )
                    })?;
                binding.project_dispatch(state).ok_or_else(|| {
                    anyhow::anyhow!(
                        "compute pass '{}' failed to project dispatch_state for '{}'",
                        node.id.as_str(),
                        binding.state_type_name()
                    )
                })?
            }
            None => {
                bail!(
                    "compute pass '{}' must declare explicit dispatch_workgroups(...) or dispatch_state(...)",
                    node.id.as_str()
                );
            }
        };
        if dispatch[0] == 0 || dispatch[1] == 0 || dispatch[2] == 0 {
            bail!(
                "compute pass '{}' resolved invalid dispatch dimensions ({}, {}, {})",
                node.id.as_str(),
                dispatch[0],
                dispatch[1],
                dispatch[2]
            );
        }

        let write_texture_ids = dedupe_render_resource_ids(&node.write_textures);
        let sampled_texture_ids = dedupe_render_resource_ids(&node.sampled_textures);
        let uniform_ids = collect_uniform_buffer_ids_for_pass(node, flow_resources)?;
        let storage_ids = collect_storage_buffer_ids_for_pass(node, runtime_resources);

        let mut write_texture_views = Vec::<(TextureView, TextureFormat)>::new();
        for texture_id in &write_texture_ids {
            let texture = runtime_resources.resolve_texture(
                node.id.as_str(),
                texture_id.as_str(),
                frame_texture,
                packet.surface_size,
                packet.surface_format,
            )?;
            if texture.is_depth {
                bail!(
                    "compute pass '{}' declares write_texture '{}' as depth; storage-texture writes require color-like resources",
                    node.id.as_str(),
                    texture.id
                );
            }
            write_texture_views.push((
                texture
                    .texture
                    .create_view(&TextureViewDescriptor::default()),
                texture.format,
            ));
        }

        let mut sampled_texture_views = Vec::<TextureView>::new();
        for texture_id in &sampled_texture_ids {
            let texture = runtime_resources.resolve_texture(
                node.id.as_str(),
                texture_id.as_str(),
                frame_texture,
                packet.surface_size,
                packet.surface_format,
            )?;
            if texture.is_depth {
                bail!(
                    "compute pass '{}' samples depth texture '{}' but builtin runtime execution currently supports only color sampled textures",
                    node.id.as_str(),
                    texture.id
                );
            }
            sampled_texture_views.push(texture.texture.create_view(&TextureViewDescriptor::default()));
        }

        let mut uniform_buffers = Vec::<&Buffer>::new();
        for uniform_id in &uniform_ids {
            let buffer =
                runtime_resources.resolve_uniform_buffer(node.id.as_str(), uniform_id.as_str())?;
            uniform_buffers.push(buffer.buffer);
        }

        let storage_write_ids = node
            .writes
            .iter()
            .map(|resource_id| resource_id.as_str().to_string())
            .collect::<BTreeSet<_>>();

        let mut storage_buffers = Vec::<(&Buffer, bool)>::new();
        for storage_id in &storage_ids {
            let buffer =
                runtime_resources.resolve_storage_buffer(node.id.as_str(), storage_id.as_str())?;
            let read_only = !storage_write_ids.contains(storage_id.as_str());
            storage_buffers.push((buffer.buffer, read_only));
        }

        let sampler = if sampled_texture_views.is_empty() {
            None
        } else {
            Some(device.create_sampler(&SamplerDescriptor::default()))
        };

        let mut bind_group_entries = Vec::<BindGroupEntry<'_>>::new();
        let mut bind_group_layout_entries = Vec::<BindGroupLayoutEntry>::new();
        let mut binding_index = 0u32;

        for (view, format) in &write_texture_views {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: BindingResource::TextureView(view),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: *format,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
        }

        for view in &sampled_texture_views {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: BindingResource::TextureView(view),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
            if let Some(sampled) = sampler.as_ref() {
                bind_group_entries.push(BindGroupEntry {
                    binding: binding_index,
                    resource: BindingResource::Sampler(sampled),
                });
                bind_group_layout_entries.push(BindGroupLayoutEntry {
                    binding: binding_index,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                });
                binding_index = binding_index.saturating_add(1);
            }
        }

        for buffer in &uniform_buffers {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: buffer.as_entire_binding(),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
        }

        for (buffer, read_only) in &storage_buffers {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: buffer.as_entire_binding(),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage {
                        read_only: *read_only,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
        }

        let bind_group_layout = if bind_group_layout_entries.is_empty() {
            None
        } else {
            Some(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_compiled_compute_bind_group_layout"),
                entries: &bind_group_layout_entries,
            }))
        };
        let pipeline_layout = bind_group_layout.as_ref().map(|layout| {
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("engine_compiled_compute_pipeline_layout"),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            })
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("engine_compiled_compute_pipeline"),
            layout: pipeline_layout.as_ref(),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });
        let bind_group = bind_group_layout.as_ref().map(|layout| {
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("engine_compiled_compute_bind_group"),
                layout,
                entries: &bind_group_entries,
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
        Ok(dispatch)
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_fullscreen_pass(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        runtime_resources: &FlowRuntimeResources,
        flow_resources: &ResourceGraph,
        node: &RenderPassNode,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<()> {
        if !node.vertex_buffers.is_empty()
            || !node.index_buffers.is_empty()
            || !node.instance_buffers.is_empty()
            || !node.indirect_buffers.is_empty()
        {
            bail!(
                "fullscreen pass '{}' cannot bind graphics vertex/index/instance/indirect buffers",
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
            .as_ref()
            .map(|reference| {
                resolve_shader_source(reference, shader_registry, DEFAULT_FULLSCREEN_SHADER)
            })
            .unwrap_or(DEFAULT_FULLSCREEN_SHADER);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_compiled_fullscreen_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });

        let sampled_texture_ids = dedupe_render_resource_ids(&node.sampled_textures);
        let uniform_ids = collect_uniform_buffer_ids_for_pass(node, flow_resources)?;
        let storage_ids = collect_storage_buffer_ids_for_pass(node, runtime_resources);
        let write_texture_ids = dedupe_render_resource_ids(&node.write_textures);

        let mut sampled_texture_views = Vec::<TextureView>::new();
        for texture_id in &sampled_texture_ids {
            let texture = runtime_resources.resolve_texture(
                node.id.as_str(),
                texture_id.as_str(),
                frame_texture,
                packet.surface_size,
                packet.surface_format,
            )?;
            sampled_texture_views
                .push(texture.texture.create_view(&TextureViewDescriptor::default()));
        }

        let mut uniform_buffers = Vec::<&Buffer>::new();
        for uniform_id in &uniform_ids {
            let buffer =
                runtime_resources.resolve_uniform_buffer(node.id.as_str(), uniform_id.as_str())?;
            uniform_buffers.push(buffer.buffer);
        }

        let mut storage_buffers = Vec::<&Buffer>::new();
        for storage_id in &storage_ids {
            let buffer =
                runtime_resources.resolve_storage_buffer(node.id.as_str(), storage_id.as_str())?;
            storage_buffers.push(buffer.buffer);
        }

        let mut write_texture_views = Vec::<(TextureView, TextureFormat)>::new();
        for texture_id in &write_texture_ids {
            let texture = runtime_resources.resolve_texture(
                node.id.as_str(),
                texture_id.as_str(),
                frame_texture,
                packet.surface_size,
                packet.surface_format,
            )?;
            if texture.is_depth {
                bail!(
                    "fullscreen pass '{}' declares write_texture '{}' as depth; storage-texture writes require color-like resources",
                    node.id.as_str(),
                    texture.id
                );
            }
            write_texture_views.push((
                texture
                    .texture
                    .create_view(&TextureViewDescriptor::default()),
                texture.format,
            ));
        }

        let sampler = if sampled_texture_views.is_empty() {
            None
        } else {
            Some(device.create_sampler(&SamplerDescriptor::default()))
        };

        let mut bind_group_entries = Vec::<BindGroupEntry<'_>>::new();
        let mut bind_group_layout_entries = Vec::<BindGroupLayoutEntry>::new();
        let mut binding_index = 0u32;

        for view in &sampled_texture_views {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: BindingResource::TextureView(view),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
            if let Some(sampled) = sampler.as_ref() {
                bind_group_entries.push(BindGroupEntry {
                    binding: binding_index,
                    resource: BindingResource::Sampler(sampled),
                });
                bind_group_layout_entries.push(BindGroupLayoutEntry {
                    binding: binding_index,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                });
                binding_index = binding_index.saturating_add(1);
            }
        }

        for buffer in &uniform_buffers {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: buffer.as_entire_binding(),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
        }

        for buffer in &storage_buffers {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: buffer.as_entire_binding(),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
        }

        for (view, format) in &write_texture_views {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: BindingResource::TextureView(view),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: *format,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
        }

        let bind_group_layout = if bind_group_layout_entries.is_empty() {
            None
        } else {
            Some(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_compiled_fullscreen_bind_group_layout"),
                entries: &bind_group_layout_entries,
            }))
        };
        let pipeline_layout = bind_group_layout.as_ref().map(|layout| {
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("engine_compiled_fullscreen_pipeline_layout"),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            })
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_compiled_fullscreen_pipeline"),
            layout: pipeline_layout.as_ref(),
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
        let bind_group = bind_group_layout.as_ref().map(|layout| {
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("engine_compiled_fullscreen_bind_group"),
                layout,
                entries: &bind_group_entries,
            })
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
        if let Some(bind_group) = bind_group.as_ref() {
            pass.set_bind_group(0, bind_group, &[]);
        }
        pass.draw(0..3, 0..1);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_graphics_pass(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        runtime_resources: &FlowRuntimeResources,
        flow_resources: &ResourceGraph,
        node: &RenderPassNode,
        shader_registry: &ShaderRegistryResource,
    ) -> Result<()> {
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
            .as_ref()
            .map(|reference| resolve_shader_source(reference, shader_registry, DEFAULT_GRAPHICS_SHADER))
            .unwrap_or(DEFAULT_GRAPHICS_SHADER);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_compiled_graphics_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });

        let sampled_texture_ids = dedupe_render_resource_ids(&node.sampled_textures);
        let uniform_ids = collect_uniform_buffer_ids_for_pass(node, flow_resources)?;
        let storage_ids = collect_storage_buffer_ids_for_pass(node, runtime_resources);
        let write_texture_ids = dedupe_render_resource_ids(&node.write_textures);

        let mut sampled_texture_views = Vec::<TextureView>::new();
        for texture_id in &sampled_texture_ids {
            let texture = runtime_resources.resolve_texture(
                node.id.as_str(),
                texture_id.as_str(),
                frame_texture,
                packet.surface_size,
                packet.surface_format,
            )?;
            sampled_texture_views
                .push(texture.texture.create_view(&TextureViewDescriptor::default()));
        }

        let mut uniform_buffers = Vec::<&Buffer>::new();
        for uniform_id in &uniform_ids {
            let buffer =
                runtime_resources.resolve_uniform_buffer(node.id.as_str(), uniform_id.as_str())?;
            uniform_buffers.push(buffer.buffer);
        }

        let storage_write_ids = node
            .writes
            .iter()
            .map(|resource_id| resource_id.as_str().to_string())
            .collect::<BTreeSet<_>>();

        let mut storage_buffers = Vec::<(&Buffer, bool)>::new();
        for storage_id in &storage_ids {
            let buffer =
                runtime_resources.resolve_storage_buffer(node.id.as_str(), storage_id.as_str())?;
            let read_only = !storage_write_ids.contains(storage_id.as_str());
            storage_buffers.push((buffer.buffer, read_only));
        }

        let mut write_texture_views = Vec::<(TextureView, TextureFormat)>::new();
        for texture_id in &write_texture_ids {
            let texture = runtime_resources.resolve_texture(
                node.id.as_str(),
                texture_id.as_str(),
                frame_texture,
                packet.surface_size,
                packet.surface_format,
            )?;
            if texture.is_depth {
                bail!(
                    "graphics pass '{}' declares write_texture '{}' as depth; storage-texture writes require color-like resources",
                    node.id.as_str(),
                    texture.id
                );
            }
            write_texture_views.push((
                texture
                    .texture
                    .create_view(&TextureViewDescriptor::default()),
                texture.format,
            ));
        }

        let sampler = if sampled_texture_views.is_empty() {
            None
        } else {
            Some(device.create_sampler(&SamplerDescriptor::default()))
        };

        let mut bind_group_entries = Vec::<BindGroupEntry<'_>>::new();
        let mut bind_group_layout_entries = Vec::<BindGroupLayoutEntry>::new();
        let mut binding_index = 0u32;

        for view in &sampled_texture_views {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: BindingResource::TextureView(view),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
            if let Some(sampled) = sampler.as_ref() {
                bind_group_entries.push(BindGroupEntry {
                    binding: binding_index,
                    resource: BindingResource::Sampler(sampled),
                });
                bind_group_layout_entries.push(BindGroupLayoutEntry {
                    binding: binding_index,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                });
                binding_index = binding_index.saturating_add(1);
            }
        }

        for buffer in &uniform_buffers {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: buffer.as_entire_binding(),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
        }

        for (buffer, read_only) in &storage_buffers {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: buffer.as_entire_binding(),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage {
                        read_only: *read_only,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
        }

        for (view, format) in &write_texture_views {
            bind_group_entries.push(BindGroupEntry {
                binding: binding_index,
                resource: BindingResource::TextureView(view),
            });
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding_index,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: *format,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            });
            binding_index = binding_index.saturating_add(1);
        }

        let bind_group_layout = if bind_group_layout_entries.is_empty() {
            None
        } else {
            Some(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_compiled_graphics_bind_group_layout"),
                entries: &bind_group_layout_entries,
            }))
        };
        let pipeline_layout = bind_group_layout.as_ref().map(|layout| {
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("engine_compiled_graphics_pipeline_layout"),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            })
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_compiled_graphics_pipeline"),
            layout: pipeline_layout.as_ref(),
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
        let bind_group = bind_group_layout.as_ref().map(|layout| {
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("engine_compiled_graphics_bind_group"),
                layout,
                entries: &bind_group_entries,
            })
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
        if let Some(bind_group) = bind_group.as_ref() {
            pass.set_bind_group(0, bind_group, &[]);
        }

        let mut vertex_slot = 0u32;
        for resource_id in &node.vertex_buffers {
            let buffer =
                runtime_resources.resolve_storage_buffer(node.id.as_str(), resource_id.as_str())?;
            pass.set_vertex_buffer(vertex_slot, buffer.buffer.slice(..));
            vertex_slot = vertex_slot.saturating_add(1);
        }
        for resource_id in &node.instance_buffers {
            let buffer =
                runtime_resources.resolve_storage_buffer(node.id.as_str(), resource_id.as_str())?;
            pass.set_vertex_buffer(vertex_slot, buffer.buffer.slice(..));
            vertex_slot = vertex_slot.saturating_add(1);
        }

        let index_buffer = match node.index_buffers.as_slice() {
            [] => None,
            [only] => Some(
                runtime_resources.resolve_storage_buffer(node.id.as_str(), only.as_str())?,
            ),
            _ => {
                bail!(
                    "graphics pass '{}' declares multiple index_buffer(...) resources; runtime currently supports exactly one",
                    node.id.as_str()
                );
            }
        };
        if let Some(ref index) = index_buffer {
            pass.set_index_buffer(index.buffer.slice(..), IndexFormat::Uint32);
        }

        let indirect_buffer = match node.indirect_buffers.as_slice() {
            [] => None,
            [only] => Some(
                runtime_resources.resolve_storage_buffer(node.id.as_str(), only.as_str())?,
            ),
            _ => {
                bail!(
                    "graphics pass '{}' declares multiple indirect_buffer(...) resources; runtime currently supports exactly one",
                    node.id.as_str()
                );
            }
        };

        match (index_buffer.is_some(), indirect_buffer) {
            (true, Some(indirect)) => pass.draw_indexed_indirect(indirect.buffer, 0),
            (false, Some(indirect)) => pass.draw_indirect(indirect.buffer, 0),
            (true, None) => pass.draw_indexed(0..3, 0, 0..1),
            (false, None) => pass.draw(0..3, 0..1),
        }
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
        node: &RenderPassNode,
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
        node: &RenderPassNode,
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
            id: "surface.color".to_string(),
            texture: frame_texture,
            format: packet.surface_format,
            size: packet.surface_size,
            is_depth: false,
        };
        self.encode_texture_copy(encoder, node.id.as_str(), source, destination)
    }
}

fn pass_kind_name(node: &RenderPassNode) -> &'static str {
    match node.kind {
        crate::plugins::render::RenderPassKind::Compute => "compute",
        crate::plugins::render::RenderPassKind::Fullscreen => "fullscreen",
        crate::plugins::render::RenderPassKind::Graphics => "graphics",
        crate::plugins::render::RenderPassKind::Copy => "copy",
        crate::plugins::render::RenderPassKind::Present => "present",
        crate::plugins::render::RenderPassKind::BuiltinUiComposite => "builtin_ui_composite",
    }
}

fn resolve_shader_source<'a>(
    reference: &RenderShaderReference,
    shader_registry: &'a ShaderRegistryResource,
    fallback: &'a str,
) -> &'a str {
    match reference {
        RenderShaderReference::AssetPath(path) => shader_registry.source_or(path, fallback),
        RenderShaderReference::RegistryHandle(handle) => {
            shader_registry.source_or_handle(*handle, fallback)
        }
    }
}

fn dedupe_render_resource_ids(ids: &[RenderResourceId]) -> Vec<String> {
    let mut seen = BTreeSet::<String>::new();
    let mut ordered = Vec::<String>::new();
    for id in ids {
        let key = id.as_str().to_string();
        if seen.insert(key.clone()) {
            ordered.push(key);
        }
    }
    ordered
}

fn collect_uniform_buffer_ids_for_pass(
    node: &RenderPassNode,
    flow_resources: &ResourceGraph,
) -> Result<Vec<String>> {
    let mut seen = BTreeSet::<String>::new();
    let mut ordered = Vec::<String>::new();

    for binding in &node.uniform_bindings {
        if !flow_resources.has_uniform_buffer(binding.uniform_id()) {
            bail!(
                "pass '{}' uniform binding references missing uniform buffer '{}'",
                node.id.as_str(),
                binding.uniform_id().as_str()
            );
        }

        let key = binding.uniform_id().as_str().to_string();
        if seen.insert(key.clone()) {
            ordered.push(key);
        }
    }

    Ok(ordered)
}

fn collect_storage_buffer_ids_for_pass(
    node: &RenderPassNode,
    runtime_resources: &FlowRuntimeResources,
) -> Vec<String> {
    let mut seen = BTreeSet::<String>::new();
    let mut ordered = Vec::<String>::new();

    for id in node.reads.iter().chain(node.writes.iter()) {
        let key = id.as_str();
        let Some(descriptor) = runtime_resources.descriptor_of(key) else {
            continue;
        };
        if !matches!(
            descriptor,
            RenderResourceDescriptor::StorageBuffer(_) | RenderResourceDescriptor::ImportedBuffer(_)
        ) {
            continue;
        }
        let owned = key.to_string();
        if seen.insert(owned.clone()) {
            ordered.push(owned);
        }
    }

    ordered
}
