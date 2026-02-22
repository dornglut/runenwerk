use super::text::{FileFontProvider, TextRenderer};
use crate::ui::{UiDrawCmd, UiDrawList};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu::*;

const CLEAR_COLOR: Color = Color {
    r: 0.02,
    g: 0.02,
    b: 0.03,
    a: 1.0,
};

const UI_RECT_SHADER: &str = r#"
struct VsIn {
    @location(0) rect : vec4<f32>,
    @location(1) color : vec4<f32>,
    @location(2) radius : f32,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) local_px : vec2<f32>,
    @location(1) half_size : vec2<f32>,
    @location(2) color : vec4<f32>,
    @location(3) radius : f32,
};

struct ScreenUniform {
    size : vec2<f32>,
    _pad : vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen : ScreenUniform;

@vertex
fn vs_main(input: VsIn, @builtin(vertex_index) vertex_index: u32) -> VsOut {
    let uv = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );

    let p = uv[vertex_index];
    let local = p * 2.0 - vec2<f32>(1.0, 1.0);
    let center = vec2<f32>(input.rect.x + input.rect.z * 0.5, input.rect.y + input.rect.w * 0.5);
    let half_size = vec2<f32>(input.rect.z * 0.5, input.rect.w * 0.5);
    let pixel = center + local * half_size;

    let x_ndc = (pixel.x / screen.size.x) * 2.0 - 1.0;
    let y_ndc = 1.0 - (pixel.y / screen.size.y) * 2.0;

    var out: VsOut;
    out.clip_position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.local_px = local * half_size;
    out.half_size = half_size;
    out.color = input.color;
    out.radius = input.radius;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let safe_half = max(input.half_size, vec2<f32>(0.0001, 0.0001));
    let max_radius = min(safe_half.x, safe_half.y);
    let radius = clamp(input.radius, 0.0, max_radius);

    let q = abs(input.local_px) - (safe_half - vec2<f32>(radius, radius));
    let outside = length(max(q, vec2<f32>(0.0, 0.0)));
    let inside = min(max(q.x, q.y), 0.0);
    let sdf = outside + inside - radius;

    if (sdf > 0.0) {
        discard;
    }

    return input.color;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RectInstanceRaw {
    rect: [f32; 4],
    color: [f32; 4],
    radius: f32,
    _pad: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ScreenUniformRaw {
    size: [f32; 2],
    _pad: [f32; 2],
}

#[derive(Debug)]
struct RectPass {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
}

#[derive(Debug)]
pub struct Renderer {
    rect_pass: Option<RectPass>,
    rect_pass_format: Option<TextureFormat>,
    text_renderer: Option<TextRenderer>,
    text_renderer_format: Option<TextureFormat>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            rect_pass: None,
            rect_pass_format: None,
            text_renderer: None,
            text_renderer_format: None,
        }
    }

    fn ensure_rect_pass(&mut self, device: &Device, format: TextureFormat) {
        if self.rect_pass.is_some() && self.rect_pass_format == Some(format) {
            return;
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_v2_ui_rect_shader"),
            source: ShaderSource::Wgsl(UI_RECT_SHADER.into()),
        });

        let screen_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_v2_ui_screen_uniform"),
            size: std::mem::size_of::<ScreenUniformRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_v2_ui_rect_bind_group_layout"),
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
            label: Some("engine_v2_ui_rect_bind_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_v2_ui_rect_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_v2_ui_rect_pipeline"),
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
    }

    fn extract_rect_instances(draw_list: &UiDrawList) -> Vec<RectInstanceRaw> {
        let mut instances = Vec::new();
        for cmd in &draw_list.commands {
            if let UiDrawCmd::Rect {
                x,
                y,
                w,
                h,
                color,
                radius,
            } = cmd
            {
                instances.push(RectInstanceRaw {
                    rect: [*x, *y, *w, *h],
                    color: *color,
                    radius: *radius,
                    _pad: [0.0; 3],
                });
            }
        }
        instances
    }

    fn ensure_text_renderer(&mut self, device: &Device, queue: &Queue, format: TextureFormat) {
        if self.text_renderer.is_some() && self.text_renderer_format == Some(format) {
            return;
        }

        let provider = FileFontProvider;
        self.text_renderer = Some(TextRenderer::new(device, queue, format, &provider));
        self.text_renderer_format = Some(format);
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_view: &TextureView,
        draw_list: &UiDrawList,
        surface_format: TextureFormat,
        surface_width: f32,
        surface_height: f32,
    ) {
        self.ensure_rect_pass(device, surface_format);
        self.ensure_text_renderer(device, queue, surface_format);
        let Some(rect_pass) = self.rect_pass.as_ref() else {
            return;
        };

        let instances = Self::extract_rect_instances(draw_list);
        let instance_buffer = if instances.is_empty() {
            None
        } else {
            Some(device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("engine_v2_ui_rect_instances"),
                contents: bytemuck::cast_slice(&instances),
                usage: BufferUsages::VERTEX,
            }))
        };

        let screen = ScreenUniformRaw {
            size: [surface_width.max(1.0), surface_height.max(1.0)],
            _pad: [0.0; 2],
        };
        queue.write_buffer(&rect_pass.screen_buffer, 0, bytemuck::bytes_of(&screen));

        let text_instances = self
            .text_renderer
            .as_ref()
            .and_then(|renderer| {
                renderer.write_screen_uniform(queue, surface_width, surface_height);
                renderer.build_instance_buffer(device, draw_list)
            });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_v2_render_encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("engine_v2_clear_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: frame_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(CLEAR_COLOR),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(instance_buffer) = instance_buffer.as_ref() {
                pass.set_pipeline(&rect_pass.pipeline);
                pass.set_bind_group(0, &rect_pass.screen_bind_group, &[]);
                pass.set_vertex_buffer(0, instance_buffer.slice(..));
                pass.draw(0..6, 0..instances.len() as u32);
            }

            if let (Some(text_renderer), Some((text_buffer, text_count))) =
                (self.text_renderer.as_ref(), text_instances.as_ref())
            {
                text_renderer.encode_draw(&mut pass, text_buffer, *text_count);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
