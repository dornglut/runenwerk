use bytemuck::{Pod, Zeroable};
use engine::plugins::gpu::*;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::{Window, WindowId};

const COMPUTE_SOURCE_KEY: &str = "proof.reaction-diffusion.compute";
const RENDER_SOURCE_KEY: &str = "proof.reaction-diffusion.render";
const SOURCE_REVISION: u64 = 1;
const MANIFEST_SCHEMA_VERSION: u32 = 1;
const WORKGROUP: u32 = 8;
const FRAME_COUNT: u32 = 8;
const SURFACE_WIDTH: u32 = 64;
const SURFACE_HEIGHT: u32 = 64;
const SURFACE_FORMAT_CANDIDATES: [GpuTextureFormat; 4] = [
    GpuTextureFormat::Bgra8UnormSrgb,
    GpuTextureFormat::Rgba8UnormSrgb,
    GpuTextureFormat::Bgra8Unorm,
    GpuTextureFormat::Rgba8Unorm,
];

const REACTION_DIFFUSION_COMPUTE_WGSL: &str = r#"
struct Params {
    width: u32,
    height: u32,
    dt: f32,
    feed: f32,
    kill: f32,
    diffusion_a: f32,
    diffusion_b: f32,
    _pad: f32,
}

@group(0) @binding(0)
var<storage, read> state_in: array<vec2<f32>>;

@group(0) @binding(1)
var<storage, read_write> state_out: array<vec2<f32>>;

@group(0) @binding(2)
var<storage, read> params: Params;

fn wrapped_index(x: i32, y: i32) -> u32 {
    let width = i32(params.width);
    let height = i32(params.height);
    let wx = (x + width) % width;
    let wy = (y + height) % height;
    return u32(wy) * params.width + u32(wx);
}

fn sample(x: i32, y: i32) -> vec2<f32> {
    return state_in[wrapped_index(x, y)];
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.width || gid.y >= params.height) {
        return;
    }

    let x = i32(gid.x);
    let y = i32(gid.y);
    let center = sample(x, y);

    let cardinal =
        sample(x - 1, y) + sample(x + 1, y) +
        sample(x, y - 1) + sample(x, y + 1);
    let diagonal =
        sample(x - 1, y - 1) + sample(x + 1, y - 1) +
        sample(x - 1, y + 1) + sample(x + 1, y + 1);
    let laplacian = center * -1.0 + cardinal * 0.2 + diagonal * 0.05;

    let a = center.x;
    let b = center.y;
    let reaction = a * b * b;
    let next_a = clamp(
        a + (params.diffusion_a * laplacian.x - reaction + params.feed * (1.0 - a)) * params.dt,
        0.0,
        1.0,
    );
    let next_b = clamp(
        b + (params.diffusion_b * laplacian.y + reaction - (params.kill + params.feed) * b) * params.dt,
        0.0,
        1.0,
    );
    state_out[gid.y * params.width + gid.x] = vec2<f32>(next_a, next_b);
}
"#;

const REACTION_DIFFUSION_RENDER_WGSL: &str = r#"
struct Params {
    width: u32,
    height: u32,
    dt: f32,
    feed: f32,
    kill: f32,
    diffusion_a: f32,
    diffusion_b: f32,
    _pad: f32,
}

@group(0) @binding(0)
var<storage, read> state: array<vec2<f32>>;

@group(0) @binding(1)
var<storage, read> params: Params;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let x = min(u32(position.x), params.width - 1u);
    let y = min(u32(position.y), params.height - 1u);
    let cell = state[y * params.width + x];
    let b = clamp(cell.y, 0.0, 1.0);
    let contrast = clamp((cell.x - cell.y) * 0.5 + 0.5, 0.0, 1.0);
    return vec4<f32>(b, b * b, contrast, 1.0);
}
"#;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ReactionCell {
    a: f32,
    b: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ReactionParams {
    width: u32,
    height: u32,
    dt: f32,
    feed: f32,
    kill: f32,
    diffusion_a: f32,
    diffusion_b: f32,
    _pad: f32,
}

#[derive(Debug, Clone, Copy)]
struct Envelope {
    name: &'static str,
    width: u32,
    height: u32,
    frames: u32,
    iterations_per_frame: u32,
}

const ENVELOPES: [Envelope; 2] = [
    Envelope {
        name: "64x64-4-iterations",
        width: 64,
        height: 64,
        frames: FRAME_COUNT,
        iterations_per_frame: 4,
    },
    Envelope {
        name: "128x128-8-iterations",
        width: 128,
        height: 128,
        frames: FRAME_COUNT,
        iterations_per_frame: 8,
    },
];

#[derive(Clone)]
struct ProgramSources {
    compute: GpuAdmittedProgramSource,
    render: GpuAdmittedProgramSource,
}

fn label(value: &str) -> GpuResourceLabel {
    GpuResourceLabel::new(value).unwrap()
}

fn provenance(value: &str) -> GpuResourceProvenance {
    GpuResourceProvenance::new(label(value), None, None)
}

fn common(value: &str, lifetime: GpuResourceLifetime) -> GpuResourceCommon {
    GpuResourceCommon::owned(
        label(value),
        lifetime,
        GpuMemoryIntent::Device,
        GpuReconstruction::SourceBacked,
        provenance(value),
    )
    .unwrap()
}

fn reaction_params(width: u32, height: u32) -> ReactionParams {
    ReactionParams {
        width,
        height,
        dt: 1.0,
        feed: 0.055,
        kill: 0.062,
        diffusion_a: 1.0,
        diffusion_b: 0.5,
        _pad: 0.0,
    }
}
fn fixed_seed(width: u32, height: u32) -> Vec<ReactionCell> {
    let mut cells = vec![ReactionCell { a: 1.0, b: 0.0 }; usize::try_from(width * height).unwrap()];
    let half_w = width / 2;
    let half_h = height / 2;
    let radius = (width.min(height) / 10).max(2);
    for y in (half_h - radius)..(half_h + radius) {
        for x in (half_w - radius)..(half_w + radius) {
            let index = usize::try_from(y * width + x).unwrap();
            cells[index] = ReactionCell { a: 0.0, b: 1.0 };
        }
    }
    cells
}

fn admitted_sources() -> ProgramSources {
    let [compute, render] = admit_static_wgsl_sources([
        (
            COMPUTE_SOURCE_KEY,
            SOURCE_REVISION,
            REACTION_DIFFUSION_COMPUTE_WGSL,
        ),
        (
            RENDER_SOURCE_KEY,
            SOURCE_REVISION,
            REACTION_DIFFUSION_RENDER_WGSL,
        ),
    ])
    .unwrap();
    ProgramSources { compute, render }
}

fn compute_pipeline(source: &GpuAdmittedProgramSource) -> GpuComputePipelineDescriptor {
    GpuComputePipelineDescriptor::ordinary(source.clone(), "cs_main").unwrap()
}

fn render_pipeline(
    source: &GpuAdmittedProgramSource,
    format: GpuTextureFormat,
) -> GpuRenderPipelineDescriptor {
    GpuRenderPipelineDescriptor::ordinary_color(source.clone(), "vs_main", "fs_main", format)
        .unwrap()
}

fn buffer(resources: &mut GpuResourceScope, name: &str, byte_len: u64) -> GpuBufferHandle {
    let resource_label = label(name);
    resources
        .buffer(
            GpuBufferDescriptor::new(
                common(name, GpuResourceLifetime::Retained),
                byte_len,
                GpuBufferUsages::new(
                    &resource_label,
                    [GpuBufferUsage::Storage, GpuBufferUsage::CopyDestination],
                )
                .unwrap(),
                GpuBufferInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap()
}

fn runtime_binding(binding: u32, buffer: &GpuBufferHandle) -> GpuRuntimeBindingValue {
    GpuRuntimeBindingValue::new(
        GpuBindingKey::try_new(0, u64::from(binding)).unwrap(),
        [GpuRuntimeBindingResource::Buffer(
            GpuRuntimeBufferBinding::whole(buffer),
        )],
    )
    .unwrap()
}

fn compute_operation(
    pipeline: &GpuComputePipelineDescriptor,
    input: &GpuBufferHandle,
    output: &GpuBufferHandle,
    params: &GpuBufferHandle,
    width: u32,
    height: u32,
) -> GpuComputeOperation {
    let bindings = GpuRuntimeBindingSet::new(
        pipeline.layout().clone(),
        [
            runtime_binding(0, input),
            runtime_binding(1, output),
            runtime_binding(2, params),
        ],
    )
    .unwrap();
    let dispatch = GpuDispatchIntent::direct(
        GpuDispatchSize::new(width.div_ceil(WORKGROUP), height.div_ceil(WORKGROUP), 1).unwrap(),
    );
    GpuComputeOperation::new(pipeline.clone(), bindings, dispatch).unwrap()
}

fn render_operation(
    pipeline: &GpuRenderPipelineDescriptor,
    state: &GpuBufferHandle,
    params: &GpuBufferHandle,
    view: &GpuTextureViewHandle,
    width: u32,
    height: u32,
) -> GpuRenderOperation {
    let bindings = GpuRuntimeBindingSet::new(
        pipeline.layout().clone(),
        [runtime_binding(0, state), runtime_binding(1, params)],
    )
    .unwrap();
    let draw = GpuRenderDraw::new(
        pipeline.clone(),
        bindings,
        [],
        None,
        GpuDrawIntent::direct(
            GpuDrawRange::new(0, 3).unwrap(),
            GpuDrawRange::new(0, 1).unwrap(),
        ),
        GpuViewport::new(0.0, 0.0, width as f32, height as f32, 0.0, 1.0).unwrap(),
        GpuScissorRect::new(0, 0, width, height).unwrap(),
        GpuBlendConstant::new(0.0, 0.0, 0.0, 0.0).unwrap(),
        0,
    )
    .unwrap();
    let attachment = GpuRenderColorAttachment::new(
        view.clone(),
        GpuColorAttachmentLoad::Clear(GpuColorClearValue::new(0.0, 0.0, 0.0, 1.0).unwrap()),
        GpuAttachmentStore::Store,
        None,
    )
    .unwrap();
    GpuRenderOperation::new([attachment], None, [draw], None).unwrap()
}

fn upload_operation<T: Pod>(
    name: &str,
    buffer: &GpuBufferHandle,
    values: &[T],
) -> GpuUploadOperation {
    let region = GpuBufferRegion::new(buffer, GpuBufferRange::whole(buffer).unwrap()).unwrap();
    let data =
        PreparedGpuData::<TransferData>::from_pod_transfer(name, values, provenance(name)).unwrap();
    GpuUploadOperation::new(region.into(), data).unwrap()
}

fn add_operation(
    builder: &mut GpuWorkFragmentBuilder,
    name: &str,
    operation: GpuWorkOperation,
) -> Result<(), GpuWorkAuthoringError> {
    builder.operation(name, operation).map(|_| ())
}

fn state_resources(
    resources: &mut GpuResourceScope,
    envelope: Envelope,
) -> (
    GpuBufferHandle,
    GpuBufferHandle,
    GpuBufferHandle,
    Vec<ReactionCell>,
    ReactionParams,
) {
    let seed = fixed_seed(envelope.width, envelope.height);
    let state_bytes = u64::try_from(seed.len() * core::mem::size_of::<ReactionCell>()).unwrap();
    let params = reaction_params(envelope.width, envelope.height);
    let params_bytes = u64::try_from(core::mem::size_of::<ReactionParams>()).unwrap();
    (
        buffer(
            resources,
            &format!("{} state a", envelope.name),
            state_bytes,
        ),
        buffer(
            resources,
            &format!("{} state b", envelope.name),
            state_bytes,
        ),
        buffer(
            resources,
            &format!("{} params", envelope.name),
            params_bytes,
        ),
        seed,
        params,
    )}

fn offscreen_target(
    resources: &mut GpuResourceScope,
    envelope: Envelope,
) -> (GpuTextureHandle, GpuTextureViewHandle) {
    let name = format!("{} offscreen target", envelope.name);
    let texture_label = label(&name);
    let texture = resources
        .texture(
            GpuTextureDescriptor::new(
                common(&name, GpuResourceLifetime::Transient),
                GpuTextureDimension::D2,
                GpuTextureExtent::new(
                    &texture_label,
                    GpuTextureDimension::D2,
                    envelope.width,
                    envelope.height,
                    1,
                )
                .unwrap(),
                1,
                1,
                GpuTextureFormat::Rgba8Unorm,
                GpuTextureUsages::new(
                    &texture_label,
                    [
                        GpuTextureUsage::ColorAttachment,
                        GpuTextureUsage::CopySource,
                    ],
                )
                .unwrap(),
                GpuTextureInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap();
    let subresources = GpuTextureSubresourceRange::new(
        texture.descriptor().common().label(),
        0,
        1,
        0,
        1,
        GpuTextureAspect::Color,
    )
    .unwrap();
    let view = resources
        .texture_view(
            GpuTextureViewDescriptor::new(
                common(
                    &format!("{} offscreen target view", envelope.name),
                    GpuResourceLifetime::Transient,
                ),
                &texture,
                None,
                GpuTextureDimension::D2,
                subresources,
            )
            .unwrap(),
        )
        .unwrap();
    (texture, view)
}

fn add_initialization(
    builder: &mut GpuWorkFragmentBuilder,
    state_a: &GpuBufferHandle,
    state_b: &GpuBufferHandle,
    params_buffer: &GpuBufferHandle,
    seed: &[ReactionCell],
    params: ReactionParams,
) -> Result<(), GpuWorkAuthoringError> {
    add_operation(
        builder,
        "reaction diffusion initialize state a",
        GpuWorkOperation::Upload(upload_operation(
            "reaction diffusion state a seed",
            state_a,
            seed,
        )),
    )?;
    add_operation(
        builder,
        "reaction diffusion initialize state b",
        GpuWorkOperation::Upload(upload_operation(
            "reaction diffusion state b seed",
            state_b,
            seed,
        )),
    )?;
    add_operation(
        builder,
        "reaction diffusion upload parameters",
        GpuWorkOperation::Upload(upload_operation(
            "reaction diffusion parameters",
            params_buffer,
            &[params],
        )),
    )?;
    Ok(())
}

fn assert_prepared_graph_evidence(graph: &GpuPreparedWorkGraph) {
    assert!(
        graph.initialization().len() >= 3,
        "prepared graph must retain initialization evidence for both ping-pong states and parameters"
    );
    assert!(
        !graph.dependencies().is_empty(),
        "compute/render/readback ordering must be inferred from accepted resource hazards"
    );
    assert!(
        graph
            .dependencies()
            .iter()
            .flat_map(|dependency| dependency.reasons())
            .all(|reason| !matches!(reason, GpuDependencyReason::ExplicitNonData { .. })),
        "reaction-diffusion proof must not rely on duplicate manual ordering"
    );
    for feature in [
        GpuCapabilityFeature::Compute,
        GpuCapabilityFeature::RenderPipeline,
        GpuCapabilityFeature::Copy,
    ] {
        assert!(
            graph.requirements().get(feature).is_some(),
            "prepared graph must mechanically require {feature:?}"
        );
    }
}

fn offscreen_work(
    sources: &ProgramSources,
    envelope: Envelope,
) -> (GpuResourceLabel, GpuWorkFragment, Vec<GpuReadbackId>) {
    assert!(envelope.width >= 32 && envelope.height >= 32);
    assert!(envelope.frames >= 8);
    assert!(envelope.iterations_per_frame > 0);
    let mut resources = GpuResourceScope::new();
    let (state_a, state_b, params_buffer, seed, params) = state_resources(&mut resources, envelope);
    let (texture, view) = offscreen_target(&mut resources, envelope);
    let compute = compute_pipeline(&sources.compute);
    let render = render_pipeline(&sources.render, GpuTextureFormat::Rgba8Unorm);

    let name = format!("{} reaction diffusion sequence", envelope.name);
    let graph_label = label(&format!("{} reaction diffusion graph", envelope.name));
    let mut readbacks = Vec::with_capacity(usize::try_from(envelope.frames).unwrap());
    let fragment = GpuWorkFragment::build(&name, |builder| {
        add_initialization(builder, &state_a, &state_b, &params_buffer, &seed, params)?;

        let mut current_is_a = true;
        for frame in 0..envelope.frames {
            for iteration in 0..envelope.iterations_per_frame {
                let (input, output) = if current_is_a {
                    (&state_a, &state_b)
                } else {
                    (&state_b, &state_a)
                };
                let operation = compute_operation(
                    &compute,
                    input,
                    output,
                    &params_buffer,
                    envelope.width,
                    envelope.height,
                );
                builder.compute(
                    format!(
                        "{} frame {frame:03} iteration {iteration:03}",
                        envelope.name
                    ),
                    operation,
                )?;
                current_is_a = !current_is_a;
            }

            let state = if current_is_a { &state_a } else { &state_b };
            add_operation(
                builder,
                &format!("{} render frame {frame:03}", envelope.name),
                GpuWorkOperation::Render(render_operation(
                    &render,
                    state,
                    &params_buffer,
                    &view,
                    envelope.width,
                    envelope.height,
                )),
            )?;
            let region = GpuTextureCopyRegion::new(
                &texture,
                0,
                GpuTextureOrigin::new(0, 0, 0),
                GpuTextureAspect::Color,
                GpuCopyExtent::new(envelope.width, envelope.height, 1).unwrap(),
            )
            .unwrap();
            let readback_id = GpuReadbackId::allocate().unwrap();
            add_operation(
                builder,
                &format!("{} readback frame {frame:03}", envelope.name),
                GpuWorkOperation::Readback(
                    GpuReadbackOperation::new(region.into(), readback_id).unwrap(),
                ),
            )?;
            readbacks.push(readback_id);
        }
        Ok(())
    })
    .unwrap();

    (graph_label, fragment, readbacks)
}

fn progress_to_readbacks(
    context: &GpuContext,
    submission: &GpuSubmission,
    ids: &[GpuReadbackId],
) -> Vec<GpuReadbackBytes> {
    let handles = ids
        .iter()
        .map(|id| {
            submission
                .readback(*id)
                .expect("readback must remain observable")
                .clone()
        })
        .collect::<Vec<_>>();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        context.progress();
        let mut all_ready = true;
        for handle in &handles {
            match handle.status() {
                GpuReadbackStatus::Ready(_) => {}
                GpuReadbackStatus::Failed(failure) => {
                    panic!("reaction-diffusion readback failed: {failure:?}")
                }
                GpuReadbackStatus::Pending => all_ready = false,
            }
        }
        match submission.status() {
            GpuSubmissionStatus::Failed(failure) => {
                panic!("reaction-diffusion submission failed: {failure:?}")
            }
            GpuSubmissionStatus::Completed if all_ready => break,
            GpuSubmissionStatus::Accepted | GpuSubmissionStatus::Completed => {}
        }
        assert!(
            Instant::now() < deadline,
            "reaction-diffusion submission/readbacks timed out"
        );
        std::thread::yield_now();
    }
    handles
        .into_iter()
        .map(|handle| match handle.status() {
            GpuReadbackStatus::Ready(bytes) => bytes,
            other => panic!("terminal readback must be ready, got {other:?}"),
        })
        .collect()
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
    }
    digest
}

fn validate_frame(bytes: &GpuReadbackBytes, envelope: Envelope) -> u64 {
    assert_eq!(bytes.texture_format(), Some(GpuTextureFormat::Rgba8Unorm));
    assert_eq!(
        bytes.as_bytes().len(),
        usize::try_from(envelope.width * envelope.height * 4).unwrap()
    );
    let (pixels, remainder) = bytes.as_bytes().as_chunks::<4>();
    assert!(remainder.is_empty());
    assert!(pixels.iter().all(|pixel| pixel[3] == 255));
    let first = pixels[0];
    assert!(
        pixels.iter().any(|pixel| *pixel != first),
        "reaction-diffusion frame must contain spatially varying output"
    );
    fnv1a64(bytes.as_bytes())
}

fn artifact_root() -> PathBuf {
    std::env::var_os("RUNEN_GPU_PROOF_ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/runengpu-proof-artifacts"))
        .join("reaction-diffusion")
}

fn write_png(path: &Path, bytes: &GpuReadbackBytes, envelope: Envelope) {
    image::save_buffer_with_format(
        path,
        bytes.as_bytes(),
        envelope.width,
        envelope.height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap();
}

fn native_offscreen_context() -> GpuContext {
    let requirements = GpuCapabilityProfile::ComputeBaseline
        .requirements()
        .merge(&GpuCapabilityProfile::OffscreenGraphicsBaseline.requirements())
        .unwrap();
    let descriptor = GpuContextDescriptor::new(requirements)
        .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::ColorAttachment)
        .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::CopySource)
        .with_fallback_policy(GpuSoftwareFallbackPolicy::Require)
        .with_allowed_backends([GpuBackendFamily::Vulkan])
        .with_label("reaction-diffusion offscreen sequence proof");
    let context = pollster::block_on(GpuContext::request(descriptor))
        .expect("native conformance environment must provide a Vulkan fallback adapter");
    assert_eq!(context.adapter_facts().backend(), GpuBackendFamily::Vulkan);
    assert_eq!(
        context.adapter_facts().fallback(),
        GpuFallbackStatus::ConfirmedFallback
    );
    context
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by RunenGPU Native Conformance CI"]
fn native_reaction_diffusion_retains_bounded_png_sequences_and_manifest() {
    let context = native_offscreen_context();
    let sources = admitted_sources();
    let root = artifact_root();
    std::fs::create_dir_all(&root).unwrap();

    let mut jobs = Vec::new();
    for (envelope_index, envelope) in ENVELOPES.into_iter().enumerate() {
        let (graph_label, fragment, ids) = offscreen_work(&sources, envelope);
        let (submission, submission_path) = if envelope_index == 0 {
            let graph = GpuPreparedWorkGraph::prepare(graph_label, [fragment]).unwrap();
            assert_prepared_graph_evidence(&graph);
            let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
            (
                context.submit_prepared(prepared).unwrap(),
                "advanced-inspected",
            )
        } else {
            (
                pollster::block_on(context.submit_work(graph_label, [fragment])).unwrap(),
                "ordinary",
            )
        };
        let frames = progress_to_readbacks(&context, &submission, &ids);
        assert_eq!(frames.len(), usize::try_from(envelope.frames).unwrap());

        let job_dir = root.join(envelope.name);
        std::fs::create_dir_all(&job_dir).unwrap();
        let mut frame_records = Vec::new();
        for (frame_index, bytes) in frames.iter().enumerate() {
            let hash = validate_frame(bytes, envelope);
            let file_name = format!("frame_{frame_index:03}.png");
            let path = job_dir.join(&file_name);
            write_png(&path, bytes, envelope);
            assert!(path.metadata().unwrap().len() > 0);
            frame_records.push(json!({
                "logical_frame": frame_index,
                "png": format!("{}/{}", envelope.name, file_name),
                "fnv1a64": format!("{hash:016x}"),
            }));
        }
        assert_ne!(
            frame_records.first().unwrap()["fnv1a64"],
            frame_records.last().unwrap()["fnv1a64"],
            "bounded sequence must visibly evolve"
        );
        jobs.push(json!({
            "name": envelope.name,
            "width": envelope.width,
            "height": envelope.height,
            "logical_frames": envelope.frames,
            "iterations_per_frame": envelope.iterations_per_frame,
            "total_compute_iterations": envelope.frames * envelope.iterations_per_frame,
            "workgroup": [WORKGROUP, WORKGROUP, 1],
            "submission_path": submission_path,
            "frames": frame_records,
        }));
    }

    let enabled_features = context
        .device_facts()
        .enabled_features()
        .map(|feature| format!("{feature:?}"))
        .collect::<Vec<_>>();
    let manifest = json!({
        "schema_version": MANIFEST_SCHEMA_VERSION,
        "workload": "runengpu-reaction-diffusion",
        "model": "Gray-Scott",
        "seed": "A=1,B=0 with deterministic centered square A=0,B=1",
        "boundary": "toroidal-wrap",
        "parameters": {
            "dt": 1.0,
            "feed": 0.055,
            "kill": 0.062,
            "diffusion_a": 1.0,
            "diffusion_b": 0.5,
        },
        "backend": format!("{:?}", context.adapter_facts().backend()),
        "fallback": format!("{:?}", context.adapter_facts().fallback()),
        "enabled_capabilities": enabled_features,
        "programs": {
            "compute": {
                "source_key": sources.compute.identity().key().as_str(),
                "source_revision": sources.compute.identity().revision().get(),
                "canonical_wgsl_digest": sources.compute.digest().to_string(),
            },
            "render": {
                "source_key": sources.render.identity().key().as_str(),
                "source_revision": sources.render.identity().revision().get(),
                "canonical_wgsl_digest": sources.render.digest().to_string(),
            },
        },
        "jobs": jobs,
    });
    let manifest_path = root.join("manifest.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    assert!(manifest_path.metadata().unwrap().len() > 0);

    let stats = context.execution_stats();
    assert_eq!(stats.prepared_submissions(), 0);
    assert_eq!(stats.in_flight_submissions(), 0);
    assert_eq!(stats.upload_bytes_in_flight(), 0);
    assert_eq!(stats.readback_bytes_in_flight(), 0);
    assert_eq!(stats.pending_readbacks(), 0);
}

fn native_surface_context(window: Arc<Window>) -> (GpuContext, GpuSurfaceHandle, GpuTextureFormat) {
    let requirements = GpuCapabilityProfile::ComputeBaseline
        .requirements()
        .merge(&GpuCapabilityProfile::DesktopPresentationBaseline.requirements())
        .unwrap();
    let mut descriptor = GpuContextDescriptor::new(requirements);
    for format in SURFACE_FORMAT_CANDIDATES {
        descriptor = descriptor.require_format_role(format, GpuFormatRole::ColorAttachment);
    }
    let descriptor = descriptor
        .with_fallback_policy(GpuSoftwareFallbackPolicy::Require)
        .with_allowed_backends([GpuBackendFamily::Vulkan])
        .with_label("reaction-diffusion surface proof");
    let (context, surface) =
        pollster::block_on(GpuContext::request_for_surface(descriptor, window))
            .expect("Xvfb/Lavapipe must admit reaction-diffusion presentation");
    let capabilities = context.surface_capabilities(surface).unwrap();
    assert!(capabilities.supports_usage(GpuTextureUsage::ColorAttachment));
    let format = capabilities
        .formats()
        .iter()
        .copied()
        .find(|format| SURFACE_FORMAT_CANDIDATES.contains(format) && format.is_srgb())
        .or_else(|| {
            capabilities
                .formats()
                .iter()
                .copied()
                .find(|format| SURFACE_FORMAT_CANDIDATES.contains(format))
        })
        .expect("surface must expose an admitted SDR color-attachment format");
    let present_mode = capabilities
        .present_modes()
        .iter()
        .copied()
        .find(|mode| *mode == GpuSurfacePresentMode::Fifo)
        .or_else(|| capabilities.present_modes().first().copied())
        .unwrap();
    let alpha_mode = capabilities
        .alpha_modes()
        .iter()
        .copied()
        .find(|mode| *mode == GpuSurfaceAlphaMode::Opaque)
        .or_else(|| capabilities.alpha_modes().first().copied())
        .unwrap();
    let surface = context
        .configure_surface(
            surface,
            GpuSurfaceConfiguration::new(
                SURFACE_WIDTH,
                SURFACE_HEIGHT,
                format,
                [GpuTextureUsage::ColorAttachment],
                present_mode,
                alpha_mode,
                2,
                [],
            )
            .unwrap(),
        )
        .unwrap();
    (context, surface, format)
}

fn surface_graph(
    sources: &ProgramSources,
    image: &GpuAcquiredSurfaceImage,
    format: GpuTextureFormat,
) -> GpuPreparedWorkGraph {
    let envelope = ENVELOPES[0];
    let mut resources = GpuResourceScope::new();
    let (state_a, state_b, params_buffer, seed, params) = state_resources(&mut resources, envelope);
    let compute = compute_pipeline(&sources.compute);
    let render = render_pipeline(&sources.render, format);
    let view = image.default_view().clone();

    let fragment = GpuWorkFragment::build("reaction diffusion surface replay", |builder| {
        add_initialization(builder, &state_a, &state_b, &params_buffer, &seed, params)?;

        let mut current_is_a = true;
        for frame in 0..envelope.frames {
            for iteration in 0..envelope.iterations_per_frame {
                let (input, output) = if current_is_a {
                    (&state_a, &state_b)
                } else {
                    (&state_b, &state_a)
                };
                builder.compute(
                    format!("surface frame {frame:03} iteration {iteration:03}"),
                    compute_operation(
                        &compute,
                        input,
                        output,
                        &params_buffer,
                        envelope.width,
                        envelope.height,
                    ),
                )?;
                current_is_a = !current_is_a;
            }
        }
        let state = if current_is_a { &state_a } else { &state_b };
        add_operation(
            builder,
            "reaction diffusion surface render",
            GpuWorkOperation::Render(render_operation(
                &render,
                state,
                &params_buffer,
                &view,
                SURFACE_WIDTH,
                SURFACE_HEIGHT,
            )),
        )?;
        add_operation(
            builder,
            "reaction diffusion surface Present",
            GpuWorkOperation::Present(
                GpuPresentOperation::new(view.clone().into(), view.descriptor().subresources())
                    .unwrap(),
            ),
        )?;
        Ok(())
    })
    .unwrap();

    let graph =
        GpuPreparedWorkGraph::prepare(label("reaction diffusion surface graph"), [fragment])
            .unwrap();
    assert_prepared_graph_evidence(&graph);
    graph
}

fn progress_to_completion(context: &GpuContext, submission: &GpuSubmission) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Completed => break,
            GpuSubmissionStatus::Failed(failure) => {
                panic!("reaction-diffusion surface submission failed: {failure:?}")
            }
            GpuSubmissionStatus::Accepted => {}
        }
        assert!(Instant::now() < deadline, "surface proof timed out");
        std::thread::yield_now();
    }
}

fn run_surface_proof(window: Arc<Window>) {
    let (context, surface, format) = native_surface_context(window);
    let image = context.acquire_surface_image(surface).unwrap();
    let sources = admitted_sources();
    let graph = surface_graph(&sources, &image, format);
    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    let submission = context.submit_prepared(prepared).unwrap();
    progress_to_completion(&context, &submission);

    let next = context
        .acquire_surface_image(surface)
        .expect("successful reaction-diffusion Present must release the surface image lease");
    assert_ne!(next.lease_id(), image.lease_id());
    next.abandon();
    context.detach_surface(surface).unwrap();
    drop(image);

    let stats = context.execution_stats();
    assert_eq!(stats.prepared_submissions(), 0);
    assert_eq!(stats.in_flight_submissions(), 0);
    assert_eq!(stats.upload_bytes_in_flight(), 0);
    assert_eq!(stats.readback_bytes_in_flight(), 0);
    assert_eq!(stats.pending_readbacks(), 0);
}

#[derive(Default)]
struct ReactionDiffusionSurfaceProof {
    ran: bool,
}

impl ApplicationHandler for ReactionDiffusionSurfaceProof {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ran {
            return;
        }
        self.ran = true;
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("RunenGPU reaction diffusion surface proof")
                        .with_inner_size(PhysicalSize::new(SURFACE_WIDTH, SURFACE_HEIGHT)),
                )
                .unwrap(),
        );
        run_surface_proof(window);
        event_loop.exit();
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}

#[test]
#[ignore = "requires Xvfb plus a real Vulkan fallback adapter; executed by RunenGPU Native Conformance CI"]
fn native_reaction_diffusion_replays_same_compute_to_render_path_to_surface_present() {
    let mut event_loop_builder = EventLoop::builder();
    #[cfg(target_os = "linux")]
    event_loop_builder.with_x11().with_any_thread(true);
    let event_loop = event_loop_builder.build().unwrap();
    let mut proof = ReactionDiffusionSurfaceProof::default();
    event_loop.run_app(&mut proof).unwrap();
    assert!(proof.ran);
}