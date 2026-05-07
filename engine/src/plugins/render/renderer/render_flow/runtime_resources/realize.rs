use super::*;

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
                    let label = format!("engine_render_resource_{id}");
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
                        id,
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
            RenderResourceDescriptor::TargetAlias(_)
            | RenderResourceDescriptor::ImportedTexture(_)
            | RenderResourceDescriptor::UniformBuffer(_)
            | RenderResourceDescriptor::StorageBuffer(_)
            | RenderResourceDescriptor::ImportedBuffer(_) => None,
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
