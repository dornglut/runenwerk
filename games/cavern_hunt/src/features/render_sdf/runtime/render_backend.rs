use super::*;

// Owner: Cavern Hunt SDF Renderer - Render Backend and GPU Executors
pub(crate) fn project_mouse_to_world(
    camera: &CavernCameraState,
    window: &WindowState,
    layout: &CavernLayout,
    cursor: (f32, f32),
) -> [f32; 2] {
    let size = (
        window.size_px.0.max(1) as f32,
        window.size_px.1.max(1) as f32,
    );
    let aspect = size.0 / size.1.max(1.0);
    let view_h = camera.distance * 0.55;
    let view_w = view_h * aspect;
    let ndc_x = (cursor.0 / size.0) * 2.0 - 1.0;
    let ndc_y = 1.0 - (cursor.1 / size.1) * 2.0;
    [
        (camera.target[0] + ndc_x * view_w).clamp(layout.world_bounds[0], layout.world_bounds[2]),
        (camera.target[2] - ndc_y * view_h).clamp(layout.world_bounds[1], layout.world_bounds[3]),
    ]
}

pub(super) fn build_feature_graph_spec() -> Result<RenderFeatureGraphSpec> {
    let mut builder = RenderFeatureGraphSpec::builder(FEATURE_ID)
        .resource("cavern.params")
        .resource("cavern.primitives")
        .resource("cavern.agents")
        .resource("cavern.color")
        .resource("surface.color")
        .resource("ui.draw_list")
        .pipeline_compute("cavern_hunt.compute.raymarch", SHADER_PATH)
        .pipeline_render_builtin("cavern_hunt.compose.fullscreen", "compose.fullscreen")
        .pipeline_render_builtin("ui.compose", "ui.composite");

    builder = builder
        .compute_pass("cavern_hunt.compute")
        .pipeline("cavern_hunt.compute.raymarch")
        .executor(COMPUTE_EXECUTOR_ID)
        .reads(["cavern.params", "cavern.primitives", "cavern.agents"])
        .writes(["cavern.color"])
        .finish();
    builder = builder
        .render_pass("cavern_hunt.compose")
        .pipeline("cavern_hunt.compose.fullscreen")
        .executor(COMPOSE_EXECUTOR_ID)
        .reads(["cavern.color"])
        .writes(["surface.color"])
        .depends_on(["cavern_hunt.compute"])
        .finish();
    builder = builder
        .render_pass("ui_composite")
        .pipeline("ui.compose")
        .executor_builtin_ui_composite()
        .reads(["ui.draw_list"])
        .writes(["surface.color"])
        .depends_on(["cavern_hunt.compose"])
        .finish();
    builder.build()
}

fn build_gpu_pass(
    device: &Device,
    surface_format: TextureFormat,
    size: (u32, u32),
) -> CavernGpuPass {
    let compute_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("cavern_hunt_compute_shader"),
        source: ShaderSource::Wgsl(COMPUTE_SHADER.into()),
    });
    let compose_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("cavern_hunt_compose_shader"),
        source: ShaderSource::Wgsl(COMPOSE_SHADER.into()),
    });

    let params_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_params_buffer"),
        size: std::mem::size_of::<CavernWorldParamsRaw>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let primitives_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_geometry_primitives_buffer"),
        size: (std::mem::size_of::<CavernGeometryPrimitiveRaw>() * MAX_GEOMETRY_PRIMITIVES) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let agents_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_agents_buffer"),
        size: (std::mem::size_of::<CavernAgentRaw>() * MAX_AGENTS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let material_program_headers_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_material_program_headers_buffer"),
        size: (std::mem::size_of::<CavernMaterialProgramHeaderRaw>() * MAX_MATERIAL_PROGRAMS)
            as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let material_ops_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_material_ops_buffer"),
        size: (std::mem::size_of::<CavernMaterialOpRaw>() * MAX_MATERIAL_OPS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let material_constants_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_material_constants_buffer"),
        size: (std::mem::size_of::<[f32; 4]>() * MAX_MATERIAL_CONSTANTS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let world_texture = device.create_texture(&TextureDescriptor {
        label: Some("cavern_hunt_world_texture"),
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
        label: Some("cavern_hunt_world_sampler"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let compute_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("cavern_hunt_compute_layout"),
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
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 5,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 6,
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
        label: Some("cavern_hunt_compute_bind_group"),
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
                resource: primitives_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: agents_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 4,
                resource: material_program_headers_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 5,
                resource: material_ops_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 6,
                resource: material_constants_buffer.as_entire_binding(),
            },
        ],
    });
    let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("cavern_hunt_compute_pipeline_layout"),
        bind_group_layouts: &[&compute_bind_group_layout],
        push_constant_ranges: &[],
    });
    let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("cavern_hunt_compute_pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &compute_shader,
        entry_point: Some("cs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    });

    let compose_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("cavern_hunt_compose_layout"),
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
        label: Some("cavern_hunt_compose_bind_group"),
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
        label: Some("cavern_hunt_compose_pipeline_layout"),
        bind_group_layouts: &[&compose_bind_group_layout],
        push_constant_ranges: &[],
    });
    let compose_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("cavern_hunt_compose_pipeline"),
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

    CavernGpuPass {
        surface_format,
        size,
        params_buffer,
        primitives_buffer,
        agents_buffer,
        material_program_headers_buffer,
        material_ops_buffer,
        material_constants_buffer,
        compute_bind_group,
        compose_bind_group,
        compute_pipeline,
        compose_pipeline,
        _world_texture: world_texture,
        _world_texture_view: world_texture_view,
        _world_sampler: world_sampler,
    }
}

#[path = "render_backend_executors.rs"]
mod render_backend_executors;

pub(super) use render_backend_executors::{CavernComposeExecutor, CavernComputeExecutor};
