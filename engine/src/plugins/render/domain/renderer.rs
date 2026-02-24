use super::PipelineKey;
use super::frame_graph::{FrameGraph, PassHandle, PassKind};
use super::render_executor_registry::{
    BuiltinRenderPassExecutor, RenderFrameDataRegistry, RenderPassEncodeContext,
    RenderPassExecutorRegistryResource, RenderPassPrepareContext,
};
use super::render_graph_registry::{
    RegisteredPassKind, RegisteredPipelineRef, RenderGraphRegistryResource,
};
use super::shader_manager::{ShaderHandle, ShaderRegistryResource};
use crate::plugins::ui::domain::{FileFontProvider, TextRenderer};
use crate::plugins::ui::domain::{UiDrawCmd, UiDrawList};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
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

impl Renderer {
    fn builtin_pass_executor(
        executor: BuiltinRenderPassExecutor,
    ) -> &'static dyn FramePassExecutor {
        match executor {
            BuiltinRenderPassExecutor::Compute => &BUILTIN_COMPUTE_NOOP_PASS_EXECUTOR,
            BuiltinRenderPassExecutor::Compose => &BUILTIN_COMPOSE_NOOP_PASS_EXECUTOR,
            BuiltinRenderPassExecutor::MeshOverlay => &MESH_OVERLAY_NOOP_PASS_EXECUTOR,
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
        frame_data: &RenderFrameDataRegistry<'_>,
        packet: &RendererPreparedPacket,
        active_executors: &BTreeSet<String>,
        render_executor_registry: &RenderPassExecutorRegistryResource,
        timings: &mut RendererFrameTimings,
    ) {
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
                    frame_data,
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
            rect_pass: None,
            rect_pass_format: None,
            rect_pass_shader_revision: 0,
            text_renderer: None,
            text_renderer_format: None,
            last_frame_graph_diagnostics_hash: None,
            last_missing_executors_hash: None,
            last_execution_order_error_hash: None,
        }
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
        _frame_data: &RenderFrameDataRegistry<'_>,
        draw_list: &UiDrawList,
        shader_registry: &mut ShaderRegistryResource,
        ui_rect_shader_handle: Option<ShaderHandle>,
        surface_format: TextureFormat,
        surface_width: f32,
        surface_height: f32,
    ) -> RendererPreparedPacket {
        let mut prepare_timings = RendererFrameTimings::default();
        let ui_rect_shader = ui_rect_shader_handle
            .map(|handle| shader_registry.source_or_handle(handle, DEFAULT_UI_RECT_SHADER))
            .unwrap_or(DEFAULT_UI_RECT_SHADER)
            .to_string();
        let ui_rect_revision = ui_rect_shader_handle
            .map(|handle| shader_registry.revision_for_handle(handle))
            .unwrap_or(0);

        self.ensure_rect_pass(device, surface_format, &ui_rect_shader, ui_rect_revision);
        self.ensure_text_renderer(device, queue, surface_format);
        let surface_size = (
            surface_width.max(1.0).round() as u32,
            surface_height.max(1.0).round() as u32,
        );
        let world_scene_label = "unbound_world_scene".to_string();
        let overlay_scene_label = "unbound_overlay_scene".to_string();
        let prepare_ui_start = Instant::now();
        let prepared_ui = {
            let _span = tracing::info_span!("renderer.prepare_ui_draws").entered();
            self.prepare_ui_draws(device, queue, draw_list, surface_width, surface_height)
        };
        prepare_timings.prepare_ui_ms = prepare_ui_start.elapsed().as_secs_f32() * 1000.0;
        prepare_timings.prepare_mesh_ms = 0.0;
        prepare_timings.mesh_hot_path = MeshPrepareHotPath::default();

        RendererPreparedPacket {
            surface_format,
            surface_size,
            world_scene_label,
            overlay_scene_label,
            prepared_ui,
            prepare_timings,
        }
    }

    pub(crate) fn render_packet(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_view: &TextureView,
        frame_data: &RenderFrameDataRegistry<'_>,
        packet: RendererPreparedPacket,
        render_graph_registry: &RenderGraphRegistryResource,
        render_executor_registry: &RenderPassExecutorRegistryResource,
    ) -> RendererFrameTimings {
        let world_scene = packet.world_scene_label.as_str();
        let overlay_scene = packet.overlay_scene_label.as_str();
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
            frame_data,
            &packet,
            &active_executors,
            render_executor_registry,
            &mut timings,
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("engine_render_encoder"),
        });

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
                        frame_data,
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
                        frame_data,
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
        frame_data: &RenderFrameDataRegistry<'_>,
        draw_list: &UiDrawList,
        shader_registry: &mut ShaderRegistryResource,
        render_graph_registry: &RenderGraphRegistryResource,
        render_executor_registry: &RenderPassExecutorRegistryResource,
        ui_rect_shader: Option<ShaderHandle>,
        surface_format: TextureFormat,
        surface_width: f32,
        surface_height: f32,
    ) -> RendererFrameTimings {
        let packet = self.prepare_packet(
            device,
            queue,
            frame_data,
            draw_list,
            shader_registry,
            ui_rect_shader,
            surface_format,
            surface_width,
            surface_height,
        );
        self.render_packet(
            device,
            queue,
            frame_view,
            frame_data,
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
