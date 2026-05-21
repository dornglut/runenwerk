use crate::plugins::render::RenderFlowId;
use crate::plugins::render::backend::WgpuCtx;
use crate::plugins::render::features::{
    FeatureContributionStatus, FeatureFallbackPolicy, UiFontAtlasResource,
};
use crate::plugins::render::frame::PreparedRenderFrame;
use crate::plugins::render::graph::{CompiledRenderFlowPlan, RenderExecutionGraphPreparedReport};
use crate::plugins::render::inspect::{
    PassTimingSample, RenderCaptureSelectorResult, RenderCapturedTexture,
    RenderDebugConfigResource, RenderDebugControlResource, RenderPassProvenanceRecord,
    ResolvedRenderCapturePlan, RuntimeResourceInspectionEntry,
};
use crate::plugins::render::shader::{ShaderHandle, ShaderRegistryResource};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;
use ui_render_data::{
    ProductSurfaceTextureBindingSource, ViewportSurfaceBindingRegistry,
    ViewportSurfaceBindingSource, ViewportSurfaceEmbedSlotId,
};
use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;

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

pub const DEFAULT_UI_STROKE_SHADER: &str = r#"
struct VsIn {
    @location(0) start : vec2<f32>,
    @location(1) end : vec2<f32>,
    @location(2) color : vec4<f32>,
    @location(3) width : f32,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) pixel : vec2<f32>,
    @location(1) segment_start : vec2<f32>,
    @location(2) segment_end : vec2<f32>,
    @location(3) color : vec4<f32>,
    @location(4) half_width : f32,
};

struct ScreenUniform {
    size : vec2<f32>,
    _pad : vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen : ScreenUniform;

@vertex
fn vs_main(input: VsIn, @builtin(vertex_index) vertex_index: u32) -> VsOut {
    let local = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    )[vertex_index];

    let half_width = max(input.width * 0.5, 0.5);
    let delta = input.end - input.start;
    let raw_segment_length = length(delta);
    let segment_length = max(raw_segment_length, 0.0001);
    let direction = select(vec2<f32>(1.0, 0.0), delta / segment_length, raw_segment_length > 0.0001);
    let normal = vec2<f32>(-direction.y, direction.x);
    let along = ((local.x + 1.0) * 0.5) * segment_length + local.x * half_width;
    let pixel = input.start + direction * along + normal * local.y * half_width;

    let x_ndc = (pixel.x / screen.size.x) * 2.0 - 1.0;
    let y_ndc = 1.0 - (pixel.y / screen.size.y) * 2.0;

    var out: VsOut;
    out.clip_position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.pixel = pixel;
    out.segment_start = input.start;
    out.segment_end = input.end;
    out.color = input.color;
    out.half_width = half_width;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let segment = input.segment_end - input.segment_start;
    let segment_len2 = max(dot(segment, segment), 0.0001);
    let t = clamp(dot(input.pixel - input.segment_start, segment) / segment_len2, 0.0, 1.0);
    let nearest = input.segment_start + segment * t;
    let dist = length(input.pixel - nearest);
    let aa = max(fwidth(dist), 0.75);
    let coverage = 1.0 - smoothstep(input.half_width - aa, input.half_width + aa, dist);
    if (coverage <= 0.001) {
        discard;
    }
    return vec4<f32>(input.color.rgb, input.color.a * coverage);
}
"#;

pub const DEFAULT_UI_GLYPH_SHADER: &str = r#"
struct VsIn {
    @location(0) rect : vec4<f32>,
    @location(1) uv_rect : vec4<f32>,
    @location(2) color : vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
    @location(1) color : vec4<f32>,
};

struct ScreenUniform {
    size : vec2<f32>,
    _pad : vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen : ScreenUniform;

@group(1) @binding(0)
var glyph_texture : texture_2d<f32>;

@group(1) @binding(1)
var glyph_sampler : sampler;

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
    let pixel = vec2<f32>(
        input.rect.x + input.rect.z * p.x,
        input.rect.y + input.rect.w * p.y
    );

    let x_ndc = (pixel.x / screen.size.x) * 2.0 - 1.0;
    let y_ndc = 1.0 - (pixel.y / screen.size.y) * 2.0;

    var out: VsOut;
    out.clip_position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.uv = vec2<f32>(
        input.uv_rect.x + input.uv_rect.z * p.x,
        input.uv_rect.y + input.uv_rect.w * p.y
    );
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let coverage = textureSample(glyph_texture, glyph_sampler, input.uv).r;
    if (coverage <= 0.001) {
        discard;
    }
    return vec4<f32>(input.color.rgb, input.color.a * coverage);
}
"#;

pub const DEFAULT_UI_VIEWPORT_EMBED_SHADER: &str = r#"
struct VsIn {
    @location(0) rect : vec4<f32>,
    @location(1) uv_rect : vec4<f32>,
    @location(2) tint : vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) tint : vec4<f32>,
    @location(1) local : vec2<f32>,
    @location(2) uv_rect : vec4<f32>,
};

struct ScreenUniform {
    size : vec2<f32>,
    _pad : vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen : ScreenUniform;

@group(1) @binding(0)
var viewport_texture : texture_2d<f32>;

@group(1) @binding(1)
var viewport_sampler : sampler;

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
    let pixel = vec2<f32>(
        input.rect.x + input.rect.z * p.x,
        input.rect.y + input.rect.w * p.y
    );

    let x_ndc = (pixel.x / screen.size.x) * 2.0 - 1.0;
    let y_ndc = 1.0 - (pixel.y / screen.size.y) * 2.0;

    var out: VsOut;
    out.clip_position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.tint = input.tint;
    out.local = p;
    out.uv_rect = input.uv_rect;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let sample_uv = vec2<f32>(
        input.uv_rect.x + input.uv_rect.z * input.local.x,
        input.uv_rect.y + input.uv_rect.w * input.local.y,
    );
    let sample_color = textureSample(viewport_texture, viewport_sampler, sample_uv);
    return sample_color * input.tint;
}
"#;

pub const DEFAULT_UI_PRODUCT_SURFACE_SHADER: &str = r#"
struct VsIn {
    @location(0) rect : vec4<f32>,
    @location(1) uv_rect : vec4<f32>,
    @location(2) tint : vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) tint : vec4<f32>,
    @location(1) local : vec2<f32>,
    @location(2) uv_rect : vec4<f32>,
};

struct ScreenUniform {
    size : vec2<f32>,
    _pad : vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen : ScreenUniform;

@group(1) @binding(0)
var product_texture : texture_2d<f32>;

@group(1) @binding(1)
var product_sampler : sampler;

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
    let pixel = vec2<f32>(
        input.rect.x + input.rect.z * p.x,
        input.rect.y + input.rect.w * p.y
    );

    let x_ndc = (pixel.x / screen.size.x) * 2.0 - 1.0;
    let y_ndc = 1.0 - (pixel.y / screen.size.y) * 2.0;

    var out: VsOut;
    out.clip_position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.tint = input.tint;
    out.local = p;
    out.uv_rect = input.uv_rect;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let sample_uv = vec2<f32>(
        input.uv_rect.x + input.uv_rect.z * input.local.x,
        input.uv_rect.y + input.uv_rect.w * input.local.y,
    );
    let sample_color = textureSample(product_texture, product_sampler, sample_uv);
    return sample_color * input.tint;
}
"#;

pub const DEFAULT_FULLSCREEN_SHADER: &str = r#"
struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );
    var out: VsOut;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.12, 0.14, 0.18, 1.0);
}
"#;

pub const DEFAULT_GRAPHICS_SHADER: &str = r#"
struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-0.6, -0.4),
        vec2<f32>(0.0, 0.6),
        vec2<f32>(0.6, -0.4),
    );
    var out: VsOut;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.9, 0.55, 0.22, 1.0);
}
"#;

pub const DEFAULT_COMPUTE_SHADER: &str = r#"
@compute @workgroup_size(1, 1, 1)
fn cs_main() {}
"#;

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
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct RectInstanceRaw {
    rect: [f32; 4],
    color: [f32; 4],
    radius: f32,
    _pad: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
struct FlattenedUiRectInstance {
    raw: RectInstanceRaw,
    clip: Option<[f32; 4]>,
    layer_order: u32,
    primitive_order: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct StrokeSegmentInstanceRaw {
    start: [f32; 2],
    end: [f32; 2],
    color: [f32; 4],
    width: f32,
    _pad: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
struct FlattenedUiStrokeSegmentInstance {
    raw: StrokeSegmentInstanceRaw,
    clip: Option<[f32; 4]>,
    layer_order: u32,
    primitive_order: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct GlyphInstanceRaw {
    rect: [f32; 4],
    uv_rect: [f32; 4],
    color: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
struct FlattenedUiGlyphInstance {
    raw: GlyphInstanceRaw,
    clip: Option<[f32; 4]>,
    texture_id: u64,
    layer_order: u32,
    primitive_order: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ViewportEmbedInstanceRaw {
    rect: [f32; 4],
    uv_rect: [f32; 4],
    tint: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
struct FlattenedUiViewportEmbedInstance {
    raw: ViewportEmbedInstanceRaw,
    clip: Option<[f32; 4]>,
    viewport_id: u64,
    slot: ViewportSurfaceEmbedSlotId,
    layer_order: u32,
    primitive_order: u32,
}

#[derive(Debug, Clone)]
struct FlattenedUiProductSurfaceInstance {
    raw: ViewportEmbedInstanceRaw,
    clip: Option<[f32; 4]>,
    source: ProductSurfaceTextureBindingSource,
    layer_order: u32,
    primitive_order: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ScreenUniformRaw {
    size: [f32; 2],
    _pad: [f32; 2],
}

#[derive(Debug)]
struct RectPass {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
}

#[derive(Debug)]
struct StrokePass {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
}

#[derive(Debug)]
struct GlyphPass {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
    texture_bind_group_layout: BindGroupLayout,
    texture_sampler: Sampler,
}

#[derive(Debug)]
struct ViewportEmbedPass {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
    texture_bind_group_layout: BindGroupLayout,
    texture_sampler: Sampler,
}

#[derive(Debug)]
struct ProductSurfacePass {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
    texture_bind_group_layout: BindGroupLayout,
    texture_sampler: Sampler,
}

#[derive(Debug, Clone)]
struct UiRectBatch {
    layer_order: u32,
    first_primitive_order: u32,
    last_primitive_order: u32,
    scissor: (u32, u32, u32, u32),
    instance_count: u32,
    instance_buffer: Buffer,
}

#[derive(Debug, Clone)]
struct UiStrokeBatch {
    layer_order: u32,
    first_primitive_order: u32,
    last_primitive_order: u32,
    scissor: (u32, u32, u32, u32),
    instance_count: u32,
    instance_buffer: Buffer,
}

#[derive(Debug, Clone)]
struct UiGlyphBatch {
    layer_order: u32,
    first_primitive_order: u32,
    last_primitive_order: u32,
    scissor: (u32, u32, u32, u32),
    instance_count: u32,
    instance_buffer: Buffer,
    texture_id: u64,
}

#[derive(Debug, Clone)]
struct UiViewportEmbedBatch {
    layer_order: u32,
    first_primitive_order: u32,
    last_primitive_order: u32,
    scissor: (u32, u32, u32, u32),
    instance_count: u32,
    instance_buffer: Buffer,
    viewport_id: u64,
    slot: ViewportSurfaceEmbedSlotId,
}

#[derive(Debug, Clone)]
struct UiProductSurfaceBatch {
    layer_order: u32,
    first_primitive_order: u32,
    last_primitive_order: u32,
    scissor: (u32, u32, u32, u32),
    instance_count: u32,
    instance_buffer: Buffer,
    source: ProductSurfaceTextureBindingSource,
}

#[derive(Debug)]
struct UiGlyphAtlasGpu {
    _texture: Texture,
    _view: TextureView,
    bind_group: BindGroup,
}

#[derive(Debug, Clone, Default)]
struct UiPreparedDraws {
    rect_batches: Vec<UiRectBatch>,
    stroke_batches: Vec<UiStrokeBatch>,
    glyph_batches: Vec<UiGlyphBatch>,
    viewport_embed_batches: Vec<UiViewportEmbedBatch>,
    product_surface_batches: Vec<UiProductSurfaceBatch>,
    draw_plan: Vec<UiPreparedDrawCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiPreparedDrawCommand {
    Rect(usize),
    Stroke(usize),
    ViewportEmbed(usize),
    ProductSurface(usize),
    Glyph(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FeatureExecutionGate {
    status: FeatureContributionStatus,
    fallback_policy: FeatureFallbackPolicy,
}

impl Default for FeatureExecutionGate {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedMaterialGpuResources {
    layout: BindGroupLayout,
    bind_group: BindGroup,
    _textures: Vec<Texture>,
    _texture_views: Vec<TextureView>,
    _samplers: Vec<Sampler>,
}

impl PreparedMaterialGpuResources {
    pub(crate) fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RendererPreparedPacket {
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    view_id: String,
    feature_gates: BTreeMap<RenderFeatureId, FeatureExecutionGate>,
    feature_runtime_signatures: BTreeMap<RenderFeatureId, u64>,
    prepared_material: Option<crate::plugins::render::PreparedMaterialFeatureContribution>,
    prepared_material_gpu_resources: Option<PreparedMaterialGpuResources>,
    prepared_ui: UiPreparedDraws,
    viewport_surface_bindings: ViewportSurfaceBindingRegistry,
    prepare_timings: RendererFrameTimings,
}

#[derive(Debug)]
pub struct Renderer {
    rect_pass: Option<RectPass>,
    rect_pass_format: Option<TextureFormat>,
    rect_pass_shader_revision: u64,
    stroke_pass: Option<StrokePass>,
    stroke_pass_format: Option<TextureFormat>,
    glyph_pass: Option<GlyphPass>,
    glyph_pass_format: Option<TextureFormat>,
    viewport_embed_pass: Option<ViewportEmbedPass>,
    viewport_embed_pass_format: Option<TextureFormat>,
    product_surface_pass: Option<ProductSurfacePass>,
    product_surface_pass_format: Option<TextureFormat>,
    glyph_atlas_gpu: BTreeMap<u64, UiGlyphAtlasGpu>,
    dynamic_texture_targets: dynamic_targets::RendererDynamicTextureTargetCache,
    flow_runtime_cache: BTreeMap<RenderFlowId, render_flow::FlowRuntimeResources>,
    flow_pipeline_cache: pipeline_cache::FlowPipelineArtifactCache,
    last_good_ui_prepared: Option<UiPreparedDraws>,
    last_pass_timings: Vec<PassTimingSample>,
    last_runtime_resources: Vec<RuntimeResourceInspectionEntry>,
    last_pass_provenance: Vec<RenderPassProvenanceRecord>,
    last_preflight_report: RenderExecutionGraphPreparedReport,
    last_capture_plan: ResolvedRenderCapturePlan,
    last_capture_selector_results: Vec<RenderCaptureSelectorResult>,
    last_captured_textures: Vec<RenderCapturedTexture>,
}

#[derive(Debug, ecs::Component, ecs::Resource)]
pub struct Gfx {
    pub ctx: WgpuCtx<'static>,
    pub renderer: Renderer,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GfxFrameTimings {
    pub acquire_ms: f32,
    pub renderer: RendererFrameTimings,
    pub present_ms: f32,
}

impl Gfx {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let ctx = WgpuCtx::new(window)?;
        Ok(Self {
            ctx,
            renderer: Renderer::new(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ctx.resize(width, height);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        prepared_frame: &PreparedRenderFrame,
        shader_registry: &mut ShaderRegistryResource,
        compiled_flows: &[CompiledRenderFlowPlan],
        ui_rect_shader: Option<ShaderHandle>,
        ui_font_atlas: &UiFontAtlasResource,
        viewport_surface_bindings: &ViewportSurfaceBindingRegistry,
        debug_control: &RenderDebugControlResource,
        debug_config: &RenderDebugConfigResource,
    ) -> Result<GfxFrameTimings> {
        let mut timings = GfxFrameTimings::default();
        let acquire_start = Instant::now();
        let frame = self.ctx.get_current_texture()?;
        timings.acquire_ms = acquire_start.elapsed().as_secs_f32() * 1000.0;
        let view = frame.texture.create_view(&Default::default());
        timings.renderer = self.renderer.render(
            &self.ctx.device,
            &self.ctx.queue,
            &frame.texture,
            &view,
            prepared_frame,
            shader_registry,
            compiled_flows,
            ui_rect_shader,
            ui_font_atlas,
            viewport_surface_bindings,
            self.ctx.surface_config.format,
            debug_control,
            debug_config,
        )?;

        let present_start = Instant::now();
        frame.present();
        timings.present_ms = present_start.elapsed().as_secs_f32() * 1000.0;
        Ok(timings)
    }
}

mod dynamic_targets;
mod extract;
mod pipeline_cache;
mod prepare;
mod render_flow;
mod setup;

pub mod frame_bindings;
use crate::plugins::RenderFeatureId;
pub use frame_bindings::RenderFrameDataRegistry;

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
