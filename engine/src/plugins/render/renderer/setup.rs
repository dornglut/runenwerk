use super::*;

impl Renderer {
    // Owner: Engine Renderer - UI Pipeline Setup and Encoding
    pub fn new() -> Self {
        Self {
            rect_pass: None,
            rect_pass_format: None,
            rect_pass_shader_revision: 0,
            glyph_pass: None,
            glyph_pass_format: None,
            viewport_embed_pass: None,
            viewport_embed_pass_format: None,
            glyph_atlas_gpu: std::collections::BTreeMap::new(),
            flow_runtime_cache: std::collections::BTreeMap::new(),
            flow_pipeline_cache: super::pipeline_cache::FlowPipelineArtifactCache::default(),
            last_good_ui_prepared: None,
            last_pass_timings: Vec::new(),
            last_runtime_resources: Vec::new(),
            last_pass_provenance: Vec::new(),
            last_capture_plan: crate::plugins::render::inspect::ResolvedRenderCapturePlan::default(
            ),
            last_capture_selector_results: Vec::new(),
            last_captured_textures: Vec::new(),
        }
    }

    pub fn last_pass_timings(&self) -> &[crate::plugins::render::inspect::PassTimingSample] {
        &self.last_pass_timings
    }

    pub fn last_runtime_resources(
        &self,
    ) -> &[crate::plugins::render::inspect::RuntimeResourceInspectionEntry] {
        &self.last_runtime_resources
    }

    pub fn last_pass_provenance(
        &self,
    ) -> &[crate::plugins::render::inspect::RenderPassProvenanceRecord] {
        &self.last_pass_provenance
    }

    pub fn last_capture_plan(&self) -> &crate::plugins::render::inspect::ResolvedRenderCapturePlan {
        &self.last_capture_plan
    }

    pub fn last_capture_selector_results(
        &self,
    ) -> &[crate::plugins::render::inspect::RenderCaptureSelectorResult] {
        &self.last_capture_selector_results
    }

    pub fn last_captured_textures(
        &self,
    ) -> &[crate::plugins::render::inspect::RenderCapturedTexture] {
        &self.last_captured_textures
    }

    pub fn flow_pipeline_cache_stats(&self) -> super::pipeline_cache::RendererPipelineCacheStats {
        self.flow_pipeline_cache.stats()
    }

    pub(super) fn ensure_rect_pass(
        &mut self,
        device: &Device,
        format: TextureFormat,
        shader_source: &str,
        shader_revision: u64,
    ) {
        if self.rect_pass.is_some()
            && self.rect_pass_format == Some(format)
            && self.rect_pass_shader_revision == shader_revision
        {
            return;
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_ui_rect_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });

        let screen_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_ui_screen_uniform"),
            size: std::mem::size_of::<ScreenUniformRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_ui_rect_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let screen_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_ui_rect_bind_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_ui_rect_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_ui_rect_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<RectInstanceRaw>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32,
                            offset: 32,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        self.rect_pass = Some(RectPass {
            pipeline,
            screen_buffer,
            screen_bind_group,
        });
        self.rect_pass_format = Some(format);
        self.rect_pass_shader_revision = shader_revision;
    }

    pub(super) fn ensure_glyph_pass(&mut self, device: &Device, format: TextureFormat) {
        if self.glyph_pass.is_some() && self.glyph_pass_format == Some(format) {
            return;
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_ui_glyph_shader"),
            source: ShaderSource::Wgsl(DEFAULT_UI_GLYPH_SHADER.into()),
        });

        let screen_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_ui_glyph_screen_uniform"),
            size: std::mem::size_of::<ScreenUniformRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let screen_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_ui_glyph_screen_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let screen_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_ui_glyph_screen_bind_group"),
            layout: &screen_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_ui_glyph_texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("engine_ui_glyph_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_ui_glyph_pipeline_layout"),
            bind_group_layouts: &[&screen_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_ui_glyph_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<GlyphInstanceRaw>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format,
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

        self.glyph_pass = Some(GlyphPass {
            pipeline,
            screen_buffer,
            screen_bind_group,
            texture_bind_group_layout,
            texture_sampler,
        });
        self.glyph_pass_format = Some(format);
        self.glyph_atlas_gpu.clear();
    }

    pub(super) fn ensure_viewport_embed_pass(
        &mut self,
        device: &Device,
        format: TextureFormat,
    ) {
        if self.viewport_embed_pass.is_some() && self.viewport_embed_pass_format == Some(format) {
            return;
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_ui_viewport_embed_shader"),
            source: ShaderSource::Wgsl(DEFAULT_UI_VIEWPORT_EMBED_SHADER.into()),
        });

        let screen_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_ui_viewport_embed_screen_uniform"),
            size: std::mem::size_of::<ScreenUniformRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let screen_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_ui_viewport_embed_screen_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let screen_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_ui_viewport_embed_screen_bind_group"),
            layout: &screen_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_ui_viewport_embed_texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("engine_ui_viewport_embed_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_ui_viewport_embed_pipeline_layout"),
            bind_group_layouts: &[&screen_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_ui_viewport_embed_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<ViewportEmbedInstanceRaw>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format,
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

        self.viewport_embed_pass = Some(ViewportEmbedPass {
            pipeline,
            screen_buffer,
            screen_bind_group,
            texture_bind_group_layout,
            texture_sampler,
        });
        self.viewport_embed_pass_format = Some(format);
    }

    pub(super) fn ensure_glyph_atlas_gpu(
        &mut self,
        device: &Device,
        queue: &Queue,
        atlas: &crate::plugins::render::features::UiFontAtlasResource,
        texture_id: u64,
    ) -> Option<()> {
        if self.glyph_atlas_gpu.contains_key(&texture_id) {
            return Some(());
        }

        let glyph_pass = self.glyph_pass.as_ref()?;
        let (_, atlas_image) = atlas.atlas_for_texture_id(texture_id)?;
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("engine_ui_glyph_atlas_texture"),
            size: Extent3d {
                width: atlas_image.width.max(1),
                height: atlas_image.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &atlas_image.pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_image.width.max(1)),
                rows_per_image: Some(atlas_image.height.max(1)),
            },
            Extent3d {
                width: atlas_image.width.max(1),
                height: atlas_image.height.max(1),
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_ui_glyph_atlas_bind_group"),
            layout: &glyph_pass.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&glyph_pass.texture_sampler),
                },
            ],
        });

        self.glyph_atlas_gpu.insert(
            texture_id,
            UiGlyphAtlasGpu {
                _texture: texture,
                _view: view,
                bind_group,
            },
        );
        Some(())
    }

    pub(super) fn full_scissor(surface_width: u32, surface_height: u32) -> (u32, u32, u32, u32) {
        (0, 0, surface_width.max(1), surface_height.max(1))
    }

    pub(super) fn clip_to_scissor(
        clip: [f32; 4],
        surface_width: u32,
        surface_height: u32,
    ) -> Option<(u32, u32, u32, u32)> {
        let max_x = surface_width.max(1) as i32;
        let max_y = surface_height.max(1) as i32;

        let x0 = (clip[0].floor() as i32).clamp(0, max_x);
        let y0 = (clip[1].floor() as i32).clamp(0, max_y);
        let x1 = ((clip[0] + clip[2]).ceil() as i32).clamp(0, max_x);
        let y1 = ((clip[1] + clip[3]).ceil() as i32).clamp(0, max_y);

        if x1 <= x0 || y1 <= y0 {
            return None;
        }

        Some((x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32))
    }

    pub(super) fn encode_ui_pass(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_texture: &Texture,
        frame_view: &TextureView,
        prepared: &UiPreparedDraws,
        flow_id: &str,
        runtime_resources: &super::render_flow::FlowRuntimeResources,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
        surface_size: (u32, u32),
        surface_format: TextureFormat,
    ) {
        let Some(rect_pass) = self.rect_pass.as_ref() else {
            return;
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_ui_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if !prepared.rect_batches.is_empty() {
            pass.set_pipeline(&rect_pass.pipeline);
            pass.set_bind_group(0, &rect_pass.screen_bind_group, &[]);
            for batch in &prepared.rect_batches {
                pass.set_scissor_rect(
                    batch.scissor.0,
                    batch.scissor.1,
                    batch.scissor.2,
                    batch.scissor.3,
                );
                pass.set_vertex_buffer(0, batch.instance_buffer.slice(..));
                pass.draw(0..6, 0..batch.instance_count);
            }
        }

        if let Some(viewport_embed_pass) = self.viewport_embed_pass.as_ref() {
            let mut viewport_bind_groups = std::collections::BTreeMap::<String, BindGroup>::new();
            pass.set_pipeline(&viewport_embed_pass.pipeline);
            pass.set_bind_group(0, &viewport_embed_pass.screen_bind_group, &[]);
            for batch in &prepared.viewport_embed_batches {
                let Some(binding) = viewport_surface_bindings.get(batch.viewport_id, batch.slot)
                else {
                    continue;
                };
                if binding.flow_id.as_str() != flow_id {
                    continue;
                }

                if !viewport_bind_groups.contains_key(binding.resource_id.as_str()) {
                    let Ok(view) = runtime_resources.resolve_ui_texture_view(
                        "builtin_ui_viewport_embed",
                        binding.resource_id.as_str(),
                        frame_texture,
                        surface_size,
                        surface_format,
                    ) else {
                        continue;
                    };
                    let bind_group = device.create_bind_group(&BindGroupDescriptor {
                        label: Some("engine_ui_viewport_embed_bind_group"),
                        layout: &viewport_embed_pass.texture_bind_group_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::TextureView(&view),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::Sampler(
                                    &viewport_embed_pass.texture_sampler,
                                ),
                            },
                        ],
                    });
                    viewport_bind_groups.insert(binding.resource_id.clone(), bind_group);
                }

                let Some(bind_group) = viewport_bind_groups.get(binding.resource_id.as_str())
                else {
                    continue;
                };
                pass.set_bind_group(1, bind_group, &[]);
                pass.set_scissor_rect(
                    batch.scissor.0,
                    batch.scissor.1,
                    batch.scissor.2,
                    batch.scissor.3,
                );
                pass.set_vertex_buffer(0, batch.instance_buffer.slice(..));
                pass.draw(0..6, 0..batch.instance_count);
            }
        }

        if let Some(glyph_pass) = self.glyph_pass.as_ref() {
            pass.set_pipeline(&glyph_pass.pipeline);
            pass.set_bind_group(0, &glyph_pass.screen_bind_group, &[]);
            for batch in &prepared.glyph_batches {
                let Some(atlas_gpu) = self.glyph_atlas_gpu.get(&batch.texture_id) else {
                    continue;
                };
                pass.set_bind_group(1, &atlas_gpu.bind_group, &[]);
                pass.set_scissor_rect(
                    batch.scissor.0,
                    batch.scissor.1,
                    batch.scissor.2,
                    batch.scissor.3,
                );
                pass.set_vertex_buffer(0, batch.instance_buffer.slice(..));
                pass.draw(0..6, 0..batch.instance_count);
            }
        }
    }
}
