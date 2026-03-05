use crate::domain::{
    CavernCameraState, CavernGeometryGraph, CavernLayout, CavernMaterialQualityConfig,
    CavernMaterialRuntimeState, CavernSdfAgent, CavernSdfGeometryPrimitive, CavernSdfMaterialOp,
    CavernSdfWorldFrame, CavernTopology, Chest, ColliderRadius, EnemyKind, ExtractionZone,
    GeometryMaterial, GeometryOp, GeometryPrimitiveShape3, Health, LocalPlayerRef, LootDrop,
    Pickup, Player, PlayerCompanion, PlayerId, PlayerSpectator, Projectile, ProjectileVisualState,
    Transform2, is_active_player_entity,
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
const MAX_GEOMETRY_PRIMITIVES: usize = 384;
const MAX_AGENTS: usize = 96;
const MAX_MATERIAL_PROGRAMS: usize = 16;
const MAX_MATERIAL_OPS: usize = 512;
const MAX_MATERIAL_CONSTANTS: usize = 256;
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
    primitive_count: u32,
    agent_count: u32,
    material_program_count: u32,
    material_op_count: u32,
    material_constant_count: u32,
    render_mode: u32,
    gi_mode: u32,
    gi_quality: u32,
    gi_sample_budget: u32,
    _pad3: [u32; 3],
    floor_rock_height: [f32; 4],
    camera_target_time: [f32; 4],
    camera_orbit: [f32; 4],
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
struct CavernGeometryPrimitiveRaw {
    shape_kind: u32,
    op_kind: u32,
    material_class: u32,
    material_instance: u32,
    p0: [f32; 4],
    p1: [f32; 4],
    p2: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CavernMaterialProgramHeaderRaw {
    class_id: u32,
    op_offset: u32,
    op_count: u32,
    const_offset: u32,
    const_count: u32,
    base_color_slot: u32,
    roughness_slot: u32,
    metallic_slot: u32,
    normal_perturb_slot: u32,
    ao_slot: u32,
    emissive_slot: u32,
    _pad0: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CavernMaterialOpRaw {
    op: u32,
    dst: u32,
    src_a: u32,
    src_b: u32,
    src_c: u32,
    const_idx: u32,
    flags: u32,
    _pad0: u32,
}

#[derive(Default)]
struct CavernGpuSharedState {
    pass: Option<CavernGpuPass>,
}

struct CavernGpuPass {
    surface_format: TextureFormat,
    size: (u32, u32),
    params_buffer: Buffer,
    primitives_buffer: Buffer,
    agents_buffer: Buffer,
    material_program_headers_buffer: Buffer,
    material_ops_buffer: Buffer,
    material_constants_buffer: Buffer,
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
    let (world_bounds, geometry_primitives) =
        if let Ok(graph) = world.resource::<CavernGeometryGraph>() {
            (
                [
                    graph.bounds.min[0],
                    graph.bounds.min[2],
                    graph.bounds.max[0],
                    graph.bounds.max[2],
                ],
                geometry_primitives_from_graph(&graph),
            )
        } else if let Ok(topology) = world.resource::<CavernTopology>() {
            (
                [
                    topology.world_bounds.min[0],
                    topology.world_bounds.min[2],
                    topology.world_bounds.max[0],
                    topology.world_bounds.max[2],
                ],
                geometry_primitives_from_topology(&topology),
            )
        } else {
            (
                layout.world_bounds,
                geometry_primitives_from_layout(&layout),
            )
        };

    frame.world_bounds = world_bounds;
    frame.camera = camera.clone();
    frame.material_program_headers.clear();
    frame.material_ops.clear();
    frame.material_constants.clear();
    frame.agents.clear();
    frame.geometry_primitives = geometry_primitives;

    if let Ok(quality) = world.resource::<CavernMaterialQualityConfig>() {
        frame.render_mode = quality.render_mode.as_gpu_u32();
        frame.gi_mode = quality.gi.mode.as_gpu_u32();
        frame.gi_quality = quality.gi.quality.as_gpu_u32();
        frame.gi_sample_budget = quality.gi.sample_budget.max(1);
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
            let player_palette_slot = world
                .get::<PlayerId>(entity)
                .map(|player_id| player_id.0.saturating_sub(1) % 8)
                .unwrap_or(0);
            frame.agents.push(CavernSdfAgent {
                pos: [transform.x, transform.y],
                radius: radius,
                health_ratio: health.ratio(),
                team: player_palette_slot,
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

    if let Ok(runtime) = world.resource::<CavernMaterialRuntimeState>() {
        let payload = runtime.build_gpu_payload(
            MAX_MATERIAL_PROGRAMS,
            MAX_MATERIAL_OPS,
            MAX_MATERIAL_CONSTANTS,
        );
        frame.material_program_headers = payload
            .headers
            .iter()
            .map(|header| crate::domain::CavernSdfMaterialProgramHeader {
                class_id: header.class_id,
                op_offset: header.op_offset,
                op_count: header.op_count,
                const_offset: header.const_offset,
                const_count: header.const_count,
                base_color_slot: header.base_color_slot,
                roughness_slot: header.roughness_slot,
                metallic_slot: header.metallic_slot,
                normal_perturb_slot: header.normal_perturb_slot,
                ao_slot: header.ao_slot,
                emissive_slot: header.emissive_slot,
            })
            .collect();
        frame.material_ops = payload
            .ops
            .iter()
            .map(|op| CavernSdfMaterialOp {
                op: op.op,
                dst: op.dst,
                src_a: op.src_a,
                src_b: op.src_b,
                src_c: op.src_c,
                const_idx: op.const_idx,
                flags: op.flags,
            })
            .collect();
        frame.material_constants = payload.constants;
    }

    Ok(())
}

const SHAPE_SPHERE: u32 = 0;
const SHAPE_ELLIPSOID: u32 = 1;
const SHAPE_CAPSULE: u32 = 2;
const SHAPE_BOX: u32 = 3;
const SHAPE_ROUNDED_BOX: u32 = 4;
const SHAPE_CYLINDER: u32 = 5;

const OP_ADD_SOLID: u32 = 0;
const OP_SUBTRACT_VOID: u32 = 1;
const OP_MASK_WALKABLE: u32 = 2;
const OP_BLOCKER: u32 = 3;
const OP_HAZARD: u32 = 4;

fn material_class_from_geometry(material: GeometryMaterial) -> u32 {
    match material {
        GeometryMaterial::Rock | GeometryMaterial::CavernVoid => crate::domain::MATERIAL_CLASS_ROCK,
        GeometryMaterial::Barrier => crate::domain::MATERIAL_CLASS_BARRIER,
        GeometryMaterial::Hazard => crate::domain::MATERIAL_CLASS_HAZARD,
        GeometryMaterial::Marker => crate::domain::MATERIAL_CLASS_MARKER,
    }
}

fn op_kind(op: GeometryOp) -> u32 {
    match op {
        GeometryOp::AddSolid => OP_ADD_SOLID,
        GeometryOp::SubtractVoid => OP_SUBTRACT_VOID,
        GeometryOp::MaskWalkable => OP_MASK_WALKABLE,
        GeometryOp::Blocker => OP_BLOCKER,
        GeometryOp::HazardVolume => OP_HAZARD,
    }
}

fn geometry_primitives_from_graph(graph: &CavernGeometryGraph) -> Vec<CavernSdfGeometryPrimitive> {
    let mut out = Vec::with_capacity(graph.primitives.len());
    for primitive in graph
        .primitives
        .iter()
        .filter(|primitive| primitive.enabled)
    {
        append_shape_primitive(
            &mut out,
            primitive.op,
            &primitive.shape,
            material_class_from_geometry(primitive.material),
            primitive.id.0 as u32,
        );
    }
    out
}

fn geometry_primitives_from_topology(topology: &CavernTopology) -> Vec<CavernSdfGeometryPrimitive> {
    let mut out = Vec::with_capacity(topology.rooms.len() + topology.connections.len() + 1);
    let center = [
        (topology.world_bounds.min[0] + topology.world_bounds.max[0]) * 0.5,
        (topology.world_bounds.min[1] + topology.world_bounds.max[1]) * 0.5,
        (topology.world_bounds.min[2] + topology.world_bounds.max[2]) * 0.5,
    ];
    let half_extents = [
        (topology.world_bounds.max[0] - topology.world_bounds.min[0]) * 0.5,
        (topology.world_bounds.max[1] - topology.world_bounds.min[1]) * 0.5,
        (topology.world_bounds.max[2] - topology.world_bounds.min[2]) * 0.5,
    ];
    out.push(CavernSdfGeometryPrimitive {
        shape_kind: SHAPE_BOX,
        op_kind: OP_ADD_SOLID,
        material_class: crate::domain::MATERIAL_CLASS_ROCK,
        material_instance: 0,
        p0: [center[0], center[1], center[2], 0.0],
        p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
        p2: [0.0; 4],
    });
    for room in &topology.rooms {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CYLINDER,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::domain::MATERIAL_CLASS_ROCK,
            material_instance: room.id.0 as u32,
            p0: [
                room.center[0],
                room.center[1],
                room.center[2],
                room.radii[0].max(room.radii[2]),
            ],
            p1: [room.radii[1], 0.0, 0.0, 0.0],
            p2: [0.0; 4],
        });
    }
    for connection in &topology.connections {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CAPSULE,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::domain::MATERIAL_CLASS_ROCK,
            material_instance: 0,
            p0: [
                connection.start[0],
                connection.start[1],
                connection.start[2],
                connection.radius,
            ],
            p1: [connection.end[0], connection.end[1], connection.end[2], 0.0],
            p2: [0.0; 4],
        });
    }
    out
}

fn geometry_primitives_from_layout(layout: &CavernLayout) -> Vec<CavernSdfGeometryPrimitive> {
    let mut out = Vec::with_capacity(layout.rooms.len() + layout.connections.len() + 1);
    let center = [
        (layout.world_bounds[0] + layout.world_bounds[2]) * 0.5,
        2.2,
        (layout.world_bounds[1] + layout.world_bounds[3]) * 0.5,
    ];
    let half_extents = [
        (layout.world_bounds[2] - layout.world_bounds[0]) * 0.5,
        5.8,
        (layout.world_bounds[3] - layout.world_bounds[1]) * 0.5,
    ];
    out.push(CavernSdfGeometryPrimitive {
        shape_kind: SHAPE_BOX,
        op_kind: OP_ADD_SOLID,
        material_class: crate::domain::MATERIAL_CLASS_ROCK,
        material_instance: 0,
        p0: [center[0], center[1], center[2], 0.0],
        p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
        p2: [0.0; 4],
    });
    for room in &layout.rooms {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CYLINDER,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::domain::MATERIAL_CLASS_ROCK,
            material_instance: room.id.0 as u32,
            p0: [
                room.center[0],
                2.4,
                room.center[1],
                room.radii[0].max(room.radii[1]),
            ],
            p1: [2.2, 0.0, 0.0, 0.0],
            p2: [0.0; 4],
        });
    }
    for tunnel in &layout.connections {
        out.push(CavernSdfGeometryPrimitive {
            shape_kind: SHAPE_CAPSULE,
            op_kind: OP_SUBTRACT_VOID,
            material_class: crate::domain::MATERIAL_CLASS_ROCK,
            material_instance: 0,
            p0: [tunnel.start[0], 2.2, tunnel.start[1], tunnel.radius],
            p1: [tunnel.end[0], 2.2, tunnel.end[1], 0.0],
            p2: [0.0; 4],
        });
    }
    out
}

fn append_shape_primitive(
    out: &mut Vec<CavernSdfGeometryPrimitive>,
    op: GeometryOp,
    shape: &GeometryPrimitiveShape3,
    material_class: u32,
    material_instance: u32,
) {
    let op_kind = op_kind(op);
    match shape {
        GeometryPrimitiveShape3::Sphere { center, radius } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_SPHERE,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], *radius],
                p1: [0.0; 4],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Ellipsoid { center, radii } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_ELLIPSOID,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], 0.0],
                p1: [radii[0], radii[1], radii[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Capsule { start, end, radius } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_CAPSULE,
                op_kind,
                material_class,
                material_instance,
                p0: [start[0], start[1], start[2], *radius],
                p1: [end[0], end[1], end[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Box {
            center,
            half_extents,
        } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_BOX,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], 0.0],
                p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::RoundedBox {
            center,
            half_extents,
            radius,
        } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_ROUNDED_BOX,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], *radius],
                p1: [half_extents[0], half_extents[1], half_extents[2], 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::Cylinder {
            center,
            radius,
            half_height,
        } => {
            out.push(CavernSdfGeometryPrimitive {
                shape_kind: SHAPE_CYLINDER,
                op_kind,
                material_class,
                material_instance,
                p0: [center[0], center[1], center[2], *radius],
                p1: [*half_height, 0.0, 0.0, 0.0],
                p2: [0.0; 4],
            });
        }
        GeometryPrimitiveShape3::TunnelSplineCapsuleChain { points, radius } => {
            for segment in points.windows(2) {
                out.push(CavernSdfGeometryPrimitive {
                    shape_kind: SHAPE_CAPSULE,
                    op_kind,
                    material_class,
                    material_instance,
                    p0: [segment[0][0], segment[0][1], segment[0][2], *radius],
                    p1: [segment[1][0], segment[1][1], segment[1][2], 0.0],
                    p2: [0.0; 4],
                });
            }
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
            primitive_count: frame.geometry_primitives.len().min(MAX_GEOMETRY_PRIMITIVES) as u32,
            agent_count: frame.agents.len().min(MAX_AGENTS) as u32,
            material_program_count: frame
                .material_program_headers
                .len()
                .min(MAX_MATERIAL_PROGRAMS) as u32,
            material_op_count: frame.material_ops.len().min(MAX_MATERIAL_OPS) as u32,
            material_constant_count: frame.material_constants.len().min(MAX_MATERIAL_CONSTANTS)
                as u32,
            render_mode: frame.render_mode,
            gi_mode: frame.gi_mode,
            gi_quality: frame.gi_quality,
            gi_sample_budget: frame.gi_sample_budget.max(1),
            _pad3: [0; 3],
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
        let roof_clip_y = frame.camera.target[1] + 1.6;
        let params = CavernWorldParamsRaw {
            floor_rock_height: [frame.floor_height, frame.rock_height, roof_clip_y, 0.0],
            ..params
        };
        ctx.queue()
            .write_buffer(&pass.params_buffer, 0, bytemuck::bytes_of(&params));

        let primitives = frame
            .geometry_primitives
            .iter()
            .take(MAX_GEOMETRY_PRIMITIVES)
            .map(|primitive| CavernGeometryPrimitiveRaw {
                shape_kind: primitive.shape_kind,
                op_kind: primitive.op_kind,
                material_class: primitive.material_class,
                material_instance: primitive.material_instance,
                p0: primitive.p0,
                p1: primitive.p1,
                p2: primitive.p2,
            })
            .collect::<Vec<_>>();
        if !primitives.is_empty() {
            ctx.queue().write_buffer(
                &pass.primitives_buffer,
                0,
                bytemuck::cast_slice(&primitives),
            );
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

        let program_headers = frame
            .material_program_headers
            .iter()
            .take(MAX_MATERIAL_PROGRAMS)
            .map(|header| CavernMaterialProgramHeaderRaw {
                class_id: header.class_id,
                op_offset: header.op_offset,
                op_count: header.op_count,
                const_offset: header.const_offset,
                const_count: header.const_count,
                base_color_slot: header.base_color_slot,
                roughness_slot: header.roughness_slot,
                metallic_slot: header.metallic_slot,
                normal_perturb_slot: header.normal_perturb_slot,
                ao_slot: header.ao_slot,
                emissive_slot: header.emissive_slot,
                _pad0: [0; 3],
            })
            .collect::<Vec<_>>();
        if !program_headers.is_empty() {
            ctx.queue().write_buffer(
                &pass.material_program_headers_buffer,
                0,
                bytemuck::cast_slice(&program_headers),
            );
        }

        let material_ops = frame
            .material_ops
            .iter()
            .take(MAX_MATERIAL_OPS)
            .map(|op| CavernMaterialOpRaw {
                op: op.op,
                dst: op.dst,
                src_a: op.src_a,
                src_b: op.src_b,
                src_c: op.src_c,
                const_idx: op.const_idx,
                flags: op.flags,
                _pad0: 0,
            })
            .collect::<Vec<_>>();
        if !material_ops.is_empty() {
            ctx.queue().write_buffer(
                &pass.material_ops_buffer,
                0,
                bytemuck::cast_slice(&material_ops),
            );
        }

        let material_constants = frame
            .material_constants
            .iter()
            .take(MAX_MATERIAL_CONSTANTS)
            .copied()
            .collect::<Vec<_>>();
        if !material_constants.is_empty() {
            ctx.queue().write_buffer(
                &pass.material_constants_buffer,
                0,
                bytemuck::cast_slice(&material_constants),
            );
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

#[cfg(test)]
mod tests {
    use super::{
        OP_ADD_SOLID, OP_BLOCKER, OP_SUBTRACT_VOID, SHAPE_CAPSULE, SHAPE_CYLINDER, SHAPE_ELLIPSOID,
        SHAPE_SPHERE, geometry_primitives_from_graph, geometry_primitives_from_topology,
    };
    use crate::domain::{CavernGeometryGraph, GeometryPrimitiveShape3, GeometryRevision};

    #[test]
    fn geometry_primitives_from_graph_preserves_ops_and_flattens_splines() {
        let mut graph = CavernGeometryGraph {
            revision: GeometryRevision(1),
            bounds: crate::domain::GeometryBounds3::default(),
            primitives: Vec::new(),
        };
        graph.primitives.push(crate::domain::GeometryPrimitive3 {
            id: crate::domain::GeometryPrimitiveId(1),
            layer: crate::domain::GeometryLayer::Terrain,
            material: crate::domain::GeometryMaterial::Rock,
            op: crate::domain::GeometryOp::AddSolid,
            enabled: true,
            shape: GeometryPrimitiveShape3::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 5.0,
            },
        });
        graph.primitives.push(crate::domain::GeometryPrimitive3 {
            id: crate::domain::GeometryPrimitiveId(2),
            layer: crate::domain::GeometryLayer::Walkable,
            material: crate::domain::GeometryMaterial::CavernVoid,
            op: crate::domain::GeometryOp::SubtractVoid,
            enabled: true,
            shape: GeometryPrimitiveShape3::TunnelSplineCapsuleChain {
                points: vec![[0.0, 2.0, 0.0], [2.0, 2.0, 0.0], [4.0, 2.0, 0.0]],
                radius: 1.0,
            },
        });
        graph.primitives.push(crate::domain::GeometryPrimitive3 {
            id: crate::domain::GeometryPrimitiveId(3),
            layer: crate::domain::GeometryLayer::Blocker,
            material: crate::domain::GeometryMaterial::Barrier,
            op: crate::domain::GeometryOp::Blocker,
            enabled: true,
            shape: GeometryPrimitiveShape3::Ellipsoid {
                center: [1.0, 1.0, 1.0],
                radii: [0.5, 0.7, 0.9],
            },
        });
        let primitives = geometry_primitives_from_graph(&graph);
        assert_eq!(
            primitives.len(),
            4,
            "spline chain should flatten to two capsules"
        );
        assert_eq!(primitives[0].shape_kind, SHAPE_SPHERE);
        assert_eq!(primitives[0].op_kind, OP_ADD_SOLID);
        assert_eq!(primitives[1].shape_kind, SHAPE_CAPSULE);
        assert_eq!(primitives[1].op_kind, OP_SUBTRACT_VOID);
        assert_eq!(primitives[2].shape_kind, SHAPE_CAPSULE);
        assert_eq!(primitives[2].op_kind, OP_SUBTRACT_VOID);
        assert_eq!(primitives[3].shape_kind, SHAPE_ELLIPSOID);
        assert_eq!(primitives[3].op_kind, OP_BLOCKER);
    }

    #[test]
    fn geometry_primitives_from_topology_keeps_topology_heights() {
        let layout = crate::domain::CavernLayout::generate(
            crate::domain::CavernSeed(1337),
            &crate::domain::CavernRunConfig::default(),
        );
        let topology =
            crate::domain::CavernTopology::from_layout(&layout, crate::domain::CavernSeed(1337));
        let primitives = geometry_primitives_from_topology(&topology);
        assert!(
            !primitives.is_empty(),
            "topology conversion should produce primitives"
        );
        assert_eq!(primitives[0].shape_kind, super::SHAPE_BOX);
        assert_eq!(primitives[0].op_kind, OP_ADD_SOLID);
        let first_room = topology
            .rooms
            .first()
            .expect("topology should contain rooms");
        let first_room_prim = primitives
            .iter()
            .find(|primitive| {
                primitive.shape_kind == SHAPE_CYLINDER && primitive.op_kind == OP_SUBTRACT_VOID
            })
            .expect("expected room cylinder primitive");
        assert_eq!(first_room_prim.p0[1], first_room.center[1]);
        assert_eq!(
            first_room_prim.p0[3],
            first_room.radii[0].max(first_room.radii[2])
        );
        assert_eq!(first_room_prim.p1[0], first_room.radii[1]);
    }
}
