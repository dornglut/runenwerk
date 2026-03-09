// Owner: SDF Renderer Example - GPU Pass and Custom Executors
use super::*;

const SDF_CLEAR_COLOR: Color = Color {
    r: 0.02,
    g: 0.02,
    b: 0.03,
    a: 1.0,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SdfWorldParamsRaw {
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
    camera_target_time: [f32; 4],
    camera_orbit: [f32; 4],
    debug_view_mode: u32,
    _pad4: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SdfWorldAgentRaw {
    pos: [f32; 2],
    radius: f32,
    health: f32,
    team: u32,
    _pad0: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SdfWorldModelRaw {
    pos: [f32; 2],
    radius: f32,
    _pad0: f32,
    color: [f32; 4],
}

#[derive(Default)]
pub(super) struct SdfGpuSharedState {
    pass: Option<SdfGpuPass>,
}

struct SdfGpuPass {
    surface_format: TextureFormat,
    size: (u32, u32),
    params_buffer: Buffer,
    agents_buffer: Buffer,
    _models_buffer: Buffer,
    compute_bind_group: BindGroup,
    compose_bind_group: BindGroup,
    compute_pipeline: ComputePipeline,
    compose_pipeline: RenderPipeline,
    _world_texture: Texture,
    _world_texture_view: TextureView,
    _world_sampler: Sampler,
}

fn build_sdf_gpu_pass(
    device: &Device,
    surface_format: TextureFormat,
    size: (u32, u32),
) -> SdfGpuPass {
    let compute_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("sdf_example_compute_shader"),
        source: ShaderSource::Wgsl(SDF_COMPUTE_SHADER.into()),
    });
    let compose_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("sdf_example_compose_shader"),
        source: ShaderSource::Wgsl(SDF_COMPOSE_SHADER.into()),
    });

    let params_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("sdf_example_params_buffer"),
        size: std::mem::size_of::<SdfWorldParamsRaw>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let agents_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("sdf_example_agents_buffer"),
        size: (std::mem::size_of::<SdfWorldAgentRaw>() * SDF_MAX_AGENTS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let models_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("sdf_example_models_buffer"),
        size: (std::mem::size_of::<SdfWorldModelRaw>() * SDF_MAX_MODELS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let world_texture = device.create_texture(&TextureDescriptor {
        label: Some("sdf_example_world_color"),
        size: Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
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
        label: Some("sdf_example_world_sampler"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let compute_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("sdf_example_compute_bind_group_layout"),
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
        label: Some("sdf_example_compute_bind_group"),
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
        label: Some("sdf_example_compute_pipeline_layout"),
        bind_group_layouts: &[&compute_bind_group_layout],
        push_constant_ranges: &[],
    });
    let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("sdf_example_compute_pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &compute_shader,
        entry_point: Some("cs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    });

    let compose_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("sdf_example_compose_bind_group_layout"),
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
        label: Some("sdf_example_compose_bind_group"),
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
        label: Some("sdf_example_compose_pipeline_layout"),
        bind_group_layouts: &[&compose_bind_group_layout],
        push_constant_ranges: &[],
    });
    let compose_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("sdf_example_compose_pipeline"),
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

    SdfGpuPass {
        surface_format,
        size,
        params_buffer,
        agents_buffer,
        _models_buffer: models_buffer,
        compute_bind_group,
        compose_bind_group,
        compute_pipeline,
        compose_pipeline,
        _world_texture: world_texture,
        _world_texture_view: world_texture_view,
        _world_sampler: world_sampler,
    }
}

fn ensure_sdf_gpu_pass(
    shared: &mut SdfGpuSharedState,
    device: &Device,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
) {
    let size = (surface_size.0.max(1), surface_size.1.max(1));
    let needs_rebuild = shared
        .pass
        .as_ref()
        .is_none_or(|pass| pass.surface_format != surface_format || pass.size != size);
    if needs_rebuild {
        shared.pass = Some(build_sdf_gpu_pass(device, surface_format, size));
    }
}

pub(super) struct SdfComputeExecutor {
    shared: Arc<Mutex<SdfGpuSharedState>>,
}

impl SdfComputeExecutor {
    pub(super) fn new(shared: Arc<Mutex<SdfGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for SdfComputeExecutor {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compute shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compute pass unavailable after setup"))?;

        let world_frame = ctx
            .frame_data::<SdfWorldState>()
            .ok_or_else(|| anyhow!("missing SdfWorldState in render pass prepare context"))?;
        let agent_count = world_frame.agents.len().min(SDF_MAX_AGENTS);
        let model_count = 0usize;
        let params = SdfWorldParamsRaw {
            screen_size: [pass.size.0 as f32, pass.size.1 as f32],
            _pad0: [0.0; 2],
            world_min: [world_frame.world_bounds[0], world_frame.world_bounds[1]],
            _pad1: [0.0; 2],
            world_max: [world_frame.world_bounds[2], world_frame.world_bounds[3]],
            _pad2: [0.0; 2],
            agent_count: agent_count as u32,
            model_count: model_count as u32,
            paused: u32::from(world_frame.world_paused),
            _pad3: 0,
            camera_target_time: [
                world_frame.camera_target[0],
                world_frame.camera_target[1],
                world_frame.camera_target[2],
                world_frame.elapsed_time_seconds.max(0.0),
            ],
            camera_orbit: [
                world_frame.camera_yaw,
                world_frame.camera_pitch,
                world_frame.camera_distance.max(0.1),
                world_frame
                    .camera_fov_y
                    .clamp(0.1, std::f32::consts::PI - 0.1),
            ],
            debug_view_mode: world_frame.debug_view_mode,
            _pad4: [0; 3],
        };
        ctx.queue()
            .write_buffer(&pass.params_buffer, 0, bytemuck::bytes_of(&params));

        let mut agents = Vec::with_capacity(agent_count);
        for agent in world_frame.agents.iter().take(agent_count) {
            agents.push(SdfWorldAgentRaw {
                pos: [agent.x, agent.y],
                radius: agent.radius.max(0.2),
                health: agent.health_ratio.clamp(0.0, 1.0),
                team: agent.team,
                _pad0: [0; 3],
            });
        }
        if !agents.is_empty() {
            ctx.queue()
                .write_buffer(&pass.agents_buffer, 0, bytemuck::cast_slice(&agents));
        }

        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compute shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compute pass unavailable during encode"))?;
        let mut compute = ctx.encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("sdf_example.compute"),
            timestamp_writes: None,
        });
        compute.set_pipeline(&pass.compute_pipeline);
        compute.set_bind_group(0, &pass.compute_bind_group, &[]);
        compute.dispatch_workgroups(pass.size.0.div_ceil(8), pass.size.1.div_ceil(8), 1);
        Ok(())
    }
}

pub(super) struct SdfComposeExecutor {
    shared: Arc<Mutex<SdfGpuSharedState>>,
}

impl SdfComposeExecutor {
    pub(super) fn new(shared: Arc<Mutex<SdfGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for SdfComposeExecutor {
    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compose shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compose pass unavailable during encode"))?;
        let frame_view = ctx.frame_view();
        let mut compose = ctx.encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("sdf_example.compose"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(SDF_CLEAR_COLOR),
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
        Ok(())
    }
}

pub(super) struct SdfUiCompositeExecutor;

impl RenderPassExecutor for SdfUiCompositeExecutor {
    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        ctx.run_ui()
    }
}

