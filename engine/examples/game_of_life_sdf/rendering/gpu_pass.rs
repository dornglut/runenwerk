// Owner: Game of Life SDF Example - GPU Pass Construction
use crate::runtime::GameOfLifeSdfState;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Color, ColorTargetState,
    ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, FilterMode, FragmentState,
    FrontFace, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    StorageTextureAccess, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexState,
};

pub(crate) const GOL_SHADER_SOURCE: &str =
    include_str!("../../../../assets/shaders/game_of_life_sdf.wgsl");
pub(crate) const WORKGROUP_SIZE: u32 = 8;
pub(crate) const DEFAULT_CLEAR_COLOR: Color = Color {
    r: 0.03,
    g: 0.045,
    b: 0.042,
    a: 1.0,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct GameOfLifeComputeParamsRaw {
    pub(crate) grid_size: [u32; 2],
    pub(crate) step: u32,
    pub(crate) _pad0: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct GameOfLifeComposeParamsRaw {
    pub(crate) output_size: [f32; 2],
    pub(crate) grid_size: [f32; 2],
    pub(crate) cell_radius: f32,
    pub(crate) edge_softness: f32,
    pub(crate) grid_line_width: f32,
    pub(crate) glow_strength: f32,
    pub(crate) alive_color: [f32; 4],
    pub(crate) dead_color: [f32; 4],
    pub(crate) grid_color: [f32; 4],
    pub(crate) background_color: [f32; 4],
}

#[derive(Default)]
pub(crate) struct GameOfLifeGpuSharedState {
    pub(crate) pass: Option<GameOfLifeGpuPass>,
}

pub(crate) struct GameOfLifeGpuPass {
    pub(crate) surface_format: TextureFormat,
    pub(crate) surface_size: (u32, u32),
    pub(crate) grid_size: (u32, u32),
    pub(crate) phase: usize,
    pub(crate) params_buffer: Buffer,
    pub(crate) compose_params_buffer: Buffer,
    pub(crate) compute_bind_groups: [BindGroup; 2],
    pub(crate) compose_bind_group: BindGroup,
    pub(crate) compute_pipeline: ComputePipeline,
    pub(crate) compose_pipeline: RenderPipeline,
    pub(crate) _cells_a_buffer: Buffer,
    pub(crate) _cells_b_buffer: Buffer,
    pub(crate) _cells_texture: Texture,
    pub(crate) _cells_texture_view: TextureView,
    pub(crate) _cells_sampler: Sampler,
}

pub(crate) fn ensure_game_of_life_gpu_pass(
    shared: &mut GameOfLifeGpuSharedState,
    device: &Device,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    state: &GameOfLifeSdfState,
) {
    let desired_grid_size = clamped_grid_size(state);
    let desired_surface_size = (surface_size.0.max(1), surface_size.1.max(1));

    let needs_rebuild = shared.pass.as_ref().is_none_or(|pass| {
        pass.surface_format != surface_format || pass.grid_size != desired_grid_size
    });
    if needs_rebuild {
        shared.pass = Some(build_game_of_life_gpu_pass(
            device,
            surface_format,
            desired_surface_size,
            state,
        ));
    }
    if let Some(pass) = shared.pass.as_mut() {
        pass.surface_size = desired_surface_size;
    }
}

fn build_game_of_life_gpu_pass(
    device: &Device,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    state: &GameOfLifeSdfState,
) -> GameOfLifeGpuPass {
    let grid_size = clamped_grid_size(state);
    let cells = seed_cells(grid_size, state.initial_alive_density);

    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("gol_sdf_shader"),
        source: ShaderSource::Wgsl(GOL_SHADER_SOURCE.into()),
    });

    let params_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("gol_params_buffer"),
        size: std::mem::size_of::<GameOfLifeComputeParamsRaw>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let compose_params_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("gol_compose_params_buffer"),
        size: std::mem::size_of::<GameOfLifeComposeParamsRaw>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let cells_a_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("gol_cells_a_buffer"),
        contents: bytemuck::cast_slice(&cells),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });
    let cells_b_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("gol_cells_b_buffer"),
        contents: bytemuck::cast_slice(&cells),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    let cells_texture = device.create_texture(&TextureDescriptor {
        label: Some("gol_cells_texture"),
        size: Extent3d {
            width: grid_size.0,
            height: grid_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let cells_texture_view = cells_texture.create_view(&TextureViewDescriptor::default());
    let cells_sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("gol_cells_sampler"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let compute_bind_group_layout = compute_bind_group_layout(device);
    let compute_bind_group_a_to_b = create_compute_bind_group(
        device,
        "gol_compute_bind_group_a_to_b",
        &compute_bind_group_layout,
        &params_buffer,
        &cells_a_buffer,
        &cells_b_buffer,
        &cells_texture_view,
    );
    let compute_bind_group_b_to_a = create_compute_bind_group(
        device,
        "gol_compute_bind_group_b_to_a",
        &compute_bind_group_layout,
        &params_buffer,
        &cells_b_buffer,
        &cells_a_buffer,
        &cells_texture_view,
    );

    let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("gol_compute_pipeline_layout"),
        bind_group_layouts: &[&compute_bind_group_layout],
        push_constant_ranges: &[],
    });
    let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("gol_compute_pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    });

    let compose_bind_group_layout = compose_bind_group_layout(device);
    let compose_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("gol_compose_bind_group"),
        layout: &compose_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 4,
                resource: BindingResource::TextureView(&cells_texture_view),
            },
            BindGroupEntry {
                binding: 5,
                resource: BindingResource::Sampler(&cells_sampler),
            },
            BindGroupEntry {
                binding: 6,
                resource: compose_params_buffer.as_entire_binding(),
            },
        ],
    });
    let compose_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("gol_compose_pipeline_layout"),
        bind_group_layouts: &[&compose_bind_group_layout],
        push_constant_ranges: &[],
    });
    let compose_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("gol_compose_pipeline"),
        layout: Some(&compose_pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format: surface_format,
                blend: Some(BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
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

    GameOfLifeGpuPass {
        surface_format,
        surface_size,
        grid_size,
        phase: 0,
        params_buffer,
        compose_params_buffer,
        compute_bind_groups: [compute_bind_group_a_to_b, compute_bind_group_b_to_a],
        compose_bind_group,
        compute_pipeline,
        compose_pipeline,
        _cells_a_buffer: cells_a_buffer,
        _cells_b_buffer: cells_b_buffer,
        _cells_texture: cells_texture,
        _cells_texture_view: cells_texture_view,
        _cells_sampler: cells_sampler,
    }
}

fn clamped_grid_size(state: &GameOfLifeSdfState) -> (u32, u32) {
    (
        state.grid_size[0].clamp(8, 512),
        state.grid_size[1].clamp(8, 512),
    )
}

fn compute_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("gol_compute_bind_group_layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::Rgba8Unorm,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    })
}

fn compose_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("gol_compose_bind_group_layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 5,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 6,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn create_compute_bind_group(
    device: &Device,
    label: &str,
    layout: &BindGroupLayout,
    params_buffer: &Buffer,
    cells_in: &Buffer,
    cells_out: &Buffer,
    cells_texture_view: &TextureView,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: cells_in.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: cells_out.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::TextureView(cells_texture_view),
            },
        ],
    })
}

fn seed_cells(grid_size: (u32, u32), density: f32) -> Vec<u32> {
    let count = (grid_size.0 as usize).saturating_mul(grid_size.1 as usize);
    let threshold = (density.clamp(0.0, 1.0) * 1024.0).round() as u32;

    let mut cells = Vec::with_capacity(count);
    for index in 0..count {
        let hash = hash_u32(index as u32 ^ 0xA512_9EC3);
        let alive = u32::from((hash & 1023) <= threshold);
        cells.push(alive);
    }

    stamp_glider(&mut cells, grid_size, 12, 12);
    stamp_glider(&mut cells, grid_size, 46, 28);
    stamp_glider(&mut cells, grid_size, 84, 51);
    cells
}

fn stamp_glider(cells: &mut [u32], grid_size: (u32, u32), origin_x: u32, origin_y: u32) {
    const GLIDER: &[(u32, u32)] = &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];

    for (dx, dy) in GLIDER {
        let x = origin_x.saturating_add(*dx);
        let y = origin_y.saturating_add(*dy);
        if x < grid_size.0 && y < grid_size.1 {
            let idx = (y as usize)
                .saturating_mul(grid_size.0 as usize)
                .saturating_add(x as usize);
            if let Some(cell) = cells.get_mut(idx) {
                *cell = 1;
            }
        }
    }
}

fn hash_u32(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7FEB_352D);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846C_A68B);
    value ^ (value >> 16)
}
