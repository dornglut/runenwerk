use super::*;
use crate::plugins::render::{RenderFeatureId, RenderPassId};

enum RuntimeBindingResource<'a> {
    TextureView(TextureView),
    SamplerPlaceholder,
    Buffer(&'a Buffer),
}

struct RuntimeBindingResolved<'a> {
    layout_ty: BindingType,
    resource: RuntimeBindingResource<'a>,
    resource_identity: Option<RuntimeResourceKey>,
    generation_token: Option<u64>,
    cacheable: bool,
}

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve_compiled_bind_group<'a>(
        &mut self,
        device: &Device,
        frame_texture: &'a Texture,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        pass_id: RenderPassId,
        pass_kind: FlowPassKind,
        pass_feature_id: Option<RenderFeatureId>,
        shader_identity: &str,
        shader_revision: u64,
        bindings: &CompiledPassBindings,
        visibility: ShaderStages,
        allow_depth_sampling: bool,
        color_formats: Vec<TextureFormat>,
        depth_format: Option<TextureFormat>,
        vertex_layout_signature_hash: u64,
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
                    let resource_key = runtime_resources.resolve_resource_key(
                        pass_id,
                        resource,
                        "sampled_texture",
                    )?;
                    let texture = match resource_key.clone() {
                        RuntimeResourceKey::DynamicTexture(key) => {
                            self.dynamic_texture_targets.texture_ref(pass_id, &key)?
                        }
                        _ => runtime_resources.resolve_texture(
                            pass_id,
                            resource_key,
                            frame_texture,
                            packet.surface_size,
                            packet.surface_format,
                        )?,
                    };
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
                        resource_identity: Some(texture.id),
                        generation_token: texture.generation,
                        cacheable: texture.generation.is_some(),
                    });
                }
                CompiledBindingEntry::Sampler => {
                    resolved_entries.push(RuntimeBindingResolved {
                        layout_ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        resource: RuntimeBindingResource::SamplerPlaceholder,
                        resource_identity: None,
                        generation_token: Some(0),
                        cacheable: true,
                    });
                }
                CompiledBindingEntry::StorageTexture { resource, access } => {
                    let resource_key = runtime_resources.resolve_resource_key(
                        pass_id,
                        resource,
                        "storage_texture",
                    )?;
                    let texture = match resource_key.clone() {
                        RuntimeResourceKey::DynamicTexture(key) => {
                            self.dynamic_texture_targets.texture_ref(pass_id, &key)?
                        }
                        _ => runtime_resources.resolve_texture(
                            pass_id,
                            resource_key,
                            frame_texture,
                            packet.surface_size,
                            packet.surface_format,
                        )?,
                    };
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
                        resource_identity: Some(texture.id),
                        generation_token: texture.generation,
                        cacheable: texture.generation.is_some(),
                    });
                }
                CompiledBindingEntry::UniformBuffer { resource } => {
                    let buffer =
                        runtime_resources.resolve_uniform_buffer_for_pass(pass_id, *resource)?;
                    resolved_entries.push(RuntimeBindingResolved {
                        layout_ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        resource: RuntimeBindingResource::Buffer(buffer.buffer),
                        resource_identity: Some(buffer.id),
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
                        resource_identity: Some(buffer.id),
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
            flow_id: flow.flow_id,
            pass_id,
            pass_kind,
            feature_id: pass_feature_id,
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
            vertex_layout_signature_hash,
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
                value.resource_identity.hash(&mut signature_hasher);
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
}
