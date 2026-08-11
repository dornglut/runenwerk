use super::resource_descriptors::{
    buffer_descriptor, linear_sampler_descriptor, texture_descriptor, whole_texture_view_descriptor,
};
use super::*;
use crate::plugins::gpu::{
    CurrentRenderBufferBindingTerminal, CurrentRenderSampledTextureBindingTerminal,
    CurrentRenderTextureUploadTerminal, CurrentRenderTimestampWritesTerminal,
    CurrentRenderVertexBufferTerminal, GpuBufferUsage, GpuMemoryIntent, GpuResourceLifetime,
    GpuTextureFormat, GpuTextureUsage,
};

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub(super) fn realize_buffer_resource(
        &mut self,
        context: &GpuContext,
        label: &str,
        size: u64,
        usages: impl IntoIterator<Item = GpuBufferUsage>,
        lifetime: GpuResourceLifetime,
        memory_intent: GpuMemoryIntent,
    ) -> Result<RendererBufferResource> {
        let handle = self.resource_ids.allocate_buffer_handle(buffer_descriptor(
            label,
            size,
            usages,
            lifetime,
            memory_intent,
        )?)?;
        let realized = context.realize_buffer(&handle)?;
        Ok(RendererBufferResource {
            _handle: handle,
            realized,
        })
    }

    fn realize_linear_sampler_resource(
        &mut self,
        context: &GpuContext,
        label: &str,
        lifetime: GpuResourceLifetime,
    ) -> Result<RendererSamplerResource> {
        let handle = self
            .resource_ids
            .allocate_sampler_handle(linear_sampler_descriptor(label, lifetime)?)?;
        let realized = context.realize_sampler(&handle)?;
        Ok(RendererSamplerResource {
            _handle: handle,
            realized,
        })
    }

    // Owner: Engine Renderer - UI Pipeline Setup and Encoding
    pub fn new() -> Self {
        Self {
            resource_ids: GpuWorkResourceIdAllocator::new(),
            rect_pass: None,
            rect_pass_format: None,
            rect_pass_shader_revision: 0,
            stroke_pass: None,
            stroke_pass_format: None,
            glyph_pass: None,
            glyph_pass_format: None,
            viewport_embed_pass: None,
            viewport_embed_pass_format: None,
            product_surface_pass: None,
            product_surface_pass_format: None,
            glyph_atlas_gpu: std::collections::BTreeMap::new(),
            dynamic_texture_targets:
                super::dynamic_targets::RendererDynamicTextureTargetCache::default(),
            flow_runtime_cache: std::collections::BTreeMap::new(),
            flow_pipeline_cache: super::pipeline_cache::FlowPipelineArtifactCache::default(),
            preflight_cache: None,
            last_good_ui_prepared: None,
            last_pass_timings: Vec::new(),
            last_gpu_pass_timing_evidence: Vec::new(),
            last_runtime_resources: Vec::new(),
            last_pass_provenance: Vec::new(),
            last_preflight_report:
                crate::plugins::render::graph::RenderExecutionGraphPreparedReport::default(),
            last_preflight_cache_state:
                crate::plugins::render::graph::RenderPreparedFramePreflightCacheState::default(),
            last_capture_plan: crate::plugins::render::inspect::ResolvedRenderCapturePlan::default(
            ),
            last_capture_selector_results: Vec::new(),
            last_captured_textures: Vec::new(),
        }
    }

    pub fn last_pass_timings(&self) -> &[crate::plugins::render::inspect::PassTimingSample] {
        &self.last_pass_timings
    }

    pub fn last_gpu_pass_timing_evidence(
        &self,
    ) -> &[crate::plugins::render::inspect::RenderPassTimingEvidence] {
        &self.last_gpu_pass_timing_evidence
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

    pub fn last_preflight_report(
        &self,
    ) -> &crate::plugins::render::graph::RenderExecutionGraphPreparedReport {
        &self.last_preflight_report
    }

    pub fn last_preflight_cache_state(
        &self,
    ) -> &crate::plugins::render::graph::RenderPreparedFramePreflightCacheState {
        &self.last_preflight_cache_state
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
        context: &GpuContext,
        device: &Device,
        format: TextureFormat,
        shader_source: &str,
        shader_revision: u64,
    ) -> Result<()> {
        if self.rect_pass.is_some()
            && self.rect_pass_format == Some(format)
            && self.rect_pass_shader_revision == shader_revision
        {
            return Ok(());
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_ui_rect_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });

        let screen_buffer = self.realize_buffer_resource(
            context,
            "engine_ui_screen_uniform",
            std::mem::size_of::<ScreenUniformRaw>() as u64,
            [GpuBufferUsage::Uniform, GpuBufferUsage::CopyDestination],
            GpuResourceLifetime::Retained,
            GpuMemoryIntent::Device,
        )?;

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

        let screen_bind_group = create_single_buffer_bind_group(
            context,
            device,
            "engine_ui_rect_bind_group",
            &bind_group_layout,
            &screen_buffer.realized,
        )?;

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
                        VertexAttribute {
                            format: VertexFormat::Float32,
                            offset: 36,
                            shader_location: 3,
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
        Ok(())
    }

    pub(super) fn ensure_stroke_pass(
        &mut self,
        context: &GpuContext,
        device: &Device,
        format: TextureFormat,
    ) -> Result<()> {
        if self.stroke_pass.is_some() && self.stroke_pass_format == Some(format) {
            return Ok(());
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_ui_stroke_shader"),
            source: ShaderSource::Wgsl(DEFAULT_UI_STROKE_SHADER.into()),
        });

        let screen_buffer = self.realize_buffer_resource(
            context,
            "engine_ui_stroke_screen_uniform",
            std::mem::size_of::<ScreenUniformRaw>() as u64,
            [GpuBufferUsage::Uniform, GpuBufferUsage::CopyDestination],
            GpuResourceLifetime::Retained,
            GpuMemoryIntent::Device,
        )?;

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_ui_stroke_bind_group_layout"),
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

        let screen_bind_group = create_single_buffer_bind_group(
            context,
            device,
            "engine_ui_stroke_bind_group",
            &bind_group_layout,
            &screen_buffer.realized,
        )?;

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_ui_stroke_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_ui_stroke_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<StrokeSegmentInstanceRaw>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32,
                            offset: 32,
                            shader_location: 3,
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

        self.stroke_pass = Some(StrokePass {
            pipeline,
            screen_buffer,
            screen_bind_group,
        });
        self.stroke_pass_format = Some(format);
        Ok(())
    }

    pub(super) fn ensure_glyph_pass(
        &mut self,
        context: &GpuContext,
        device: &Device,
        format: TextureFormat,
    ) -> Result<()> {
        if self.glyph_pass.is_some() && self.glyph_pass_format == Some(format) {
            return Ok(());
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_ui_glyph_shader"),
            source: ShaderSource::Wgsl(DEFAULT_UI_GLYPH_SHADER.into()),
        });

        let screen_buffer = self.realize_buffer_resource(
            context,
            "engine_ui_glyph_screen_uniform",
            std::mem::size_of::<ScreenUniformRaw>() as u64,
            [GpuBufferUsage::Uniform, GpuBufferUsage::CopyDestination],
            GpuResourceLifetime::Retained,
            GpuMemoryIntent::Device,
        )?;

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

        let screen_bind_group = create_single_buffer_bind_group(
            context,
            device,
            "engine_ui_glyph_screen_bind_group",
            &screen_bind_group_layout,
            &screen_buffer.realized,
        )?;

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

        let texture_sampler = self.realize_linear_sampler_resource(
            context,
            "engine_ui_glyph_sampler",
            GpuResourceLifetime::Retained,
        )?;

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
        Ok(())
    }

    pub(super) fn ensure_viewport_embed_pass(
        &mut self,
        context: &GpuContext,
        device: &Device,
        format: TextureFormat,
    ) -> Result<()> {
        if self.viewport_embed_pass.is_some() && self.viewport_embed_pass_format == Some(format) {
            return Ok(());
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_ui_viewport_embed_shader"),
            source: ShaderSource::Wgsl(DEFAULT_UI_VIEWPORT_EMBED_SHADER.into()),
        });

        let screen_buffer = self.realize_buffer_resource(
            context,
            "engine_ui_viewport_embed_screen_uniform",
            std::mem::size_of::<ScreenUniformRaw>() as u64,
            [GpuBufferUsage::Uniform, GpuBufferUsage::CopyDestination],
            GpuResourceLifetime::Retained,
            GpuMemoryIntent::Device,
        )?;

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

        let screen_bind_group = create_single_buffer_bind_group(
            context,
            device,
            "engine_ui_viewport_embed_screen_bind_group",
            &screen_bind_group_layout,
            &screen_buffer.realized,
        )?;

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

        let texture_sampler = self.realize_linear_sampler_resource(
            context,
            "engine_ui_viewport_embed_sampler",
            GpuResourceLifetime::Retained,
        )?;

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
        Ok(())
    }

    pub(super) fn ensure_product_surface_pass(
        &mut self,
        context: &GpuContext,
        device: &Device,
        format: TextureFormat,
    ) -> Result<()> {
        if self.product_surface_pass.is_some() && self.product_surface_pass_format == Some(format) {
            return Ok(());
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_ui_product_surface_shader"),
            source: ShaderSource::Wgsl(DEFAULT_UI_PRODUCT_SURFACE_SHADER.into()),
        });

        let screen_buffer = self.realize_buffer_resource(
            context,
            "engine_ui_product_surface_screen_uniform",
            std::mem::size_of::<ScreenUniformRaw>() as u64,
            [GpuBufferUsage::Uniform, GpuBufferUsage::CopyDestination],
            GpuResourceLifetime::Retained,
            GpuMemoryIntent::Device,
        )?;

        let screen_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_ui_product_surface_screen_bind_group_layout"),
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

        let screen_bind_group = create_single_buffer_bind_group(
            context,
            device,
            "engine_ui_product_surface_screen_bind_group",
            &screen_bind_group_layout,
            &screen_buffer.realized,
        )?;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_ui_product_surface_texture_bind_group_layout"),
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

        let texture_sampler = self.realize_linear_sampler_resource(
            context,
            "engine_ui_product_surface_sampler",
            GpuResourceLifetime::Retained,
        )?;

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_ui_product_surface_pipeline_layout"),
            bind_group_layouts: &[&screen_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_ui_product_surface_pipeline"),
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

        self.product_surface_pass = Some(ProductSurfacePass {
            pipeline,
            screen_buffer,
            screen_bind_group,
            texture_bind_group_layout,
            texture_sampler,
        });
        self.product_surface_pass_format = Some(format);
        Ok(())
    }

    pub(super) fn ensure_glyph_atlas_gpu(
        &mut self,
        context: &GpuContext,
        device: &Device,
        queue: &Queue,
        atlas: &crate::plugins::render::features::UiFontAtlasResource,
        texture_id: u64,
    ) -> Result<bool> {
        if self.glyph_atlas_gpu.contains_key(&texture_id) {
            return Ok(true);
        }

        let Some(glyph_pass) = self.glyph_pass.as_ref() else {
            return Ok(false);
        };
        let texture_bind_group_layout = glyph_pass.texture_bind_group_layout.clone();
        let texture_sampler = glyph_pass.texture_sampler.clone();
        let Some((_, atlas_image)) = atlas.atlas_for_texture_id(texture_id) else {
            return Ok(false);
        };
        let texture_handle = self
            .resource_ids
            .allocate_texture_handle(texture_descriptor(
                "engine_ui_glyph_atlas_texture",
                (atlas_image.width.max(1), atlas_image.height.max(1)),
                GpuTextureFormat::R8Unorm,
                [GpuTextureUsage::Sampled, GpuTextureUsage::CopyDestination],
                GpuResourceLifetime::Retained,
            )?)?;
        let texture = RendererTextureResource {
            realized: context.realize_texture(&texture_handle)?,
            _handle: texture_handle,
        };
        context
            .current_render_resource_bridge()
            .for_texture_upload(
                &texture.realized,
                UploadGlyphAtlasTexture { queue, atlas_image },
            )?;

        let view_handle =
            self.resource_ids
                .allocate_texture_view_handle(whole_texture_view_descriptor(
                    "engine_ui_glyph_atlas_view",
                    &texture._handle,
                )?)?;
        let view = RendererTextureViewResource {
            realized: context.realize_texture_view(&view_handle, &texture.realized)?,
            _handle: view_handle,
        };
        let bind_group = create_sampled_texture_bind_group(
            context,
            device,
            "engine_ui_glyph_atlas_bind_group",
            &texture_bind_group_layout,
            &view.realized,
            &texture_sampler.realized,
        )?;

        self.glyph_atlas_gpu.insert(
            texture_id,
            UiGlyphAtlasGpu {
                _texture: texture,
                _view: view,
                bind_group,
            },
        );
        Ok(true)
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

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_ui_pass(
        &self,
        context: &GpuContext,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        prepared: &UiPreparedDraws,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
        gpu_timestamp_writes: Option<super::render_flow::GpuPassTimestampWrites>,
    ) -> Result<()> {
        if let Some(writes) = gpu_timestamp_writes {
            let mut output = Ok(());
            context
                .current_render_resource_bridge()
                .for_timestamp_writes(
                    &writes.query_set,
                    EncodeTimestampedUiPass {
                        renderer: self,
                        context,
                        device,
                        encoder,
                        frame_view,
                        prepared,
                        viewport_surface_bindings,
                        indices: writes.indices,
                        output: &mut output,
                    },
                )?;
            output
        } else {
            self.encode_ui_pass_current(
                context,
                device,
                encoder,
                frame_view,
                prepared,
                viewport_surface_bindings,
                None,
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_ui_pass_current(
        &self,
        context: &GpuContext,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        prepared: &UiPreparedDraws,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
        timestamp: Option<(&QuerySet, super::render_flow::GpuPassTimestampIndices)>,
    ) -> Result<()> {
        let Some(rect_pass) = self.rect_pass.as_ref() else {
            return Ok(());
        };

        let timestamp_writes = timestamp.map(|(query_set, indices)| RenderPassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(indices.begin),
            end_of_pass_write_index: Some(indices.end),
        });
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
            timestamp_writes,
            occlusion_query_set: None,
        });

        let mut viewport_bind_groups =
            std::collections::BTreeMap::<ViewportSurfaceBindingSource, BindGroup>::new();
        let mut product_surface_bind_groups =
            std::collections::BTreeMap::<ProductSurfaceTextureBindingSource, BindGroup>::new();
        for command in &prepared.draw_plan {
            match *command {
                UiPreparedDrawCommand::Rect(index) => {
                    let Some(batch) = prepared.rect_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(&rect_pass.pipeline);
                    pass.set_bind_group(0, &rect_pass.screen_bind_group, &[]);
                    pass.set_scissor_rect(
                        batch.scissor.0,
                        batch.scissor.1,
                        batch.scissor.2,
                        batch.scissor.3,
                    );
                    context.current_render_resource_bridge().for_vertex_buffer(
                        &batch.instance_buffer.realized,
                        DrawUiInstances {
                            pass: &mut pass,
                            instance_count: batch.instance_count,
                        },
                    )?;
                }
                UiPreparedDrawCommand::Stroke(index) => {
                    let Some(stroke_pass) = self.stroke_pass.as_ref() else {
                        continue;
                    };
                    let Some(batch) = prepared.stroke_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(&stroke_pass.pipeline);
                    pass.set_bind_group(0, &stroke_pass.screen_bind_group, &[]);
                    pass.set_scissor_rect(
                        batch.scissor.0,
                        batch.scissor.1,
                        batch.scissor.2,
                        batch.scissor.3,
                    );
                    context.current_render_resource_bridge().for_vertex_buffer(
                        &batch.instance_buffer.realized,
                        DrawUiInstances {
                            pass: &mut pass,
                            instance_count: batch.instance_count,
                        },
                    )?;
                }
                UiPreparedDrawCommand::ViewportEmbed(index) => {
                    let Some(viewport_embed_pass) = self.viewport_embed_pass.as_ref() else {
                        continue;
                    };
                    let Some(batch) = prepared.viewport_embed_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(&viewport_embed_pass.pipeline);
                    pass.set_bind_group(0, &viewport_embed_pass.screen_bind_group, &[]);
                    let Some(binding) =
                        viewport_surface_bindings.get(batch.viewport_id, batch.slot)
                    else {
                        continue;
                    };

                    if !viewport_bind_groups.contains_key(&binding.source) {
                        let ViewportSurfaceBindingSource::DynamicTexture {
                            namespace,
                            target_id,
                        } = &binding.source;
                        let key = crate::plugins::render::RenderDynamicTextureTargetKey::new(
                            namespace.clone(),
                            target_id.clone(),
                        );
                        let Ok(view) = self.dynamic_texture_targets.ui_texture_view(&key) else {
                            continue;
                        };
                        context
                            .current_render_resource_bridge()
                            .for_sampled_texture_binding(
                                &view,
                                &viewport_embed_pass.texture_sampler.realized,
                                CreateUiTextureBindGroup {
                                    device,
                                    label: "engine_ui_viewport_embed_bind_group",
                                    layout: &viewport_embed_pass.texture_bind_group_layout,
                                    key: binding.source.clone(),
                                    destination: &mut viewport_bind_groups,
                                },
                            )?;
                    }

                    let Some(bind_group) = viewport_bind_groups.get(&binding.source) else {
                        continue;
                    };
                    pass.set_bind_group(1, bind_group, &[]);
                    pass.set_scissor_rect(
                        batch.scissor.0,
                        batch.scissor.1,
                        batch.scissor.2,
                        batch.scissor.3,
                    );
                    context.current_render_resource_bridge().for_vertex_buffer(
                        &batch.instance_buffer.realized,
                        DrawUiInstances {
                            pass: &mut pass,
                            instance_count: batch.instance_count,
                        },
                    )?;
                }
                UiPreparedDrawCommand::ProductSurface(index) => {
                    let Some(product_surface_pass) = self.product_surface_pass.as_ref() else {
                        continue;
                    };
                    let Some(batch) = prepared.product_surface_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(&product_surface_pass.pipeline);
                    pass.set_bind_group(0, &product_surface_pass.screen_bind_group, &[]);

                    if !product_surface_bind_groups.contains_key(&batch.source) {
                        let ProductSurfaceTextureBindingSource::DynamicTexture {
                            namespace,
                            target_id,
                        } = &batch.source;
                        let key = crate::plugins::render::RenderDynamicTextureTargetKey::new(
                            namespace.clone(),
                            target_id.clone(),
                        );
                        let Ok(view) = self.dynamic_texture_targets.ui_texture_view(&key) else {
                            continue;
                        };
                        context
                            .current_render_resource_bridge()
                            .for_sampled_texture_binding(
                                &view,
                                &product_surface_pass.texture_sampler.realized,
                                CreateUiTextureBindGroup {
                                    device,
                                    label: "engine_ui_product_surface_bind_group",
                                    layout: &product_surface_pass.texture_bind_group_layout,
                                    key: batch.source.clone(),
                                    destination: &mut product_surface_bind_groups,
                                },
                            )?;
                    }

                    let Some(bind_group) = product_surface_bind_groups.get(&batch.source) else {
                        continue;
                    };
                    pass.set_bind_group(1, bind_group, &[]);
                    pass.set_scissor_rect(
                        batch.scissor.0,
                        batch.scissor.1,
                        batch.scissor.2,
                        batch.scissor.3,
                    );
                    context.current_render_resource_bridge().for_vertex_buffer(
                        &batch.instance_buffer.realized,
                        DrawUiInstances {
                            pass: &mut pass,
                            instance_count: batch.instance_count,
                        },
                    )?;
                }
                UiPreparedDrawCommand::Glyph(index) => {
                    let Some(glyph_pass) = self.glyph_pass.as_ref() else {
                        continue;
                    };
                    let Some(batch) = prepared.glyph_batches.get(index) else {
                        continue;
                    };
                    pass.set_pipeline(&glyph_pass.pipeline);
                    pass.set_bind_group(0, &glyph_pass.screen_bind_group, &[]);
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
                    context.current_render_resource_bridge().for_vertex_buffer(
                        &batch.instance_buffer.realized,
                        DrawUiInstances {
                            pass: &mut pass,
                            instance_count: batch.instance_count,
                        },
                    )?;
                }
            }
        }
        Ok(())
    }
}

struct EncodeTimestampedUiPass<'a> {
    renderer: &'a Renderer,
    context: &'a GpuContext,
    device: &'a Device,
    encoder: &'a mut CommandEncoder,
    frame_view: &'a TextureView,
    prepared: &'a UiPreparedDraws,
    viewport_surface_bindings: &'a ViewportSurfaceBindingRegistry,
    indices: super::render_flow::GpuPassTimestampIndices,
    output: &'a mut Result<()>,
}

impl CurrentRenderTimestampWritesTerminal for EncodeTimestampedUiPass<'_> {
    fn write_timestamps(self, query_set: &QuerySet) {
        *self.output = self.renderer.encode_ui_pass_current(
            self.context,
            self.device,
            self.encoder,
            self.frame_view,
            self.prepared,
            self.viewport_surface_bindings,
            Some((query_set, self.indices)),
        );
    }
}

struct DrawUiInstances<'a, 'pass> {
    pass: &'a mut RenderPass<'pass>,
    instance_count: u32,
}

impl CurrentRenderVertexBufferTerminal for DrawUiInstances<'_, '_> {
    fn use_vertex_buffer(self, buffer: &Buffer) {
        self.pass.set_vertex_buffer(0, buffer.slice(..));
        self.pass.draw(0..6, 0..self.instance_count);
    }
}

fn create_single_buffer_bind_group(
    context: &GpuContext,
    device: &Device,
    label: &'static str,
    layout: &BindGroupLayout,
    buffer: &GpuRealizedBuffer,
) -> Result<BindGroup> {
    let mut output = None;
    context
        .current_render_resource_bridge()
        .for_buffer_binding(
            buffer,
            CreateSingleBufferBindGroup {
                device,
                label,
                layout,
                output: &mut output,
            },
        )?;
    output.ok_or_else(|| anyhow::anyhow!("current render resource bridge did not create '{label}'"))
}

struct CreateSingleBufferBindGroup<'a> {
    device: &'a Device,
    label: &'static str,
    layout: &'a BindGroupLayout,
    output: &'a mut Option<BindGroup>,
}

impl CurrentRenderBufferBindingTerminal for CreateSingleBufferBindGroup<'_> {
    fn bind_buffer(self, buffer: &Buffer) {
        *self.output = Some(self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(self.label),
            layout: self.layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        }));
    }
}

fn create_sampled_texture_bind_group(
    context: &GpuContext,
    device: &Device,
    label: &'static str,
    layout: &BindGroupLayout,
    view: &GpuRealizedTextureView,
    sampler: &GpuRealizedSampler,
) -> Result<BindGroup> {
    let mut output = None;
    context
        .current_render_resource_bridge()
        .for_sampled_texture_binding(
            view,
            sampler,
            CreateSampledTextureBindGroup {
                device,
                label,
                layout,
                output: &mut output,
            },
        )?;
    output.ok_or_else(|| anyhow::anyhow!("current render resource bridge did not create '{label}'"))
}

struct CreateSampledTextureBindGroup<'a> {
    device: &'a Device,
    label: &'static str,
    layout: &'a BindGroupLayout,
    output: &'a mut Option<BindGroup>,
}

impl CurrentRenderSampledTextureBindingTerminal for CreateSampledTextureBindGroup<'_> {
    fn bind_sampled_texture(self, view: &TextureView, sampler: &Sampler) {
        *self.output = Some(self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(self.label),
            layout: self.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        }));
    }
}

struct UploadGlyphAtlasTexture<'a> {
    queue: &'a Queue,
    atlas_image: &'a crate::plugins::render::features::UiFontAtlasImage,
}

impl CurrentRenderTextureUploadTerminal for UploadGlyphAtlasTexture<'_> {
    fn upload_texture(self, texture: &Texture) {
        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &self.atlas_image.pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.atlas_image.width.max(1)),
                rows_per_image: Some(self.atlas_image.height.max(1)),
            },
            Extent3d {
                width: self.atlas_image.width.max(1),
                height: self.atlas_image.height.max(1),
                depth_or_array_layers: 1,
            },
        );
    }
}

struct CreateUiTextureBindGroup<'a, K> {
    device: &'a Device,
    label: &'static str,
    layout: &'a BindGroupLayout,
    key: K,
    destination: &'a mut std::collections::BTreeMap<K, BindGroup>,
}

impl<K: Ord> CurrentRenderSampledTextureBindingTerminal for CreateUiTextureBindGroup<'_, K> {
    fn bind_sampled_texture(self, view: &TextureView, sampler: &Sampler) {
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(self.label),
            layout: self.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        });
        self.destination.insert(self.key, bind_group);
    }
}
