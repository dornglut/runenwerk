use super::*;
use crate::plugins::gpu::{GpuResourceDescriptor, GpuTextureDimension, GpuWorkResourceId};
use crate::plugins::render::renderer::dynamic_targets::{
    dynamic_format_to_wgpu, dynamic_usage_to_wgpu,
};
use crate::plugins::render::{
    RenderGpuResourceLowering, RenderTextureFormatPolicy, legacy_surface_validation_format,
};

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

            match Self::current_runtime_resource_disposition(
                descriptor,
                frame_size,
                surface_format,
            )? {
                CurrentRuntimeResourceDisposition::Buffer(spec) => {
                    self.textures.remove(&id);
                    self.realize_flow_buffer(device, id, descriptor.lifetime(), spec);
                }
                CurrentRuntimeResourceDisposition::FlowTexture(spec) => {
                    self.buffers.remove(&id);
                    self.realize_flow_texture(device, id, descriptor.lifetime(), spec);
                }
                CurrentRuntimeResourceDisposition::InvocationHistoryTexture(_) => {
                    self.textures.remove(&id);
                    self.buffers.remove(&id);
                }
                CurrentRuntimeResourceDisposition::ImportedTexture(_) => {
                    self.textures.remove(&id);
                    self.buffers.remove(&id);
                }
                CurrentRuntimeResourceDisposition::ImportedBuffer(_) => {
                    self.textures.remove(&id);
                    self.buffers.remove(&id);
                }
                CurrentRuntimeResourceDisposition::TargetAlias(_) => {
                    self.textures.remove(&id);
                    self.buffers.remove(&id);
                }
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

    fn realize_flow_texture(
        &mut self,
        device: &Device,
        id: GpuWorkResourceId,
        lifetime: crate::plugins::gpu::GpuResourceLifetime,
        spec: TextureAllocationSpec,
    ) {
        let previous_generation = self
            .textures
            .get(&id)
            .map(|existing| existing.generation)
            .unwrap_or(0);
        let should_recreate = match self.textures.get(&id) {
            Some(existing) => {
                lifetime.is_transient()
                    || existing.format != spec.format
                    || existing.size != spec.size
                    || existing.usage != spec.usage
                    || existing.is_depth != spec.is_depth
            }
            None => true,
        };

        if should_recreate {
            let label = format!("engine_render_resource_{id}");
            let texture = device.create_texture(&TextureDescriptor {
                label: Some(label.as_str()),
                size: Extent3d {
                    width: spec.size.0,
                    height: spec.size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: spec.format,
                usage: spec.usage,
                view_formats: &[],
            });
            self.textures.insert(
                id,
                RuntimeTextureResource {
                    texture,
                    format: spec.format,
                    size: spec.size,
                    usage: spec.usage,
                    is_depth: spec.is_depth,
                    history_signature: None,
                    generation: previous_generation.saturating_add(1),
                    reused_last_frame: false,
                },
            );
        } else if let Some(existing) = self.textures.get_mut(&id) {
            existing.reused_last_frame = true;
        }
    }

    fn realize_flow_buffer(
        &mut self,
        device: &Device,
        id: GpuWorkResourceId,
        lifetime: crate::plugins::gpu::GpuResourceLifetime,
        spec: BufferAllocationSpec,
    ) {
        let size = spec.size.max(1);
        let previous_generation = self
            .buffers
            .get(&id)
            .map(|existing| existing.generation)
            .unwrap_or(0);
        let should_recreate = match self.buffers.get(&id) {
            Some(existing) => {
                lifetime.is_transient() || existing.size != size || existing.kind != spec.kind
            }
            None => true,
        };

        if should_recreate {
            let label = format!("engine_render_resource_{id}");
            let buffer = device.create_buffer(&BufferDescriptor {
                label: Some(label.as_str()),
                size,
                usage: spec.usage,
                mapped_at_creation: false,
            });
            self.buffers.insert(
                id,
                RuntimeBufferResource {
                    buffer,
                    size,
                    kind: spec.kind,
                    generation: previous_generation.saturating_add(1),
                    reused_last_frame: false,
                },
            );
        } else if let Some(existing) = self.buffers.get_mut(&id) {
            existing.reused_last_frame = true;
        }
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
            let texture_spec = match Self::current_runtime_resource_disposition(
                &descriptor,
                surface_size,
                surface_format,
            )? {
                CurrentRuntimeResourceDisposition::InvocationHistoryTexture(spec) => spec,
                CurrentRuntimeResourceDisposition::Buffer(_)
                | CurrentRuntimeResourceDisposition::FlowTexture(_)
                | CurrentRuntimeResourceDisposition::ImportedTexture(_)
                | CurrentRuntimeResourceDisposition::ImportedBuffer(_)
                | CurrentRuntimeResourceDisposition::TargetAlias(_) => {
                    bail!(
                        "history declaration '{}' did not produce invocation-scoped texture allocation facts",
                        resource_id
                    );
                }
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
        let spec = match Self::current_runtime_resource_disposition(
            descriptor,
            (1, 1),
            TextureFormat::Rgba8Unorm,
        )? {
            CurrentRuntimeResourceDisposition::Buffer(spec) => spec,
            CurrentRuntimeResourceDisposition::FlowTexture(_)
            | CurrentRuntimeResourceDisposition::InvocationHistoryTexture(_)
            | CurrentRuntimeResourceDisposition::ImportedTexture(_)
            | CurrentRuntimeResourceDisposition::ImportedBuffer(_)
            | CurrentRuntimeResourceDisposition::TargetAlias(_) => {
                bail!(
                    "prepared invocation '{}' uploads '{}' but it is not a buffer resource",
                    invocation_id,
                    resource_id
                );
            }
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

    pub(super) fn current_runtime_resource_disposition(
        descriptor: &RenderResourceDeclaration,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
    ) -> core::result::Result<
        CurrentRuntimeResourceDisposition,
        CurrentRuntimeResourceRealizationError,
    > {
        let lowering =
            descriptor.lower_gpu_resource(surface_size, legacy_surface_validation_format())?;
        Self::current_runtime_resource_disposition_from_lowering(
            descriptor,
            lowering,
            surface_format,
        )
    }

    pub(super) fn current_runtime_resource_disposition_from_lowering(
        descriptor: &RenderResourceDeclaration,
        lowering: RenderGpuResourceLowering,
        surface_format: TextureFormat,
    ) -> core::result::Result<
        CurrentRuntimeResourceDisposition,
        CurrentRuntimeResourceRealizationError,
    > {
        let resource_id = *descriptor.id();
        match lowering {
            RenderGpuResourceLowering::Normalized(normalized) => match normalized.as_ref() {
                GpuResourceDescriptor::Buffer(buffer) => {
                    let kind = match descriptor {
                        RenderResourceDeclaration::Uniform(_) => RuntimeBufferKind::Uniform,
                        RenderResourceDeclaration::Storage(_) => RuntimeBufferKind::Storage,
                        RenderResourceDeclaration::Sampled(_)
                        | RenderResourceDeclaration::StorageImage(_)
                        | RenderResourceDeclaration::ColorAttachment(_)
                        | RenderResourceDeclaration::DepthAttachment(_)
                        | RenderResourceDeclaration::History(_)
                        | RenderResourceDeclaration::TargetAlias(_)
                        | RenderResourceDeclaration::ImportedTexture(_)
                        | RenderResourceDeclaration::ImportedBuffer(_) => {
                            return Err(
                                CurrentRuntimeResourceRealizationError::NormalizedDeclarationMismatch {
                                    resource_id,
                                    normalized_kind: "buffer",
                                },
                            );
                        }
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
                    Ok(CurrentRuntimeResourceDisposition::Buffer(
                        BufferAllocationSpec {
                            size: buffer.size_bytes(),
                            usage,
                            kind,
                        },
                    ))
                }
                GpuResourceDescriptor::Texture(texture) => {
                    if texture.dimension() != GpuTextureDimension::D2
                        || texture.extent().depth_or_layers() != 1
                    {
                        return Err(
                            CurrentRuntimeResourceRealizationError::UnsupportedTextureShape {
                                resource_id,
                                dimension: texture.dimension(),
                                depth_or_layers: texture.extent().depth_or_layers(),
                            },
                        );
                    }
                    let render_texture = match descriptor {
                        RenderResourceDeclaration::Sampled(value)
                        | RenderResourceDeclaration::StorageImage(value)
                        | RenderResourceDeclaration::ColorAttachment(value)
                        | RenderResourceDeclaration::DepthAttachment(value)
                        | RenderResourceDeclaration::History(value) => value.texture(),
                        RenderResourceDeclaration::Uniform(_)
                        | RenderResourceDeclaration::Storage(_)
                        | RenderResourceDeclaration::TargetAlias(_)
                        | RenderResourceDeclaration::ImportedTexture(_)
                        | RenderResourceDeclaration::ImportedBuffer(_) => {
                            return Err(
                                CurrentRuntimeResourceRealizationError::NormalizedDeclarationMismatch {
                                    resource_id,
                                    normalized_kind: "texture",
                                },
                            );
                        }
                    };
                    let format = match render_texture.format {
                        RenderTextureFormatPolicy::Surface => surface_format,
                        RenderTextureFormatPolicy::Exact(format) => dynamic_format_to_wgpu(format),
                    };
                    let spec = TextureAllocationSpec {
                        size: (texture.extent().width(), texture.extent().height()),
                        format,
                        usage: dynamic_usage_to_wgpu(render_texture.usage),
                        is_depth: texture.format().is_depth(),
                    };
                    if matches!(descriptor, RenderResourceDeclaration::History(_)) {
                        Ok(CurrentRuntimeResourceDisposition::InvocationHistoryTexture(
                            spec,
                        ))
                    } else {
                        Ok(CurrentRuntimeResourceDisposition::FlowTexture(spec))
                    }
                }
                GpuResourceDescriptor::TextureView(_) => Err(
                    CurrentRuntimeResourceRealizationError::UnsupportedNormalizedKind {
                        resource_id,
                        kind: "texture-view",
                    },
                ),
                GpuResourceDescriptor::Sampler(_) => Err(
                    CurrentRuntimeResourceRealizationError::UnsupportedNormalizedKind {
                        resource_id,
                        kind: "sampler",
                    },
                ),
                GpuResourceDescriptor::QuerySet(_) => Err(
                    CurrentRuntimeResourceRealizationError::UnsupportedNormalizedKind {
                        resource_id,
                        kind: "query-set",
                    },
                ),
            },
            RenderGpuResourceLowering::ImportedTexture(intent) => {
                Ok(CurrentRuntimeResourceDisposition::ImportedTexture(intent))
            }
            RenderGpuResourceLowering::ImportedBuffer(intent) => {
                Ok(CurrentRuntimeResourceDisposition::ImportedBuffer(intent))
            }
            RenderGpuResourceLowering::TargetAlias(alias) => {
                Ok(CurrentRuntimeResourceDisposition::TargetAlias(alias))
            }
        }
    }
}
