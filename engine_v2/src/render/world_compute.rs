use super::PipelineKey;
use bytemuck::{Pod, Zeroable};
use wgpu::*;

pub const MAX_WORLD_RENDER_AGENTS: usize = 512;
pub const MAX_WORLD_RENDER_MODELS: usize = 1024;

const WORLD_CLEAR_COLOR: Color = Color {
    r: 0.02,
    g: 0.02,
    b: 0.03,
    a: 1.0,
};

pub const DEFAULT_WORLD_COMPUTE_SHADER_BASIC: &str = r#"
struct WorldParams {
    screen_size : vec2<f32>,
    _pad0 : vec2<f32>,
    world_min : vec2<f32>,
    _pad1 : vec2<f32>,
    world_max : vec2<f32>,
    _pad2 : vec2<f32>,
    agent_count : u32,
    model_count : u32,
    paused : u32,
    _pad3 : u32,
};

struct Agent {
    pos : vec2<f32>,
    radius : f32,
    health : f32,
    team : u32,
    _pad0 : vec3<u32>,
};

struct ModelProxy {
    pos : vec2<f32>,
    radius : f32,
    _pad0 : f32,
    color : vec4<f32>,
};

@group(0) @binding(0)
var output_tex : texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> params : WorldParams;

@group(0) @binding(2)
var<storage, read> agents : array<Agent>;
@group(0) @binding(3)
var<storage, read> models : array<ModelProxy>;

fn team_color(team: u32) -> vec3<f32> {
    if (team == 0u) {
        return vec3<f32>(0.28, 0.72, 0.98);
    }
    return vec3<f32>(0.95, 0.34, 0.30);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if (x >= u32(params.screen_size.x) || y >= u32(params.screen_size.y)) {
        return;
    }

    let uv = vec2<f32>(f32(x) / max(params.screen_size.x - 1.0, 1.0), f32(y) / max(params.screen_size.y - 1.0, 1.0));
    let world_span = max(params.world_max - params.world_min, vec2<f32>(0.001, 0.001));
    let world_pos = params.world_min + vec2<f32>(uv.x * world_span.x, (1.0 - uv.y) * world_span.y);

    var color = vec3<f32>(0.07, 0.09, 0.12);

    let grid_x = abs(fract((world_pos.x - params.world_min.x) / 4.0) - 0.5);
    let grid_y = abs(fract((world_pos.y - params.world_min.y) / 4.0) - 0.5);
    if (grid_x < 0.015 || grid_y < 0.015) {
        color += vec3<f32>(0.03, 0.03, 0.04);
    }

    var idx: u32 = 0u;
    loop {
        if (idx >= params.agent_count) {
            break;
        }
        let agent = agents[idx];
        let delta = world_pos - agent.pos;
        let dist = length(delta);
        if (dist <= agent.radius) {
            let edge = smoothstep(agent.radius, agent.radius * 0.7, dist);
            let hp = clamp(agent.health, 0.0, 1.0);
            let tint = team_color(agent.team) * (0.55 + hp * 0.45);
            color = mix(color, tint, edge);
        }
        idx = idx + 1u;
    }

    var model_idx: u32 = 0u;
    loop {
        if (model_idx >= params.model_count) {
            break;
        }
        let model = models[model_idx];
        let delta = world_pos - model.pos;
        let dist = length(delta);
        let sdf = dist - model.radius;
        let band = smoothstep(0.45, -0.15, sdf);
        color = mix(color, model.color.xyz, band * model.color.w);
        model_idx = model_idx + 1u;
    }

    if (params.paused == 1u) {
        color = mix(color, vec3<f32>(0.62, 0.66, 0.74), 0.25);
    }

    textureStore(output_tex, vec2<i32>(i32(x), i32(y)), vec4<f32>(color, 1.0));
}
"#;

pub const DEFAULT_WORLD_COMPUTE_SHADER_HIGH_CONTRAST: &str = r#"
struct WorldParams {
    screen_size : vec2<f32>,
    _pad0 : vec2<f32>,
    world_min : vec2<f32>,
    _pad1 : vec2<f32>,
    world_max : vec2<f32>,
    _pad2 : vec2<f32>,
    agent_count : u32,
    model_count : u32,
    paused : u32,
    _pad3 : u32,
};

struct Agent {
    pos : vec2<f32>,
    radius : f32,
    health : f32,
    team : u32,
    _pad0 : vec3<u32>,
};

struct ModelProxy {
    pos : vec2<f32>,
    radius : f32,
    _pad0 : f32,
    color : vec4<f32>,
};

@group(0) @binding(0)
var output_tex : texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> params : WorldParams;

@group(0) @binding(2)
var<storage, read> agents : array<Agent>;
@group(0) @binding(3)
var<storage, read> models : array<ModelProxy>;

fn team_color(team: u32) -> vec3<f32> {
    if (team == 0u) {
        return vec3<f32>(0.15, 0.88, 1.00);
    }
    return vec3<f32>(1.00, 0.30, 0.16);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if (x >= u32(params.screen_size.x) || y >= u32(params.screen_size.y)) {
        return;
    }

    let uv = vec2<f32>(
        f32(x) / max(params.screen_size.x - 1.0, 1.0),
        f32(y) / max(params.screen_size.y - 1.0, 1.0)
    );
    let world_span = max(params.world_max - params.world_min, vec2<f32>(0.001, 0.001));
    let world_pos = params.world_min + vec2<f32>(uv.x * world_span.x, (1.0 - uv.y) * world_span.y);

    var color = vec3<f32>(0.01, 0.01, 0.01);
    let grid_x = abs(fract((world_pos.x - params.world_min.x) / 3.0) - 0.5);
    let grid_y = abs(fract((world_pos.y - params.world_min.y) / 3.0) - 0.5);
    if (grid_x < 0.018 || grid_y < 0.018) {
        color += vec3<f32>(0.10, 0.10, 0.10);
    }

    var idx: u32 = 0u;
    loop {
        if (idx >= params.agent_count) {
            break;
        }
        let agent = agents[idx];
        let delta = world_pos - agent.pos;
        let dist = length(delta);
        if (dist <= agent.radius) {
            let edge = smoothstep(agent.radius, agent.radius * 0.55, dist);
            let hp = clamp(agent.health, 0.0, 1.0);
            let tint = team_color(agent.team) * (0.35 + hp * 0.65);
            color = mix(color, tint, edge);
        }
        idx = idx + 1u;
    }

    var model_idx: u32 = 0u;
    loop {
        if (model_idx >= params.model_count) {
            break;
        }
        let model = models[model_idx];
        let delta = world_pos - model.pos;
        let dist = length(delta);
        let sdf = dist - model.radius;
        let band = smoothstep(0.42, -0.12, sdf);
        color = mix(color, model.color.xyz, band * model.color.w);
        model_idx = model_idx + 1u;
    }

    if (params.paused == 1u) {
        color = mix(color, vec3<f32>(0.96, 0.80, 0.32), 0.22);
    }

    textureStore(output_tex, vec2<i32>(i32(x), i32(y)), vec4<f32>(color, 1.0));
}
"#;

pub const DEFAULT_WORLD_COMPOSE_SHADER_FULLSCREEN: &str = r#"
@group(0) @binding(0)
var world_tex : texture_2d<f32>;

@group(0) @binding(1)
var world_sampler : sampler;

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    let p = pos[vertex_index];
    var out: VsOut;
    out.clip_position = vec4<f32>(p, 0.0, 1.0);
    out.uv = vec2<f32>((p.x + 1.0) * 0.5, 1.0 - (p.y + 1.0) * 0.5);
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    return textureSample(world_tex, world_sampler, input.uv);
}
"#;

#[derive(Debug, Clone)]
pub struct WorldRenderAgent {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub health_ratio: f32,
    pub team: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct WorldRenderModelProxy {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct WorldRenderFrame {
    pub world_bounds: [f32; 4],
    pub world_paused: bool,
    pub agents: Vec<WorldRenderAgent>,
    pub model_proxies: Vec<WorldRenderModelProxy>,
}

#[derive(Debug, Clone, Copy)]
pub struct WorldShaderSources<'a> {
    pub compute_basic: &'a str,
    pub compute_high_contrast: &'a str,
    pub compose_fullscreen: &'a str,
    pub revisions: [u64; 3],
}

impl Default for WorldRenderFrame {
    fn default() -> Self {
        Self {
            world_bounds: [-32.0, -18.0, 32.0, 18.0],
            world_paused: false,
            agents: Vec::new(),
            model_proxies: Vec::new(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WorldParamsRaw {
    screen_size: [f32; 2],
    _pad0: [f32; 2],
    world_min: [f32; 2],
    _pad1: [f32; 2],
    world_max: [f32; 2],
    _pad2: [f32; 2],
    agent_count: u32,
    model_count: u32,
    paused: u32,
    _pad3: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WorldAgentRaw {
    pos: [f32; 2],
    radius: f32,
    health: f32,
    team: u32,
    _pad0: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WorldModelRaw {
    pos: [f32; 2],
    radius: f32,
    _pad0: f32,
    color: [f32; 4],
}

#[derive(Debug)]
struct WorldComputePass {
    compute_pipeline_basic: ComputePipeline,
    compute_pipeline_high_contrast: ComputePipeline,
    compose_pipeline: RenderPipeline,
    params_buffer: Buffer,
    agents_buffer: Buffer,
    models_buffer: Buffer,
    compute_bind_group: BindGroup,
    compose_bind_group: BindGroup,
    world_texture: Texture,
    size: (u32, u32),
    surface_format: TextureFormat,
    shader_revisions: [u64; 3],
}

#[derive(Debug, Default)]
pub struct WorldComputeRenderer {
    pass: Option<WorldComputePass>,
}

impl WorldComputeRenderer {
    pub fn new() -> Self {
        Self { pass: None }
    }

    fn rebuild_pass(
        &mut self,
        device: &Device,
        surface_format: TextureFormat,
        width: u32,
        height: u32,
        shaders: &WorldShaderSources<'_>,
    ) {
        let compute_shader_basic = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_v2_world_compute_shader_basic"),
            source: ShaderSource::Wgsl(shaders.compute_basic.into()),
        });
        let compute_shader_high_contrast = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_v2_world_compute_shader_high_contrast"),
            source: ShaderSource::Wgsl(shaders.compute_high_contrast.into()),
        });
        let compose_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_v2_world_compose_shader"),
            source: ShaderSource::Wgsl(shaders.compose_fullscreen.into()),
        });

        let params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_v2_world_params_buffer"),
            size: std::mem::size_of::<WorldParamsRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let agents_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_v2_world_agents_buffer"),
            size: (std::mem::size_of::<WorldAgentRaw>() * MAX_WORLD_RENDER_AGENTS) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let models_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_v2_world_models_buffer"),
            size: (std::mem::size_of::<WorldModelRaw>() * MAX_WORLD_RENDER_MODELS) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let world_texture = device.create_texture(&TextureDescriptor {
            label: Some("engine_v2_world_compute_texture"),
            size: Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let world_texture_view = world_texture.create_view(&TextureViewDescriptor::default());
        let world_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("engine_v2_world_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let compute_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_v2_world_compute_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let compute_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_v2_world_compute_bind_group"),
            layout: &compute_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&world_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: agents_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: models_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_v2_world_compute_pipeline_layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });
        let compute_pipeline_basic = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("engine_v2_world_compute_pipeline_basic"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader_basic,
            entry_point: Some("cs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });
        let compute_pipeline_high_contrast =
            device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("engine_v2_world_compute_pipeline_high_contrast"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader_high_contrast,
            entry_point: Some("cs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let compose_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_v2_world_compose_bind_group_layout"),
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

        let compose_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("engine_v2_world_compose_bind_group"),
            layout: &compose_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&world_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&world_sampler),
                },
            ],
        });

        let compose_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_v2_world_compose_pipeline_layout"),
            bind_group_layouts: &[&compose_bind_group_layout],
            push_constant_ranges: &[],
        });
        let compose_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_v2_world_compose_pipeline"),
            layout: Some(&compose_pipeline_layout),
            vertex: VertexState {
                module: &compose_shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &compose_shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
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

        self.pass = Some(WorldComputePass {
            compute_pipeline_basic,
            compute_pipeline_high_contrast,
            compose_pipeline,
            params_buffer,
            agents_buffer,
            models_buffer,
            compute_bind_group,
            compose_bind_group,
            world_texture,
            size: (width.max(1), height.max(1)),
            surface_format,
            shader_revisions: shaders.revisions,
        });
    }

    fn ensure_pass(
        &mut self,
        device: &Device,
        surface_format: TextureFormat,
        width: u32,
        height: u32,
        shaders: &WorldShaderSources<'_>,
    ) {
        let needs_rebuild = self.pass.as_ref().is_none_or(|pass| {
            pass.size != (width.max(1), height.max(1))
                || pass.surface_format != surface_format
                || pass.shader_revisions != shaders.revisions
        });
        if needs_rebuild {
            self.rebuild_pass(device, surface_format, width, height, shaders);
        }
    }

    pub fn prepare_frame(
        &mut self,
        device: &Device,
        queue: &Queue,
        surface_format: TextureFormat,
        width: u32,
        height: u32,
        shaders: &WorldShaderSources<'_>,
        frame: &WorldRenderFrame,
    ) {
        self.ensure_pass(device, surface_format, width, height, shaders);
        let Some(pass) = self.pass.as_ref() else {
            return;
        };
        let _keep_alive = &pass.world_texture;

        let world_min = [frame.world_bounds[0], frame.world_bounds[1]];
        let world_max = [frame.world_bounds[2], frame.world_bounds[3]];
        let agent_count = frame.agents.len().min(MAX_WORLD_RENDER_AGENTS);
        let model_count = frame.model_proxies.len().min(MAX_WORLD_RENDER_MODELS);
        let params = WorldParamsRaw {
            screen_size: [width.max(1) as f32, height.max(1) as f32],
            _pad0: [0.0; 2],
            world_min,
            _pad1: [0.0; 2],
            world_max,
            _pad2: [0.0; 2],
            agent_count: agent_count as u32,
            model_count: model_count as u32,
            paused: u32::from(frame.world_paused),
            _pad3: 0,
        };
        queue.write_buffer(&pass.params_buffer, 0, bytemuck::bytes_of(&params));

        let mut agents = Vec::with_capacity(MAX_WORLD_RENDER_AGENTS);
        for agent in frame.agents.iter().take(MAX_WORLD_RENDER_AGENTS) {
            agents.push(WorldAgentRaw {
                pos: [agent.x, agent.y],
                radius: agent.radius.max(0.2),
                health: agent.health_ratio.clamp(0.0, 1.0),
                team: agent.team,
                _pad0: [0; 3],
            });
        }
        if !agents.is_empty() {
            queue.write_buffer(&pass.agents_buffer, 0, bytemuck::cast_slice(&agents));
        }

        let mut models = Vec::with_capacity(MAX_WORLD_RENDER_MODELS);
        for model in frame.model_proxies.iter().take(MAX_WORLD_RENDER_MODELS) {
            models.push(WorldModelRaw {
                pos: [model.x, model.y],
                radius: model.radius.max(0.2),
                _pad0: 0.0,
                color: model.color,
            });
        }
        if !models.is_empty() {
            queue.write_buffer(&pass.models_buffer, 0, bytemuck::cast_slice(&models));
        }
    }

    pub fn encode_compute_pass(
        &self,
        encoder: &mut CommandEncoder,
        pipeline: PipelineKey,
    ) {
        let Some(pass) = self.pass.as_ref() else {
            return;
        };
        let selected_pipeline = match pipeline {
            PipelineKey::WorldComputeBasic => &pass.compute_pipeline_basic,
            PipelineKey::WorldComputeHighContrast => &pass.compute_pipeline_high_contrast,
            _ => return,
        };

        let width = pass.size.0.max(1);
        let height = pass.size.1.max(1);

        {
            let mut compute = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("engine_v2_world_compute_pass"),
                timestamp_writes: None,
            });
            compute.set_pipeline(selected_pipeline);
            compute.set_bind_group(0, &pass.compute_bind_group, &[]);
            let gx = width.max(1).div_ceil(8);
            let gy = height.max(1).div_ceil(8);
            compute.dispatch_workgroups(gx, gy, 1);
        }
    }

    pub fn encode_compose_pass(
        &self,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        pipeline: PipelineKey,
    ) {
        if pipeline != PipelineKey::WorldComposeFullscreen {
            return;
        }
        let Some(pass) = self.pass.as_ref() else {
            return;
        };

        {
            let mut compose = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("engine_v2_world_compose_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: frame_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(WORLD_CLEAR_COLOR),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            compose.set_pipeline(&pass.compose_pipeline);
            compose.set_bind_group(0, &pass.compose_bind_group, &[]);
            compose.draw(0..3, 0..1);
        }
    }

    pub fn encode(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        surface_format: TextureFormat,
        width: u32,
        height: u32,
        shaders: &WorldShaderSources<'_>,
        frame: &WorldRenderFrame,
    ) {
        self.prepare_frame(device, queue, surface_format, width, height, shaders, frame);
        self.encode_compute_pass(encoder, PipelineKey::WorldComputeBasic);
        self.encode_compose_pass(encoder, frame_view, PipelineKey::WorldComposeFullscreen);
    }
}
