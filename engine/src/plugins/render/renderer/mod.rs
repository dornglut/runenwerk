use crate::plugins::render::frame_graph::{
    BuiltinRenderPassExecutor, FrameGraph, PassHandle, PassKind, RegisteredPassKind,
    RegisteredPipelineRef, RenderGraphRegistryResource, RenderPassEncodeContext,
    RenderPassExecutorRegistryResource, RenderPassPrepareContext,
};
use crate::plugins::render::pipelines::PipelineKey;
use crate::plugins::render::resources::RenderFrameDataRegistry;
use crate::plugins::render::shader::{ShaderHandle, ShaderRegistryResource};
use crate::plugins::ui::domain::{FileFontProvider, TextRenderer, UiDrawCmd, UiDrawList};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use wgpu::util::DeviceExt;
use wgpu::*;

// Owner: Engine Renderer - Core Types and Builtin Executors

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

#[derive(Debug)]
struct RectPass {
    pipeline: RenderPipeline,
    screen_buffer: Buffer,
    screen_bind_group: BindGroup,
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
    world_scene_label: String,
    overlay_scene_label: String,
    prepared_ui: UiPreparedDraws,
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
struct MeshOverlayNoopPassExecutor;

impl FramePassExecutor for MeshOverlayNoopPassExecutor {
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
                "builtin_mesh_overlay is not implemented in core render plugin; register a custom executor instead"
            );
        }
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
const MESH_OVERLAY_NOOP_PASS_EXECUTOR: MeshOverlayNoopPassExecutor = MeshOverlayNoopPassExecutor;
const UI_COMPOSITE_PASS_EXECUTOR: UiCompositePassExecutor = UiCompositePassExecutor;

#[derive(Debug)]
pub struct Renderer {
    rect_pass: Option<RectPass>,
    rect_pass_format: Option<TextureFormat>,
    rect_pass_shader_revision: u64,
    text_renderer: Option<TextRenderer>,
    text_renderer_format: Option<TextureFormat>,
    last_frame_graph_diagnostics_hash: Option<u64>,
    last_missing_executors_hash: Option<u64>,
    last_execution_order_error_hash: Option<u64>,
}

mod extract;
mod graph_execution;
mod prepare;
mod render_flow;
mod setup;

pub mod frame_bindings;
pub mod submit;

// Owner: Engine Renderer - Tests
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
