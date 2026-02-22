use super::frame_graph::{FrameGraph, PassHandle};
use super::model_manager::{ModelManager, ModelMaterial, ModelMesh, ModelMeshVertex, ModelTextureData};
use super::pipeline_registry::{PassSlot, PipelineKey, PipelineRegistry, PipelineSelection};
use super::shader_manager::{ShaderId, ShaderManager};
use super::text::{FileFontProvider, TextRenderer};
use super::world_compute::{
    DEFAULT_WORLD_COMPOSE_SHADER_FULLSCREEN, DEFAULT_WORLD_COMPUTE_SHADER_BASIC,
    DEFAULT_WORLD_COMPUTE_SHADER_HIGH_CONTRAST, WorldComputeRenderer, WorldRenderAgent,
    WorldRenderFrame,
    WorldShaderSources,
};
use crate::ui::{UiDrawCmd, UiDrawList};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use wgpu::*;

pub const DEFAULT_UI_RECT_SHADER: &str = r#"
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

pub const DEFAULT_MESH_SHADER: &str = r#"
struct VsIn {
    @location(0) position : vec3<f32>,
    @location(1) uv : vec2<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

struct CameraUniform {
    view_proj : mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera : CameraUniform;
@group(1) @binding(0)
var<uniform> material : vec4<f32>;
@group(1) @binding(1)
var material_texture : texture_2d<f32>;
@group(1) @binding(2)
var material_sampler : sampler;

@vertex
fn vs_main(input: VsIn) -> VsOut {
    var out: VsOut;
    out.clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.uv = input.uv;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(material_texture, material_sampler, input.uv);
    return texel * material;
}
"#;

const RESOURCE_SURFACE_COLOR: &str = "surface_color";
const RESOURCE_WORLD_COLOR: &str = "world_color";
const RESOURCE_WORLD_PARAMS: &str = "world_params";
const RESOURCE_WORLD_AGENTS: &str = "world_agents";
const RESOURCE_MESH_DATA: &str = "mesh_data";
const RESOURCE_UI_DRAW_LIST: &str = "ui_draw_list";

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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MeshVertexRaw {
    position: [f32; 3],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MeshCameraRaw {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MeshMaterialRaw {
    base_color_factor: [f32; 4],
}

#[derive(Debug)]
struct RectPass {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
}

#[derive(Debug)]
struct MeshPass {
    pipeline: RenderPipeline,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    material_bind_group_layout: BindGroupLayout,
    default_texture_view: TextureView,
    default_sampler: Sampler,
}

#[derive(Debug)]
struct MeshPreparedDrawItem {
    index_count: u32,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    _material_buffer: Option<Buffer>,
    material_bind_group: Option<BindGroup>,
    _material_texture: Option<Texture>,
    _material_texture_view: Option<TextureView>,
}

#[derive(Debug)]
struct MeshPreparedDraw {
    draws: Vec<MeshPreparedDrawItem>,
}

#[derive(Debug)]
struct UiPreparedDraws {
    rect_instances: usize,
    rect_instance_buffer: Option<Buffer>,
    text_draws: Vec<(Buffer, u32, (u32, u32, u32, u32))>,
    surface_size: (u32, u32),
}

const PLAYER_CUBE_COLOR: [f32; 4] = [0.12, 0.88, 0.18, 1.0];
const ENEMY_CUBE_COLOR: [f32; 4] = [0.92, 0.18, 0.18, 1.0];
const GROUND_TILE_SIZE: f32 = 4.0;

fn checker_texture_rgba(size: u32) -> ModelTextureData {
    let size = size.max(2);
    let mut rgba8 = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let cell = ((x / 8) + (y / 8)) % 2;
            let color = if cell == 0 {
                [46u8, 50u8, 58u8, 255u8]
            } else {
                [30u8, 34u8, 40u8, 255u8]
            };
            rgba8.extend_from_slice(&color);
        }
    }
    ModelTextureData {
        width: size,
        height: size,
        rgba8,
    }
}

fn ground_mesh_for_bounds(world_bounds: [f32; 4]) -> ModelMesh {
    let min_x = world_bounds[0];
    let min_z = world_bounds[1];
    let max_x = world_bounds[2];
    let max_z = world_bounds[3];
    let span_x = (max_x - min_x).max(1.0);
    let span_z = (max_z - min_z).max(1.0);
    let tiles_u = (span_x / GROUND_TILE_SIZE).max(1.0);
    let tiles_v = (span_z / GROUND_TILE_SIZE).max(1.0);
    let y = -0.02;

    ModelMesh {
        vertices: vec![
            ModelMeshVertex {
                position: [min_x, y, min_z],
                uv: [0.0, 0.0],
            },
            ModelMeshVertex {
                position: [max_x, y, min_z],
                uv: [tiles_u, 0.0],
            },
            ModelMeshVertex {
                position: [max_x, y, max_z],
                uv: [tiles_u, tiles_v],
            },
            ModelMeshVertex {
                position: [min_x, y, max_z],
                uv: [0.0, tiles_v],
            },
        ],
        // winding chosen so top-face normal points +Y.
        indices: vec![0, 2, 1, 0, 3, 2],
        material: ModelMaterial {
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            base_color_texture: Some(checker_texture_rgba(64)),
        },
    }
}

fn create_texture_from_rgba(
    device: &Device,
    queue: &Queue,
    texture: &ModelTextureData,
    mesh_idx: u32,
) -> Texture {
    let gpu_texture = device.create_texture(&TextureDescriptor {
        label: Some("engine_v2_mesh_material_texture"),
        size: Extent3d {
            width: texture.width.max(1),
            height: texture.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let expected_len = (texture.width as usize)
        .saturating_mul(texture.height as usize)
        .saturating_mul(4);
    if texture.rgba8.len() == expected_len && expected_len > 0 {
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &gpu_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &texture.rgba8,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture.width),
                rows_per_image: Some(texture.height),
            },
            Extent3d {
                width: texture.width.max(1),
                height: texture.height.max(1),
                depth_or_array_layers: 1,
            },
        );
    } else {
        tracing::warn!(
            mesh_idx,
            expected_len,
            actual_len = texture.rgba8.len(),
            "invalid base color texture payload, using uninitialized texture"
        );
    }
    gpu_texture
}

fn agent_cube_meshes(agents: &[WorldRenderAgent]) -> Vec<ModelMesh> {
    let mut meshes = Vec::with_capacity(agents.len());
    for agent in agents {
        let color = if agent.team == 0 {
            PLAYER_CUBE_COLOR
        } else {
            ENEMY_CUBE_COLOR
        };
        let half = (agent.radius * 0.45).max(0.25);
        let cx = agent.x;
        let cy = half;
        let cz = agent.y;

        let vertices = vec![
            ModelMeshVertex {
                position: [cx - half, cy - half, cz - half],
                uv: [0.0, 0.0],
            },
            ModelMeshVertex {
                position: [cx + half, cy - half, cz - half],
                uv: [1.0, 0.0],
            },
            ModelMeshVertex {
                position: [cx + half, cy + half, cz - half],
                uv: [1.0, 1.0],
            },
            ModelMeshVertex {
                position: [cx - half, cy + half, cz - half],
                uv: [0.0, 1.0],
            },
            ModelMeshVertex {
                position: [cx - half, cy - half, cz + half],
                uv: [0.0, 0.0],
            },
            ModelMeshVertex {
                position: [cx + half, cy - half, cz + half],
                uv: [1.0, 0.0],
            },
            ModelMeshVertex {
                position: [cx + half, cy + half, cz + half],
                uv: [1.0, 1.0],
            },
            ModelMeshVertex {
                position: [cx - half, cy + half, cz + half],
                uv: [0.0, 1.0],
            },
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, // back
            4, 6, 5, 4, 7, 6, // front
            0, 4, 5, 0, 5, 1, // bottom
            3, 2, 6, 3, 6, 7, // top
            1, 5, 6, 1, 6, 2, // right
            0, 3, 7, 0, 7, 4, // left
        ];

        meshes.push(ModelMesh {
            vertices,
            indices,
            material: ModelMaterial {
                base_color_factor: color,
                base_color_texture: None,
            },
        });
    }
    meshes
}

#[derive(Debug)]
pub struct Renderer {
    pipeline_registry: PipelineRegistry,
    shader_manager: ShaderManager,
    model_manager: ModelManager,
    world_compute_renderer: WorldComputeRenderer,
    mesh_pass: Option<MeshPass>,
    mesh_pass_format: Option<TextureFormat>,
    rect_pass: Option<RectPass>,
    rect_pass_format: Option<TextureFormat>,
    rect_pass_shader_revision: u64,
    text_renderer: Option<TextRenderer>,
    text_renderer_format: Option<TextureFormat>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            pipeline_registry: PipelineRegistry::default(),
            shader_manager: ShaderManager::new(),
            model_manager: ModelManager::new(),
            world_compute_renderer: WorldComputeRenderer::new(),
            mesh_pass: None,
            mesh_pass_format: None,
            rect_pass: None,
            rect_pass_format: None,
            rect_pass_shader_revision: 0,
            text_renderer: None,
            text_renderer_format: None,
        }
    }

    fn ensure_mesh_pass(&mut self, device: &Device, queue: &Queue, format: TextureFormat) {
        if self.mesh_pass.is_some() && self.mesh_pass_format == Some(format) {
            return;
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_v2_mesh_shader"),
            source: ShaderSource::Wgsl(DEFAULT_MESH_SHADER.into()),
        });

        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_v2_mesh_camera_uniform"),
            size: std::mem::size_of::<MeshCameraRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_v2_mesh_bind_group_layout"),
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
        let material_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("engine_v2_mesh_material_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_v2_mesh_bind_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_v2_mesh_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout, &material_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_v2_mesh_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<MeshVertexRaw>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 12,
                            shader_location: 1,
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
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let default_texture = device.create_texture(&TextureDescriptor {
            label: Some("engine_v2_mesh_default_texture"),
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &default_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &[255, 255, 255, 255],
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let default_texture_view = default_texture.create_view(&TextureViewDescriptor::default());
        let default_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("engine_v2_mesh_default_sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        self.mesh_pass = Some(MeshPass {
            pipeline,
            camera_buffer,
            camera_bind_group,
            material_bind_group_layout,
            default_texture_view,
            default_sampler,
        });
        self.mesh_pass_format = Some(format);
    }

    fn ensure_rect_pass(
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
            label: Some("engine_v2_ui_rect_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
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
        self.rect_pass_shader_revision = shader_revision;
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

    fn full_scissor(surface_width: u32, surface_height: u32) -> (u32, u32, u32, u32) {
        (0, 0, surface_width.max(1), surface_height.max(1))
    }

    fn clip_to_scissor(
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

    fn ensure_text_renderer(&mut self, device: &Device, queue: &Queue, format: TextureFormat) {
        if self.text_renderer.is_some() && self.text_renderer_format == Some(format) {
            return;
        }

        let provider = FileFontProvider;
        self.text_renderer = Some(TextRenderer::new(device, queue, format, &provider));
        self.text_renderer_format = Some(format);
    }

    pub fn pipeline_selection(&self) -> PipelineSelection {
        self.pipeline_registry.selection()
    }

    pub fn set_pipeline_for_slot(&mut self, slot: PassSlot, key: PipelineKey) -> Result<()> {
        self.pipeline_registry.set_pipeline(slot, key)
    }

    pub fn poll_shader_hot_reload(&mut self) -> Vec<String> {
        self.shader_manager.poll_updates()
    }

    pub fn force_shader_reload(&mut self) -> Vec<String> {
        self.shader_manager.request_reload();
        self.shader_manager.poll_updates()
    }

    pub fn set_shader_watch_enabled(&mut self, enabled: bool) {
        self.shader_manager.set_watch_enabled(enabled);
    }

    pub fn shader_watch_enabled(&self) -> bool {
        self.shader_manager.watch_enabled()
    }

    pub fn shader_status_lines(&self) -> Vec<String> {
        self.shader_manager.status_lines()
    }

    pub fn poll_model_hot_reload(&mut self) -> Vec<String> {
        self.model_manager.poll_updates()
    }

    pub fn force_model_reload(&mut self) -> Vec<String> {
        self.model_manager.request_reload();
        self.model_manager.poll_updates()
    }

    pub fn set_model_watch_enabled(&mut self, enabled: bool) {
        self.model_manager.set_watch_enabled(enabled);
    }

    pub fn model_watch_enabled(&self) -> bool {
        self.model_manager.watch_enabled()
    }

    pub fn model_status_lines(&self) -> Vec<String> {
        self.model_manager.status_lines()
    }

    fn prepare_ui_draws(
        &self,
        device: &Device,
        queue: &Queue,
        draw_list: &UiDrawList,
        surface_width: f32,
        surface_height: f32,
    ) -> UiPreparedDraws {
        let surface_width_u32 = surface_width.max(1.0).round() as u32;
        let surface_height_u32 = surface_height.max(1.0).round() as u32;
        let instances = Self::extract_rect_instances(draw_list);
        let rect_instance_buffer = if instances.is_empty() {
            None
        } else {
            Some(device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("engine_v2_ui_rect_instances"),
                contents: bytemuck::cast_slice(&instances),
                usage: BufferUsages::VERTEX,
            }))
        };

        if let Some(rect_pass) = self.rect_pass.as_ref() {
            let screen = ScreenUniformRaw {
                size: [surface_width.max(1.0), surface_height.max(1.0)],
                _pad: [0.0; 2],
            };
            queue.write_buffer(&rect_pass.screen_buffer, 0, bytemuck::bytes_of(&screen));
        }

        if let Some(text_renderer) = self.text_renderer.as_ref() {
            text_renderer.write_screen_uniform(queue, surface_width, surface_height);
        }

        let text_draws = if let Some(text_renderer) = self.text_renderer.as_ref() {
            let full_scissor = Self::full_scissor(surface_width_u32, surface_height_u32);
            let mut draws = Vec::new();
            for cmd in &draw_list.commands {
                let UiDrawCmd::Text { clip, .. } = cmd else {
                    continue;
                };
                let scissor = clip
                    .and_then(|clip| {
                        Self::clip_to_scissor(clip, surface_width_u32, surface_height_u32)
                    })
                    .unwrap_or(full_scissor);
                let single = UiDrawList {
                    commands: vec![cmd.clone()],
                };
                if let Some((buffer, count)) = text_renderer.build_instance_buffer(device, &single)
                {
                    draws.push((buffer, count, scissor));
                }
            }
            draws
        } else {
            Vec::new()
        };

        UiPreparedDraws {
            rect_instances: instances.len(),
            rect_instance_buffer,
            text_draws,
            surface_size: (surface_width_u32, surface_height_u32),
        }
    }

    fn prepare_mesh_draw(
        &self,
        device: &Device,
        queue: &Queue,
        world_frame: &WorldRenderFrame,
        surface_width: f32,
        surface_height: f32,
    ) -> MeshPreparedDraw {
        let meshes: Vec<ModelMesh> = if world_frame.agents.is_empty() {
            self.model_manager.collect_meshes()
        } else {
            let mut meshes = Vec::new();
            meshes.push(ground_mesh_for_bounds(world_frame.world_bounds));
            meshes.extend(agent_cube_meshes(&world_frame.agents));
            meshes
        };
        if meshes.is_empty() {
            return MeshPreparedDraw { draws: Vec::new() };
        }

        if let Some(mesh_pass) = self.mesh_pass.as_ref() {
            let aspect = (surface_width.max(1.0) / surface_height.max(1.0)).max(0.1);
            let player_target = world_frame
                .agents
                .iter()
                .find(|agent| agent.team == 0)
                .or_else(|| world_frame.agents.first())
                .map(|agent| Vec3::new(agent.x, 0.0, agent.y))
                .unwrap_or(Vec3::ZERO);
            let eye = player_target + Vec3::new(5.0, 5.0, 5.0);
            let target = player_target;
            let up = Vec3::Y;
            let view = Mat4::look_at_rh(eye, target, up);
            let proj = Mat4::perspective_rh_gl(55.0f32.to_radians(), aspect, 0.01, 200.0);
            let camera = MeshCameraRaw {
                view_proj: (proj * view).to_cols_array_2d(),
            };
            queue.write_buffer(&mesh_pass.camera_buffer, 0, bytemuck::bytes_of(&camera));
        }

        let Some(mesh_pass) = self.mesh_pass.as_ref() else {
            return MeshPreparedDraw { draws: Vec::new() };
        };

        let mut draws = Vec::new();
        for (mesh_idx, mesh) in meshes.into_iter().enumerate() {
            let vertices: Vec<MeshVertexRaw> = mesh
                .vertices
                .into_iter()
                .map(|v| MeshVertexRaw {
                    position: v.position,
                    uv: v.uv,
                })
                .collect();
            if vertices.is_empty() || mesh.indices.is_empty() {
                continue;
            }

            let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("engine_v2_mesh_vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("engine_v2_mesh_indices"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: BufferUsages::INDEX,
            });

            let material_raw = MeshMaterialRaw {
                base_color_factor: mesh.material.base_color_factor,
            };
            let material_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("engine_v2_mesh_material_uniform"),
                contents: bytemuck::bytes_of(&material_raw),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let (material_texture, material_texture_view) =
                if let Some(tex) = mesh.material.base_color_texture.as_ref() {
                    let texture = create_texture_from_rgba(device, queue, tex, mesh_idx as u32);
                    let view = texture.create_view(&TextureViewDescriptor::default());
                    (Some(texture), Some(view))
                } else {
                    (None, None)
                };
            let texture_view_ref = material_texture_view
                .as_ref()
                .unwrap_or(&mesh_pass.default_texture_view);
            let material_bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("engine_v2_mesh_material_bind_group"),
                layout: &mesh_pass.material_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: material_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(texture_view_ref),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(&mesh_pass.default_sampler),
                    },
                ],
            });

            draws.push(MeshPreparedDrawItem {
                index_count: mesh.indices.len() as u32,
                vertex_buffer: Some(vertex_buffer),
                index_buffer: Some(index_buffer),
                _material_buffer: Some(material_buffer),
                material_bind_group: Some(material_bind_group),
                _material_texture: material_texture,
                _material_texture_view: material_texture_view,
            });
        }

        MeshPreparedDraw { draws }
    }

    fn encode_mesh_pass(
        &self,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        prepared: &MeshPreparedDraw,
    ) {
        if prepared.draws.is_empty() {
            return;
        }
        let Some(mesh_pass) = self.mesh_pass.as_ref() else {
            return;
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_v2_mesh_pass"),
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

        pass.set_pipeline(&mesh_pass.pipeline);
        pass.set_bind_group(0, &mesh_pass.camera_bind_group, &[]);
        for draw in &prepared.draws {
            let (Some(vertex_buffer), Some(index_buffer), Some(material_bind_group)) = (
                draw.vertex_buffer.as_ref(),
                draw.index_buffer.as_ref(),
                draw.material_bind_group.as_ref(),
            ) else {
                continue;
            };
            pass.set_bind_group(1, material_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
            pass.draw_indexed(0..draw.index_count, 0, 0..1);
        }
    }

    fn encode_ui_pass(
        &self,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        prepared: &UiPreparedDraws,
        pipeline: PipelineKey,
    ) {
        if pipeline != PipelineKey::UiCompositeSdf {
            return;
        }
        let Some(rect_pass) = self.rect_pass.as_ref() else {
            return;
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_v2_ui_pass"),
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

    fn build_frame_graph(&self) -> (FrameGraph, PassHandle, PassHandle, PassHandle, PassHandle) {
        let mut graph = FrameGraph::new();
        let world_compute = graph
            .compute_pass(
                "world_compute",
                self.pipeline_registry.key_for(PassSlot::WorldCompute),
            )
            .reads(&[RESOURCE_WORLD_PARAMS, RESOURCE_WORLD_AGENTS])
            .writes(&[RESOURCE_WORLD_COLOR])
            .build();
        let world_compose = graph
            .render_pass(
                "world_compose",
                self.pipeline_registry.key_for(PassSlot::WorldCompose),
            )
            .reads(&[RESOURCE_WORLD_COLOR])
            .writes(&[RESOURCE_SURFACE_COLOR])
            .depends_on(world_compute)
            .build();
        let mesh_pass = graph
            .render_pass("mesh_overlay", PipelineKey::WorldComposeFullscreen)
            .reads(&[RESOURCE_MESH_DATA])
            .writes(&[RESOURCE_SURFACE_COLOR])
            .depends_on(world_compose)
            .build();
        let ui_composite = graph
            .render_pass(
                "ui_composite",
                self.pipeline_registry.key_for(PassSlot::UiComposite),
            )
            .reads(&[RESOURCE_UI_DRAW_LIST])
            .writes(&[RESOURCE_SURFACE_COLOR])
            .depends_on(mesh_pass)
            .build();
        (graph, world_compute, world_compose, mesh_pass, ui_composite)
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_view: &TextureView,
        world_frame: &WorldRenderFrame,
        draw_list: &UiDrawList,
        surface_format: TextureFormat,
        surface_width: f32,
        surface_height: f32,
    ) {
        let ui_rect_shader = self
            .shader_manager
            .source_or(ShaderId::UiRect, DEFAULT_UI_RECT_SHADER)
            .to_string();
        let ui_rect_revision = self.shader_manager.revision(ShaderId::UiRect);
        let world_compute_basic = self
            .shader_manager
            .source_or(
                ShaderId::WorldComputeBasic,
                DEFAULT_WORLD_COMPUTE_SHADER_BASIC,
            )
            .to_string();
        let world_compute_high_contrast = self
            .shader_manager
            .source_or(
                ShaderId::WorldComputeHighContrast,
                DEFAULT_WORLD_COMPUTE_SHADER_HIGH_CONTRAST,
            )
            .to_string();
        let world_compose = self
            .shader_manager
            .source_or(
                ShaderId::WorldComposeFullscreen,
                DEFAULT_WORLD_COMPOSE_SHADER_FULLSCREEN,
            )
            .to_string();
        let world_shader_sources = WorldShaderSources {
            compute_basic: &world_compute_basic,
            compute_high_contrast: &world_compute_high_contrast,
            compose_fullscreen: &world_compose,
            revisions: [
                self.shader_manager.revision(ShaderId::WorldComputeBasic),
                self.shader_manager.revision(ShaderId::WorldComputeHighContrast),
                self.shader_manager.revision(ShaderId::WorldComposeFullscreen),
            ],
        };

        self.ensure_rect_pass(
            device,
            surface_format,
            &ui_rect_shader,
            ui_rect_revision,
        );
        self.ensure_mesh_pass(device, queue, surface_format);
        self.ensure_text_renderer(device, queue, surface_format);
        let surface_width_u32 = surface_width.max(1.0).round() as u32;
        let surface_height_u32 = surface_height.max(1.0).round() as u32;
        let mut merged_world_frame = world_frame.clone();
        merged_world_frame
            .model_proxies
            .extend(self.model_manager.collect_sdf_proxies());
        let prepared_ui =
            self.prepare_ui_draws(device, queue, draw_list, surface_width, surface_height);
        let prepared_mesh = self.prepare_mesh_draw(
            device,
            queue,
            &merged_world_frame,
            surface_width,
            surface_height,
        );
        self.world_compute_renderer.prepare_frame(
            device,
            queue,
            surface_format,
            surface_width_u32,
            surface_height_u32,
            &world_shader_sources,
            &merged_world_frame,
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_v2_render_encoder"),
        });

        let (graph, world_compute_pass, world_compose_pass, mesh_pass, ui_pass) =
            self.build_frame_graph();
        let order = match graph.execution_order() {
            Ok(order) => order,
            Err(err) => {
                tracing::error!(?err, "frame graph execution order failed; using fallback order");
                vec![world_compute_pass, world_compose_pass, mesh_pass, ui_pass]
            }
        };

        for handle in order {
            let Some(node) = graph.node(handle) else {
                continue;
            };
            if handle == world_compute_pass {
                self.world_compute_renderer
                    .encode_compute_pass(&mut encoder, node.pipeline);
                continue;
            }
            if handle == world_compose_pass {
                self.world_compute_renderer
                    .encode_compose_pass(&mut encoder, frame_view, node.pipeline);
                continue;
            }
            if handle == mesh_pass {
                self.encode_mesh_pass(&mut encoder, frame_view, &prepared_mesh);
                continue;
            }
            if handle == ui_pass {
                self.encode_ui_pass(&mut encoder, frame_view, &prepared_ui, node.pipeline);
                continue;
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    use super::Renderer;

    #[test]
    fn clip_to_scissor_clamps_and_rejects_empty() {
        let clipped = Renderer::clip_to_scissor([-10.0, 4.0, 20.0, 10.0], 100, 80)
            .expect("clip should intersect");
        assert_eq!(clipped, (0, 4, 10, 10));

        let none = Renderer::clip_to_scissor([200.0, 200.0, 10.0, 10.0], 100, 80);
        assert!(none.is_none());
    }
}
