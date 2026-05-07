use super::*;
use crate::plugins::render::renderer::dynamic_targets::{
    dynamic_format_to_wgpu, dynamic_usage_to_wgpu,
};
use crate::plugins::render::{
    RenderTextureDescriptor, RenderTextureFormatPolicy, RenderTextureSizePolicy,
};

impl FlowRuntimeResources {
    pub fn realize_for_frame(
        &mut self,
        device: &Device,
        flow: &CompiledRenderFlowPlan,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
    ) {
        let frame_size = (surface_size.0.max(1), surface_size.1.max(1));
        let mut declared_ids = BTreeSet::<RenderResourceId>::new();

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
                RenderResourceDescriptor::UniformBuffer(_)
                | RenderResourceDescriptor::StorageBuffer(_)
                | RenderResourceDescriptor::ImportedBuffer(_) => RuntimeResourceKind::BufferLike,
                _ => RuntimeResourceKind::TextureLike,
            };
            self.kinds.insert(id, kind);

            if !matches!(descriptor, RenderResourceDescriptor::HistoryTexture(_))
                && let Some(texture_spec) =
                    Self::texture_allocation_spec(descriptor, frame_size, surface_format)
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
                RenderResourceDescriptor::HistoryTexture(_) => Some((*id, descriptor.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        for (resource_id, descriptor) in history_descriptors {
            let Some(texture_spec) =
                Self::texture_allocation_spec(&descriptor, surface_size, surface_format)
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
        resource_id: RenderResourceId,
        size: u64,
    ) -> Result<&RuntimeBufferResource> {
        let descriptor = self.descriptors.get(&resource_id).ok_or_else(|| {
            anyhow::anyhow!(
                "prepared invocation '{}' uploads unknown uniform buffer '{}'",
                invocation_id,
                resource_id
            )
        })?;
        let Some(spec) = Self::buffer_allocation_spec(descriptor) else {
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
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
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
        descriptor: &RenderResourceDescriptor,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
    ) -> Option<TextureAllocationSpec> {
        let texture = match descriptor {
            RenderResourceDescriptor::SampledTexture(value) => &value.texture,
            RenderResourceDescriptor::StorageTexture(value) => &value.texture,
            RenderResourceDescriptor::ColorTarget(value) => &value.texture,
            RenderResourceDescriptor::DepthTarget(value) => &value.texture,
            RenderResourceDescriptor::HistoryTexture(value) => &value.texture,
            RenderResourceDescriptor::TargetAlias(_)
            | RenderResourceDescriptor::ImportedTexture(_)
            | RenderResourceDescriptor::UniformBuffer(_)
            | RenderResourceDescriptor::StorageBuffer(_)
            | RenderResourceDescriptor::ImportedBuffer(_) => return None,
        };
        Some(Self::texture_descriptor_allocation_spec(
            texture,
            surface_size,
            surface_format,
        ))
    }

    fn texture_descriptor_allocation_spec(
        texture: &RenderTextureDescriptor,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
    ) -> TextureAllocationSpec {
        let size = match texture.size {
            RenderTextureSizePolicy::Surface => surface_size,
            RenderTextureSizePolicy::Fixed { width, height } => (width.max(1), height.max(1)),
        };
        let format = match texture.format {
            RenderTextureFormatPolicy::Surface => surface_format,
            RenderTextureFormatPolicy::Exact(format) => dynamic_format_to_wgpu(format),
        };
        TextureAllocationSpec {
            size,
            format,
            usage: dynamic_usage_to_wgpu(texture.usage),
            is_depth: texture.format
                == RenderTextureFormatPolicy::Exact(
                    crate::plugins::render::RenderTextureTargetFormat::Depth32Float,
                ),
        }
    }

    pub fn buffer_allocation_spec(
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
            | RenderResourceDescriptor::TargetAlias(_)
            | RenderResourceDescriptor::ImportedTexture(_)
            | RenderResourceDescriptor::ImportedBuffer(_) => None,
        }
    }
}
