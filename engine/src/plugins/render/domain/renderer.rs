use super::PipelineKey;
use super::frame_graph::{FrameGraph, PassHandle, PassKind};
use super::model_manager::{
    ModelManager, ModelMaterial, ModelMesh, ModelMeshVertex, ModelTextureData,
};
use super::render_executor_registry::{
    BuiltinRenderPassExecutor, RenderFrameDataRegistry, RenderPassEncodeContext,
    RenderPassExecutorRegistryResource, RenderPassPrepareContext,
};
use super::render_frame::{WorldRenderAgent, WorldRenderFrame};
use super::render_graph_registry::{
    RegisteredPassKind, RegisteredPipelineRef, RenderGraphRegistryResource,
};
use super::shader_manager::ShaderRegistryResource;
use crate::plugins::ui::domain::{FileFontProvider, TextRenderer};
use crate::plugins::ui::domain::{UiDrawCmd, UiDrawList};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
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
    @location(2) origin_scale : vec4<f32>,
    @location(3) instance_color : vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
    @location(1) instance_color : vec4<f32>,
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
    let world_pos = vec3<f32>(
        input.position.x * input.origin_scale.w + input.origin_scale.x,
        input.position.y * input.origin_scale.w + input.origin_scale.y,
        input.position.z * input.origin_scale.w + input.origin_scale.z,
    );
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = input.uv;
    out.instance_color = input.instance_color;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(material_texture, material_sampler, input.uv);
    return texel * material * input.instance_color;
}
"#;

const UI_RECT_SHADER_ID: &str = "ui_rect";
const MESH_CLEAR_COLOR: Color = Color {
    r: 0.02,
    g: 0.03,
    b: 0.05,
    a: 1.0,
};
const MESH_DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

#[derive(Debug, Clone)]
struct ResolvedFramePassDescriptor {
    name: String,
    kind: PassKind,
    pipeline: PipelineKey,
    reads: Vec<String>,
    writes: Vec<String>,
    depends_on: Vec<String>,
    executor: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
struct FrameGraphCompileDiagnostics {
    empty_pass_name_count: usize,
    duplicate_pass_names: Vec<String>,
    missing_dependencies: Vec<(String, String)>,
    no_registered_passes: bool,
}

impl FrameGraphCompileDiagnostics {
    fn has_issues(&self) -> bool {
        self.issue_count() > 0
    }

    fn issue_count(&self) -> usize {
        self.empty_pass_name_count
            + self.duplicate_pass_names.len()
            + self.missing_dependencies.len()
            + usize::from(self.no_registered_passes)
    }
}

#[derive(Debug, Clone)]
struct FrameGraphBuildOutput {
    graph: FrameGraph,
    handles: Vec<PassHandle>,
    pass_executor_bindings: BTreeMap<String, String>,
    diagnostics: FrameGraphCompileDiagnostics,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RendererFrameTimings {
    pub prepare_ui_ms: f32,
    pub prepare_mesh_ms: f32,
    pub world_prepare_ms: f32,
    pub encode_submit_ms: f32,
    pub mesh_hot_path: MeshPrepareHotPath,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MeshPrepareHotPath {
    pub model_collect_ms: f32,
    pub chunk_collect_ms: f32,
    pub merge_filter_ms: f32,
    pub camera_update_ms: f32,
    pub static_upload_ms: f32,
    pub agent_upload_ms: f32,
    pub model_meshes: u32,
    pub chunk_meshes: u32,
    pub merged_meshes: u32,
    pub skipped_meshes: u32,
    pub draw_items: u32,
    pub textured_meshes: u32,
    pub texture_upload_bytes: u64,
    pub vertex_count: u64,
    pub index_count: u64,
    pub vertex_upload_bytes: u64,
    pub index_upload_bytes: u64,
    pub instance_upload_bytes: u64,
    pub uniform_upload_bytes: u64,
    pub agent_instances: u32,
    pub static_cache_hits: u32,
    pub static_cache_misses: u32,
}

impl MeshPrepareHotPath {
    pub fn is_warm_frame(&self) -> bool {
        self.static_cache_misses == 0
            && self.vertex_upload_bytes == 0
            && self.index_upload_bytes == 0
            && self.texture_upload_bytes == 0
    }
}

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
struct MeshInstanceRaw {
    origin_scale: [f32; 4],
    color: [f32; 4],
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
    nearest_sampler: Sampler,
    default_instance_buffer: Buffer,
}

#[derive(Debug)]
struct MeshSurfaceTargets {
    size: (u32, u32),
    format: TextureFormat,
    _msaa_target: Texture,
    msaa_view: TextureView,
    _depth_target: Texture,
    depth_view: TextureView,
}

#[derive(Debug)]
struct MeshPreparedDrawItem {
    index_count: u32,
    instance_count: u32,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    instance_buffer: Option<Buffer>,
    _material_buffer: Option<Buffer>,
    material_bind_group: Option<BindGroup>,
    _material_texture: Option<Texture>,
    _material_texture_view: Option<TextureView>,
}

#[derive(Debug)]
struct MeshPreparedDraw {
    draws: Vec<MeshPreparedDrawItem>,
    surface_size: (u32, u32),
}

#[derive(Debug)]
struct MeshPreparedWithHotPath {
    prepared: MeshPreparedDraw,
    hot_path: MeshPrepareHotPath,
}

#[derive(Debug)]
struct MeshCacheEntry {
    signature: u64,
    index_count: u32,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    material_buffer: Buffer,
    material_bind_group: BindGroup,
    material_texture: Option<Texture>,
    material_texture_view: Option<TextureView>,
}

#[derive(Debug)]
struct UiPreparedDraws {
    rect_instances: usize,
    rect_instance_buffer: Option<Buffer>,
    text_draws: Vec<(Buffer, u32, (u32, u32, u32, u32))>,
    surface_size: (u32, u32),
}

#[derive(Debug)]
pub(crate) struct RendererPreparedPacket {
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    merged_world_frame: WorldRenderFrame,
    prepared_ui: UiPreparedDraws,
    prepared_mesh: MeshPreparedDraw,
    prepare_timings: RendererFrameTimings,
}

trait FramePassExecutor {
    fn prepare(
        &self,
        _renderer: &mut Renderer,
        _device: &Device,
        _queue: &Queue,
        _packet: &RendererPreparedPacket,
        _timings: &mut RendererFrameTimings,
    ) {
    }

    fn encode(
        &self,
        renderer: &mut Renderer,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        pipeline: PipelineKey,
    );
}

#[derive(Debug, Default)]
struct BuiltinComputeNoopPassExecutor;

impl FramePassExecutor for BuiltinComputeNoopPassExecutor {
    fn encode(
        &self,
        _renderer: &mut Renderer,
        _device: &Device,
        _encoder: &mut CommandEncoder,
        _frame_view: &TextureView,
        _packet: &RendererPreparedPacket,
        _pipeline: PipelineKey,
    ) {
        static WARNED: AtomicBool = AtomicBool::new(false);
        if !WARNED.swap(true, Ordering::Relaxed) {
            tracing::warn!(
                "builtin_compute is not implemented in core render plugin; register a custom executor instead"
            );
        }
    }
}

#[derive(Debug, Default)]
struct BuiltinComposeNoopPassExecutor;

impl FramePassExecutor for BuiltinComposeNoopPassExecutor {
    fn encode(
        &self,
        _renderer: &mut Renderer,
        _device: &Device,
        _encoder: &mut CommandEncoder,
        _frame_view: &TextureView,
        _packet: &RendererPreparedPacket,
        _pipeline: PipelineKey,
    ) {
        static WARNED: AtomicBool = AtomicBool::new(false);
        if !WARNED.swap(true, Ordering::Relaxed) {
            tracing::warn!(
                "builtin_compose is not implemented in core render plugin; register a custom executor instead"
            );
        }
    }
}

#[derive(Debug, Default)]
struct MeshOverlayPassExecutor;

impl FramePassExecutor for MeshOverlayPassExecutor {
    fn encode(
        &self,
        renderer: &mut Renderer,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        _pipeline: PipelineKey,
    ) {
        renderer.encode_mesh_pass(device, encoder, frame_view, &packet.prepared_mesh);
    }
}

#[derive(Debug, Default)]
struct UiCompositePassExecutor;

impl FramePassExecutor for UiCompositePassExecutor {
    fn encode(
        &self,
        renderer: &mut Renderer,
        _device: &Device,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        packet: &RendererPreparedPacket,
        _pipeline: PipelineKey,
    ) {
        renderer.encode_ui_pass(encoder, frame_view, &packet.prepared_ui);
    }
}

const BUILTIN_COMPUTE_NOOP_PASS_EXECUTOR: BuiltinComputeNoopPassExecutor =
    BuiltinComputeNoopPassExecutor;
const BUILTIN_COMPOSE_NOOP_PASS_EXECUTOR: BuiltinComposeNoopPassExecutor =
    BuiltinComposeNoopPassExecutor;
const MESH_OVERLAY_PASS_EXECUTOR: MeshOverlayPassExecutor = MeshOverlayPassExecutor;
const UI_COMPOSITE_PASS_EXECUTOR: UiCompositePassExecutor = UiCompositePassExecutor;

const PLAYER_CUBE_COLOR: [f32; 4] = [0.12, 0.88, 0.18, 1.0];
const ENEMY_CUBE_COLOR: [f32; 4] = [0.92, 0.18, 0.18, 1.0];

fn ground_mesh_for_bounds(world_bounds: [f32; 4]) -> ModelMesh {
    let min_x = world_bounds[0];
    let min_z = world_bounds[1];
    let max_x = world_bounds[2];
    let max_z = world_bounds[3];
    let y = -0.02;

    ModelMesh {
        name: "generated_ground".to_string(),
        vertices: vec![
            ModelMeshVertex {
                position: [min_x, y, min_z],
                uv: [0.0, 0.0],
            },
            ModelMeshVertex {
                position: [max_x, y, min_z],
                uv: [1.0, 0.0],
            },
            ModelMeshVertex {
                position: [max_x, y, max_z],
                uv: [1.0, 1.0],
            },
            ModelMeshVertex {
                position: [min_x, y, max_z],
                uv: [0.0, 1.0],
            },
        ],
        // winding chosen so top-face normal points +Y.
        indices: vec![0, 2, 1, 0, 3, 2],
        material: ModelMaterial {
            base_color_factor: [0.12, 0.14, 0.18, 1.0],
            base_color_texture: None,
            nearest_sampling: false,
        },
    }
}

fn mesh_cache_key(mesh_name: &str, mesh_idx: usize) -> String {
    format!("{mesh_name}#{mesh_idx}")
}

fn is_template_mesh_name(name: &str) -> bool {
    template_name_matches(name, "corner_col")
        || template_name_matches(name, "diagonal_corner_col")
        || template_name_matches(name, "edge_col")
        || template_name_matches(name, "full_col")
        || template_name_matches(name, "T_col")
}

fn template_name_matches(name: &str, base: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let base = base.to_ascii_lowercase();
    lower == base
        || lower.starts_with(&format!("{base}."))
        || lower.starts_with(&format!("{base}_"))
}

fn mesh_signature(mesh: &ModelMesh) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    mesh.name.hash(&mut hasher);
    mesh.vertices.len().hash(&mut hasher);
    mesh.indices.len().hash(&mut hasher);
    for v in &mesh.vertices {
        for f in v.position {
            f.to_bits().hash(&mut hasher);
        }
        for f in v.uv {
            f.to_bits().hash(&mut hasher);
        }
    }
    for idx in &mesh.indices {
        idx.hash(&mut hasher);
    }
    for f in mesh.material.base_color_factor {
        f.to_bits().hash(&mut hasher);
    }
    mesh.material.nearest_sampling.hash(&mut hasher);
    if let Some(tex) = mesh.material.base_color_texture.as_ref() {
        tex.width.hash(&mut hasher);
        tex.height.hash(&mut hasher);
        tex.rgba8.len().hash(&mut hasher);
        tex.rgba8.hash(&mut hasher);
    } else {
        0_u8.hash(&mut hasher);
    }
    hasher.finish()
}

fn create_texture_from_rgba(
    device: &Device,
    queue: &Queue,
    texture: &ModelTextureData,
    mesh_idx: u32,
) -> Texture {
    let gpu_texture = device.create_texture(&TextureDescriptor {
        label: Some("engine_mesh_material_texture"),
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

fn unit_cube_mesh() -> ModelMesh {
    let vertices = vec![
        ModelMeshVertex {
            position: [-0.5, 0.0, -0.5],
            uv: [0.0, 0.0],
        },
        ModelMeshVertex {
            position: [0.5, 0.0, -0.5],
            uv: [1.0, 0.0],
        },
        ModelMeshVertex {
            position: [0.5, 1.0, -0.5],
            uv: [1.0, 1.0],
        },
        ModelMeshVertex {
            position: [-0.5, 1.0, -0.5],
            uv: [0.0, 1.0],
        },
        ModelMeshVertex {
            position: [-0.5, 0.0, 0.5],
            uv: [0.0, 0.0],
        },
        ModelMeshVertex {
            position: [0.5, 0.0, 0.5],
            uv: [1.0, 0.0],
        },
        ModelMeshVertex {
            position: [0.5, 1.0, 0.5],
            uv: [1.0, 1.0],
        },
        ModelMeshVertex {
            position: [-0.5, 1.0, 0.5],
            uv: [0.0, 1.0],
        },
    ];
    let indices: Vec<u32> = vec![
        0, 2, 1, 0, 3, 2, // back (-Z)
        4, 5, 6, 4, 6, 7, // front (+Z)
        0, 1, 5, 0, 5, 4, // bottom (-Y)
        3, 7, 6, 3, 6, 2, // top (+Y)
        1, 2, 6, 1, 6, 5, // right (+X)
        0, 4, 7, 0, 7, 3, // left (-X)
    ];
    ModelMesh {
        name: "generated_unit_cube".to_string(),
        vertices,
        indices,
        material: ModelMaterial {
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            base_color_texture: None,
            nearest_sampling: false,
        },
    }
}

fn agent_instance_data(agents: &[WorldRenderAgent]) -> Vec<MeshInstanceRaw> {
    let mut instances = Vec::with_capacity(agents.len());
    for agent in agents {
        let color = if agent.team == 0 {
            PLAYER_CUBE_COLOR
        } else {
            ENEMY_CUBE_COLOR
        };
        let scale = (agent.radius * 0.9).max(0.5);
        instances.push(MeshInstanceRaw {
            origin_scale: [agent.x, 0.0, agent.y, scale],
            color,
        });
    }
    instances
}

#[derive(Debug)]
pub struct Renderer {
    model_manager: ModelManager,
    mesh_pass: Option<MeshPass>,
    mesh_pass_format: Option<TextureFormat>,
    mesh_surface_targets: Option<MeshSurfaceTargets>,
    rect_pass: Option<RectPass>,
    rect_pass_format: Option<TextureFormat>,
    rect_pass_shader_revision: u64,
    text_renderer: Option<TextRenderer>,
    text_renderer_format: Option<TextureFormat>,
    camera_focus: Option<Vec3>,
    mesh_cache: BTreeMap<String, MeshCacheEntry>,
    last_frame_graph_diagnostics_hash: Option<u64>,
    last_missing_executors_hash: Option<u64>,
    last_execution_order_error_hash: Option<u64>,
}

impl Renderer {
    fn builtin_pass_executor(
        executor: BuiltinRenderPassExecutor,
    ) -> &'static dyn FramePassExecutor {
        match executor {
            BuiltinRenderPassExecutor::Compute => &BUILTIN_COMPUTE_NOOP_PASS_EXECUTOR,
            BuiltinRenderPassExecutor::Compose => &BUILTIN_COMPOSE_NOOP_PASS_EXECUTOR,
            BuiltinRenderPassExecutor::MeshOverlay => &MESH_OVERLAY_PASS_EXECUTOR,
            BuiltinRenderPassExecutor::UiComposite => &UI_COMPOSITE_PASS_EXECUTOR,
        }
    }

    fn stable_hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn log_frame_graph_diagnostics(
        &mut self,
        world_scene: &str,
        overlay_scene: &str,
        registry_revision: u64,
        diagnostics: &FrameGraphCompileDiagnostics,
    ) {
        if !diagnostics.has_issues() {
            if self.last_frame_graph_diagnostics_hash.take().is_some() {
                tracing::info!(
                    world_scene,
                    overlay_scene,
                    "frame graph compile diagnostics resolved"
                );
            }
            return;
        }

        let signature =
            Self::stable_hash(&(world_scene, overlay_scene, registry_revision, diagnostics));
        if self.last_frame_graph_diagnostics_hash == Some(signature) {
            return;
        }
        self.last_frame_graph_diagnostics_hash = Some(signature);

        let first_duplicate_pass = diagnostics
            .duplicate_pass_names
            .first()
            .map(String::as_str)
            .unwrap_or_default();
        let first_missing_dependency = diagnostics
            .missing_dependencies
            .first()
            .map(|(pass, dependency)| format!("{pass}->{dependency}"))
            .unwrap_or_default();

        tracing::warn!(
            world_scene,
            overlay_scene,
            registry_revision,
            issue_count = diagnostics.issue_count(),
            empty_pass_name_count = diagnostics.empty_pass_name_count,
            duplicate_pass_count = diagnostics.duplicate_pass_names.len(),
            missing_dependency_count = diagnostics.missing_dependencies.len(),
            no_registered_passes = diagnostics.no_registered_passes,
            first_duplicate_pass,
            first_missing_dependency,
            "frame graph compile diagnostics"
        );
    }

    fn log_missing_executors_once(&mut self, missing_executors: &[(String, String)]) {
        if missing_executors.is_empty() {
            if self.last_missing_executors_hash.take().is_some() {
                tracing::info!("frame graph executor bindings resolved");
            }
            return;
        }

        let mut unique_missing = missing_executors.to_vec();
        unique_missing.sort();
        unique_missing.dedup();

        let signature = Self::stable_hash(&unique_missing);
        if self.last_missing_executors_hash == Some(signature) {
            return;
        }
        self.last_missing_executors_hash = Some(signature);

        let first_missing = unique_missing
            .first()
            .map(|(pass, executor)| format!("{pass}->{executor}"))
            .unwrap_or_default();
        tracing::warn!(
            missing_count = unique_missing.len(),
            first_missing,
            "frame graph pass executor bindings are missing; skipped pass encoding"
        );
    }

    fn log_execution_order_error_once(&mut self, err: &anyhow::Error) {
        let err_text = err.to_string();
        let signature = Self::stable_hash(&err_text);
        if self.last_execution_order_error_hash == Some(signature) {
            return;
        }
        self.last_execution_order_error_hash = Some(signature);
        tracing::error!(
            error = err_text,
            "frame graph execution order failed; using fallback order"
        );
    }

    fn clear_execution_order_error(&mut self) {
        if self.last_execution_order_error_hash.take().is_some() {
            tracing::info!("frame graph execution ordering recovered");
        }
    }

    fn prepare_registered_passes(
        &mut self,
        device: &Device,
        queue: &Queue,
        packet: &RendererPreparedPacket,
        active_executors: &BTreeSet<String>,
        render_executor_registry: &RenderPassExecutorRegistryResource,
        timings: &mut RendererFrameTimings,
    ) {
        let frame_data = RenderFrameDataRegistry::new().with(&packet.merged_world_frame);
        for executor_name in active_executors {
            if let Some(builtin) = render_executor_registry.resolve_builtin(executor_name) {
                let executor = Self::builtin_pass_executor(builtin);
                executor.prepare(self, device, queue, packet, timings);
                continue;
            }
            if let Some(custom) = render_executor_registry.resolve_custom(executor_name) {
                let mut dispatch_builtin = |builtin: BuiltinRenderPassExecutor| -> Result<()> {
                    let executor = Self::builtin_pass_executor(builtin);
                    executor.prepare(self, device, queue, packet, timings);
                    Ok(())
                };
                let mut ctx = RenderPassPrepareContext::new(
                    device,
                    queue,
                    &frame_data,
                    packet.surface_format,
                    packet.surface_size,
                )
                .with_builtin_dispatch(&mut dispatch_builtin);
                if let Err(err) = custom.prepare(&mut ctx) {
                    tracing::error!(
                        executor = executor_name,
                        ?err,
                        "custom render pass executor prepare failed"
                    );
                }
            }
        }
    }

    fn resolve_registered_pipeline(
        &self,
        pass_name: &str,
        pipeline_ref: Option<&RegisteredPipelineRef>,
        named_pipelines: &BTreeMap<String, PipelineKey>,
    ) -> PipelineKey {
        if let Some(pipeline_ref) = pipeline_ref {
            match pipeline_ref {
                RegisteredPipelineRef::Builtin(key) => return key.clone(),
                RegisteredPipelineRef::Named(name) => {
                    if let Some(key) = named_pipelines.get(name).cloned() {
                        return key;
                    }
                    tracing::warn!(
                        pass = pass_name,
                        pipeline_id = name,
                        "registered named pipeline id not found; falling back to pass id key"
                    );
                }
            }
        }

        PipelineKey::from(pass_name.to_string())
    }

    fn resolved_registered_descriptors(
        &self,
        render_graph_registry: &RenderGraphRegistryResource,
    ) -> Vec<ResolvedFramePassDescriptor> {
        let owners = render_graph_registry.owners();
        let mut named_pipelines = BTreeMap::<String, PipelineKey>::new();
        for owner in &owners {
            for pipeline in &owner.pipelines {
                let pipeline_id = pipeline.id.trim();
                if pipeline_id.is_empty() {
                    tracing::warn!(
                        owner = owner.owner,
                        "registered named pipeline has empty id; skipping"
                    );
                    continue;
                }
                if let Some(previous) =
                    named_pipelines.insert(pipeline_id.to_string(), pipeline.key.clone())
                {
                    tracing::warn!(
                        owner = owner.owner,
                        pipeline_id,
                        previous_pipeline = previous.label(),
                        new_pipeline = pipeline.key.label(),
                        "registered named pipeline id replaced previous registration"
                    );
                }
            }
        }

        let mut out = Vec::new();
        for owner in &owners {
            for pass in &owner.passes {
                let pass_name = pass.id.trim();
                if pass_name.is_empty() {
                    tracing::warn!(
                        owner = owner.owner,
                        "registered render pass has empty id; skipping"
                    );
                    continue;
                }
                let kind = match pass.kind {
                    RegisteredPassKind::Compute => PassKind::Compute,
                    RegisteredPassKind::Render => PassKind::Render,
                };
                let pipeline = self.resolve_registered_pipeline(
                    pass_name,
                    pass.pipeline.as_ref(),
                    &named_pipelines,
                );
                let executor = pass
                    .executor
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or(pass_name)
                    .to_string();
                out.push(ResolvedFramePassDescriptor {
                    name: pass_name.to_string(),
                    kind,
                    pipeline,
                    reads: pass
                        .reads
                        .iter()
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .collect(),
                    writes: pass
                        .writes
                        .iter()
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .collect(),
                    depends_on: pass
                        .depends_on
                        .iter()
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .collect(),
                    executor,
                });
            }
        }

        out
    }

    fn build_frame_graph_from_descriptors(
        &self,
        descriptors: &[ResolvedFramePassDescriptor],
    ) -> FrameGraphBuildOutput {
        let mut graph = FrameGraph::new();
        let mut handles = Vec::with_capacity(descriptors.len());
        let mut by_name = BTreeMap::<String, PassHandle>::new();
        let mut pass_executor_bindings = BTreeMap::<String, String>::new();
        let mut diagnostics = FrameGraphCompileDiagnostics::default();

        for descriptor in descriptors {
            let pass_name = descriptor.name.trim();
            if pass_name.is_empty() {
                diagnostics.empty_pass_name_count =
                    diagnostics.empty_pass_name_count.saturating_add(1);
                continue;
            }
            if by_name.contains_key(pass_name) {
                diagnostics.duplicate_pass_names.push(pass_name.to_string());
                continue;
            }

            let mut builder = match descriptor.kind {
                PassKind::Compute => {
                    graph.compute_pass(pass_name.to_string(), descriptor.pipeline.clone())
                }
                PassKind::Render => {
                    graph.render_pass(pass_name.to_string(), descriptor.pipeline.clone())
                }
            };
            if !descriptor.reads.is_empty() {
                builder = builder.reads(descriptor.reads.clone());
            }
            if !descriptor.writes.is_empty() {
                builder = builder.writes(descriptor.writes.clone());
            }
            for dep_name in &descriptor.depends_on {
                let dep_name = dep_name.trim();
                if dep_name.is_empty() {
                    continue;
                }
                if let Some(dep_handle) = by_name.get(dep_name).copied() {
                    builder = builder.depends_on(dep_handle);
                } else {
                    diagnostics
                        .missing_dependencies
                        .push((pass_name.to_string(), dep_name.to_string()));
                }
            }

            let handle = builder.build();
            by_name.insert(pass_name.to_string(), handle);
            pass_executor_bindings.insert(pass_name.to_string(), descriptor.executor.clone());
            handles.push(handle);
        }

        FrameGraphBuildOutput {
            graph,
            handles,
            pass_executor_bindings,
            diagnostics,
        }
    }

    pub fn new() -> Self {
        Self {
            model_manager: ModelManager::new(),
            mesh_pass: None,
            mesh_pass_format: None,
            mesh_surface_targets: None,
            rect_pass: None,
            rect_pass_format: None,
            rect_pass_shader_revision: 0,
            text_renderer: None,
            text_renderer_format: None,
            camera_focus: None,
            mesh_cache: BTreeMap::new(),
            last_frame_graph_diagnostics_hash: None,
            last_missing_executors_hash: None,
            last_execution_order_error_hash: None,
        }
    }

    fn ensure_mesh_pass(&mut self, device: &Device, queue: &Queue, format: TextureFormat) {
        if self.mesh_pass.is_some() && self.mesh_pass_format == Some(format) {
            return;
        }

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("engine_mesh_shader"),
            source: ShaderSource::Wgsl(DEFAULT_MESH_SHADER.into()),
        });

        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_mesh_camera_uniform"),
            size: std::mem::size_of::<MeshCameraRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_mesh_bind_group_layout"),
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
                label: Some("engine_mesh_material_bind_group_layout"),
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
            label: Some("engine_mesh_bind_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_mesh_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout, &material_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_mesh_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[
                    VertexBufferLayout {
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
                    },
                    VertexBufferLayout {
                        array_stride: std::mem::size_of::<MeshInstanceRaw>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 2,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 3,
                            },
                        ],
                    },
                ],
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
            depth_stencil: Some(DepthStencilState {
                format: MESH_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 4,
                ..MultisampleState::default()
            },
            multiview: None,
            cache: None,
        });

        let default_texture = device.create_texture(&TextureDescriptor {
            label: Some("engine_mesh_default_texture"),
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
            label: Some("engine_mesh_default_sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });
        let nearest_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("engine_mesh_nearest_sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let default_instance = MeshInstanceRaw {
            origin_scale: [0.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let default_instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("engine_mesh_default_instance_buffer"),
            contents: bytemuck::bytes_of(&default_instance),
            usage: BufferUsages::VERTEX,
        });

        self.mesh_pass = Some(MeshPass {
            pipeline,
            camera_buffer,
            camera_bind_group,
            material_bind_group_layout,
            default_texture_view,
            default_sampler,
            nearest_sampler,
            default_instance_buffer,
        });
        self.mesh_pass_format = Some(format);
        self.mesh_surface_targets = None;
        self.mesh_cache.clear();
    }

    fn ensure_mesh_surface_targets(&mut self, device: &Device, surface_size: (u32, u32)) {
        if self.mesh_pass.is_none() {
            self.mesh_surface_targets = None;
            return;
        }
        let size = (surface_size.0.max(1), surface_size.1.max(1));
        let format = self
            .mesh_pass_format
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);
        let needs_rebuild = self
            .mesh_surface_targets
            .as_ref()
            .is_none_or(|targets| targets.size != size || targets.format != format);
        if !needs_rebuild {
            return;
        }

        let msaa_target = device.create_texture(&TextureDescriptor {
            label: Some("engine_mesh_msaa_target"),
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let msaa_view = msaa_target.create_view(&TextureViewDescriptor::default());
        let depth_target = device.create_texture(&TextureDescriptor {
            label: Some("engine_mesh_depth_target"),
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: MESH_DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_target.create_view(&TextureViewDescriptor::default());
        self.mesh_surface_targets = Some(MeshSurfaceTargets {
            size,
            format,
            _msaa_target: msaa_target,
            msaa_view,
            _depth_target: depth_target,
            depth_view,
        });
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
            label: Some("engine_ui_rect_shader"),
            source: ShaderSource::Wgsl(shader_source.into()),
        });

        let screen_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("engine_ui_screen_uniform"),
            size: std::mem::size_of::<ScreenUniformRaw>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("engine_ui_rect_bind_group_layout"),
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
            label: Some("engine_ui_rect_bind_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("engine_ui_rect_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("engine_ui_rect_pipeline"),
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
                label: Some("engine_ui_rect_instances"),
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
        &mut self,
        device: &Device,
        queue: &Queue,
        world_frame: &WorldRenderFrame,
        surface_width: f32,
        surface_height: f32,
    ) -> MeshPreparedWithHotPath {
        let _mesh_prepare_span = tracing::info_span!("renderer.prepare_mesh_draw").entered();
        let mut hot_path = MeshPrepareHotPath::default();
        let surface_size = (
            surface_width.max(1.0).round() as u32,
            surface_height.max(1.0).round() as u32,
        );
        if !world_frame.render_mesh_overlay {
            return MeshPreparedWithHotPath {
                prepared: MeshPreparedDraw {
                    draws: Vec::new(),
                    surface_size,
                },
                hot_path,
            };
        }

        let collect_models_start = Instant::now();
        let source_meshes = {
            let _span = tracing::info_span!("mesh.collect_models").entered();
            self.model_manager.collect_meshes()
        };
        hot_path.model_collect_ms = collect_models_start.elapsed().as_secs_f32() * 1000.0;
        hot_path.model_meshes = source_meshes.len() as u32;

        let merge_start = Instant::now();
        let meshes = {
            let _span = tracing::info_span!("mesh.merge_filter").entered();
            let mut meshes: Vec<_> = source_meshes
                .into_iter()
                .filter(|mesh| !is_template_mesh_name(&mesh.name))
                .collect();
            if !world_frame.agents.is_empty() {
                meshes.push(ground_mesh_for_bounds(world_frame.world_bounds));
            }
            meshes
        };
        hot_path.merge_filter_ms = merge_start.elapsed().as_secs_f32() * 1000.0;
        hot_path.merged_meshes = meshes.len() as u32;

        if meshes.is_empty() && world_frame.agents.is_empty() {
            return MeshPreparedWithHotPath {
                prepared: MeshPreparedDraw {
                    draws: Vec::new(),
                    surface_size,
                },
                hot_path,
            };
        }

        if let Some(mesh_pass) = self.mesh_pass.as_ref() {
            let camera_update_start = Instant::now();
            {
                let _span = tracing::info_span!("mesh.update_camera").entered();
                let aspect = (surface_width.max(1.0) / surface_height.max(1.0)).max(0.1);
                let player_target = world_frame
                    .agents
                    .iter()
                    .find(|agent| agent.team == 0)
                    .or_else(|| world_frame.agents.first())
                    .map(|agent| Vec3::new(agent.x, 0.0, agent.y))
                    .unwrap_or(Vec3::ZERO);
                let follow_dampening = world_frame.camera_follow_dampening.clamp(0.0, 1.0);
                let target = if let Some(prev) = self.camera_focus {
                    prev.lerp(player_target, follow_dampening)
                } else {
                    player_target
                };
                self.camera_focus = Some(target);
                let yaw = world_frame.camera_yaw;
                let pitch_min = world_frame
                    .camera_pitch_min
                    .min(world_frame.camera_pitch_max);
                let pitch_max = world_frame
                    .camera_pitch_min
                    .max(world_frame.camera_pitch_max);
                let distance_min = world_frame
                    .camera_distance_min
                    .min(world_frame.camera_distance_max)
                    .max(0.1);
                let distance_max = world_frame
                    .camera_distance_min
                    .max(world_frame.camera_distance_max)
                    .max(distance_min);
                let pitch = world_frame.camera_pitch.clamp(pitch_min, pitch_max);
                let distance = world_frame
                    .camera_distance
                    .clamp(distance_min, distance_max);
                let pivot = target + Vec3::new(0.0, 1.1, 0.0);
                // Orbit/spring arm direction from pivot to camera.
                let orbit = Vec3::new(
                    pitch.cos() * yaw.sin(),
                    pitch.sin(),
                    pitch.cos() * yaw.cos(),
                );
                let eye = pivot + orbit * distance;
                let up = Vec3::Y;
                let view = Mat4::look_at_rh(eye, pivot, up);
                let proj = Mat4::perspective_rh_gl(55.0f32.to_radians(), aspect, 0.10, 600.0);
                let camera = MeshCameraRaw {
                    view_proj: (proj * view).to_cols_array_2d(),
                };
                queue.write_buffer(&mesh_pass.camera_buffer, 0, bytemuck::bytes_of(&camera));
            }
            hot_path.camera_update_ms = camera_update_start.elapsed().as_secs_f32() * 1000.0;
        }

        let Some(mesh_pass) = self.mesh_pass.as_ref() else {
            return MeshPreparedWithHotPath {
                prepared: MeshPreparedDraw {
                    draws: Vec::new(),
                    surface_size,
                },
                hot_path,
            };
        };

        let mut draws = Vec::new();
        let mut live_cache_keys = BTreeSet::<String>::new();
        let static_upload_start = Instant::now();
        {
            let _span = tracing::info_span!("mesh.upload_static").entered();
            for (mesh_idx, mesh) in meshes.into_iter().enumerate() {
                if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                    hot_path.skipped_meshes = hot_path.skipped_meshes.saturating_add(1);
                    continue;
                }
                let key = mesh_cache_key(&mesh.name, mesh_idx);
                live_cache_keys.insert(key.clone());
                let signature = mesh_signature(&mesh);
                let mut cache_hit = false;
                if let Some(entry) = self.mesh_cache.get(&key)
                    && entry.signature == signature
                {
                    cache_hit = true;
                    hot_path.static_cache_hits = hot_path.static_cache_hits.saturating_add(1);
                    hot_path.vertex_count = hot_path
                        .vertex_count
                        .saturating_add(mesh.vertices.len() as u64);
                    hot_path.index_count = hot_path
                        .index_count
                        .saturating_add(mesh.indices.len() as u64);
                    if mesh.material.base_color_texture.is_some() {
                        hot_path.textured_meshes = hot_path.textured_meshes.saturating_add(1);
                    }
                    draws.push(MeshPreparedDrawItem {
                        index_count: entry.index_count,
                        instance_count: 1,
                        vertex_buffer: Some(entry.vertex_buffer.clone()),
                        index_buffer: Some(entry.index_buffer.clone()),
                        instance_buffer: None,
                        _material_buffer: Some(entry.material_buffer.clone()),
                        material_bind_group: Some(entry.material_bind_group.clone()),
                        _material_texture: entry.material_texture.clone(),
                        _material_texture_view: entry.material_texture_view.clone(),
                    });
                }
                if cache_hit {
                    continue;
                }

                hot_path.static_cache_misses = hot_path.static_cache_misses.saturating_add(1);
                let vertices: Vec<MeshVertexRaw> = mesh
                    .vertices
                    .iter()
                    .map(|v| MeshVertexRaw {
                        position: v.position,
                        uv: v.uv,
                    })
                    .collect();
                hot_path.vertex_count = hot_path.vertex_count.saturating_add(vertices.len() as u64);
                hot_path.index_count = hot_path
                    .index_count
                    .saturating_add(mesh.indices.len() as u64);
                hot_path.vertex_upload_bytes = hot_path
                    .vertex_upload_bytes
                    .saturating_add((vertices.len() * std::mem::size_of::<MeshVertexRaw>()) as u64);
                hot_path.index_upload_bytes = hot_path
                    .index_upload_bytes
                    .saturating_add((mesh.indices.len() * std::mem::size_of::<u32>()) as u64);
                hot_path.uniform_upload_bytes = hot_path
                    .uniform_upload_bytes
                    .saturating_add(std::mem::size_of::<MeshMaterialRaw>() as u64);

                let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_mesh_vertices"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_mesh_indices"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: BufferUsages::INDEX,
                });

                let material_raw = MeshMaterialRaw {
                    base_color_factor: mesh.material.base_color_factor,
                };
                let material_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_mesh_material_uniform"),
                    contents: bytemuck::bytes_of(&material_raw),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                });

                let (material_texture, material_texture_view) =
                    if let Some(tex) = mesh.material.base_color_texture.as_ref() {
                        hot_path.textured_meshes = hot_path.textured_meshes.saturating_add(1);
                        hot_path.texture_upload_bytes = hot_path
                            .texture_upload_bytes
                            .saturating_add(tex.rgba8.len() as u64);
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
                    label: Some("engine_mesh_material_bind_group"),
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
                            resource: BindingResource::Sampler(if mesh.material.nearest_sampling {
                                &mesh_pass.nearest_sampler
                            } else {
                                &mesh_pass.default_sampler
                            }),
                        },
                    ],
                });

                let cache_entry = MeshCacheEntry {
                    signature,
                    index_count: mesh.indices.len() as u32,
                    vertex_buffer: vertex_buffer.clone(),
                    index_buffer: index_buffer.clone(),
                    material_buffer: material_buffer.clone(),
                    material_bind_group: material_bind_group.clone(),
                    material_texture: material_texture.clone(),
                    material_texture_view: material_texture_view.clone(),
                };
                self.mesh_cache.insert(key, cache_entry);

                draws.push(MeshPreparedDrawItem {
                    index_count: mesh.indices.len() as u32,
                    instance_count: 1,
                    vertex_buffer: Some(vertex_buffer),
                    index_buffer: Some(index_buffer),
                    instance_buffer: None,
                    _material_buffer: Some(material_buffer),
                    material_bind_group: Some(material_bind_group),
                    _material_texture: material_texture,
                    _material_texture_view: material_texture_view,
                });
            }
        }
        self.mesh_cache
            .retain(|key, _| live_cache_keys.contains(key));
        hot_path.static_upload_ms = static_upload_start.elapsed().as_secs_f32() * 1000.0;

        if !world_frame.agents.is_empty() {
            let agent_upload_start = Instant::now();
            {
                let _span = tracing::info_span!("mesh.upload_agents").entered();
                let mesh = unit_cube_mesh();
                let vertices: Vec<MeshVertexRaw> = mesh
                    .vertices
                    .into_iter()
                    .map(|v| MeshVertexRaw {
                        position: v.position,
                        uv: v.uv,
                    })
                    .collect();
                let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_mesh_agent_cube_vertices"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                    label: Some("engine_mesh_agent_cube_indices"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: BufferUsages::INDEX,
                });
                hot_path.vertex_upload_bytes = hot_path
                    .vertex_upload_bytes
                    .saturating_add((vertices.len() * std::mem::size_of::<MeshVertexRaw>()) as u64);
                hot_path.index_upload_bytes = hot_path
                    .index_upload_bytes
                    .saturating_add((mesh.indices.len() * std::mem::size_of::<u32>()) as u64);
                let instances = agent_instance_data(&world_frame.agents);
                hot_path.agent_instances = instances.len() as u32;
                if !instances.is_empty() {
                    hot_path.instance_upload_bytes = hot_path.instance_upload_bytes.saturating_add(
                        (instances.len() * std::mem::size_of::<MeshInstanceRaw>()) as u64,
                    );
                    hot_path.uniform_upload_bytes = hot_path
                        .uniform_upload_bytes
                        .saturating_add(std::mem::size_of::<MeshMaterialRaw>() as u64);

                    let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                        label: Some("engine_mesh_agent_cube_instances"),
                        contents: bytemuck::cast_slice(&instances),
                        usage: BufferUsages::VERTEX,
                    });
                    let material_raw = MeshMaterialRaw {
                        base_color_factor: [1.0, 1.0, 1.0, 1.0],
                    };
                    let material_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
                        label: Some("engine_mesh_agent_material_uniform"),
                        contents: bytemuck::bytes_of(&material_raw),
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    });
                    let material_bind_group = device.create_bind_group(&BindGroupDescriptor {
                        label: Some("engine_mesh_agent_material_bind_group"),
                        layout: &mesh_pass.material_bind_group_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: material_buffer.as_entire_binding(),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::TextureView(
                                    &mesh_pass.default_texture_view,
                                ),
                            },
                            BindGroupEntry {
                                binding: 2,
                                resource: BindingResource::Sampler(&mesh_pass.default_sampler),
                            },
                        ],
                    });
                    draws.push(MeshPreparedDrawItem {
                        index_count: mesh.indices.len() as u32,
                        instance_count: instances.len() as u32,
                        vertex_buffer: Some(vertex_buffer),
                        index_buffer: Some(index_buffer),
                        instance_buffer: Some(instance_buffer),
                        _material_buffer: Some(material_buffer),
                        material_bind_group: Some(material_bind_group),
                        _material_texture: None,
                        _material_texture_view: None,
                    });
                }
            }
            hot_path.agent_upload_ms = agent_upload_start.elapsed().as_secs_f32() * 1000.0;
        }
        hot_path.draw_items = draws.len() as u32;

        MeshPreparedWithHotPath {
            prepared: MeshPreparedDraw {
                draws,
                surface_size,
            },
            hot_path,
        }
    }

    fn encode_mesh_pass(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        prepared: &MeshPreparedDraw,
    ) {
        if prepared.draws.is_empty() || self.mesh_pass.is_none() {
            return;
        }
        self.ensure_mesh_surface_targets(device, prepared.surface_size);
        let Some(mesh_pass) = self.mesh_pass.as_ref() else {
            return;
        };
        let Some(targets) = self.mesh_surface_targets.as_ref() else {
            return;
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_mesh_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &targets.msaa_view,
                depth_slice: None,
                resolve_target: Some(frame_view),
                ops: Operations {
                    load: LoadOp::Clear(MESH_CLEAR_COLOR),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &targets.depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
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
            if let Some(instance_buffer) = draw.instance_buffer.as_ref() {
                pass.set_vertex_buffer(1, instance_buffer.slice(..));
            } else {
                pass.set_vertex_buffer(1, mesh_pass.default_instance_buffer.slice(..));
            }
            pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
            pass.draw_indexed(0..draw.index_count, 0, 0..draw.instance_count.max(1));
        }
    }

    fn encode_ui_pass(
        &self,
        encoder: &mut CommandEncoder,
        frame_view: &TextureView,
        prepared: &UiPreparedDraws,
    ) {
        let Some(rect_pass) = self.rect_pass.as_ref() else {
            return;
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("engine_ui_pass"),
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

    fn build_frame_graph(
        &self,
        render_graph_registry: &RenderGraphRegistryResource,
    ) -> FrameGraphBuildOutput {
        let descriptors = self.resolved_registered_descriptors(render_graph_registry);
        let mut output = self.build_frame_graph_from_descriptors(&descriptors);
        if output.handles.is_empty() {
            output.diagnostics.no_registered_passes = true;
        }
        output
    }

    pub(crate) fn prepare_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        world_frame: &WorldRenderFrame,
        draw_list: &UiDrawList,
        shader_registry: &mut ShaderRegistryResource,
        surface_format: TextureFormat,
        surface_width: f32,
        surface_height: f32,
    ) -> RendererPreparedPacket {
        let mut prepare_timings = RendererFrameTimings::default();
        let ui_rect_shader = shader_registry
            .source_or(UI_RECT_SHADER_ID, DEFAULT_UI_RECT_SHADER)
            .to_string();
        let ui_rect_revision = shader_registry.revision_for(UI_RECT_SHADER_ID);

        self.ensure_rect_pass(device, surface_format, &ui_rect_shader, ui_rect_revision);
        self.ensure_mesh_pass(device, queue, surface_format);
        self.ensure_text_renderer(device, queue, surface_format);
        let surface_size = (
            surface_width.max(1.0).round() as u32,
            surface_height.max(1.0).round() as u32,
        );
        let mut merged_world_frame = world_frame.clone();
        merged_world_frame
            .model_proxies
            .extend(self.model_manager.collect_sdf_proxies());
        let prepare_ui_start = Instant::now();
        let prepared_ui = {
            let _span = tracing::info_span!("renderer.prepare_ui_draws").entered();
            self.prepare_ui_draws(device, queue, draw_list, surface_width, surface_height)
        };
        prepare_timings.prepare_ui_ms = prepare_ui_start.elapsed().as_secs_f32() * 1000.0;

        let prepare_mesh_start = Instant::now();
        let prepared_mesh = if merged_world_frame.render_world {
            let prepared = {
                let _span = tracing::info_span!("renderer.prepare_mesh_draws").entered();
                self.prepare_mesh_draw(
                    device,
                    queue,
                    &merged_world_frame,
                    surface_width,
                    surface_height,
                )
            };
            prepare_timings.prepare_mesh_ms = prepare_mesh_start.elapsed().as_secs_f32() * 1000.0;
            prepared
        } else {
            MeshPreparedWithHotPath {
                prepared: MeshPreparedDraw {
                    draws: Vec::new(),
                    surface_size,
                },
                hot_path: MeshPrepareHotPath::default(),
            }
        };
        prepare_timings.mesh_hot_path = prepared_mesh.hot_path;

        RendererPreparedPacket {
            surface_format,
            surface_size,
            merged_world_frame,
            prepared_ui,
            prepared_mesh: prepared_mesh.prepared,
            prepare_timings,
        }
    }

    pub(crate) fn render_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_view: &TextureView,
        packet: RendererPreparedPacket,
        render_graph_registry: &RenderGraphRegistryResource,
        render_executor_registry: &RenderPassExecutorRegistryResource,
    ) -> RendererFrameTimings {
        let world_scene = packet.merged_world_frame.world_scene_label.as_str();
        let overlay_scene = packet.merged_world_frame.overlay_scene_label.as_str();
        let frame_graph_output = self.build_frame_graph(render_graph_registry);
        self.log_frame_graph_diagnostics(
            world_scene,
            overlay_scene,
            render_graph_registry.revision(),
            &frame_graph_output.diagnostics,
        );
        let graph = frame_graph_output.graph;
        let fallback_order = frame_graph_output.handles;
        let pass_executor_bindings = frame_graph_output.pass_executor_bindings;
        let order = match graph.execution_order() {
            Ok(order) => {
                self.clear_execution_order_error();
                order
            }
            Err(err) => {
                self.log_execution_order_error_once(&err);
                fallback_order
            }
        };
        let mut active_executors = BTreeSet::new();
        for handle in &order {
            if let Some(node) = graph.node(*handle) {
                let executor_name = pass_executor_bindings
                    .get(&node.name)
                    .map(String::as_str)
                    .unwrap_or(node.name.as_str());
                active_executors.insert(executor_name.to_string());
            }
        }

        let mut timings = packet.prepare_timings;
        self.prepare_registered_passes(
            device,
            queue,
            &packet,
            &active_executors,
            render_executor_registry,
            &mut timings,
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_render_encoder"),
        });
        let frame_data = RenderFrameDataRegistry::new().with(&packet.merged_world_frame);

        let mut missing_executors = Vec::<(String, String)>::new();
        for handle in order {
            let Some(node) = graph.node(handle) else {
                continue;
            };
            let executor_name = pass_executor_bindings
                .get(&node.name)
                .map(String::as_str)
                .unwrap_or(node.name.as_str());
            if let Some(builtin) = render_executor_registry.resolve_builtin(executor_name) {
                let executor = Self::builtin_pass_executor(builtin);
                executor.encode(
                    self,
                    device,
                    &mut encoder,
                    frame_view,
                    &packet,
                    node.pipeline.clone(),
                );
                continue;
            }
            if let Some(custom) = render_executor_registry.resolve_custom(executor_name) {
                let uses_ui_dispatch = executor_name.eq_ignore_ascii_case("builtin_ui_composite")
                    || node.name.eq_ignore_ascii_case("ui_composite");
                if uses_ui_dispatch {
                    let mut dispatch_ui = |encoder: &mut CommandEncoder| -> Result<()> {
                        self.encode_ui_pass(encoder, frame_view, &packet.prepared_ui);
                        Ok(())
                    };
                    let mut ctx = RenderPassEncodeContext::new(
                        device,
                        &mut encoder,
                        frame_view,
                        &frame_data,
                        packet.surface_format,
                        packet.surface_size,
                        node.pipeline.clone(),
                    )
                    .with_ui_dispatch(&mut dispatch_ui);
                    if let Err(err) = custom.encode(&mut ctx) {
                        tracing::error!(
                            pass = node.name.as_str(),
                            executor = executor_name,
                            ?err,
                            "custom render pass executor encode failed"
                        );
                    }
                } else {
                    let mut dispatch_builtin = |encoder: &mut CommandEncoder,
                                                builtin: BuiltinRenderPassExecutor|
                     -> Result<()> {
                        let executor = Self::builtin_pass_executor(builtin);
                        executor.encode(
                            self,
                            device,
                            encoder,
                            frame_view,
                            &packet,
                            node.pipeline.clone(),
                        );
                        Ok(())
                    };
                    let mut ctx = RenderPassEncodeContext::new(
                        device,
                        &mut encoder,
                        frame_view,
                        &frame_data,
                        packet.surface_format,
                        packet.surface_size,
                        node.pipeline.clone(),
                    )
                    .with_builtin_dispatch(&mut dispatch_builtin);
                    if let Err(err) = custom.encode(&mut ctx) {
                        tracing::error!(
                            pass = node.name.as_str(),
                            executor = executor_name,
                            ?err,
                            "custom render pass executor encode failed"
                        );
                    }
                }
                continue;
            }
            missing_executors.push((node.name.clone(), executor_name.to_string()));
        }
        self.log_missing_executors_once(&missing_executors);

        let encode_submit_start = Instant::now();
        {
            let _span = tracing::info_span!("renderer.encode_submit").entered();
            queue.submit(std::iter::once(encoder.finish()));
        }
        timings.encode_submit_ms = encode_submit_start.elapsed().as_secs_f32() * 1000.0;
        timings
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_view: &TextureView,
        world_frame: &WorldRenderFrame,
        draw_list: &UiDrawList,
        shader_registry: &mut ShaderRegistryResource,
        render_graph_registry: &RenderGraphRegistryResource,
        render_executor_registry: &RenderPassExecutorRegistryResource,
        surface_format: TextureFormat,
        surface_width: f32,
        surface_height: f32,
    ) -> RendererFrameTimings {
        let packet = self.prepare_packet(
            device,
            queue,
            world_frame,
            draw_list,
            shader_registry,
            surface_format,
            surface_width,
            surface_height,
        );
        self.render_packet(
            device,
            queue,
            frame_view,
            packet,
            render_graph_registry,
            render_executor_registry,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PassKind, PipelineKey, RenderGraphRegistryResource, Renderer, ResolvedFramePassDescriptor,
    };

    #[test]
    fn clip_to_scissor_clamps_and_rejects_empty() {
        let clipped = Renderer::clip_to_scissor([-10.0, 4.0, 20.0, 10.0], 100, 80)
            .expect("clip should intersect");
        assert_eq!(clipped, (0, 4, 10, 10));

        let none = Renderer::clip_to_scissor([200.0, 200.0, 10.0, 10.0], 100, 80);
        assert!(none.is_none());
    }

    #[test]
    fn build_frame_graph_from_descriptors_collects_diagnostics() {
        let renderer = Renderer::new();
        let descriptors = vec![
            ResolvedFramePassDescriptor {
                name: "".to_string(),
                kind: PassKind::Render,
                pipeline: PipelineKey::from("world_compose_fullscreen"),
                reads: Vec::new(),
                writes: Vec::new(),
                depends_on: Vec::new(),
                executor: "ui_composite".to_string(),
            },
            ResolvedFramePassDescriptor {
                name: "builtin_compute".to_string(),
                kind: PassKind::Compute,
                pipeline: PipelineKey::from("world_compute_basic"),
                reads: vec!["world_params".to_string()],
                writes: vec!["world_color".to_string()],
                depends_on: Vec::new(),
                executor: "builtin_compute".to_string(),
            },
            ResolvedFramePassDescriptor {
                name: "builtin_compute".to_string(),
                kind: PassKind::Compute,
                pipeline: PipelineKey::from("world_compute_high_contrast"),
                reads: vec!["world_params".to_string()],
                writes: vec!["world_color".to_string()],
                depends_on: Vec::new(),
                executor: "builtin_compute".to_string(),
            },
            ResolvedFramePassDescriptor {
                name: "builtin_compose".to_string(),
                kind: PassKind::Render,
                pipeline: PipelineKey::from("world_compose_fullscreen"),
                reads: vec!["world_color".to_string()],
                writes: vec!["surface_color".to_string()],
                depends_on: vec!["missing_pass".to_string()],
                executor: "builtin_compose".to_string(),
            },
        ];

        let output = renderer.build_frame_graph_from_descriptors(&descriptors);
        assert_eq!(output.handles.len(), 2);
        assert_eq!(output.diagnostics.empty_pass_name_count, 1);
        assert_eq!(
            output.diagnostics.duplicate_pass_names,
            vec!["builtin_compute".to_string()]
        );
        assert_eq!(
            output.diagnostics.missing_dependencies,
            vec![("builtin_compose".to_string(), "missing_pass".to_string())]
        );
    }

    #[test]
    fn build_frame_graph_reports_when_no_feature_graph_is_registered() {
        let renderer = Renderer::new();
        let output = renderer.build_frame_graph(&RenderGraphRegistryResource::default());
        assert!(output.handles.is_empty());
        assert!(output.diagnostics.no_registered_passes);
    }
}
