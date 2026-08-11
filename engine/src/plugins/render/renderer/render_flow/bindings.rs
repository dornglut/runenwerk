use super::*;
use crate::plugins::gpu::{
    CurrentRenderBindGroupTerminal, GpuAdmittedProgramSource, GpuBindGroupLayoutDescriptor,
    GpuBindingClass, GpuBindingDeclaration, GpuBindingKey, GpuBindingKind, GpuBindingProvenance,
    GpuBlendMode as GpuPipelineBlendMode, GpuCapabilityRequirements, GpuColorTargetStateDescriptor,
    GpuColorWriteMask, GpuCompareFunction, GpuComputePipelineDescriptor,
    GpuCullMode as GpuPipelineCullMode, GpuDepthStencilStateDescriptor, GpuEntryPointDescriptor,
    GpuEntryPointName, GpuFragmentOutputStateDescriptor, GpuFrontFace, GpuIndexFormat,
    GpuMultisampleStateDescriptor, GpuPipelineLayoutDescriptor, GpuPrimitiveStateDescriptor,
    GpuPrimitiveTopology as GpuPipelinePrimitiveTopology, GpuProgramDescriptor,
    GpuProgramInterfaceDescriptor, GpuRealizedBuffer, GpuRealizedTextureView, GpuRenderEntryPoints,
    GpuRenderPipelineDescriptor, GpuRenderPipelineStateDescriptor, GpuSamplerClass, GpuShaderStage,
    GpuShaderStages, GpuSpecializationValueSet, GpuStorageBufferAccess, GpuStorageTextureAccess,
    GpuTextureFormat, GpuTextureSampleClass, GpuTextureViewDimension, GpuVertexAttribute,
    GpuVertexBufferLayoutDescriptor, GpuVertexFormat, GpuVertexInputStateDescriptor,
    GpuVertexStepMode,
};
use crate::plugins::render::pipelines::FlowPassPipelineDescriptor;
use crate::plugins::render::renderer::resource_descriptors::linear_sampler_descriptor;
use crate::plugins::render::{
    RenderBlendMode, RenderCullMode, RenderDepthPolicy, RenderFeatureId, RenderPassId,
    RenderPrimitiveTopology, RenderRasterState, RenderVertexFormat, RenderVertexStepMode,
};

enum RuntimeBindingResource {
    TextureView(GpuRealizedTextureView),
    SurfaceTextureView(TextureView),
    SamplerPlaceholder,
    Buffer(GpuRealizedBuffer),
}

struct RuntimeBindingResolved {
    key: GpuBindingKey,
    kind: GpuBindingKind,
    resource: RuntimeBindingResource,
    resource_identity: Option<RuntimeResourceKey>,
    generation_token: Option<u64>,
    cacheable: bool,
}

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn resolve_compiled_bind_group(
        &mut self,
        context: &GpuContext,
        device: &Device,
        frame_texture: &Texture,
        packet: &RendererPreparedPacket,
        flow: &CompiledRenderFlowPlan,
        pass_id: RenderPassId,
        pass_kind: FlowPassKind,
        pass_feature_id: Option<RenderFeatureId>,
        program_source: &GpuAdmittedProgramSource,
        specialization: GpuSpecializationValueSet,
        bindings: &CompiledPassBindings,
        visibility: ShaderStages,
        allow_depth_sampling: bool,
        color_formats: Vec<TextureFormat>,
        depth_format: Option<TextureFormat>,
        runtime_resources: &FlowRuntimeResources,
    ) -> Result<(
        FlowPassPipelineKey,
        Option<BindGroupLayout>,
        Option<BindGroup>,
    )> {
        let mut resolved_entries = Vec::<RuntimeBindingResolved>::new();
        for entry in &bindings.bind_group.entries {
            match entry {
                CompiledBindingEntry::SampledTexture { key, resource } => {
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
                        key: *key,
                        kind: GpuBindingKind::sampled_texture(
                            sample_class,
                            GpuTextureViewDimension::D2,
                            false,
                        )?,
                        resource: resolved_binding_texture_view(&texture)?,
                        resource_identity: Some(texture.id),
                        generation_token: texture.generation,
                        cacheable: texture.generation.is_some(),
                    });
                }
                CompiledBindingEntry::Sampler { key } => {
                    resolved_entries.push(RuntimeBindingResolved {
                        key: *key,
                        kind: GpuBindingKind::sampler(GpuSamplerClass::Filtering),
                        resource: RuntimeBindingResource::SamplerPlaceholder,
                        resource_identity: None,
                        generation_token: Some(0),
                        cacheable: true,
                    });
                }
                CompiledBindingEntry::StorageTexture {
                    key,
                    resource,
                    access,
                } => {
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
                        key: *key,
                        kind: GpuBindingKind::storage_texture(
                            gpu_storage_texture_access(*access),
                            gpu_texture_format_from_wgpu(texture.format)?,
                            GpuTextureViewDimension::D2,
                        )?,
                        resource: resolved_binding_texture_view(&texture)?,
                        resource_identity: Some(texture.id),
                        generation_token: texture.generation,
                        cacheable: texture.generation.is_some(),
                    });
                }
                CompiledBindingEntry::UniformBuffer { key, resource } => {
                    let buffer =
                        runtime_resources.resolve_uniform_buffer_for_pass(pass_id, *resource)?;
                    resolved_entries.push(RuntimeBindingResolved {
                        key: *key,
                        kind: GpuBindingKind::uniform_buffer(false, None),
                        resource: RuntimeBindingResource::Buffer(buffer.buffer.clone()),
                        resource_identity: Some(buffer.id),
                        generation_token: buffer.generation,
                        cacheable: buffer.generation.is_some(),
                    });
                }
                CompiledBindingEntry::StorageBuffer {
                    key,
                    resource,
                    access,
                } => {
                    let buffer = runtime_resources.resolve_storage_buffer_ref(pass_id, resource)?;
                    resolved_entries.push(RuntimeBindingResolved {
                        key: *key,
                        kind: GpuBindingKind::storage_buffer(
                            gpu_storage_buffer_access(*access),
                            false,
                            None,
                        ),
                        resource: RuntimeBindingResource::Buffer(buffer.buffer.clone()),
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
            .map(|value| {
                Ok(GpuBindingDeclaration::new(
                    value.key,
                    gpu_visibility,
                    value.kind,
                    None,
                    format!("pass-{pass_id}-binding-{}", value.key.binding()),
                    GpuBindingProvenance::new(
                        "render-flow-primary-bind-group",
                        Some(format!("pass {pass_id}")),
                    )?,
                )?)
            })
            .collect::<Result<Vec<_>>>()?;
        let primary_bind_group_layout = GpuBindGroupLayoutDescriptor::new(0, binding_declarations)?;
        let bind_group_layout_entries = primary_bind_group_layout
            .bindings()
            .map(wgpu_bind_group_layout_entry)
            .collect::<Result<Vec<_>>>()?;
        let pipeline_layout =
            gpu_pipeline_layout_for_pass(packet, flow, pass_id, &primary_bind_group_layout)?;
        let render_pipeline_state =
            gpu_render_pipeline_state_for_pass(flow, pass_id, &color_formats, depth_format)?;
        let pipeline_descriptor = gpu_pipeline_descriptor_for_pass(
            program_source,
            pass_kind,
            pipeline_layout,
            render_pipeline_state,
            specialization,
        )?;

        let pipeline_key = FlowPassPipelineKey {
            flow_id: flow.flow_id,
            pass_id,
            pass_kind,
            feature_id: pass_feature_id,
            pipeline_descriptor,
        };

        if bind_group_layout_entries.is_empty() {
            return Ok((pipeline_key, None, None));
        }

        let shared_sampler = if resolved_entries
            .iter()
            .any(|entry| matches!(entry.resource, RuntimeBindingResource::SamplerPlaceholder))
        {
            Some(self.flow_pipeline_cache.get_or_realize_sampler(
                context,
                pipeline_key.clone(),
                linear_sampler_descriptor(
                    "engine_compiled_flow_sampler",
                    crate::plugins::gpu::GpuResourceLifetime::Retained,
                )?,
            )?)
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

        let mut can_cache_bind_group = true;
        let mut signature_hasher = std::collections::hash_map::DefaultHasher::new();
        for value in &resolved_entries {
            value.key.hash(&mut signature_hasher);
            if value.cacheable {
                value.resource_identity.hash(&mut signature_hasher);
                value.generation_token.hash(&mut signature_hasher);
            } else {
                can_cache_bind_group = false;
            }
        }
        let buffers = resolved_entries
            .iter()
            .filter_map(|entry| match &entry.resource {
                RuntimeBindingResource::Buffer(buffer) => Some(buffer),
                _ => None,
            })
            .collect::<Vec<_>>();
        let views = resolved_entries
            .iter()
            .filter_map(|entry| match &entry.resource {
                RuntimeBindingResource::TextureView(view) => Some(view),
                _ => None,
            })
            .collect::<Vec<_>>();
        let samplers = shared_sampler.iter().collect::<Vec<_>>();
        let mut bind_group = None;
        context.current_render_resource_bridge().for_bind_group(
            &buffers,
            &views,
            &samplers,
            CreateCompiledBindGroup {
                device,
                layout: &bind_group_layout,
                resolved_entries: &resolved_entries,
                can_cache: can_cache_bind_group,
                signature_hash: signature_hasher.finish(),
                pipeline_key: &pipeline_key,
                cache: &mut self.flow_pipeline_cache,
                output: &mut bind_group,
            },
        )?;
        let bind_group = bind_group.ok_or_else(|| {
            anyhow::anyhow!(
                "current render resource bridge did not create pass '{pass_id}' bindings"
            )
        })?;

        Ok((pipeline_key, Some(bind_group_layout), Some(bind_group)))
    }
}

fn resolved_binding_texture_view(
    texture: &ResolvedTextureRef<'_>,
) -> Result<RuntimeBindingResource> {
    match texture.texture {
        RuntimeTextureRef::Surface(texture) => Ok(RuntimeBindingResource::SurfaceTextureView(
            texture.create_view(&TextureViewDescriptor::default()),
        )),
        RuntimeTextureRef::Realized(_) => texture
            .realized_view
            .cloned()
            .map(RuntimeBindingResource::TextureView)
            .ok_or_else(|| {
                anyhow::anyhow!("realized texture '{}' has no realized view", texture.id)
            }),
    }
}

struct CreateCompiledBindGroup<'a> {
    device: &'a Device,
    layout: &'a BindGroupLayout,
    resolved_entries: &'a [RuntimeBindingResolved],
    can_cache: bool,
    signature_hash: u64,
    pipeline_key: &'a FlowPassPipelineKey,
    cache: &'a mut FlowPipelineArtifactCache,
    output: &'a mut Option<BindGroup>,
}

impl CurrentRenderBindGroupTerminal for CreateCompiledBindGroup<'_> {
    fn bind_resources(self, buffers: &[&Buffer], views: &[&TextureView], samplers: &[&Sampler]) {
        let mut buffer_index = 0;
        let mut view_index = 0;
        let mut entries = Vec::with_capacity(self.resolved_entries.len());
        for value in self.resolved_entries {
            let resource = match &value.resource {
                RuntimeBindingResource::TextureView(_) => {
                    let view = views[view_index];
                    view_index += 1;
                    BindingResource::TextureView(view)
                }
                RuntimeBindingResource::SurfaceTextureView(view) => {
                    BindingResource::TextureView(view)
                }
                RuntimeBindingResource::SamplerPlaceholder => BindingResource::Sampler(samplers[0]),
                RuntimeBindingResource::Buffer(_) => {
                    let buffer = buffers[buffer_index];
                    buffer_index += 1;
                    buffer.as_entire_binding()
                }
            };
            entries.push(BindGroupEntry {
                binding: value.key.binding(),
                resource,
            });
        }
        let bind_group = if self.can_cache {
            let key = FlowPassBindGroupKey {
                pipeline: self.pipeline_key.clone(),
                resource_generation_signature_hash: self.signature_hash,
            };
            self.cache.get_or_create_bind_group(key, || {
                self.device.create_bind_group(&BindGroupDescriptor {
                    label: Some("engine_compiled_flow_bind_group"),
                    layout: self.layout,
                    entries: &entries,
                })
            })
        } else {
            self.device.create_bind_group(&BindGroupDescriptor {
                label: Some("engine_compiled_flow_bind_group_noncached"),
                layout: self.layout,
                entries: &entries,
            })
        };
        *self.output = Some(bind_group);
    }
}

fn gpu_pipeline_layout_for_pass(
    packet: &RendererPreparedPacket,
    flow: &CompiledRenderFlowPlan,
    pass_id: RenderPassId,
    primary_bind_group_layout: &GpuBindGroupLayoutDescriptor,
) -> Result<GpuPipelineLayoutDescriptor> {
    let pass = flow
        .execution
        .passes
        .iter()
        .find(|pass| execution_pass_id(pass) == pass_id)
        .ok_or_else(|| {
            anyhow::anyhow!("pass '{pass_id}' is missing from compiled execution plan")
        })?;

    let mut groups = Vec::new();
    if primary_bind_group_layout.bindings().len() != 0 {
        groups.push(primary_bind_group_layout.clone());
    }
    if let Some(material_group) = gpu_material_bind_group_layout_for_pass(packet, pass)? {
        groups.push(material_group);
    }
    Ok(GpuPipelineLayoutDescriptor::new(groups)?)
}

fn gpu_program_interface_for_layout(
    layout: &GpuPipelineLayoutDescriptor,
) -> Result<GpuProgramInterfaceDescriptor> {
    Ok(GpuProgramInterfaceDescriptor::new(
        layout.groups().flat_map(|group| group.bindings().cloned()),
    )?)
}

fn gpu_pipeline_descriptor_for_pass(
    source: &GpuAdmittedProgramSource,
    pass_kind: FlowPassKind,
    layout: GpuPipelineLayoutDescriptor,
    render_state: Option<GpuRenderPipelineStateDescriptor>,
    specialization: GpuSpecializationValueSet,
) -> Result<FlowPassPipelineDescriptor> {
    let interface = gpu_program_interface_for_layout(&layout)?;
    match pass_kind {
        FlowPassKind::Compute => {
            if render_state.is_some() {
                bail!("compute pipeline descriptor cannot carry render state");
            }
            let entry_point = GpuEntryPointName::new("cs_main")?;
            let program = GpuProgramDescriptor::new(
                source.clone(),
                interface.clone(),
                [GpuEntryPointDescriptor::new(
                    entry_point.clone(),
                    GpuShaderStage::Compute,
                    interface,
                )],
            )?;
            Ok(FlowPassPipelineDescriptor::Compute(
                GpuComputePipelineDescriptor::new(
                    program,
                    entry_point,
                    layout,
                    specialization,
                    GpuCapabilityRequirements::new(),
                )?,
            ))
        }
        FlowPassKind::Fullscreen | FlowPassKind::Graphics => {
            let render_state = render_state.ok_or_else(|| {
                anyhow::anyhow!("render pipeline descriptor requires typed render state")
            })?;
            let vertex = GpuEntryPointName::new("vs_main")?;
            let fragment = GpuEntryPointName::new("fs_main")?;
            let program = GpuProgramDescriptor::new(
                source.clone(),
                interface.clone(),
                [
                    GpuEntryPointDescriptor::new(
                        vertex.clone(),
                        GpuShaderStage::Vertex,
                        interface.clone(),
                    ),
                    GpuEntryPointDescriptor::new(
                        fragment.clone(),
                        GpuShaderStage::Fragment,
                        interface,
                    ),
                ],
            )?;
            Ok(FlowPassPipelineDescriptor::Render(
                GpuRenderPipelineDescriptor::new(
                    program,
                    GpuRenderEntryPoints::new(vertex, Some(fragment)),
                    render_state,
                    layout,
                    specialization,
                    GpuCapabilityRequirements::new(),
                )?,
            ))
        }
        _ => bail!("pass kind '{pass_kind:?}' cannot construct a shader pipeline descriptor"),
    }
}

fn gpu_material_bind_group_layout_for_pass(
    packet: &RendererPreparedPacket,
    pass: &CompiledPassExecutionPlan,
) -> Result<Option<GpuBindGroupLayoutDescriptor>> {
    if !pass_consumes_material_resources(
        execution_pass_feature_id(pass),
        execution_pass_shader_reference(pass),
    ) {
        return Ok(None);
    }
    let Some(material) = packet.prepared_material.as_ref() else {
        return Ok(None);
    };

    gpu_material_bind_group_layout(material)
}

fn gpu_material_bind_group_layout(
    material: &crate::plugins::render::PreparedMaterialFeatureContribution,
) -> Result<Option<GpuBindGroupLayoutDescriptor>> {
    let declarations = gpu_material_binding_declarations(material)?;
    if declarations.is_empty() {
        return Ok(None);
    }

    Ok(Some(GpuBindGroupLayoutDescriptor::new(1, declarations)?))
}

fn gpu_material_binding_declarations(
    material: &crate::plugins::render::PreparedMaterialFeatureContribution,
) -> Result<Vec<GpuBindingDeclaration>> {
    let mut bindings = material
        .instances
        .iter()
        .flat_map(|instance| instance.texture_bindings.iter())
        .collect::<Vec<_>>();
    bindings.sort_by_key(|binding| {
        (
            binding.bind_group,
            binding.texture_binding,
            binding.sampler_binding,
            binding.resource_slot_index,
        )
    });

    let visibility = GpuShaderStages::one(GpuShaderStage::Fragment);
    let mut declarations = Vec::with_capacity(bindings.len() * 2);
    for binding in bindings {
        let view_dimension = match binding.texture_kind {
            crate::plugins::render::PreparedMaterialTextureKind::Texture2D => {
                GpuTextureViewDimension::D2
            }
            crate::plugins::render::PreparedMaterialTextureKind::Texture3D => {
                GpuTextureViewDimension::D3
            }
        };
        declarations.push(GpuBindingDeclaration::new(
            GpuBindingKey::try_new(
                u64::from(binding.bind_group),
                u64::from(binding.texture_binding),
            )?,
            visibility,
            GpuBindingKind::sampled_texture(
                GpuTextureSampleClass::FloatFilterable,
                view_dimension,
                false,
            )?,
            None,
            format!("material-texture-slot-{}", binding.resource_slot_index),
            GpuBindingProvenance::new(
                "render-material-resource-bind-group",
                Some(format!("resource slot {}", binding.resource_slot_index)),
            )?,
        )?);
        declarations.push(GpuBindingDeclaration::new(
            GpuBindingKey::try_new(
                u64::from(binding.bind_group),
                u64::from(binding.sampler_binding),
            )?,
            visibility,
            GpuBindingKind::sampler(GpuSamplerClass::Filtering),
            None,
            format!("material-sampler-slot-{}", binding.resource_slot_index),
            GpuBindingProvenance::new(
                "render-material-resource-bind-group",
                Some(format!("resource slot {}", binding.resource_slot_index)),
            )?,
        )?);
    }

    Ok(declarations)
}

fn gpu_render_pipeline_state_for_pass(
    flow: &CompiledRenderFlowPlan,
    pass_id: RenderPassId,
    color_formats: &[TextureFormat],
    depth_format: Option<TextureFormat>,
) -> Result<Option<GpuRenderPipelineStateDescriptor>> {
    let pass = flow
        .execution
        .passes
        .iter()
        .find(|pass| execution_pass_id(pass) == pass_id)
        .ok_or_else(|| {
            anyhow::anyhow!("pass '{pass_id}' is missing from compiled execution plan")
        })?;
    let vertex_input = gpu_vertex_input_state_for_execution_pass(pass, pass_id)?;

    match pass {
        CompiledPassExecutionPlan::Compute(_) => {
            if !color_formats.is_empty() || depth_format.is_some() {
                bail!(
                    "compute pass '{}' cannot carry render attachment state",
                    pass_id
                );
            }
            Ok(None)
        }
        CompiledPassExecutionPlan::Fullscreen(_) => {
            if depth_format.is_some() {
                bail!("fullscreen pass '{}' cannot carry depth state", pass_id);
            }
            let fragment_output = gpu_fragment_output_state(color_formats, RenderBlendMode::Alpha)?;
            Ok(Some(GpuRenderPipelineStateDescriptor::new(
                vertex_input,
                Some(fragment_output),
                GpuPrimitiveStateDescriptor::default(),
                None,
                GpuMultisampleStateDescriptor::default(),
            )?))
        }
        CompiledPassExecutionPlan::Graphics(plan) => {
            let fragment_output =
                gpu_fragment_output_state(color_formats, plan.raster_state.state.blend_mode)?;
            let primitive = gpu_primitive_state(plan.raster_state.state)?;
            let depth_stencil =
                gpu_depth_stencil_state(depth_format, plan.raster_state.state.depth_policy)?;
            Ok(Some(GpuRenderPipelineStateDescriptor::new(
                vertex_input,
                Some(fragment_output),
                primitive,
                depth_stencil,
                GpuMultisampleStateDescriptor::default(),
            )?))
        }
        _ => bail!(
            "pass '{}' cannot construct render-pipeline state for execution kind '{}'",
            pass_id,
            execution_pass_kind_name(pass)
        ),
    }
}

fn gpu_vertex_input_state_for_execution_pass(
    pass: &CompiledPassExecutionPlan,
    pass_id: RenderPassId,
) -> Result<GpuVertexInputStateDescriptor> {
    let layouts = match pass {
        CompiledPassExecutionPlan::Graphics(plan) => plan
            .draw_buffers
            .vertex_buffers
            .iter()
            .map(|binding| &binding.layout)
            .chain(plan.draw_buffers.instance_buffer_layouts.iter())
            .map(|layout| {
                GpuVertexBufferLayoutDescriptor::new(
                    layout.slot,
                    layout.array_stride,
                    gpu_vertex_step_mode(layout.step_mode),
                    layout.attributes.iter().map(|attribute| {
                        GpuVertexAttribute::new(
                            attribute.shader_location,
                            attribute.offset,
                            gpu_vertex_format(attribute.format),
                        )
                    }),
                )
            })
            .collect::<std::result::Result<Vec<_>, _>>()?,
        CompiledPassExecutionPlan::Compute(_) | CompiledPassExecutionPlan::Fullscreen(_) => {
            Vec::new()
        }
        _ => {
            bail!(
                "pass '{}' cannot construct pipeline vertex-input state for execution kind '{}'",
                pass_id,
                execution_pass_kind_name(pass)
            );
        }
    };

    Ok(GpuVertexInputStateDescriptor::new(layouts)?)
}

fn gpu_fragment_output_state(
    color_formats: &[TextureFormat],
    blend_mode: RenderBlendMode,
) -> Result<GpuFragmentOutputStateDescriptor> {
    let targets = color_formats
        .iter()
        .copied()
        .map(|format| {
            let format = gpu_texture_format_from_wgpu(format)?;
            let blend = if format == GpuTextureFormat::R32Uint
                || matches!(blend_mode, RenderBlendMode::Replace)
            {
                GpuPipelineBlendMode::Replace
            } else {
                GpuPipelineBlendMode::Alpha
            };
            Ok(GpuColorTargetStateDescriptor::new(
                format,
                blend,
                GpuColorWriteMask::ALL,
            )?)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(GpuFragmentOutputStateDescriptor::new(targets))
}

fn gpu_primitive_state(state: RenderRasterState) -> Result<GpuPrimitiveStateDescriptor> {
    let topology = match state.primitive_topology {
        RenderPrimitiveTopology::TriangleList => GpuPipelinePrimitiveTopology::TriangleList,
        RenderPrimitiveTopology::TriangleStrip => GpuPipelinePrimitiveTopology::TriangleStrip,
        RenderPrimitiveTopology::LineList => GpuPipelinePrimitiveTopology::LineList,
        RenderPrimitiveTopology::LineStrip => GpuPipelinePrimitiveTopology::LineStrip,
        RenderPrimitiveTopology::PointList => GpuPipelinePrimitiveTopology::PointList,
    };
    let strip_index_format = topology.is_strip().then_some(GpuIndexFormat::Uint32);
    let cull_mode = match state.cull_mode {
        RenderCullMode::None => GpuPipelineCullMode::None,
        RenderCullMode::Front => GpuPipelineCullMode::Front,
        RenderCullMode::Back => GpuPipelineCullMode::Back,
    };
    Ok(GpuPrimitiveStateDescriptor::new(
        topology,
        strip_index_format,
        GpuFrontFace::CounterClockwise,
        cull_mode,
    )?)
}

fn gpu_depth_stencil_state(
    depth_format: Option<TextureFormat>,
    policy: RenderDepthPolicy,
) -> Result<Option<GpuDepthStencilStateDescriptor>> {
    let Some(format) = depth_format else {
        return Ok(None);
    };
    if matches!(policy, RenderDepthPolicy::Disabled) {
        return Ok(None);
    }
    Ok(Some(GpuDepthStencilStateDescriptor::new(
        gpu_texture_format_from_wgpu(format)?,
        !matches!(policy, RenderDepthPolicy::ReadOnly),
        GpuCompareFunction::LessEqual,
    )?))
}

fn gpu_vertex_step_mode(value: RenderVertexStepMode) -> GpuVertexStepMode {
    match value {
        RenderVertexStepMode::Vertex => GpuVertexStepMode::Vertex,
        RenderVertexStepMode::Instance => GpuVertexStepMode::Instance,
    }
}

fn gpu_vertex_format(value: RenderVertexFormat) -> GpuVertexFormat {
    match value {
        RenderVertexFormat::Float32 => GpuVertexFormat::Float32,
        RenderVertexFormat::Float32x2 => GpuVertexFormat::Float32x2,
        RenderVertexFormat::Float32x3 => GpuVertexFormat::Float32x3,
        RenderVertexFormat::Float32x4 => GpuVertexFormat::Float32x4,
        RenderVertexFormat::Uint32 => GpuVertexFormat::Uint32,
        RenderVertexFormat::Uint32x2 => GpuVertexFormat::Uint32x2,
        RenderVertexFormat::Uint32x3 => GpuVertexFormat::Uint32x3,
        RenderVertexFormat::Uint32x4 => GpuVertexFormat::Uint32x4,
        RenderVertexFormat::Sint32 => GpuVertexFormat::Sint32,
        RenderVertexFormat::Sint32x2 => GpuVertexFormat::Sint32x2,
        RenderVertexFormat::Sint32x3 => GpuVertexFormat::Sint32x3,
        RenderVertexFormat::Sint32x4 => GpuVertexFormat::Sint32x4,
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
            view_dimension: wgpu_texture_view_dimension(kind.texture_view_dimension().ok_or_else(
                || {
                    anyhow::anyhow!(
                        "sampled-texture binding is missing its normalized view dimension"
                    )
                },
            )?),
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
            view_dimension: wgpu_texture_view_dimension(kind.texture_view_dimension().ok_or_else(
                || {
                    anyhow::anyhow!(
                        "storage-texture binding is missing its normalized view dimension"
                    )
                },
            )?),
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
        TextureFormat::R8Unorm => Ok(GpuTextureFormat::R8Unorm),
        TextureFormat::Rgba8Unorm => Ok(GpuTextureFormat::Rgba8Unorm),
        TextureFormat::Rgba8UnormSrgb => Ok(GpuTextureFormat::Rgba8UnormSrgb),
        TextureFormat::Bgra8Unorm => Ok(GpuTextureFormat::Bgra8Unorm),
        TextureFormat::Bgra8UnormSrgb => Ok(GpuTextureFormat::Bgra8UnormSrgb),
        TextureFormat::R32Uint => Ok(GpuTextureFormat::R32Uint),
        TextureFormat::Depth32Float => Ok(GpuTextureFormat::Depth32Float),
        unsupported => bail!(
            "current render texture format {unsupported:?} has no accepted G4B normalized format"
        ),
    }
}

fn wgpu_texture_format(format: GpuTextureFormat) -> TextureFormat {
    match format {
        GpuTextureFormat::R8Unorm => TextureFormat::R8Unorm,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{
        PreparedMaterialBindingTable, PreparedMaterialFeatureContribution,
        PreparedMaterialInstanceInput, PreparedMaterialParameterPayloadV1,
        PreparedMaterialTextureBinding, PreparedMaterialTextureBindingLocation,
        PreparedMaterialTextureKind,
    };

    fn material_with_bindings(
        texture_bindings: Vec<PreparedMaterialTextureBinding>,
    ) -> PreparedMaterialFeatureContribution {
        PreparedMaterialFeatureContribution {
            instances: vec![PreparedMaterialInstanceInput {
                material_instance_id: "material.product.1".to_string(),
                specialization_key_fragment: "material.first_slice".to_string(),
                parameter_payload: PreparedMaterialParameterPayloadV1::default(),
                texture_bindings,
            }],
            binding_table: PreparedMaterialBindingTable::default(),
            scene_bundle: None,
            model_mesh_material_selections: Vec::new(),
        }
    }

    fn texture_binding(
        resource_slot_index: u32,
        bind_group: u32,
        texture_binding: u32,
        sampler_binding: u32,
    ) -> PreparedMaterialTextureBinding {
        PreparedMaterialTextureBinding::new(
            resource_slot_index as u64 + 1,
            format!("texture_{resource_slot_index}"),
            PreparedMaterialTextureBindingLocation::new(
                resource_slot_index,
                bind_group,
                texture_binding,
                sampler_binding,
            ),
            format!("artifact.{resource_slot_index}"),
            ".runenwerk/artifacts/texture.ktx2",
            PreparedMaterialTextureKind::Texture2D,
            "texture-cache",
        )
    }

    #[test]
    fn material_group_one_declarations_use_transported_compiler_coordinates() {
        let material = material_with_bindings(vec![texture_binding(91, 1, 31, 47)]);

        let declarations =
            gpu_material_binding_declarations(&material).expect("typed declarations should form");
        let keys = declarations
            .iter()
            .map(|declaration| declaration.key())
            .collect::<Vec<_>>();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].group(), 1);
        assert_eq!(keys[0].binding(), 31);
        assert_eq!(keys[1].group(), 1);
        assert_eq!(keys[1].binding(), 47);

        let layout = gpu_material_bind_group_layout(&material)
            .expect("typed group-one layout should form")
            .expect("one material binding should publish a layout");
        assert_eq!(layout.group(), 1);
        assert!(layout.binding(31).is_some());
        assert!(layout.binding(47).is_some());
        assert!(layout.binding(182).is_none());
    }

    #[test]
    fn material_group_one_layout_rejects_invalid_or_duplicate_transported_keys() {
        let invalid_group = material_with_bindings(vec![texture_binding(0, 2, 31, 47)]);
        let error = gpu_material_bind_group_layout(&invalid_group).expect_err(
            "material group-one layout must reject a non-group-one compiler coordinate",
        );
        assert!(error.to_string().contains("exact group"));

        let duplicate = material_with_bindings(vec![
            texture_binding(0, 1, 31, 47),
            texture_binding(1, 1, 31, 47),
        ]);
        let error = gpu_material_bind_group_layout(&duplicate)
            .expect_err("material group-one layout must reject duplicate final GPU keys");
        assert!(error.to_string().contains("DuplicateBindingKey"));
    }
}
