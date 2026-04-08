use super::*;
use crate::plugins::render::RenderResourceDescriptor;
use crate::plugins::render::api::{SURFACE_COLOR_RESOURCE_ID, SURFACE_DEPTH_RESOURCE_ID};
use crate::plugins::render::backend::ensure_compiled_pass_is_supported;
use crate::plugins::render::frame::{PreparedFlowInputs, PreparedRenderFrame};
use crate::plugins::render::graph::{
    CompiledBindingEntry, CompiledBuiltinImport, CompiledComputeExecutionPlan,
    CompiledCopyExecutionPlan, CompiledPassBindings, CompiledPassExecutionPlan,
    CompiledPresentExecutionPlan, CompiledRasterExecutionPlan, CompiledRenderFlowPlan,
    CompiledResourceRef, CompiledStorageAccess, CompiledTargetPlan, RenderShaderReference,
};
use crate::plugins::render::inspect::{
    PassTimingSample, RuntimeResourceInspectionEntry, RuntimeResourceReuse, resource_kind_name,
};
use crate::plugins::render::pipelines::{
    FlowPassBindGroupKey, FlowPassKind, FlowPassPipelineKey, FlowPrimitiveTopologyClass,
};
use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

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
    generation: Option<u64>,
}

#[derive(Debug)]
struct ResolvedBufferRef<'a> {
    id: String,
    buffer: &'a Buffer,
    size: u64,
    kind: RuntimeBufferKind,
    generation: Option<u64>,
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

#[derive(Debug)]
enum RuntimeBindingResource<'a> {
    TextureView(TextureView),
    SamplerPlaceholder,
    Buffer(&'a Buffer),
}

#[derive(Debug)]
struct RuntimeBindingResolved<'a> {
    layout_ty: BindingType,
    resource: RuntimeBindingResource<'a>,
    generation_token: Option<u64>,
    cacheable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeaturePassAction {
    Execute,
    Skip,
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

    fn buffer_allocation_spec(
        descriptor: &RenderResourceDescriptor,
    ) -> Option<BufferAllocationSpec> {
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

    fn kind_of(&self, id: &str) -> Option<RuntimeResourceKind> {
        self.kinds.get(id).copied().or_else(|| {
            if id == SURFACE_COLOR_RESOURCE_ID {
                Some(RuntimeResourceKind::TextureLike)
            } else {
                None
            }
        })
    }

    fn resolve_resource_id<'a>(
        &self,
        pass_id: &str,
        resource: &'a CompiledResourceRef,
        role: &str,
    ) -> Result<&'a str> {
        match resource {
            CompiledResourceRef::FlowOwned(id) | CompiledResourceRef::Imported(id) => {
                Ok(id.as_str())
            }
            CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceColor) => {
                Ok(SURFACE_COLOR_RESOURCE_ID)
            }
            CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceDepth) => {
                bail!(
                    "pass '{}' references imported builtin resource '{}' but surface-depth imports are not available in runtime execution yet; use flow-owned depth targets",
                    pass_id,
                    SURFACE_DEPTH_RESOURCE_ID
                )
            }
        }
    }

    fn resolve_color_target_from_plan<'a>(
        &'a self,
        pass_id: &str,
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
        let output_id = self.resolve_resource_id(pass_id, output, "color_output")?;

        if output_id == SURFACE_COLOR_RESOURCE_ID {
            return Ok(ResolvedColorTargetView {
                view: RuntimeTextureView::Borrowed(frame_view),
                format: frame_format,
            });
        }

        let kind = self.kind_of(output_id).ok_or_else(|| {
            anyhow::anyhow!(
                "pass '{}' writes unknown color target '{}' during runtime encoding",
                pass_id,
                output_id
            )
        })?;
        if matches!(kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "pass '{}' color target '{}' is buffer-like",
                pass_id,
                output_id
            );
        }

        let Some(texture) = self.textures.get(output_id) else {
            bail!(
                "pass '{}' targets imported texture '{}', but only '{}' is currently supported as imported color target",
                pass_id,
                output_id,
                SURFACE_COLOR_RESOURCE_ID
            );
        };
        if texture.is_depth {
            bail!(
                "pass '{}' color target '{}' is depth-only",
                pass_id,
                output_id
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

    fn resolve_depth_target_from_plan(
        &self,
        pass_id: &str,
        targets: &CompiledTargetPlan,
    ) -> Result<Option<ResolvedDepthTargetView>> {
        let Some(depth_target) = targets.depth_output.as_ref() else {
            return Ok(None);
        };
        let resource_id = self.resolve_resource_id(pass_id, depth_target, "depth_output")?;
        if resource_id == SURFACE_COLOR_RESOURCE_ID {
            bail!(
                "graphics pass '{}' uses '{}' as depth target, which is not supported",
                pass_id,
                SURFACE_COLOR_RESOURCE_ID
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
                "graphics pass '{}' uses '{}' as depth target, but it is buffer-like",
                pass_id,
                resource_id
            );
        }

        let Some(texture) = self.textures.get(resource_id) else {
            bail!(
                "graphics pass '{}' uses imported depth target '{}' but runtime currently supports only flow-owned depth targets",
                pass_id,
                resource_id
            );
        };
        if !texture.is_depth {
            bail!(
                "graphics pass '{}' uses '{}' as depth target, but it is not depth-capable",
                pass_id,
                resource_id
            );
        }

        Ok(Some(ResolvedDepthTargetView {
            view: texture
                .texture
                .create_view(&TextureViewDescriptor::default()),
            format: texture.format,
        }))
    }

    fn resolve_texture_ref<'a>(
        &'a self,
        pass_id: &str,
        resource: &CompiledResourceRef,
        frame_texture: &'a Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<ResolvedTextureRef<'a>> {
        let resource_id = self.resolve_resource_id(pass_id, resource, "texture")?;
        self.resolve_texture(
            pass_id,
            resource_id,
            frame_texture,
            frame_size,
            frame_format,
        )
    }

    fn resolve_storage_buffer_ref<'a>(
        &'a self,
        pass_id: &str,
        resource: &CompiledResourceRef,
    ) -> Result<ResolvedBufferRef<'a>> {
        let resource_id = self.resolve_resource_id(pass_id, resource, "storage_buffer")?;
        self.resolve_storage_buffer(pass_id, resource_id)
    }

    fn resolve_texture<'a>(
        &'a self,
        pass_id: &str,
        resource_id: &str,
        frame_texture: &'a Texture,
        frame_size: (u32, u32),
        frame_format: TextureFormat,
    ) -> Result<ResolvedTextureRef<'a>> {
        if resource_id == SURFACE_COLOR_RESOURCE_ID {
            return Ok(ResolvedTextureRef {
                id: resource_id.to_string(),
                texture: frame_texture,
                format: frame_format,
                size: frame_size,
                is_depth: false,
                generation: None,
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
            generation: Some(texture.generation),
        })
    }

    fn resolve_buffer<'a>(
        &'a self,
        pass_id: &str,
        resource_id: &str,
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
            id: resource_id.to_string(),
            buffer: &buffer.buffer,
            size: buffer.size,
            kind: buffer.kind,
            generation: Some(buffer.generation),
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
        prepared_frame: &PreparedRenderFrame,
        packet: RendererPreparedPacket,
        compiled_flows: &[CompiledRenderFlowPlan],
        shader_registry: &ShaderRegistryResource,
    ) -> Result<RendererFrameTimings> {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_render_encoder"),
        });
        self.last_pass_timings.clear();
        self.last_runtime_resources.clear();

        if packet.view_count > 1 {
            bail!(
                "prepared frame contains {} views but active runtime execution is single-view only; multi-view execution is explicitly deferred",
                packet.view_count
            );
        }

        let mut flow_runtime_cache = std::mem::take(&mut self.flow_runtime_cache);
        let render_result = (|| -> Result<()> {
            let active_flow_ids = compiled_flows
                .iter()
                .map(|flow| flow.flow_id.clone())
                .collect::<Vec<_>>();
            flow_runtime_cache.retain(|flow_id, _| active_flow_ids.iter().any(|id| id == flow_id));
            self.flow_pipeline_cache.retain_flows(&active_flow_ids);

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

                let flow_inputs = prepared_frame
                    .flow_inputs(flow.flow_id.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "missing prepared flow inputs for flow '{}' during frame execution",
                            flow.flow_id
                        )
                    })?;
                self.upload_projected_uniform_buffers(queue, flow_inputs, runtime_resources)?;

                for pass in &flow.execution.passes {
                    if !self.pass_targets_active_view(pass, packet.view_id.as_str()) {
                        continue;
                    }
                    ensure_compiled_pass_is_supported(pass)?;
                    let pass_encode_start = Instant::now();
                    let dispatch_workgroups = self.encode_compiled_pass(
                        device,
                        &mut encoder,
                        frame_texture,
                        frame_view,
                        &packet,
                        flow,
                        flow_inputs,
                        pass,
                        shader_registry,
                        runtime_resources,
                    )?;
                    self.last_pass_timings.push(PassTimingSample {
                        flow_id: flow.flow_id.clone(),
                        pass_id: execution_pass_id(pass).to_string(),
                        pass_kind: execution_pass_kind_name(pass).to_string(),
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
        prepared_frame: &PreparedRenderFrame,
        shader_registry: &mut ShaderRegistryResource,
        compiled_flows: &[CompiledRenderFlowPlan],
        ui_rect_shader: Option<ShaderHandle>,
        surface_format: TextureFormat,
    ) -> Result<RendererFrameTimings> {
        let packet = self.prepare_packet(
            device,
            queue,
            prepared_frame,
            shader_registry,
            ui_rect_shader,
            surface_format,
        );
        self.render_packet(
            device,
            queue,
            frame_texture,
            frame_view,
            prepared_frame,
            packet,
            compiled_flows,
            shader_registry,
        )
    }

    fn upload_projected_uniform_buffers(
        &self,
        queue: &Queue,
        flow_inputs: &PreparedFlowInputs,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<()> {
        for (buffer_id, bytes) in &flow_inputs.projected_uniform_bytes {
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
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        flow_inputs: &PreparedFlowInputs,
        pass: &CompiledPassExecutionPlan,
        shader_registry: &ShaderRegistryResource,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<Option<[u32; 3]>> {
        if let Some(feature_id) = execution_pass_feature_id(pass) {
            match self.resolve_feature_pass_action(feature_id, execution_pass_id(pass), packet)? {
                FeaturePassAction::Execute => {}
                FeaturePassAction::Skip => return Ok(None),
            }
        }

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
                .map(Some),
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
                .map(|()| None),
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
                .map(|()| None),
            CompiledPassExecutionPlan::Copy(value) => self
                .encode_copy_pass(encoder, frame_texture, packet, runtime_resources, value)
                .map(|()| None),
            CompiledPassExecutionPlan::Present(value) => self
                .encode_present_pass(encoder, frame_texture, packet, runtime_resources, value)
                .map(|()| None),
            CompiledPassExecutionPlan::BuiltinUiComposite(_value) => {
                self.encode_ui_pass(encoder, frame_view, &packet.prepared_ui);
                Ok(None)
            }
        }
    }

    fn pass_targets_active_view(&self, pass: &CompiledPassExecutionPlan, view_id: &str) -> bool {
        let view_mask = match pass {
            CompiledPassExecutionPlan::Compute(value) => &value.view_mask,
            CompiledPassExecutionPlan::Fullscreen(value) => &value.view_mask,
            CompiledPassExecutionPlan::Graphics(value) => &value.view_mask,
            CompiledPassExecutionPlan::Copy(value) => &value.view_mask,
            CompiledPassExecutionPlan::Present(value) => &value.view_mask,
            CompiledPassExecutionPlan::BuiltinUiComposite(value) => &value.view_mask,
        };
        view_mask.includes(view_id)
    }

    fn resolve_feature_pass_action(
        &self,
        feature_id: &str,
        pass_id: &str,
        packet: &RendererPreparedPacket,
    ) -> Result<FeaturePassAction> {
        let gate = packet
            .feature_gates
            .get(feature_id)
            .copied()
            .unwrap_or_default();

        match gate.status {
            crate::plugins::render::FeatureContributionStatus::Ready => {
                Ok(FeaturePassAction::Execute)
            }
            crate::plugins::render::FeatureContributionStatus::Stale => {
                match gate.fallback_policy {
                    crate::plugins::render::FeatureFallbackPolicy::FailFrame => {
                        bail!(
                            "feature '{}' is stale for pass '{}' and fallback policy is fail-frame",
                            feature_id,
                            pass_id
                        )
                    }
                    crate::plugins::render::FeatureFallbackPolicy::SkipFeaturePasses => {
                        Ok(FeaturePassAction::Skip)
                    }
                    crate::plugins::render::FeatureFallbackPolicy::ReuseLastGood
                    | crate::plugins::render::FeatureFallbackPolicy::EmptyContribution => {
                        Ok(FeaturePassAction::Execute)
                    }
                }
            }
            crate::plugins::render::FeatureContributionStatus::Disabled
            | crate::plugins::render::FeatureContributionStatus::Missing => {
                match gate.fallback_policy {
                    crate::plugins::render::FeatureFallbackPolicy::FailFrame => {
                        bail!(
                            "feature '{}' is {:?} for pass '{}' and fallback policy is fail-frame",
                            feature_id,
                            gate.status,
                            pass_id
                        )
                    }
                    crate::plugins::render::FeatureFallbackPolicy::SkipFeaturePasses => {
                        Ok(FeaturePassAction::Skip)
                    }
                    crate::plugins::render::FeatureFallbackPolicy::ReuseLastGood
                    | crate::plugins::render::FeatureFallbackPolicy::EmptyContribution => {
                        Ok(FeaturePassAction::Execute)
                    }
                }
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
    ) -> Result<[u32; 3]> {
        let shader = resolve_shader_material(
            pass.shader.as_ref(),
            shader_registry,
            DEFAULT_COMPUTE_SHADER,
            "builtin:compute",
        );
        let dispatch = flow_inputs
            .projected_dispatch_workgroups
            .get(pass.pass_id.as_str())
            .copied()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "missing prepared dispatch for pass '{}' in flow '{}'",
                    pass.pass_id.as_str(),
                    flow.flow_id
                )
            })?;
        if dispatch[0] == 0 || dispatch[1] == 0 || dispatch[2] == 0 {
            bail!(
                "compute pass '{}' resolved invalid dispatch dimensions ({}, {}, {})",
                pass.pass_id.as_str(),
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
            pass.pass_id.as_str(),
            FlowPassKind::Compute,
            pass.feature_id.as_deref(),
            shader.identity.as_str(),
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
        Ok(dispatch)
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
    ) -> Result<()> {
        if !plan.draw_buffers.vertex_buffers.is_empty()
            || !plan.draw_buffers.index_buffers.is_empty()
            || !plan.draw_buffers.instance_buffers.is_empty()
            || !plan.draw_buffers.indirect_buffers.is_empty()
        {
            bail!(
                "fullscreen pass '{}' cannot bind graphics vertex/index/instance/indirect buffers",
                plan.pass_id.as_str()
            );
        }

        let color_target = runtime_resources.resolve_color_target_from_plan(
            plan.pass_id.as_str(),
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
            plan.pass_id.as_str(),
            FlowPassKind::Fullscreen,
            plan.feature_id.as_deref(),
            shader.identity.as_str(),
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
        Ok(())
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
    ) -> Result<()> {
        let color_target = runtime_resources.resolve_color_target_from_plan(
            plan.pass_id.as_str(),
            &plan.targets,
            frame_view,
            packet.surface_format,
        )?;
        let depth_target = runtime_resources
            .resolve_depth_target_from_plan(plan.pass_id.as_str(), &plan.targets)?;

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
            plan.pass_id.as_str(),
            FlowPassKind::Graphics,
            plan.feature_id.as_deref(),
            shader.identity.as_str(),
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
            let buffer =
                runtime_resources.resolve_storage_buffer_ref(plan.pass_id.as_str(), resource)?;
            pass.set_vertex_buffer(vertex_slot, buffer.buffer.slice(..));
            vertex_slot = vertex_slot.saturating_add(1);
        }
        for resource in &plan.draw_buffers.instance_buffers {
            let buffer =
                runtime_resources.resolve_storage_buffer_ref(plan.pass_id.as_str(), resource)?;
            pass.set_vertex_buffer(vertex_slot, buffer.buffer.slice(..));
            vertex_slot = vertex_slot.saturating_add(1);
        }

        let index_buffer = match plan.draw_buffers.index_buffers.as_slice() {
            [] => None,
            [only] => {
                Some(runtime_resources.resolve_storage_buffer_ref(plan.pass_id.as_str(), only)?)
            }
            _ => {
                bail!(
                    "graphics pass '{}' declares multiple index_buffer(...) resources; runtime currently supports exactly one",
                    plan.pass_id.as_str()
                );
            }
        };
        if let Some(ref index) = index_buffer {
            pass.set_index_buffer(index.buffer.slice(..), IndexFormat::Uint32);
        }

        let indirect_buffer = match plan.draw_buffers.indirect_buffers.as_slice() {
            [] => None,
            [only] => {
                Some(runtime_resources.resolve_storage_buffer_ref(plan.pass_id.as_str(), only)?)
            }
            _ => {
                bail!(
                    "graphics pass '{}' declares multiple indirect_buffer(...) resources; runtime currently supports exactly one",
                    plan.pass_id.as_str()
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

    #[allow(clippy::too_many_arguments)]
    fn resolve_compiled_bind_group<'a>(
        &mut self,
        device: &Device,
        frame_texture: &'a Texture,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        pass_id: &str,
        pass_kind: FlowPassKind,
        pass_feature_id: Option<&str>,
        shader_identity: &str,
        shader_revision: u64,
        bindings: &CompiledPassBindings,
        visibility: ShaderStages,
        allow_depth_sampling: bool,
        color_formats: Vec<TextureFormat>,
        depth_format: Option<TextureFormat>,
        primitive_topology_class: FlowPrimitiveTopologyClass,
        runtime_resources: &'a FlowRuntimeResources,
    ) -> Result<(
        FlowPassPipelineKey,
        Option<BindGroupLayout>,
        Option<BindGroup>,
    )> {
        let mut resolved_entries = Vec::<RuntimeBindingResolved<'a>>::new();
        for entry in &bindings.bind_group.entries {
            match entry {
                CompiledBindingEntry::SampledTexture { resource } => {
                    let texture = runtime_resources.resolve_texture_ref(
                        pass_id,
                        resource,
                        frame_texture,
                        packet.surface_size,
                        packet.surface_format,
                    )?;
                    if !allow_depth_sampling && texture.is_depth {
                        bail!(
                            "pass '{}' samples depth texture '{}' but this pass type only supports color sampled textures",
                            pass_id,
                            texture.id
                        );
                    }
                    let sample_type = if texture.is_depth {
                        TextureSampleType::Depth
                    } else {
                        TextureSampleType::Float { filterable: true }
                    };
                    resolved_entries.push(RuntimeBindingResolved {
                        layout_ty: BindingType::Texture {
                            sample_type,
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        resource: RuntimeBindingResource::TextureView(
                            texture
                                .texture
                                .create_view(&TextureViewDescriptor::default()),
                        ),
                        generation_token: texture.generation,
                        cacheable: texture.generation.is_some(),
                    });
                }
                CompiledBindingEntry::Sampler => {
                    resolved_entries.push(RuntimeBindingResolved {
                        layout_ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        resource: RuntimeBindingResource::SamplerPlaceholder,
                        generation_token: Some(0),
                        cacheable: true,
                    });
                }
                CompiledBindingEntry::StorageTexture { resource, access } => {
                    let texture = runtime_resources.resolve_texture_ref(
                        pass_id,
                        resource,
                        frame_texture,
                        packet.surface_size,
                        packet.surface_format,
                    )?;
                    if texture.is_depth {
                        bail!(
                            "pass '{}' declares storage texture '{}' as depth; storage-texture bindings require color-like resources",
                            pass_id,
                            texture.id
                        );
                    }
                    resolved_entries.push(RuntimeBindingResolved {
                        layout_ty: BindingType::StorageTexture {
                            access: compiled_storage_access_to_storage_texture_access(*access),
                            format: texture.format,
                            view_dimension: TextureViewDimension::D2,
                        },
                        resource: RuntimeBindingResource::TextureView(
                            texture
                                .texture
                                .create_view(&TextureViewDescriptor::default()),
                        ),
                        generation_token: texture.generation,
                        cacheable: texture.generation.is_some(),
                    });
                }
                CompiledBindingEntry::UniformBuffer { resource } => {
                    let buffer =
                        runtime_resources.resolve_uniform_buffer(pass_id, resource.as_str())?;
                    resolved_entries.push(RuntimeBindingResolved {
                        layout_ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        resource: RuntimeBindingResource::Buffer(buffer.buffer),
                        generation_token: buffer.generation,
                        cacheable: buffer.generation.is_some(),
                    });
                }
                CompiledBindingEntry::StorageBuffer { resource, access } => {
                    let buffer = runtime_resources.resolve_storage_buffer_ref(pass_id, resource)?;
                    resolved_entries.push(RuntimeBindingResolved {
                        layout_ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage {
                                read_only: matches!(access, CompiledStorageAccess::ReadOnly),
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        resource: RuntimeBindingResource::Buffer(buffer.buffer),
                        generation_token: buffer.generation,
                        cacheable: buffer.generation.is_some(),
                    });
                }
            }
        }

        let mut bind_group_layout_entries = Vec::<BindGroupLayoutEntry>::new();
        for (binding, value) in resolved_entries.iter().enumerate() {
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: binding as u32,
                visibility,
                ty: value.layout_ty,
                count: None,
            });
        }

        let pipeline_key = FlowPassPipelineKey {
            flow_id: flow.flow_id.clone(),
            pass_id: pass_id.to_string(),
            pass_kind,
            feature_id: pass_feature_id.map(|value| value.to_string()),
            shader_identity: shader_identity.to_string(),
            shader_revision,
            bind_group_layout_signature_hash: hash_bind_group_layout_entries(
                &bind_group_layout_entries,
            ),
            material_specialization_fragment_hash: material_specialization_fragment_hash(
                packet,
                pass_feature_id,
            ),
            view_signature_hash: hash_view_signature(packet.view_id.as_str(), packet.surface_size),
            feature_runtime_version: feature_runtime_version(packet, pass_feature_id),
            color_formats,
            depth_format,
            sample_count: 1,
            primitive_topology_class,
        };

        if bind_group_layout_entries.is_empty() {
            return Ok((pipeline_key, None, None));
        }

        let shared_sampler = if resolved_entries
            .iter()
            .any(|entry| matches!(entry.resource, RuntimeBindingResource::SamplerPlaceholder))
        {
            Some(
                self.flow_pipeline_cache
                    .get_or_create_sampler(pipeline_key.clone(), || {
                        device.create_sampler(&SamplerDescriptor::default())
                    }),
            )
        } else {
            None
        };

        let bind_group_layout =
            self.flow_pipeline_cache
                .get_or_create_bind_group_layout(pipeline_key.clone(), || {
                    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("engine_compiled_flow_bind_group_layout"),
                        entries: &bind_group_layout_entries,
                    })
                });

        let mut bind_group_entries = Vec::<BindGroupEntry<'_>>::new();
        let mut can_cache_bind_group = true;
        let mut signature_hasher = std::collections::hash_map::DefaultHasher::new();
        for (binding, value) in resolved_entries.iter().enumerate() {
            (binding as u32).hash(&mut signature_hasher);
            if value.cacheable {
                value.generation_token.hash(&mut signature_hasher);
            } else {
                can_cache_bind_group = false;
            }
            let resource = match &value.resource {
                RuntimeBindingResource::TextureView(view) => BindingResource::TextureView(view),
                RuntimeBindingResource::SamplerPlaceholder => {
                    let sampler = shared_sampler.as_ref().ok_or_else(|| {
                        anyhow::anyhow!(
                            "pass '{}' resolved sampler placeholder but no sampler instance was available",
                            pass_id
                        )
                    })?;
                    BindingResource::Sampler(sampler)
                }
                RuntimeBindingResource::Buffer(buffer) => buffer.as_entire_binding(),
            };
            bind_group_entries.push(BindGroupEntry {
                binding: binding as u32,
                resource,
            });
        }

        let bind_group = if can_cache_bind_group {
            let bind_group_key = FlowPassBindGroupKey {
                pipeline: pipeline_key.clone(),
                resource_generation_signature_hash: signature_hasher.finish(),
            };
            self.flow_pipeline_cache
                .get_or_create_bind_group(bind_group_key, || {
                    device.create_bind_group(&BindGroupDescriptor {
                        label: Some("engine_compiled_flow_bind_group"),
                        layout: &bind_group_layout,
                        entries: &bind_group_entries,
                    })
                })
        } else {
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("engine_compiled_flow_bind_group_noncached"),
                layout: &bind_group_layout,
                entries: &bind_group_entries,
            })
        };

        Ok((pipeline_key, Some(bind_group_layout), Some(bind_group)))
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
            runtime_resources.resolve_resource_id(pass.pass_id.as_str(), source, "copy_source")?;
        let destination_id = runtime_resources.resolve_resource_id(
            pass.pass_id.as_str(),
            destination,
            "copy_destination",
        )?;
        if source_id == destination_id {
            return Ok(());
        }

        let source_kind = runtime_resources.kind_of(source_id).ok_or_else(|| {
            anyhow::anyhow!(
                "copy pass '{}' references unknown source resource '{}'",
                pass.pass_id.as_str(),
                source_id
            )
        })?;
        let destination_kind = runtime_resources.kind_of(destination_id).ok_or_else(|| {
            anyhow::anyhow!(
                "copy pass '{}' references unknown destination resource '{}'",
                pass.pass_id.as_str(),
                destination_id
            )
        })?;

        match (source_kind, destination_kind) {
            (RuntimeResourceKind::BufferLike, RuntimeResourceKind::BufferLike) => {
                let source = runtime_resources.resolve_buffer(pass.pass_id.as_str(), source_id)?;
                let destination =
                    runtime_resources.resolve_buffer(pass.pass_id.as_str(), destination_id)?;
                self.encode_buffer_copy(encoder, pass.pass_id.as_str(), source, destination)
            }
            (RuntimeResourceKind::BufferLike, RuntimeResourceKind::TextureLike)
            | (RuntimeResourceKind::TextureLike, RuntimeResourceKind::BufferLike) => {
                bail!(
                    "copy pass '{}' mixes incompatible resource classes '{}' -> '{}'",
                    pass.pass_id.as_str(),
                    source_id,
                    destination_id
                );
            }
            (RuntimeResourceKind::TextureLike, RuntimeResourceKind::TextureLike) => {
                let source = runtime_resources.resolve_texture(
                    pass.pass_id.as_str(),
                    source_id,
                    frame_texture,
                    packet.surface_size,
                    packet.surface_format,
                )?;
                let destination = runtime_resources.resolve_texture(
                    pass.pass_id.as_str(),
                    destination_id,
                    frame_texture,
                    packet.surface_size,
                    packet.surface_format,
                )?;
                self.encode_texture_copy(encoder, pass.pass_id.as_str(), source, destination)
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
        let source_id = runtime_resources.resolve_resource_id(
            pass.pass_id.as_str(),
            source,
            "present_source",
        )?;
        if source_id == SURFACE_COLOR_RESOURCE_ID {
            return Ok(());
        }

        let source_kind = runtime_resources.kind_of(source_id).ok_or_else(|| {
            anyhow::anyhow!(
                "present pass '{}' references unknown source resource '{}'",
                pass.pass_id.as_str(),
                source_id
            )
        })?;
        if matches!(source_kind, RuntimeResourceKind::BufferLike) {
            bail!(
                "present pass '{}' reads buffer-like resource '{}' but present requires a texture-like source",
                pass.pass_id.as_str(),
                source_id
            );
        }

        let source = runtime_resources.resolve_texture(
            pass.pass_id.as_str(),
            source_id,
            frame_texture,
            packet.surface_size,
            packet.surface_format,
        )?;
        let destination = ResolvedTextureRef {
            id: SURFACE_COLOR_RESOURCE_ID.to_string(),
            texture: frame_texture,
            format: packet.surface_format,
            size: packet.surface_size,
            is_depth: false,
            generation: None,
        };
        self.encode_texture_copy(encoder, pass.pass_id.as_str(), source, destination)
    }
}

fn execution_pass_kind_name(pass: &CompiledPassExecutionPlan) -> &'static str {
    match pass {
        CompiledPassExecutionPlan::Compute(_) => "compute",
        CompiledPassExecutionPlan::Fullscreen(_) => "fullscreen",
        CompiledPassExecutionPlan::Graphics(_) => "graphics",
        CompiledPassExecutionPlan::Copy(_) => "copy",
        CompiledPassExecutionPlan::Present(_) => "present",
        CompiledPassExecutionPlan::BuiltinUiComposite(_) => "builtin_ui_composite",
    }
}

fn execution_pass_id(pass: &CompiledPassExecutionPlan) -> &str {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.pass_id.as_str(),
        CompiledPassExecutionPlan::Fullscreen(value) => value.pass_id.as_str(),
        CompiledPassExecutionPlan::Graphics(value) => value.pass_id.as_str(),
        CompiledPassExecutionPlan::Copy(value) => value.pass_id.as_str(),
        CompiledPassExecutionPlan::Present(value) => value.pass_id.as_str(),
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => value.pass_id.as_str(),
    }
}

fn execution_pass_feature_id(pass: &CompiledPassExecutionPlan) -> Option<&str> {
    match pass {
        CompiledPassExecutionPlan::Compute(value) => value.feature_id.as_deref(),
        CompiledPassExecutionPlan::Fullscreen(value) => value.feature_id.as_deref(),
        CompiledPassExecutionPlan::Graphics(value) => value.feature_id.as_deref(),
        CompiledPassExecutionPlan::Copy(value) => value.feature_id.as_deref(),
        CompiledPassExecutionPlan::Present(value) => value.feature_id.as_deref(),
        CompiledPassExecutionPlan::BuiltinUiComposite(value) => Some(value.feature_id.as_str()),
    }
}

#[derive(Debug, Clone)]
struct ResolvedShaderMaterial<'a> {
    source: &'a str,
    identity: String,
    revision: u64,
}

fn resolve_shader_material<'a>(
    reference: Option<&RenderShaderReference>,
    shader_registry: &'a ShaderRegistryResource,
    fallback_source: &'a str,
    fallback_identity: &'static str,
) -> ResolvedShaderMaterial<'a> {
    match reference {
        Some(RenderShaderReference::AssetPath(path)) => ResolvedShaderMaterial {
            source: shader_registry.source_or(path, fallback_source),
            identity: format!("asset:{path}"),
            revision: shader_registry.revision_for(path),
        },
        Some(RenderShaderReference::RegistryHandle(handle)) => ResolvedShaderMaterial {
            source: shader_registry.source_or_handle(*handle, fallback_source),
            identity: format!("handle:{}", handle.index()),
            revision: shader_registry.revision_for_handle(*handle),
        },
        None => ResolvedShaderMaterial {
            source: fallback_source,
            identity: fallback_identity.to_string(),
            revision: 0,
        },
    }
}

fn hash_bind_group_layout_entries(entries: &[BindGroupLayoutEntry]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for entry in entries {
        entry.binding.hash(&mut hasher);
        entry.visibility.bits().hash(&mut hasher);
        format!("{:?}", entry.ty).hash(&mut hasher);
    }
    hasher.finish()
}

fn hash_view_signature(view_id: &str, surface_size: (u32, u32)) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    view_id.hash(&mut hasher);
    surface_size.hash(&mut hasher);
    hasher.finish()
}

fn material_specialization_fragment_hash(
    packet: &RendererPreparedPacket,
    pass_feature_id: Option<&str>,
) -> u64 {
    if !matches!(
        pass_feature_id,
        Some(crate::plugins::render::MATERIAL_RENDER_FEATURE_ID)
    ) {
        return 0;
    }

    pass_feature_id
        .and_then(|feature_id| packet.feature_runtime_signatures.get(feature_id).copied())
        .unwrap_or_default()
}

fn feature_runtime_version(packet: &RendererPreparedPacket, pass_feature_id: Option<&str>) -> u64 {
    let Some(feature_id) = pass_feature_id else {
        return 0;
    };

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    feature_id.hash(&mut hasher);
    if let Some(gate) = packet.feature_gates.get(feature_id) {
        gate.status.hash(&mut hasher);
        gate.fallback_policy.hash(&mut hasher);
    }
    packet
        .feature_runtime_signatures
        .get(feature_id)
        .copied()
        .unwrap_or_default()
        .hash(&mut hasher);
    hasher.finish()
}

fn compiled_storage_access_to_storage_texture_access(
    access: CompiledStorageAccess,
) -> StorageTextureAccess {
    match access {
        CompiledStorageAccess::ReadOnly => StorageTextureAccess::ReadOnly,
        CompiledStorageAccess::ReadWrite => StorageTextureAccess::ReadWrite,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{CompiledPresentExecutionPlan, CompiledViewMask};

    fn packet_with_feature_gate(
        feature_id: &str,
        gate: FeatureExecutionGate,
    ) -> RendererPreparedPacket {
        let mut feature_gates = BTreeMap::new();
        feature_gates.insert(feature_id.to_string(), gate);
        let mut feature_runtime_signatures = BTreeMap::new();
        feature_runtime_signatures.insert(feature_id.to_string(), 1);
        RendererPreparedPacket {
            surface_format: TextureFormat::Rgba8Unorm,
            surface_size: (1, 1),
            view_id: "main".to_string(),
            view_count: 1,
            feature_gates,
            feature_runtime_signatures,
            prepared_ui: UiPreparedDraws::default(),
            prepare_timings: RendererFrameTimings::default(),
        }
    }

    #[test]
    fn ui_feature_gate_skips_when_missing_and_policy_is_skip() {
        let renderer = Renderer::new();
        let packet = packet_with_feature_gate(
            "ui",
            FeatureExecutionGate {
                status: FeatureContributionStatus::Missing,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            },
        );
        let action = renderer
            .resolve_feature_pass_action("ui", "ui", &packet)
            .expect("skip policy should not error");
        assert_eq!(action, FeaturePassAction::Skip);
    }

    #[test]
    fn ui_feature_gate_fails_when_missing_and_policy_is_fail_frame() {
        let renderer = Renderer::new();
        let packet = packet_with_feature_gate(
            "ui",
            FeatureExecutionGate {
                status: FeatureContributionStatus::Missing,
                fallback_policy: FeatureFallbackPolicy::FailFrame,
            },
        );
        assert!(
            renderer
                .resolve_feature_pass_action("ui", "ui", &packet)
                .is_err(),
            "missing + fail-frame should produce an execution error"
        );
    }

    #[test]
    fn generic_feature_gate_applies_to_non_ui_passes() {
        let renderer = Renderer::new();
        let packet = packet_with_feature_gate(
            "world.draw",
            FeatureExecutionGate {
                status: FeatureContributionStatus::Missing,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            },
        );

        let action = renderer
            .resolve_feature_pass_action("world.draw", "compose", &packet)
            .expect("skip policy should not error");
        assert_eq!(action, FeaturePassAction::Skip);
    }

    #[test]
    fn feature_runtime_version_changes_when_runtime_signature_changes() {
        let mut packet = packet_with_feature_gate(
            "world.draw",
            FeatureExecutionGate {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            },
        );
        let base = feature_runtime_version(&packet, Some("world.draw"));
        packet
            .feature_runtime_signatures
            .insert("world.draw".to_string(), 99);
        let changed = feature_runtime_version(&packet, Some("world.draw"));
        assert_ne!(base, changed);
    }

    #[test]
    fn material_specialization_hash_uses_material_feature_signature() {
        let mut packet = packet_with_feature_gate(
            crate::plugins::render::MATERIAL_RENDER_FEATURE_ID,
            FeatureExecutionGate {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            },
        );
        packet.feature_runtime_signatures.insert(
            crate::plugins::render::MATERIAL_RENDER_FEATURE_ID.to_string(),
            1234,
        );
        assert_eq!(
            material_specialization_fragment_hash(
                &packet,
                Some(crate::plugins::render::MATERIAL_RENDER_FEATURE_ID),
            ),
            1234
        );
        assert_eq!(
            material_specialization_fragment_hash(&packet, Some("world.draw")),
            0
        );
    }

    #[test]
    fn pass_view_mask_filters_non_matching_views() {
        let renderer = Renderer::new();
        let mut explicit = BTreeSet::new();
        explicit.insert("main".to_string());
        let pass = CompiledPassExecutionPlan::Present(CompiledPresentExecutionPlan {
            pass_id: "present".to_string(),
            order_index: 0,
            feature_id: None,
            view_mask: CompiledViewMask::Explicit(explicit),
            source: None,
        });

        assert!(renderer.pass_targets_active_view(&pass, "main"));
        assert!(!renderer.pass_targets_active_view(&pass, "minimap"));
    }
}
