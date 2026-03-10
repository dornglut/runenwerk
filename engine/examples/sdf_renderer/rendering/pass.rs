// Owner: SDF Renderer Example - GPU Pass Construction
use crate::*;

pub(crate) const SDF_CLEAR_COLOR: Color = Color {
    r: 0.02,
    g: 0.02,
    b: 0.03,
    a: 1.0,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct SdfWorldParamsRaw {
    pub(crate) screen_size: [f32; 2],
    pub(crate) _pad0: [f32; 2],
    pub(crate) world_min: [f32; 2],
    pub(crate) _pad1: [f32; 2],
    pub(crate) world_max: [f32; 2],
    pub(crate) _pad2: [f32; 2],
    pub(crate) agent_count: u32,
    pub(crate) model_count: u32,
    pub(crate) paused: u32,
    pub(crate) _pad3: u32,
    pub(crate) camera_target_time: [f32; 4],
    pub(crate) camera_orbit: [f32; 4],
    pub(crate) debug_view_mode: u32,
    pub(crate) display_fit_mode: u32,
    pub(crate) display_target_aspect: f32,
    pub(crate) _pad4: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct SdfWorldAgentRaw {
    pub(crate) pos: [f32; 2],
    pub(crate) radius: f32,
    pub(crate) health: f32,
    pub(crate) team: u32,
    pub(crate) _pad0: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct SdfWorldModelRaw {
    pub(crate) pos: [f32; 2],
    pub(crate) radius: f32,
    pub(crate) _pad0: f32,
    pub(crate) color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct SdfComposeParamsRaw {
    pub(crate) output_size: [f32; 2],
    pub(crate) target_aspect: f32,
    pub(crate) fit_mode: u32,
    pub(crate) bar_color: [f32; 4],
}

#[derive(Default)]
pub(crate) struct SdfGpuSharedState {
    pub(crate) pass: Option<SdfGpuPass>,
}

pub(crate) struct SdfGpuPass {
    pub(crate) surface_format: TextureFormat,
    pub(crate) size: (u32, u32),
    pub(crate) params_buffer: Buffer,
    pub(crate) agents_buffer: Buffer,
    pub(crate) _models_buffer: Buffer,
    pub(crate) compose_params_buffer: Buffer,
    pub(crate) compute_bind_group: BindGroup,
    pub(crate) compose_bind_group: BindGroup,
    pub(crate) compute_pipeline: ComputePipeline,
    pub(crate) compose_pipeline: RenderPipeline,
    pub(crate) _world_texture: Texture,
    pub(crate) _world_texture_view: TextureView,
    pub(crate) _world_sampler: Sampler,
}

pub(crate) fn build_sdf_gpu_pass(
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
    let compose_params_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("sdf_example_compose_params_buffer"),
        size: std::mem::size_of::<SdfComposeParamsRaw>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
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
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
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
            BindGroupEntry {
                binding: 2,
                resource: compose_params_buffer.as_entire_binding(),
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
        compose_params_buffer,
        compute_bind_group,
        compose_bind_group,
        compute_pipeline,
        compose_pipeline,
        _world_texture: world_texture,
        _world_texture_view: world_texture_view,
        _world_sampler: world_sampler,
    }
}

pub(crate) fn ensure_sdf_gpu_pass(
    shared: &mut SdfGpuSharedState,
    device: &Device,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    render_scale: f32,
) {
    let scale = render_scale.clamp(0.25, 4.0);
    let size = (
        ((surface_size.0.max(1) as f32) * scale).round() as u32,
        ((surface_size.1.max(1) as f32) * scale).round() as u32,
    );
    let size = (size.0.max(1), size.1.max(1));
    let needs_rebuild = shared
        .pass
        .as_ref()
        .is_none_or(|pass| pass.surface_format != surface_format || pass.size != size);
    if needs_rebuild {
        shared.pass = Some(build_sdf_gpu_pass(device, surface_format, size));
    }
}
