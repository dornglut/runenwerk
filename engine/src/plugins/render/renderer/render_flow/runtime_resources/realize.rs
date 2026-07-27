use super::*;
use crate::plugins::gpu::{
    GpuResourceDescriptor, GpuTextureDimension, GpuTextureFormat, GpuWorkResourceId,
};
use crate::plugins::render::renderer::dynamic_targets::{
    dynamic_format_to_wgpu, dynamic_usage_to_wgpu,
};
use crate::plugins::render::{RenderTextureFormatPolicy, legacy_surface_validation_format};

impl FlowRuntimeResources {
    pub fn realize_for_frame(
        &mut self,
        device: &Device,
        flow: &CompiledRenderFlowPlan,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
    ) -> Result<()> {
        let frame_size = (surface_size.0.max(1), surface_size.1.max(1));
        let mut declared_ids = BTreeSet::<GpuWorkResourceId>::new();

        self.kinds.clear();
        self.descriptors.clear();
        self.resource_ids_by_label.clear();
        for (label, id) in &flow.resource_ids_by_label {
            self.resource_ids_by_label.insert(label.clone(), *id);
        }

        for descriptor in &flow.resources.resources {
            let id = *descriptor.id();
            declared_ids.insert(id);
            self.descriptors.insert(id, descriptor.clone());

            let kind = match descriptor {
                RenderResourceDeclaration::Uniform(_)
                | RenderResourceDeclaration::Storage(_)
                | RenderResourceDeclaration::ImportedBuffer(_) => RuntimeResourceKind::BufferLike,
                _ => RuntimeResourceKind::TextureLike,
            };
            self.kinds.insert(id, kind);

            if !matches!(descriptor, RenderResourceDeclaration::History(_))
                && let Some(texture_spec) =
                    Self::texture_allocation_spec(descriptor, frame_size, surface_format)?
            {
                let previous_generation = self
                    .textures
                    .get(&id)
                    .map(|existing| existing.generation)
                    .unwrap_or(0);
                let should_recreate = match self.textures.get(&id) {
                    Some(existing) => {
                        descriptor.lifetime().is_transient()
                            || existing.format != texture_spec.format
                            || existing.size != texture_spec.size
                            || existing.usage != texture_spec.usage
                            || existing.is_depth != texture_spec.is_depth
                    }
                    None => true,
                };

                if should_recreate {
                    let label = format!("engine_render_resource_{id}");
                    let texture = device.create_texture(&TextureDescriptor {
                        label: Some(label.as_str()),
                        size: Extent3d {
                            width: texture_spec.size.0,
                            height: texture_spec.size.1,
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
                        id,
                        RuntimeTextureResource {
                            texture,
                            format: texture_spec.format,
                            size: texture_spec.size,
                            usage: texture_spec.usage,
                            is_depth: texture_spec.is_depth,
                            history_signature: None,
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

            if let Some(buffer_spec) = Self::buffer_allocation_spec(descriptor)? {
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
                    let label = format!("engine_render_resource_{id}");
                    let buffer = device.create_buffer(&BufferDescriptor {
                        label: Some(label.as_str()),
                        size: buffer_spec.size.max(1),
                        usage: buffer_spec.usage,
                        mapped_at_creation: false,
                    });
                    self.buffers.insert(
                        id,
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
        self.invocation_uniform_buffers
            .retain(|(_, id), _| declared_ids.contains(id));
        self.invocation_history_textures
            .retain(|(_, id), _| declared_ids.contains(id));
        self.active_invocation_uniform_scope = None;
        Ok(())
    }

    pub fn set_active_invocation_uniform_scope(&mut self, invocation_id: impl Into<String>) {
        self.active_invocation_uniform_scope = Some(invocation_id.into());
    }

    pub fn clear_active_invocation_uniform_scope(&mut self) {
        self.active_invocation_uniform_scope = None;
    }

    pub fn retain_invocation_uniform_scopes<'a>(
        &mut self,
        invocation_ids: impl IntoIterator<Item = &'a str>,
    ) {
        let active = invocation_ids
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<BTreeSet<_>>();
        self.invocation_uniform_buffers
            .retain(|(invocation_id, _), _| active.contains(invocation_id));
        self.invocation_history_textures
            .retain(|(invocation_id, _), _| active.contains(invocation_id));
    }

    pub fn realize_invocation_history_textures(
        &mut self,
        device: &Device,
        invocation_id: &str,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
        history_signature: Option<&str>,
    ) -> Result<()> {
        let history_descriptors = self
            .descriptors
            .iter()
            .filter_map(|(id, descriptor)| match descriptor {
                RenderResourceDeclaration::History(_) => Some((*id, descriptor.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        for (resource_id, descriptor) in history_descriptors {
            let Some(texture_spec) =
                Self::texture_allocation_spec(&descriptor, surface_size, surface_format)?
            else {
                continue;
            };
            let key = (invocation_id.to_string(), resource_id);
            let next_history_signature = history_signature.map(ToOwned::to_owned);
            let previous_generation = self
                .invocation_history_textures
                .get(&key)
                .map(|existing| existing.generation)
                .unwrap_or(0);
            let should_recreate = self
                .invocation_history_textures
                .get(&key)
                .map(|existing| {
                    existing.format != texture_spec.format
                        || existing.size != texture_spec.size
                        || existing.usage != texture_spec.usage
                        || existing.is_depth != texture_spec.is_depth
                        || existing.history_signature != next_history_signature
                })
                .unwrap_or(true);

            if should_recreate {
                let label = format!("engine_invocation_history_{invocation_id}_{resource_id}");
                let texture = device.create_texture(&TextureDescriptor {
                    label: Some(label.as_str()),
                    size: Extent3d {
                        width: texture_spec.size.0,
                        height: texture_spec.size.1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: texture_spec.format,
                    usage: texture_spec.usage,
                    view_formats: &[],
                });
                self.invocation_history_textures.insert(
                    key,
                    RuntimeTextureResource {
                        texture,
                        format: texture_spec.format,
                        size: texture_spec.size,
                        usage: texture_spec.usage,
                        is_depth: texture_spec.is_depth,
                        history_signature: next_history_signature,
                        generation: previous_generation.saturating_add(1),
                        reused_last_frame: false,
                    },
                );
            } else if let Some(existing) = self.invocation_history_textures.get_mut(&key) {
                existing.reused_last_frame = true;
            }
        }

        Ok(())
    }

    pub fn realize_invocation_uniform_buffer(
        &mut self,
        device: &Device,
        invocation_id: &str,
        resource_id: GpuWorkResourceId,
        size: u64,
    ) -> Result<&RuntimeBufferResource> {
        let descriptor = self.descriptors.get(&resource_id).ok_or_else(|| {
            anyhow::anyhow!(
                "prepared invocation '{}' uploads unknown uniform buffer '{}'",
                invocation_id,
                resource_id
            )
        })?;
        let Some(spec) = Self::buffer_allocation_spec(descriptor)? else {
            bail!(
                "prepared invocation '{}' uploads '{}' but it is not a buffer resource",
                invocation_id,
                resource_id
            );
        };
        if !matches!(spec.kind, RuntimeBufferKind::Uniform) {
            bail!(
                "prepared invocation '{}' uploads '{}' but it is not a uniform buffer",
                invocation_id,
                resource_id
            );
        }

        let size = size.max(spec.size).max(1);
        let key = (invocation_id.to_string(), resource_id);
        let previous_generation = self
            .invocation_uniform_buffers
            .get(&key)
            .map(|existing| existing.generation)
            .unwrap_or(0);
        let should_recreate = self
            .invocation_uniform_buffers
            .get(&key)
            .map(|existing| existing.size != size || existing.kind != RuntimeBufferKind::Uniform)
            .unwrap_or(true);

        if should_recreate {
            let label = format!("engine_invocation_uniform_{invocation_id}_{resource_id}");
            let buffer = device.create_buffer(&BufferDescriptor {
                label: Some(label.as_str()),
                size,
                usage: spec.usage,
                mapped_at_creation: false,
            });
            self.invocation_uniform_buffers.insert(
                key.clone(),
                RuntimeBufferResource {
                    buffer,
                    size,
                    kind: RuntimeBufferKind::Uniform,
                    generation: previous_generation.saturating_add(1),
                    reused_last_frame: false,
                },
            );
        } else if let Some(existing) = self.invocation_uniform_buffers.get_mut(&key) {
            existing.reused_last_frame = true;
        }

        self.invocation_uniform_buffers
            .get(&key)
            .ok_or_else(|| anyhow::anyhow!("failed to realize invocation uniform buffer"))
    }

    pub fn texture_allocation_spec(
        descriptor: &RenderResourceDeclaration,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
    ) -> Result<Option<TextureAllocationSpec>> {
        let Some(normalized) =
            descriptor.gpu_descriptor(surface_size, legacy_surface_validation_format())?
        else {
            return Ok(None);
        };
        let GpuResourceDescriptor::Texture(texture) = normalized else {
            return Ok(None);
        };
        if texture.dimension() != GpuTextureDimension::D2 || texture.extent().depth_or_layers() != 1
        {
            bail!("legacy render runtime can realize only normalized 2D single-layer textures");
        }
        let extent = texture.extent();
        let render_texture = descriptor
            .texture_intent()
            .expect("normalized texture descriptors originate from render texture intent")
            .texture();
        let format = match render_texture.format {
            RenderTextureFormatPolicy::Surface => surface_format,
            RenderTextureFormatPolicy::Exact(format) => dynamic_format_to_wgpu(format),
        };
        Ok(Some(TextureAllocationSpec {
            size: (extent.width(), extent.height()),
            format,
            usage: dynamic_usage_to_wgpu(render_texture.usage),
            is_depth: texture.format().is_depth(),
        }))
    }

    pub fn buffer_allocation_spec(
        descriptor: &RenderResourceDeclaration,
    ) -> Result<Option<BufferAllocationSpec>> {
        let Some(normalized) = descriptor.gpu_descriptor((1, 1), GpuTextureFormat::Rgba8Unorm)?
        else {
            return Ok(None);
        };
        let GpuResourceDescriptor::Buffer(buffer) = normalized else {
            return Ok(None);
        };
        let kind = match descriptor {
            RenderResourceDeclaration::Uniform(_) => RuntimeBufferKind::Uniform,
            RenderResourceDeclaration::Storage(_) => RuntimeBufferKind::Storage,
            _ => return Ok(None),
        };
        let usage = match kind {
            RuntimeBufferKind::Uniform => {
                BufferUsages::UNIFORM | BufferUsages::COPY_SRC | BufferUsages::COPY_DST
            }
            RuntimeBufferKind::Storage => {
                BufferUsages::STORAGE
                    | BufferUsages::COPY_SRC
                    | BufferUsages::COPY_DST
                    | BufferUsages::VERTEX
                    | BufferUsages::INDEX
                    | BufferUsages::INDIRECT
            }
        };
        Ok(Some(BufferAllocationSpec {
            size: buffer.size_bytes(),
            usage,
            kind,
        }))
    }
}
