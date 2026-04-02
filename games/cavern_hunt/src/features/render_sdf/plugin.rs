use crate::{
    CavernCameraState, CavernLayout, CavernMaterialQualityConfig,
    CavernMaterialRuntimeState, CavernSdfAgent, CavernSdfGeometryPrimitive, CavernSdfMaterialOp,
    CavernSdfWorldFrame, CavernTopology, Chest, ColliderRadius, EnemyKind, ExtractionZone,
    GeometryMaterial, GeometryOp, GeometryPrimitiveShape3, Health, LocalPlayerRef, LootDrop,
    Pickup, Player, PlayerCompanion, PlayerId, PlayerSpectator, Projectile, ProjectileVisualState,
    Transform2, is_active_player_entity,
};
use anyhow::{Result, anyhow};
use bytemuck::{Pod, Zeroable};
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
const COMPOSE_SHADER: &str =
    include_str!("../../../../../assets/shaders/world_compose_fullscreen.wgsl");
const COMPUTE_SHADER: &str = include_str!("../../../../../assets/shaders/cavern_hunt_sdf.wgsl");
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
#[derive(Clone, Copy, Pod, Zeroable, ecs::Resource)]
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
#[derive(Clone, Copy, Pod, Zeroable, ecs::Resource)]
struct CavernAgentRaw {
    pos: [f32; 2],
    radius: f32,
    health: f32,
    team: u32,
    kind: u32,
    _pad0: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ecs::Resource)]
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
#[derive(Clone, Copy, Pod, Zeroable, ecs::Resource)]
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
#[derive(Clone, Copy, Pod, Zeroable, ecs::Resource)]
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, ecs::Resource)]
struct CavernComposeParamsRaw {
    output_size: [f32; 2],
    target_aspect: f32,
    fit_mode: u32,
    bar_color: [f32; 4],
}

#[derive(Default, ecs::Resource)]
struct CavernGpuSharedState {
    pass: Option<CavernGpuPass>,
}

struct CavernGpuPass {
    surface_format: TextureFormat,
    size: (u32, u32),
    params_buffer: Buffer,
    primitives_buffer: Buffer,
    agents_buffer: Buffer,
    compose_params_buffer: Buffer,
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

#[path = "runtime/mod.rs"]
mod runtime;

pub use runtime::build_cavern_render_flow;
pub(crate) use runtime::{
    build_sdf_world_frame_system, project_mouse_to_world, setup_render_resources,
    update_camera_and_hud_system,
};
