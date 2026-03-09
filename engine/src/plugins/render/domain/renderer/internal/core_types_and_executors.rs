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
