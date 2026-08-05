use super::*;
use crate::plugins::gpu::{
    GpuBindGroupLayoutDescriptor, GpuBindingClass, GpuBindingDeclaration, GpuBindingKey,
    GpuBindingKind, GpuBindingProvenance, GpuProgramSourceIdentity, GpuSamplerClass,
    GpuShaderStage, GpuShaderStages, GpuStorageBufferAccess, GpuStorageTextureAccess,
    GpuTextureFormat, GpuTextureSampleClass, GpuTextureViewDimension,
};
use crate::plugins::render::pipelines::FlowPassPipelineVariant;
use crate::plugins::render::{RenderFeatureId, RenderPassId};

enum RuntimeBindingResource<'a> {
    TextureView(TextureView),
    SamplerPlaceholder,
    Buffer(&'a Buffer),
}

struct RuntimeBindingResolved<'a> {
    kind: GpuBindingKind,
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
        program_source_identity: &GpuProgramSourceIdentity,
        pipeline_variant: FlowPassPipelineVariant,
        bindings: &CompiledPassBindings,
        visibility: ShaderStages,
        allow_depth_sampling: bool,
        color_formats: Vec<TextureFormat>,
        depth_format: Option<TextureFormat>,
        vertex_layout_signature_hash: u64,
        raster_state_signature_hash: u64,
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
                    let sample_class = if texture.is_depth {
                        GpuTextureSampleClass::Depth
                    } else {
                        GpuTextureSampleClass::FloatFilterable
                    };
                    resolved_entries.push(RuntimeBindingResolved {
                        kind: GpuBindingKind::sampled_texture(
                            sample_class,
                            GpuTextureViewDimension::D2,
                            false,
                        )?,
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
                        kind: GpuBindingKind::sampler(GpuSamplerClass::Filtering),
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
                        kind: GpuBindingKind::storage_texture(
                            gpu_storage_texture_access(*access),
                            gpu_texture_format_from_wgpu(texture.format)?,
                            GpuTextureViewDimension::D2,
                        )?,
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
                        kind: GpuBindingKind::uniform_buffer(false, None),
                        resource: RuntimeBindingResource::Buffer(buffer.buffer),
                        resource_identity: Some(buffer.id),
                        generation_token: buffer.generation,
                        cacheable: buffer.generation.is_some(),
                    });
                }
                CompiledBindingEntry::StorageBuffer { resource, access } => {
                    let buffer = runtime_resources.resolve_storage_buffer_ref(pass_id, resource)?;
                    resolved_entries.push(RuntimeBindingResolved {
                        kind: GpuBindingKind::storage_buffer(
                            gpu_storage_buffer_access(*access),
                            false,
                            None,
                        ),
                        resource: RuntimeBindingResource::Buffer(buffer.buffer),
                        resource_identity: Some(buffer.id),
                        generation_token: buffer.generation,
                        cacheable: buffer.generation.is_some(),
                    });
                }
            }
        }

        let gpu_visibility = gpu_shader_stages_from_wgpu(visibility)?;
        let binding_declarations = resolved_entries
            .iter()
            .enumerate()
            .map(|(binding, value)| {
                let binding = u64::try_from(binding).map_err(|_| {
                    anyhow::anyhow!(
                        "pass '{}' declares more bindings than can be represented by G4B binding identity",
                        pass_id
                    )
                })?;
                Ok(GpuBindingDeclaration::new(
                    GpuBindingKey::try_new(0, binding)?,
                    gpu_visibility,
                    value.kind,
                    None,
                    format!("pass-{pass_id}-binding-{binding}"),
                    GpuBindingProvenance::new(
                        "render-flow-primary-bind-group",
                        Some(format!("pass {pass_id}")),
                    )?,
                )?)
            })
            .collect::<Result<Vec<_>>>()?;
        let primary_bind_group_layout =
            GpuBindGroupLayoutDescriptor::new(0, binding_declarations)?;
        let bind_group_layout_entries = primary_bind_group_layout
            .bindings()
            .map(wgpu_bind_group_layout_entry)
            .collect::<Result<Vec<_>>>()?;

        let pipeline_key = FlowPassPipelineKey {
            flow_id: flow.flow_id,
            pass_id,
            pass_kind,
            feature_id: pass_feature_id,
            program_source_identity: program_source_identity.clone(),
            pipeline_variant,
            primary_bind_group_layout,
            material_specialization_fragment_hash: material_specialization_fragment_hash(
                packet,
                pass_feature_id,
            ),
            view_signature_hash: hash_view_signature(packet.view_id.as_str(), packet.surface_size),
            feature_runtime_version: feature_runtime_version(packet, pass_feature_id),
            color_formats,
            depth_format,
            vertex_layout_signature_hash,
            raster_state_signature_hash,
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

fn gpu_shader_stages_from_wgpu(visibility: ShaderStages) -> Result<GpuShaderStages> {
    let supported = ShaderStages::COMPUTE | ShaderStages::VERTEX | ShaderStages::FRAGMENT;
    let unsupported = visibility.difference(supported);
    if !unsupported.is_empty() {
        bail!(
            "current render binding visibility contains unsupported backend stages: {unsupported:?}"
        );
    }

    Ok(GpuShaderStages::new(
        [
            (ShaderStages::COMPUTE, GpuShaderStage::Compute),
            (ShaderStages::VERTEX, GpuShaderStage::Vertex),
            (ShaderStages::FRAGMENT, GpuShaderStage::Fragment),
        ]
        .into_iter()
        .filter_map(|(wgpu_stage, gpu_stage)| visibility.contains(wgpu_stage).then_some(gpu_stage)),
    )?)
}

fn wgpu_shader_stages(visibility: GpuShaderStages) -> ShaderStages {
    visibility
        .iter()
        .fold(ShaderStages::empty(), |stages, stage| {
            stages
                | match stage {
                    GpuShaderStage::Compute => ShaderStages::COMPUTE,
                    GpuShaderStage::Vertex => ShaderStages::VERTEX,
                    GpuShaderStage::Fragment => ShaderStages::FRAGMENT,
                }
        })
}

fn wgpu_bind_group_layout_entry(
    declaration: &GpuBindingDeclaration,
) -> Result<BindGroupLayoutEntry> {
    Ok(BindGroupLayoutEntry {
        binding: declaration.key().binding(),
        visibility: wgpu_shader_stages(declaration.visibility()),
        ty: wgpu_binding_type(*declaration.kind())?,
        count: declaration.array_count(),
    })
}

fn wgpu_binding_type(kind: GpuBindingKind) -> Result<BindingType> {
    Ok(match kind.class() {
        GpuBindingClass::UniformBuffer => BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: kind.uses_dynamic_offset(),
            min_binding_size: kind.minimum_buffer_size(),
        },
        GpuBindingClass::StorageBuffer => BindingType::Buffer {
            ty: BufferBindingType::Storage {
                read_only: matches!(
                    kind.storage_buffer_access(),
                    Some(GpuStorageBufferAccess::ReadOnly)
                ),
            },
            has_dynamic_offset: kind.uses_dynamic_offset(),
            min_binding_size: kind.minimum_buffer_size(),
        },
        GpuBindingClass::SampledTexture => BindingType::Texture {
            sample_type: match kind.texture_sample_class().ok_or_else(|| {
                anyhow::anyhow!("sampled-texture binding is missing its normalized sample class")
            })? {
                GpuTextureSampleClass::FloatFilterable => {
                    TextureSampleType::Float { filterable: true }
                }
                GpuTextureSampleClass::FloatUnfilterable => {
                    TextureSampleType::Float { filterable: false }
                }
                GpuTextureSampleClass::Depth => TextureSampleType::Depth,
                GpuTextureSampleClass::Sint => TextureSampleType::Sint,
                GpuTextureSampleClass::Uint => TextureSampleType::Uint,
            },
            view_dimension: wgpu_texture_view_dimension(
                kind.texture_view_dimension().ok_or_else(|| {
                    anyhow::anyhow!(
                        "sampled-texture binding is missing its normalized view dimension"
                    )
                })?,
            ),
            multisampled: kind.is_multisampled_texture(),
        },
        GpuBindingClass::StorageTexture => BindingType::StorageTexture {
            access: match kind.storage_texture_access().ok_or_else(|| {
                anyhow::anyhow!("storage-texture binding is missing normalized access")
            })? {
                GpuStorageTextureAccess::ReadOnly => StorageTextureAccess::ReadOnly,
                GpuStorageTextureAccess::WriteOnly => StorageTextureAccess::WriteOnly,
                GpuStorageTextureAccess::ReadWrite => StorageTextureAccess::ReadWrite,
            },
            format: wgpu_texture_format(kind.storage_texture_format().ok_or_else(|| {
                anyhow::anyhow!("storage-texture binding is missing its normalized format")
            })?),
            view_dimension: wgpu_texture_view_dimension(
                kind.texture_view_dimension().ok_or_else(|| {
                    anyhow::anyhow!(
                        "storage-texture binding is missing its normalized view dimension"
                    )
                })?,
            ),
        },
        GpuBindingClass::Sampler => BindingType::Sampler(
            match kind.sampler_class().ok_or_else(|| {
                anyhow::anyhow!("sampler binding is missing its normalized sampler class")
            })? {
                GpuSamplerClass::Filtering => SamplerBindingType::Filtering,
                GpuSamplerClass::NonFiltering => SamplerBindingType::NonFiltering,
                GpuSamplerClass::Comparison => SamplerBindingType::Comparison,
            },
        ),
    })
}

fn gpu_storage_buffer_access(access: CompiledStorageAccess) -> GpuStorageBufferAccess {
    match access {
        CompiledStorageAccess::ReadOnly => GpuStorageBufferAccess::ReadOnly,
        CompiledStorageAccess::WriteOnly | CompiledStorageAccess::ReadWrite => {
            GpuStorageBufferAccess::ReadWrite
        }
    }
}

fn gpu_storage_texture_access(access: CompiledStorageAccess) -> GpuStorageTextureAccess {
    match access {
        CompiledStorageAccess::ReadOnly => GpuStorageTextureAccess::ReadOnly,
        CompiledStorageAccess::WriteOnly => GpuStorageTextureAccess::WriteOnly,
        CompiledStorageAccess::ReadWrite => GpuStorageTextureAccess::ReadWrite,
    }
}

fn gpu_texture_format_from_wgpu(format: TextureFormat) -> Result<GpuTextureFormat> {
    match format {
        TextureFormat::Rgba8Unorm => Ok(GpuTextureFormat::Rgba8Unorm),
        TextureFormat::Rgba8UnormSrgb => Ok(GpuTextureFormat::Rgba8UnormSrgb),
        TextureFormat::Bgra8Unorm => Ok(GpuTextureFormat::Bgra8Unorm),
        TextureFormat::Bgra8UnormSrgb => Ok(GpuTextureFormat::Bgra8UnormSrgb),
        TextureFormat::R32Uint => Ok(GpuTextureFormat::R32Uint),
        TextureFormat::Depth32Float => Ok(GpuTextureFormat::Depth32Float),
        unsupported => bail!(
            "current render storage texture format {unsupported:?} has no accepted G4B normalized format"
        ),
    }
}

fn wgpu_texture_format(format: GpuTextureFormat) -> TextureFormat {
    match format {
        GpuTextureFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
        GpuTextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        GpuTextureFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
        GpuTextureFormat::Bgra8UnormSrgb => TextureFormat::Bgra8UnormSrgb,
        GpuTextureFormat::R32Uint => TextureFormat::R32Uint,
        GpuTextureFormat::Depth32Float => TextureFormat::Depth32Float,
    }
}

fn wgpu_texture_view_dimension(dimension: GpuTextureViewDimension) -> TextureViewDimension {
    match dimension {
        GpuTextureViewDimension::D1 => TextureViewDimension::D1,
        GpuTextureViewDimension::D2 => TextureViewDimension::D2,
        GpuTextureViewDimension::D2Array => TextureViewDimension::D2Array,
        GpuTextureViewDimension::Cube => TextureViewDimension::Cube,
        GpuTextureViewDimension::CubeArray => TextureViewDimension::CubeArray,
        GpuTextureViewDimension::D3 => TextureViewDimension::D3,
    }
}

