use crate::domain::{
    CavernCameraState, CavernGeometryGraph, CavernLayout, CavernSdfAgent, CavernSdfBlocker,
    CavernSdfWorldFrame, CavernTopology, Chest, ColliderRadius, EnemyKind, ExtractionZone,
    GeometryOp, GeometryPrimitiveShape3, Health, LocalPlayerRef, LootDrop, Pickup, Player,
    PlayerCompanion, PlayerSpectator, Projectile, ProjectileVisualState, Transform2,
    is_active_player_entity,
};
use anyhow::{Result, anyhow};
use bytemuck::{Pod, Zeroable};
use engine::plugins::render::domain::{
    RenderFeatureGraphSpec, RenderFrameResourceBindings, RenderGraphRegistryResource,
    RenderPassEncodeContext, RenderPassExecutor, RenderPassExecutorRegistryResource,
    RenderPassPrepareContext,
};
use engine::prelude::{Entity, InputState, Res, ResMut, Time, WindowState, World, WorldRef};
use std::sync::{Arc, Mutex};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, FilterMode, FragmentState,
    FrontFace, LoadOp, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StorageTextureAccess, StoreOp, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension, VertexState,
};

const FEATURE_ID: &str = "cavern_hunt";
const COMPUTE_EXECUTOR_ID: &str = "cavern_hunt.compute";
const COMPOSE_EXECUTOR_ID: &str = "cavern_hunt.compose";
const SHADER_PATH: &str = "assets/shaders/cavern_hunt_sdf.wgsl";
const COMPOSE_SHADER: &str = include_str!("../../../assets/shaders/world_compose_fullscreen.wgsl");
const COMPUTE_SHADER: &str = include_str!("../../../assets/shaders/cavern_hunt_sdf.wgsl");
const MAX_ROOMS: usize = 16;
const MAX_TUNNELS: usize = 24;
const MAX_BLOCKERS: usize = 64;
const MAX_AGENTS: usize = 96;
const CLEAR_COLOR: Color = Color {
    r: 0.01,
    g: 0.015,
    b: 0.02,
    a: 1.0,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CavernWorldParamsRaw {
    screen_size: [f32; 2],
    _pad0: [f32; 2],
    world_min: [f32; 2],
    _pad1: [f32; 2],
    world_max: [f32; 2],
    _pad2: [f32; 2],
    room_count: u32,
    tunnel_count: u32,
    blocker_count: u32,
    agent_count: u32,
    floor_rock_height: [f32; 4],
    camera_target_time: [f32; 4],
    camera_orbit: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CavernRoomRaw {
    center: [f32; 2],
    radii: [f32; 2],
    role: u32,
    _pad0: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CavernTunnelRaw {
    start: [f32; 2],
    end: [f32; 2],
    radius: f32,
    _pad0: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CavernAgentRaw {
    pos: [f32; 2],
    radius: f32,
    health: f32,
    team: u32,
    kind: u32,
    _pad0: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CavernBlockerRaw {
    center: [f32; 3],
    radius: f32,
    half_height: f32,
    _pad0: [f32; 3],
}

#[derive(Default)]
struct CavernGpuSharedState {
    pass: Option<CavernGpuPass>,
}

struct CavernGpuPass {
    surface_format: TextureFormat,
    size: (u32, u32),
    params_buffer: Buffer,
    rooms_buffer: Buffer,
    tunnels_buffer: Buffer,
    blockers_buffer: Buffer,
    agents_buffer: Buffer,
    compute_bind_group: BindGroup,
    compose_bind_group: BindGroup,
    compute_pipeline: ComputePipeline,
    compose_pipeline: RenderPipeline,
    _world_texture: Texture,
    _world_texture_view: TextureView,
    _world_sampler: Sampler,
}

pub(crate) fn setup_render_resources(world: &mut World) -> Result<()> {
    let mut frame_bindings = world
        .resource_mut::<RenderFrameResourceBindings>()
        .map_err(|_| anyhow!("RenderPlugin must be installed before Cavern Hunt client plugin"))?;
    if !frame_bindings.contains_resource::<CavernSdfWorldFrame>() {
        frame_bindings.register_resource::<CavernSdfWorldFrame>();
    }
    drop(frame_bindings);

    let spec = build_feature_graph_spec()?;
    world
        .resource_mut::<RenderGraphRegistryResource>()?
        .register_feature_graph(spec);

    let shared = Arc::new(Mutex::new(CavernGpuSharedState::default()));
    let mut executors = world.resource_mut::<RenderPassExecutorRegistryResource>()?;
    executors.register_custom(
        COMPUTE_EXECUTOR_ID,
        Arc::new(CavernComputeExecutor::new(Arc::clone(&shared))),
    );
    executors.register_custom(
        COMPOSE_EXECUTOR_ID,
        Arc::new(CavernComposeExecutor::new(shared)),
    );
    Ok(())
}

pub(crate) fn update_camera_and_hud_system(
    world: WorldRef,
    input: Res<InputState>,
    _time: Res<Time>,
    mut camera: ResMut<CavernCameraState>,
) -> Result<()> {
    let local_player_ref = world.resource::<LocalPlayerRef>()?;
    let local_player = local_player_ref.entity.and_then(|entity| {
        world.get::<Transform2>(entity).copied().map(|transform| {
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            (transform, health)
        })
    });
    let Some((transform, _health)) = local_player else {
        return Ok(());
    };

    let zoom_delta = input.scroll_delta * 1.5;
    if zoom_delta.abs() > f32::EPSILON {
        camera.distance =
            (camera.distance - zoom_delta).clamp(camera.distance_min, camera.distance_max);
    }

    camera.target = [transform.x, 1.55, transform.y];

    Ok(())
}

pub(crate) fn build_sdf_world_frame_system(
    world: WorldRef,
    layout: Res<CavernLayout>,
    camera: Res<CavernCameraState>,
    mut frame: ResMut<CavernSdfWorldFrame>,
) -> Result<()> {
    let render_layout = world
        .resource::<CavernTopology>()
        .map(|topology| topology.to_layout_2d())
        .unwrap_or_else(|_| layout.clone());
    frame.rebuild_from_layout(&render_layout, &camera);
    if let Ok(graph) = world.resource::<CavernGeometryGraph>() {
        for primitive in graph
            .primitives
            .iter()
            .filter(|primitive| primitive.enabled && primitive.op == GeometryOp::Blocker)
        {
            if let Some(blocker) = blocker_from_shape(&primitive.shape) {
                frame.blockers.push(blocker);
            }
        }
    }

    for (entity, transform) in world.query::<(Entity, &Transform2)>().iter() {
        if is_active_player_entity(&world, entity) {
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            let radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.45))
                .0;
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: radius,
                health_ratio: health.ratio(),
                team: if world.get::<PlayerCompanion>(entity).is_some() {
                    4
                } else {
                    0
                },
                kind: if world.get::<PlayerSpectator>(entity).is_some() {
                    13
                } else if world.get::<PlayerCompanion>(entity).is_some() {
                    12
                } else {
                    0
                },
            });
            continue;
        }

        if let Some(kind) = world.get::<EnemyKind>(entity).copied() {
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            let radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(match kind {
                    EnemyKind::Swarmer => ColliderRadius(0.42),
                    EnemyKind::Bruiser => ColliderRadius(0.78),
                    EnemyKind::Spitter => ColliderRadius(0.58),
                    EnemyKind::NestGuardian => ColliderRadius(0.92),
                })
                .0;
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius,
                health_ratio: health.ratio(),
                team: 1,
                kind: match kind {
                    EnemyKind::Swarmer => 1,
                    EnemyKind::Bruiser => 2,
                    EnemyKind::Spitter => 3,
                    EnemyKind::NestGuardian => 4,
                },
            });
            continue;
        }

        if let Some(pickup) = world.get::<Pickup>(entity).copied() {
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: if world.get::<LootDrop>(entity).is_some() {
                    0.34
                } else {
                    0.48
                },
                health_ratio: 1.0,
                team: 2,
                kind: match pickup.kind {
                    crate::domain::PickupKind::Scrap(_) => 7,
                    crate::domain::PickupKind::WeaponMod(_) => 8,
                    crate::domain::PickupKind::Relic(_) => 9,
                    crate::domain::PickupKind::HealingCharge(_) => 10,
                },
            });
            continue;
        }

        if world.get::<Chest>(entity).is_some() {
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: 0.55,
                health_ratio: 1.0,
                team: 2,
                kind: 5,
            });
            continue;
        }

        if world.get::<Projectile>(entity).is_some() {
            let team = if world.get::<Player>(entity).is_some() {
                0
            } else {
                world
                    .get::<crate::domain::Faction>(entity)
                    .map(|faction| {
                        if *faction == crate::domain::Faction::Hunters {
                            0
                        } else {
                            1
                        }
                    })
                    .unwrap_or(1)
            };
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: if let Some(visual) = world.get::<ProjectileVisualState>(entity) {
                    (0.16 + visual.life_elapsed_seconds.min(0.12) * 0.3).max(0.16)
                } else {
                    0.16
                },
                health_ratio: 1.0,
                team,
                kind: 11,
            });
            continue;
        }

        if world.get::<ExtractionZone>(entity).is_some() {
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: 1.15,
                health_ratio: 1.0,
                team: 3,
                kind: 6,
            });
        }
    }

    Ok(())
}

fn blocker_from_shape(shape: &GeometryPrimitiveShape3) -> Option<CavernSdfBlocker> {
    match shape {
        GeometryPrimitiveShape3::Cylinder {
            center,
            radius,
            half_height,
        } => Some(CavernSdfBlocker {
            center: *center,
            radius: *radius,
            half_height: *half_height,
        }),
        GeometryPrimitiveShape3::Sphere { center, radius } => Some(CavernSdfBlocker {
            center: *center,
            radius: *radius,
            half_height: *radius,
        }),
        GeometryPrimitiveShape3::Capsule { start, end, radius } => {
            let center = [
                (start[0] + end[0]) * 0.5,
                (start[1] + end[1]) * 0.5,
                (start[2] + end[2]) * 0.5,
            ];
            let half_height = ((start[1] - end[1]).abs() * 0.5 + *radius).max(0.1);
            Some(CavernSdfBlocker {
                center,
                radius: *radius,
                half_height,
            })
        }
        GeometryPrimitiveShape3::Box {
            center,
            half_extents,
        } => Some(CavernSdfBlocker {
            center: *center,
            radius: half_extents[0].max(half_extents[2]).max(0.1),
            half_height: half_extents[1].max(0.1),
        }),
        GeometryPrimitiveShape3::RoundedBox {
            center,
            half_extents,
            radius,
        } => Some(CavernSdfBlocker {
            center: *center,
            radius: (half_extents[0].max(half_extents[2]) + *radius).max(0.1),
            half_height: (half_extents[1] + *radius).max(0.1),
        }),
        GeometryPrimitiveShape3::Ellipsoid { center, radii } => Some(CavernSdfBlocker {
            center: *center,
            radius: radii[0].max(radii[2]).max(0.1),
            half_height: radii[1].max(0.1),
        }),
        GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
            let center = points.first().copied()?;
            Some(CavernSdfBlocker {
                center,
                radius: *radius,
                half_height: *radius,
            })
        }
    }
}

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

fn build_feature_graph_spec() -> Result<RenderFeatureGraphSpec> {
    let mut builder = RenderFeatureGraphSpec::builder(FEATURE_ID)
        .resource("cavern.params")
        .resource("cavern.rooms")
        .resource("cavern.tunnels")
        .resource("cavern.blockers")
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
        .reads([
            "cavern.params",
            "cavern.rooms",
            "cavern.tunnels",
            "cavern.blockers",
            "cavern.agents",
        ])
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
    let rooms_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_rooms_buffer"),
        size: (std::mem::size_of::<CavernRoomRaw>() * MAX_ROOMS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let tunnels_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_tunnels_buffer"),
        size: (std::mem::size_of::<CavernTunnelRaw>() * MAX_TUNNELS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let blockers_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_blockers_buffer"),
        size: (std::mem::size_of::<CavernBlockerRaw>() * MAX_BLOCKERS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let agents_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("cavern_hunt_agents_buffer"),
        size: (std::mem::size_of::<CavernAgentRaw>() * MAX_AGENTS) as u64,
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
                resource: rooms_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: tunnels_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 4,
                resource: agents_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 5,
                resource: blockers_buffer.as_entire_binding(),
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
        rooms_buffer,
        tunnels_buffer,
        blockers_buffer,
        agents_buffer,
        compute_bind_group,
        compose_bind_group,
        compute_pipeline,
        compose_pipeline,
        _world_texture: world_texture,
        _world_texture_view: world_texture_view,
        _world_sampler: world_sampler,
    }
}

fn ensure_gpu_pass(
    shared: &mut CavernGpuSharedState,
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
        shared.pass = Some(build_gpu_pass(device, surface_format, size));
    }
}

struct CavernComputeExecutor {
    shared: Arc<Mutex<CavernGpuSharedState>>,
}

impl CavernComputeExecutor {
    fn new(shared: Arc<Mutex<CavernGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for CavernComputeExecutor {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("cavern gpu shared state lock poisoned"))?;
        ensure_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("cavern gpu pass unavailable after setup"))?;

        let frame = ctx
            .frame_data::<CavernSdfWorldFrame>()
            .ok_or_else(|| anyhow!("missing CavernSdfWorldFrame in render prepare context"))?;
        let params = CavernWorldParamsRaw {
            screen_size: [pass.size.0 as f32, pass.size.1 as f32],
            _pad0: [0.0; 2],
            world_min: [frame.world_bounds[0], frame.world_bounds[1]],
            _pad1: [0.0; 2],
            world_max: [frame.world_bounds[2], frame.world_bounds[3]],
            _pad2: [0.0; 2],
            room_count: frame.rooms.len().min(MAX_ROOMS) as u32,
            tunnel_count: frame.tunnels.len().min(MAX_TUNNELS) as u32,
            blocker_count: frame.blockers.len().min(MAX_BLOCKERS) as u32,
            agent_count: frame.agents.len().min(MAX_AGENTS) as u32,
            floor_rock_height: [frame.floor_height, frame.rock_height, 0.0, 0.0],
            camera_target_time: [
                frame.camera.target[0],
                frame.camera.target[1],
                frame.camera.target[2],
                0.0,
            ],
            camera_orbit: [
                frame.camera.yaw,
                frame.camera.pitch,
                frame.camera.distance,
                frame.camera.fov_y_radians,
            ],
        };
        ctx.queue()
            .write_buffer(&pass.params_buffer, 0, bytemuck::bytes_of(&params));

        let rooms = frame
            .rooms
            .iter()
            .take(MAX_ROOMS)
            .map(|room| CavernRoomRaw {
                center: room.center,
                radii: room.radii,
                role: match room.role {
                    crate::domain::RoomRole::Start => 0,
                    crate::domain::RoomRole::Combat => 1,
                    crate::domain::RoomRole::Loot => 2,
                    crate::domain::RoomRole::Fork => 3,
                    crate::domain::RoomRole::Elite => 4,
                    crate::domain::RoomRole::Extraction => 5,
                },
                _pad0: [0; 3],
            })
            .collect::<Vec<_>>();
        if !rooms.is_empty() {
            ctx.queue()
                .write_buffer(&pass.rooms_buffer, 0, bytemuck::cast_slice(&rooms));
        }

        let tunnels = frame
            .tunnels
            .iter()
            .take(MAX_TUNNELS)
            .map(|tunnel| CavernTunnelRaw {
                start: tunnel.start,
                end: tunnel.end,
                radius: tunnel.radius,
                _pad0: 0.0,
            })
            .collect::<Vec<_>>();
        if !tunnels.is_empty() {
            ctx.queue()
                .write_buffer(&pass.tunnels_buffer, 0, bytemuck::cast_slice(&tunnels));
        }

        let blockers = frame
            .blockers
            .iter()
            .take(MAX_BLOCKERS)
            .map(|blocker| CavernBlockerRaw {
                center: blocker.center,
                radius: blocker.radius,
                half_height: blocker.half_height,
                _pad0: [0.0; 3],
            })
            .collect::<Vec<_>>();
        if !blockers.is_empty() {
            ctx.queue()
                .write_buffer(&pass.blockers_buffer, 0, bytemuck::cast_slice(&blockers));
        }

        let agents = frame
            .agents
            .iter()
            .take(MAX_AGENTS)
            .map(|agent| CavernAgentRaw {
                pos: agent.pos,
                radius: agent.radius,
                health: agent.health_ratio,
                team: agent.team,
                kind: agent.kind,
                _pad0: [0; 2],
            })
            .collect::<Vec<_>>();
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
            .map_err(|_| anyhow!("cavern gpu shared state lock poisoned"))?;
        ensure_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("cavern gpu pass unavailable during encode"))?;
        let mut compute = ctx.encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("cavern_hunt.compute"),
            timestamp_writes: None,
        });
        compute.set_pipeline(&pass.compute_pipeline);
        compute.set_bind_group(0, &pass.compute_bind_group, &[]);
        compute.dispatch_workgroups(pass.size.0.div_ceil(8), pass.size.1.div_ceil(8), 1);
        Ok(())
    }
}

struct CavernComposeExecutor {
    shared: Arc<Mutex<CavernGpuSharedState>>,
}

impl CavernComposeExecutor {
    fn new(shared: Arc<Mutex<CavernGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for CavernComposeExecutor {
    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("cavern gpu shared state lock poisoned"))?;
        ensure_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("cavern gpu pass unavailable during encode"))?;
        let frame_view = ctx.frame_view();
        let mut render = ctx.encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("cavern_hunt.compose"),
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
        render.set_pipeline(&pass.compose_pipeline);
        render.set_bind_group(0, &pass.compose_bind_group, &[]);
        render.draw(0..3, 0..1);
        Ok(())
    }
}
