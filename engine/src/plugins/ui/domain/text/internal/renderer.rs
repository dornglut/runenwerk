// Owner: Engine UI Text - Renderer Pipeline and Draw Encoding
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlyphInstanceRaw {
    rect: [f32; 4],
    uv: [f32; 4],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ScreenUniformRaw {
    size: [f32; 2],
    _pad: [f32; 2],
}

#[derive(Debug)]
pub struct TextRenderer {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
    atlas_bind_group: BindGroup,
    sampling: TextSampling,
    glyphs: HashMap<char, GlyphMetrics>,
    base_size: f32,
    line_height: f32,
    ascent: f32,
}

impl TextRenderer {
    pub fn new(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        provider: &dyn FontAtlasProvider,
    ) -> Self {
        let loaded = provider.load_default_font();

        let atlas_texture = device.create_texture(&TextureDescriptor {
            label: Some("engine_text_atlas"),
            size: Extent3d {
                width: loaded.width.max(1),
                height: loaded.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let atlas_view = atlas_texture.create_view(&TextureViewDescriptor::default());
        let atlas_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("engine_text_atlas_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let screen_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_text_screen_uniform"),
            size: std::mem::size_of::<ScreenUniformRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let screen_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_text_screen_bgl"),
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
        let atlas_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_text_atlas_bgl"),
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

        let screen_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_text_screen_bg"),
            layout: &screen_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });
        let atlas_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_text_atlas_bg"),
            layout: &atlas_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&atlas_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_ui_text_shader"),
            source: ShaderSource::Wgsl(
                match loaded.sampling {
                    TextSampling::Msdf => UI_TEXT_SHADER_MSDF,
                    TextSampling::Alpha => UI_TEXT_SHADER_ALPHA,
                }
                .into(),
            ),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_ui_text_pipeline_layout"),
            bind_group_layouts: &[&screen_bind_group_layout, &atlas_bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_ui_text_pipeline"),
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

        let bytes_per_row = loaded.width.max(1) * 4;
        let rows_per_image = loaded.height.max(1);
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &loaded.rgba8_pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(rows_per_image),
            },
            Extent3d {
                width: loaded.width.max(1),
                height: loaded.height.max(1),
                depth_or_array_layers: 1,
            },
        );

        Self {
            pipeline,
            screen_buffer,
            screen_bind_group,
            atlas_bind_group,
            sampling: loaded.sampling,
            glyphs: loaded.glyphs,
            base_size: loaded.base_size,
            line_height: loaded.line_height,
            ascent: loaded.ascent,
        }
    }

    pub fn write_screen_uniform(&self, queue: &Queue, surface_width: f32, surface_height: f32) {
        let screen = ScreenUniformRaw {
            size: [surface_width.max(1.0), surface_height.max(1.0)],
            _pad: [0.0; 2],
        };
        queue.write_buffer(&self.screen_buffer, 0, bytemuck::bytes_of(&screen));
    }

    pub fn build_instance_buffer(
        &self,
        device: &Device,
        draw_list: &UiDrawList,
    ) -> Option<(Buffer, u32)> {
        let instances = build_glyph_instances(
            draw_list,
            &self.glyphs,
            self.base_size,
            self.line_height,
            self.ascent,
            self.sampling,
        );
        if instances.is_empty() {
            return None;
        }

        let buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("engine_ui_text_instances"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX,
        });
        Some((buffer, instances.len() as u32))
    }

    pub fn encode_draw<'a>(
        &'a self,
        pass: &mut RenderPass<'a>,
        instance_buffer: &'a Buffer,
        count: u32,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.screen_bind_group, &[]);
        pass.set_bind_group(1, &self.atlas_bind_group, &[]);
        pass.set_vertex_buffer(0, instance_buffer.slice(..));
        pass.draw(0..6, 0..count);
    }
}
