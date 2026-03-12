use super::*;

impl Renderer {
    // Owner: Engine Renderer - UI Pipeline Setup and Encoding
    pub fn new() -> Self {
        Self {
            rect_pass: None,
            rect_pass_format: None,
            rect_pass_shader_revision: 0,
            text_renderer: None,
            text_renderer_format: None,
        }
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

    pub(super) fn ensure_text_renderer(
        &mut self,
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
    ) {
        if self.text_renderer.is_some() && self.text_renderer_format == Some(format) {
            return;
        }

        let provider = FileFontProvider;
        self.text_renderer = Some(TextRenderer::new(device, queue, format, &provider));
        self.text_renderer_format = Some(format);
    }

    pub(super) fn encode_ui_pass(
        &self,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        prepared: &UiPreparedDraws,
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

        if let Some(instance_buffer) = prepared.rect_instance_buffer.as_ref() {
            pass.set_pipeline(&rect_pass.pipeline);
            pass.set_bind_group(0, &rect_pass.screen_bind_group, &[]);
            pass.set_vertex_buffer(0, instance_buffer.slice(..));
            pass.draw(0..6, 0..prepared.rect_instances as u32);
        }

        if let Some(text_renderer) = self.text_renderer.as_ref() {
            let full_scissor = Self::full_scissor(prepared.surface_size.0, prepared.surface_size.1);
            pass.set_scissor_rect(
                full_scissor.0,
                full_scissor.1,
                full_scissor.2,
                full_scissor.3,
            );
            for (text_buffer, text_count, scissor) in &prepared.text_draws {
                pass.set_scissor_rect(scissor.0, scissor.1, scissor.2, scissor.3);
                text_renderer.encode_draw(&mut pass, text_buffer, *text_count);
            }
        }
    }
}
