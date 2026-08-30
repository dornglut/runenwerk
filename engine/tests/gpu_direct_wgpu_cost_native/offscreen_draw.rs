use super::common::{
    DirectWgpuContext, MEASURED_SAMPLES, Measurements, WARMUP_SAMPLES, micros,
    padded_bytes_per_row, ratio_summary, submit_and_map, tightly_pack_texture_rows,
};
use engine::plugins::gpu::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;

const WIDTH: u32 = 8;
const HEIGHT: u32 = 8;
const CLEAR_PIXEL: [u8; 4] = [0, 0, 0, 255];
const DRAW_PIXEL: [u8; 4] = [255, 0, 0, 255];
const INDICES: [u32; 3] = [0, 1, 2];

// Byte-for-byte copy of the retained G6-C01 inline shader. Before G6-P01 acceptance this
// fixture should be extracted into shared test support so the retained proof and comparison
// consume one source of shader truth.
const KNOWN_PATTERN_WGSL: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let positions = array<vec2f, 3>(
        vec2f(-1.0, -1.0),
        vec2f(3.0, -1.0),
        vec2f(-1.0, 3.0),
    );
    return vec4f(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.0, 0.0, 1.0);
}
"#;

fn label(value: &str) -> GpuResourceLabel {
    GpuResourceLabel::new(value).unwrap()
}

fn provenance(value: &str) -> GpuResourceProvenance {
    GpuResourceProvenance::new(label(value), None, None)
}

fn common(value: &str) -> GpuResourceCommon {
    GpuResourceCommon::owned(
        label(value),
        GpuResourceLifetime::Transient,
        GpuMemoryIntent::Device,
        GpuReconstruction::SourceBacked,
        provenance(value),
    )
    .unwrap()
}

fn runengpu_context() -> GpuContext {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::OffscreenGraphicsBaseline.requirements())
            .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::ColorAttachment)
            .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::CopySource)
            .with_fallback_policy(GpuSoftwareFallbackPolicy::Require)
            .with_allowed_backends([GpuBackendFamily::Vulkan])
            .with_label("G6-P01 known-pattern RunenGPU comparison");
    let context = pollster::block_on(GpuContext::request(descriptor)).unwrap();
    assert_eq!(context.adapter_facts().backend(), GpuBackendFamily::Vulkan);
    assert_eq!(
        context.adapter_facts().fallback(),
        GpuFallbackStatus::ConfirmedFallback
    );
    context
}

fn admitted_render_source() -> GpuAdmittedProgramSource {
    let identity = GpuProgramSourceIdentity::new(
        GpuProgramSourceOwnerId::allocate().unwrap(),
        GpuProgramSourceKey::new("proof.direct-cost.offscreen.indexed-known-pattern").unwrap(),
        GpuProgramSourceRevision::try_from_raw(1).unwrap(),
    );
    let mut sources = GpuProgramSourceRegistry::new(4, 16 * 1024).unwrap();
    sources
        .admit_wgsl(
            identity,
            KNOWN_PATTERN_WGSL,
            GpuProgramSourceProvenance::new("direct-cost-indexed-offscreen", None).unwrap(),
        )
        .unwrap()
}

fn render_pipeline() -> GpuRenderPipelineDescriptor {
    let vertex = GpuEntryPointName::new("vs_main").unwrap();
    let fragment = GpuEntryPointName::new("fs_main").unwrap();
    let program = GpuProgramDescriptor::new(
        admitted_render_source(),
        [vertex.clone(), fragment.clone()],
        std::iter::empty::<GpuBindingLayoutRefinement>(),
    )
    .unwrap();
    let color_target = GpuColorTargetStateDescriptor::new(
        GpuTextureFormat::Rgba8Unorm,
        GpuBlendMode::Replace,
        GpuColorWriteMask::ALL,
    )
    .unwrap();
    let state = GpuRenderPipelineStateDescriptor::new(
        GpuVertexInputStateDescriptor::new([]).unwrap(),
        Some(GpuFragmentOutputStateDescriptor::new([color_target])),
        GpuPrimitiveStateDescriptor::default(),
        None,
        GpuMultisampleStateDescriptor::default(),
    )
    .unwrap();
    GpuRenderPipelineDescriptor::new(
        program,
        GpuRenderEntryPoints::new(vertex, Some(fragment)),
        state,
        GpuPipelineConfiguration::default(),
    )
    .unwrap()
}

fn runengpu_fragment() -> (GpuWorkFragment, GpuReadbackId) {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let texture_label = label("direct-cost indexed offscreen color target");
    let texture = allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::new(
                common("direct-cost indexed offscreen color target"),
                GpuTextureDimension::D2,
                GpuTextureExtent::new(&texture_label, GpuTextureDimension::D2, WIDTH, HEIGHT, 1)
                    .unwrap(),
                1,
                1,
                GpuTextureFormat::Rgba8Unorm,
                GpuTextureUsages::new(
                    &texture_label,
                    [GpuTextureUsage::ColorAttachment, GpuTextureUsage::CopySource],
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
    let view = allocator
        .allocate_texture_view_handle(
            GpuTextureViewDescriptor::new(
                common("direct-cost indexed offscreen color target view"),
                &texture,
                None,
                GpuTextureDimension::D2,
                subresources,
            )
            .unwrap(),
        )
        .unwrap();
    let index_label = label("direct-cost indexed offscreen index buffer");
    let index_data = PreparedGpuData::<TransferData>::from_pod_transfer(
        "direct-cost indexed offscreen indices",
        &INDICES,
        provenance("direct-cost indexed offscreen indices"),
    )
    .unwrap();
    let index_buffer = allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                common("direct-cost indexed offscreen index buffer"),
                u64::try_from(core::mem::size_of_val(&INDICES)).unwrap(),
                GpuBufferUsages::new(
                    &index_label,
                    [GpuBufferUsage::Index, GpuBufferUsage::CopyDestination],
                )
                .unwrap(),
                GpuBufferInitialization::Prepared(index_data),
            )
            .unwrap(),
        )
        .unwrap();

    let pipeline = render_pipeline();
    let bindings = GpuRuntimeBindingSet::new(pipeline.layout().clone(), []).unwrap();
    let index_binding = GpuIndexBufferBinding::new(
        &index_buffer,
        GpuBufferRange::whole(&index_buffer).unwrap(),
        GpuIndexFormat::Uint32,
    )
    .unwrap();
    let draw = GpuRenderDraw::new(
        pipeline,
        bindings,
        [],
        Some(index_binding),
        GpuDrawIntent::indexed(
            GpuDrawRange::new(0, u32::try_from(INDICES.len()).unwrap()).unwrap(),
            0,
            GpuDrawRange::new(0, 1).unwrap(),
        ),
        GpuViewport::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32, 0.0, 1.0).unwrap(),
        GpuScissorRect::new(0, 0, WIDTH / 2, HEIGHT).unwrap(),
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
    let render = GpuRenderOperation::new([attachment], None, [draw], None).unwrap();
    let readback_region = GpuTextureCopyRegion::new(
        &texture,
        0,
        GpuTextureOrigin::new(0, 0, 0),
        GpuTextureAspect::Color,
        GpuCopyExtent::new(WIDTH, HEIGHT, 1).unwrap(),
    )
    .unwrap();
    let readback_id = GpuReadbackId::allocate().unwrap();
    let readback = GpuReadbackOperation::new(readback_region.into(), readback_id).unwrap();

    let mut builder = GpuWorkFragmentBuilder::new(
        label("direct-cost indexed offscreen fragment"),
        provenance("direct-cost indexed offscreen fragment"),
    );
    builder.declare_resource(texture.into()).unwrap();
    builder.declare_resource(view.into()).unwrap();
    builder.declare_resource(index_buffer.into()).unwrap();
    builder
        .add_node(
            label("direct-cost indexed offscreen draw"),
            GpuWorkOperation::Render(render),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::Automatic,
            provenance("direct-cost indexed offscreen draw"),
        )
        .unwrap();
    builder
        .add_node(
            label("direct-cost indexed offscreen readback"),
            GpuWorkOperation::Readback(readback),
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::Automatic,
            provenance("direct-cost indexed offscreen readback"),
        )
        .unwrap();
    (builder.finish().unwrap(), readback_id)
}

fn progress_runengpu(
    context: &GpuContext,
    submission: &GpuSubmission,
    readback_id: GpuReadbackId,
) -> GpuReadbackBytes {
    let readback = submission.readback(readback_id).unwrap().clone();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        context.progress();
        match readback.status() {
            GpuReadbackStatus::Ready(bytes) => {
                loop {
                    context.progress();
                    match submission.status() {
                        GpuSubmissionStatus::Completed => return bytes,
                        GpuSubmissionStatus::Failed(failure) => {
                            panic!("RunenGPU known-pattern submission failed: {failure:?}")
                        }
                        GpuSubmissionStatus::Accepted => {}
                    }
                    assert!(Instant::now() < deadline, "RunenGPU completion timed out");
                    std::thread::yield_now();
                }
            }
            GpuReadbackStatus::Failed(failure) => {
                panic!("RunenGPU known-pattern readback failed: {failure:?}")
            }
            GpuReadbackStatus::Pending => {}
        }
        if let GpuSubmissionStatus::Failed(failure) = submission.status() {
            panic!("RunenGPU known-pattern submission failed: {failure:?}");
        }
        assert!(Instant::now() < deadline, "RunenGPU readback timed out");
        std::thread::yield_now();
    }
}

fn pixel_at(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = usize::try_from((y * WIDTH + x) * 4).unwrap();
    bytes[offset..offset + 4].try_into().unwrap()
}

fn assert_known_pattern(bytes: &[u8]) {
    assert_eq!(bytes.len(), usize::try_from(WIDTH * HEIGHT * 4).unwrap());
    for (x, y) in [(1, 1), (2, 6)] {
        assert_eq!(pixel_at(bytes, x, y), DRAW_PIXEL);
    }
    for (x, y) in [(5, 1), (6, 6)] {
        assert_eq!(pixel_at(bytes, x, y), CLEAR_PIXEL);
    }
}

fn runengpu_sample(context: &GpuContext) -> BTreeMap<String, f64> {
    let total_start = Instant::now();
    let author_start = Instant::now();
    let (fragment, readback_id) = runengpu_fragment();
    let author_us = micros(author_start.elapsed());

    let prepare_start = Instant::now();
    let graph = GpuPreparedWorkGraph::prepare(
        label("G6-P01 known-pattern RunenGPU graph"),
        [fragment],
    )
    .unwrap();
    let graph_prepare_us = micros(prepare_start.elapsed());

    let backend_prepare_start = Instant::now();
    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    let backend_prepare_us = micros(backend_prepare_start.elapsed());

    let submit_start = Instant::now();
    let submission = context.submit_prepared(prepared).unwrap();
    let submit_call_us = micros(submit_start.elapsed());

    let completion_start = Instant::now();
    let bytes = progress_runengpu(context, &submission, readback_id);
    let completion_readback_us = micros(completion_start.elapsed());
    assert_eq!(bytes.texture_format(), Some(GpuTextureFormat::Rgba8Unorm));
    assert_known_pattern(bytes.as_bytes());

    let mut phases = BTreeMap::new();
    phases.insert("authoring".to_owned(), author_us);
    phases.insert("graph_prepare".to_owned(), graph_prepare_us);
    phases.insert("backend_prepare".to_owned(), backend_prepare_us);
    phases.insert(
        "boundary_prepare_or_record".to_owned(),
        graph_prepare_us + backend_prepare_us,
    );
    phases.insert("submit_call".to_owned(), submit_call_us);
    phases.insert("completion_readback".to_owned(), completion_readback_us);
    phases.insert("total".to_owned(), micros(total_start.elapsed()));
    phases
}

struct DirectPipeline {
    pipeline: wgpu::RenderPipeline,
    cold_pipeline_us: f64,
}

fn direct_pipeline(context: &DirectWgpuContext) -> DirectPipeline {
    let start = Instant::now();
    let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("G6-P01 known-pattern shader"),
        source: wgpu::ShaderSource::Wgsl(KNOWN_PATTERN_WGSL.into()),
    });
    let layout = context
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("G6-P01 known-pattern pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
    let targets = [Some(wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rgba8Unorm,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    })];
    let pipeline = context
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("G6-P01 known-pattern pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &targets,
            }),
            multiview_mask: None,
            cache: None,
        });
    DirectPipeline {
        pipeline,
        cold_pipeline_us: micros(start.elapsed()),
    }
}

fn direct_sample(
    context: &DirectWgpuContext,
    pipeline: &wgpu::RenderPipeline,
) -> BTreeMap<String, f64> {
    let total_start = Instant::now();
    let resource_start = Instant::now();
    let texture = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("G6-P01 known-pattern target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let index_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("G6-P01 known-pattern index buffer"),
        size: u64::try_from(core::mem::size_of_val(&INDICES)).unwrap(),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let index_upload = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("G6-P01 known-pattern index upload"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsages::COPY_SRC,
        });
    let physical_row = padded_bytes_per_row(WIDTH * 4);
    let readback_size = u64::from(physical_row) * u64::from(HEIGHT);
    let readback = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("G6-P01 known-pattern readback"),
        size: readback_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let resource_setup_us = micros(resource_start.elapsed());

    let record_start = Instant::now();
    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("G6-P01 known-pattern encoder"),
        });
    encoder.copy_buffer_to_buffer(
        &index_upload,
        0,
        &index_buffer,
        0,
        u64::try_from(core::mem::size_of_val(&INDICES)).unwrap(),
    );
    {
        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("G6-P01 known-pattern pass"),
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.set_viewport(0.0, 0.0, WIDTH as f32, HEIGHT as f32, 0.0, 1.0);
        pass.set_scissor_rect(0, 0, WIDTH / 2, HEIGHT);
        pass.set_blend_constant(wgpu::Color::TRANSPARENT);
        pass.set_stencil_reference(0);
        pass.draw_indexed(0..u32::try_from(INDICES.len()).unwrap(), 0, 0..1);
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(physical_row),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
    );
    let command_buffer = encoder.finish();
    let command_record_us = micros(record_start.elapsed());

    let submitted = submit_and_map(context, command_buffer, &[&readback]);
    let bytes = tightly_pack_texture_rows(&submitted.mapped[0], WIDTH, HEIGHT, 4, physical_row);
    assert_known_pattern(&bytes);

    let mut phases = BTreeMap::new();
    phases.insert("resource_setup".to_owned(), resource_setup_us);
    phases.insert(
        "boundary_prepare_or_record".to_owned(),
        command_record_us,
    );
    phases.insert("command_record".to_owned(), command_record_us);
    phases.insert("submit_call".to_owned(), submitted.submit_call_us);
    phases.insert(
        "completion_readback".to_owned(),
        submitted.completion_readback_us,
    );
    phases.insert("total".to_owned(), micros(total_start.elapsed()));
    phases
}

pub(crate) fn compare() -> Value {
    let direct_context = DirectWgpuContext::request("G6-P01 known-pattern direct WGPU");
    let direct_pipeline = direct_pipeline(&direct_context);

    let runengpu_context_start = Instant::now();
    let runengpu_context = runengpu_context();
    let runengpu_context_us = micros(runengpu_context_start.elapsed());

    let mut runengpu_cold = None;
    let mut direct_cold = None;
    for _ in 0..WARMUP_SAMPLES {
        runengpu_cold = Some(runengpu_sample(&runengpu_context));
        direct_cold = Some(direct_sample(&direct_context, &direct_pipeline.pipeline));
    }

    let mut runengpu = Measurements::default();
    let mut direct = Measurements::default();
    for _ in 0..MEASURED_SAMPLES {
        runengpu.push(runengpu_sample(&runengpu_context));
        direct.push(direct_sample(&direct_context, &direct_pipeline.pipeline));
    }
    assert_eq!(runengpu.len(), MEASURED_SAMPLES);
    assert_eq!(direct.len(), MEASURED_SAMPLES);

    json!({
        "workload": "G6-C01-known-pattern-offscreen-draw",
        "comparison_envelope": {
            "width": WIDTH,
            "height": HEIGHT,
            "format": "Rgba8Unorm",
            "clear": [0, 0, 0, 255],
            "indices": INDICES,
            "draw": "one indexed triangle",
            "viewport": [0, 0, WIDTH, HEIGHT],
            "scissor": [0, 0, WIDTH / 2, HEIGHT],
            "shader_source": "byte-equivalent to retained inline G6-C01 WGSL",
            "readback_logical_bytes": WIDTH * HEIGHT * 4,
            "direct_readback_staging_bytes": padded_bytes_per_row(WIDTH * 4) * HEIGHT,
            "index_upload_bytes": core::mem::size_of_val(&INDICES),
        },
        "cold": {
            "runengpu_context_us": runengpu_context_us,
            "direct_context_us": direct_context.setup_us,
            "direct_pipeline_us": direct_pipeline.cold_pipeline_us,
            "runengpu_first_submission_phases_us": runengpu_cold,
            "direct_first_submission_phases_us": direct_cold,
        },
        "runengpu": runengpu.to_json(),
        "direct_wgpu": direct.to_json(),
        "runengpu_over_direct_ratio": ratio_summary(
            &runengpu,
            &direct,
            &["boundary_prepare_or_record", "submit_call", "completion_readback", "total"],
        ),
        "direct_adapter": direct_context.facts_json(),
        "timestamp_evidence": {
            "supported_by_direct_adapter": direct_context.timestamp_supported,
            "status": "not-yet-instrumented-in-first implementation slice",
        },
        "correctness": "passed exact retained selected-pixel oracle on every sample",
        "allocation_bytes": null,
        "allocation_bytes_status": "unavailable without new backend allocator instrumentation",
    })
}
