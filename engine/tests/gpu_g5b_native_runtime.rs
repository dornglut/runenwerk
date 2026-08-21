use engine::plugins::gpu::*;
use std::num::{NonZeroU64, NonZeroUsize};
use std::time::{Duration, Instant};

const DYNAMIC_COMPUTE_WGSL: &str = r#"
@group(0) @binding(0)
var<storage, read_write> values: array<u32>;

@compute @workgroup_size(1)
fn cs_main() {
    values[0] = values[0] + 1u;
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

fn add_operation(builder: &mut GpuWorkFragmentBuilder, name: &str, operation: GpuWorkOperation) {
    builder
        .add_node(
            label(name),
            operation,
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::Automatic,
            provenance(name),
        )
        .unwrap();
}

fn compute_buffer(
    allocator: &mut GpuWorkResourceIdAllocator,
    name: &str,
    byte_len: u64,
) -> GpuBufferHandle {
    let resource_label = label(name);
    allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                common(name),
                byte_len,
                GpuBufferUsages::new(
                    &resource_label,
                    [
                        GpuBufferUsage::Storage,
                        GpuBufferUsage::CopySource,
                        GpuBufferUsage::CopyDestination,
                    ],
                )
                .unwrap(),
                GpuBufferInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap()
}

fn admitted_compute_source() -> GpuAdmittedProgramSource {
    let owner =
        GpuProgramSourceOwnerId::allocate().expect("native proof source owner should allocate");
    let identity = GpuProgramSourceIdentity::new(
        owner,
        GpuProgramSourceKey::new("g5b.native.dynamic-compute").unwrap(),
        GpuProgramSourceRevision::try_from_raw(1).unwrap(),
    );
    let mut sources = GpuProgramSourceRegistry::new(4, 16 * 1024).unwrap();
    sources
        .admit_wgsl(
            identity,
            DYNAMIC_COMPUTE_WGSL,
            GpuProgramSourceProvenance::new("g5b-native-headless-proof", None).unwrap(),
        )
        .unwrap()
}

fn dynamic_compute_pipeline() -> GpuComputePipelineDescriptor {
    let binding = GpuBindingDeclaration::new(
        GpuBindingKey::try_new(0, 0).unwrap(),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::storage_buffer(GpuStorageBufferAccess::ReadWrite, true, NonZeroU64::new(4)),
        None,
        "values",
        GpuBindingProvenance::new("g5b-native-headless-proof", None).unwrap(),
    )
    .unwrap();
    let interface = GpuProgramInterfaceDescriptor::new([binding]).unwrap();
    let entry = GpuEntryPointName::new("cs_main").unwrap();
    let program = GpuProgramDescriptor::new(
        admitted_compute_source(),
        interface.clone(),
        [GpuEntryPointDescriptor::new(
            entry.clone(),
            GpuShaderStage::Compute,
            interface,
        )],
    )
    .unwrap();
    let layout = GpuPipelineLayoutDescriptor::from_interface(program.interface()).unwrap();
    let specialization = GpuSpecializationValueSet::new(
        GpuSpecializationSchema::new(std::iter::empty::<GpuSpecializationDeclaration>()).unwrap(),
        std::iter::empty::<GpuSpecializationEntry>(),
    )
    .unwrap();
    GpuComputePipelineDescriptor::new(
        program,
        entry,
        layout,
        specialization,
        GpuCapabilityRequirements::new(),
    )
    .unwrap()
}

fn dynamic_compute_bindings(
    context: &GpuContext,
    pipeline: &GpuComputePipelineDescriptor,
    buffer: &GpuBufferHandle,
    dynamic_offset: u64,
) -> GpuRuntimeBindingSet {
    let binding = GpuRuntimeBindingValue::new(
        GpuBindingKey::try_new(0, 0).unwrap(),
        [GpuRuntimeBindingResource::Buffer(
            GpuRuntimeBufferBinding::new(
                buffer.clone(),
                0,
                NonZeroU64::new(4).unwrap(),
                Some(dynamic_offset),
            ),
        )],
    )
    .unwrap();
    let facts = context
        .runtime_binding_device_facts()
        .expect("admitted compute context must publish dynamic binding facts");
    GpuRuntimeBindingSet::new(pipeline.layout().clone(), [binding], &facts).unwrap()
}

fn dynamic_compute_operation(
    context: &GpuContext,
    pipeline: &GpuComputePipelineDescriptor,
    buffer: &GpuBufferHandle,
    dynamic_offset: u64,
    x: u32,
) -> GpuComputeOperation {
    let bindings = dynamic_compute_bindings(context, pipeline, buffer, dynamic_offset);
    let dispatch = GpuDispatchIntent::direct(
        GpuDispatchSize::new(x, 1, 1).unwrap(),
        context.device_facts().workload_budget().limits(),
    )
    .unwrap();
    GpuComputeOperation::new(pipeline.clone(), bindings, dispatch).unwrap()
}

fn greatest_common_divisor(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn checked_least_common_multiple(left: u64, right: u64) -> Option<u64> {
    if left == 0 || right == 0 {
        return None;
    }
    left.checked_div(greatest_common_divisor(left, right))?
        .checked_mul(right)
}

fn native_context() -> GpuContext {
    let descriptor =
        GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
            .with_fallback_policy(GpuSoftwareFallbackPolicy::Require)
            .with_allowed_backends([GpuBackendFamily::Vulkan])
            .with_label("G5B native headless compute proof");
    let context = pollster::block_on(GpuContext::request(descriptor))
        .expect("native conformance environment must provide a Vulkan fallback adapter");
    assert_eq!(context.adapter_facts().backend(), GpuBackendFamily::Vulkan);
    assert_eq!(
        context.adapter_facts().fallback(),
        GpuFallbackStatus::ConfirmedFallback,
        "native conformance must execute through the explicitly required fallback path"
    );
    context
}

fn dynamic_compute_graph(
    context: &GpuContext,
) -> (GpuPreparedWorkGraph, GpuReadbackId, Vec<u32>, usize) {
    let storage_alignment = context
        .device_facts()
        .device_limits()
        .alignments()
        .storage_dynamic_offset
        .expect("compute context must publish storage dynamic-offset alignment");
    let copy_alignment = context
        .device_facts()
        .device_limits()
        .alignments()
        .copy_buffer_offset
        .expect("compute context must publish buffer-copy alignment");
    let dynamic_stride = checked_least_common_multiple(storage_alignment, copy_alignment)
        .and_then(|stride| checked_least_common_multiple(stride, 4))
        .expect("native proof stride must satisfy storage, copy, and u32 alignment");

    let byte_len = dynamic_stride
        .checked_mul(2)
        .expect("native proof buffer size must fit u64");
    let value_count = usize::try_from(byte_len / 4).unwrap();
    let second_index = usize::try_from(dynamic_stride / 4).unwrap();
    let mut values = vec![0_u32; value_count];
    values[0] = 10;
    values[second_index] = 20;

    let mut allocator = GpuWorkResourceIdAllocator::new();
    let values_buffer = compute_buffer(&mut allocator, "native dynamic compute values", byte_len);
    let whole = GpuBufferRegion::new(
        &values_buffer,
        GpuBufferRange::whole(&values_buffer).unwrap(),
    )
    .unwrap();
    let payload = PreparedGpuData::<TransferData>::from_pod_transfer(
        "native dynamic compute payload",
        &values,
        provenance("native dynamic compute payload"),
    )
    .unwrap();
    let upload = GpuUploadOperation::new(whole.clone().into(), payload).unwrap();
    let readback_id = GpuReadbackId::allocate().unwrap();
    let readback = GpuReadbackOperation::new(whole.into(), readback_id).unwrap();

    let pipeline = dynamic_compute_pipeline();
    let first = dynamic_compute_operation(context, &pipeline, &values_buffer, 0, 1);
    let second = dynamic_compute_operation(context, &pipeline, &values_buffer, dynamic_stride, 1);
    let zero = dynamic_compute_operation(context, &pipeline, &values_buffer, 0, 0);

    let name = "native dynamic compute";
    let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
    builder.declare_resource(values_buffer.into()).unwrap();
    add_operation(
        &mut builder,
        "native dynamic compute upload",
        GpuWorkOperation::Upload(upload),
    );
    let first_id = builder
        .compute("native dynamic compute first", first)
        .unwrap();
    let second_id = builder
        .compute("native dynamic compute second", second)
        .unwrap();
    let zero_id = builder
        .compute("native dynamic compute zero", zero)
        .unwrap();
    builder
        .add_explicit_order(
            GpuExplicitOrder::new(
                &first_id,
                &second_id,
                "prove ordered native dynamic-offset reuse",
            )
            .unwrap(),
        )
        .unwrap();
    builder
        .add_explicit_order(
            GpuExplicitOrder::new(
                &second_id,
                &zero_id,
                "prove native zero-dispatch execution semantics",
            )
            .unwrap(),
        )
        .unwrap();
    add_operation(
        &mut builder,
        "native dynamic compute readback",
        GpuWorkOperation::Readback(readback),
    );

    (
        GpuPreparedWorkGraph::prepare(
            label("native dynamic compute graph"),
            [builder.finish().unwrap()],
        )
        .unwrap(),
        readback_id,
        values,
        second_index,
    )
}

fn progress_to_readback(
    context: &GpuContext,
    submission: &GpuSubmission,
    readback: &GpuReadback,
) -> GpuReadbackBytes {
    let deadline = Instant::now() + Duration::from_secs(15);
    let bytes = loop {
        context.progress();
        match readback.status() {
            GpuReadbackStatus::Ready(bytes) => break bytes,
            GpuReadbackStatus::Failed(failure) => panic!("native G5B readback failed: {failure:?}"),
            GpuReadbackStatus::Pending => {}
        }
        if let GpuSubmissionStatus::Failed(failure) = submission.status() {
            panic!("native G5B submission failed before readback: {failure:?}");
        }
        assert!(Instant::now() < deadline, "native G5B readback timed out");
        std::thread::yield_now();
    };

    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Completed => break,
            GpuSubmissionStatus::Failed(failure) => {
                panic!("native G5B submission failed: {failure:?}")
            }
            GpuSubmissionStatus::Accepted => {}
        }
        assert!(
            Instant::now() < deadline,
            "native G5B submission did not terminalize"
        );
        std::thread::yield_now();
    }
    bytes
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by RunenGPU Native Conformance CI"]
fn native_headless_compute_executes_shader_and_reuses_dynamic_bind_group() {
    let context = native_context();
    let (graph, readback_id, mut expected, second_index) = dynamic_compute_graph(&context);

    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    assert_eq!(
        context.program_binding_realization_stats().bind_groups(),
        1,
        "two dynamic offsets must reuse one physical bind-group realization"
    );
    let submission = context.submit_prepared(prepared).unwrap();
    let readback = submission
        .readback(readback_id)
        .expect("accepted native readback must remain observable")
        .clone();
    let bytes = progress_to_readback(&context, &submission, &readback);

    let (words, remainder) = bytes.as_bytes().as_chunks::<4>();
    assert!(remainder.is_empty());
    let actual = words
        .iter()
        .map(|bytes| u32::from_ne_bytes(*bytes))
        .collect::<Vec<_>>();
    expected[0] += 1;
    expected[second_index] += 1;
    assert_eq!(
        actual, expected,
        "real Vulkan execution must apply both nonzero dispatches exactly once while zero dispatch remains non-executing"
    );

    let stats = context.execution_stats();
    assert_eq!(stats.prepared_submissions(), 0);
    assert_eq!(stats.in_flight_submissions(), 0);
    assert_eq!(stats.upload_bytes_in_flight(), 0);
    assert_eq!(stats.readback_bytes_in_flight(), 0);
    assert_eq!(stats.pending_readbacks(), 0);
}

fn native_copy_context() -> GpuContext {
    native_copy_context_with_policy(GpuExecutionPolicy::default())
}

fn native_copy_context_with_policy(policy: GpuExecutionPolicy) -> GpuContext {
    let mut requirements = GpuCapabilityRequirements::new();
    requirements
        .insert(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::Copy,
        ))
        .unwrap();
    let descriptor = GpuContextDescriptor::new(requirements)
        .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::CopySource)
        .require_format_role(GpuTextureFormat::Rgba8Unorm, GpuFormatRole::CopyDestination)
        .with_fallback_policy(GpuSoftwareFallbackPolicy::Require)
        .with_allowed_backends([GpuBackendFamily::Vulkan])
        .with_label("G5B native texture transfer proof");
    let context = pollster::block_on(GpuContext::request_with_policies(
        descriptor,
        GpuRealizationPolicies::default(),
        policy,
    ))
    .expect("native conformance environment must provide a Vulkan fallback adapter");
    assert_eq!(context.adapter_facts().backend(), GpuBackendFamily::Vulkan);
    assert_eq!(
        context.adapter_facts().fallback(),
        GpuFallbackStatus::ConfirmedFallback,
        "native texture proof must execute through the explicitly required fallback path"
    );
    context
}

fn native_texture_round_trip_graph() -> (GpuPreparedWorkGraph, GpuReadbackId, Vec<u8>) {
    const WIDTH: u32 = 3;
    const HEIGHT: u32 = 2;
    const LAYERS: u32 = 2;
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let texture_label = label("native texture transfer target");
    let texture = allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::new(
                common("native texture transfer target"),
                GpuTextureDimension::D2,
                GpuTextureExtent::new(
                    &texture_label,
                    GpuTextureDimension::D2,
                    WIDTH,
                    HEIGHT,
                    LAYERS,
                )
                .unwrap(),
                1,
                1,
                GpuTextureFormat::Rgba8Unorm,
                GpuTextureUsages::new(
                    &texture_label,
                    [
                        GpuTextureUsage::CopySource,
                        GpuTextureUsage::CopyDestination,
                    ],
                )
                .unwrap(),
                GpuTextureInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap();
    let region = GpuTextureCopyRegion::new(
        &texture,
        0,
        GpuTextureOrigin::new(0, 0, 0),
        GpuTextureAspect::Color,
        GpuCopyExtent::new(WIDTH, HEIGHT, LAYERS).unwrap(),
    )
    .unwrap();
    let expected = (0..WIDTH * HEIGHT * LAYERS * 4)
        .map(|value| u8::try_from((value * 17 + 5) % 251).unwrap())
        .collect::<Vec<_>>();
    let upload = GpuUploadOperation::new(
        region.clone().into(),
        PreparedGpuData::<TransferData>::from_pod_transfer(
            "native texture transfer payload",
            expected.as_slice(),
            provenance("native texture transfer payload"),
        )
        .unwrap(),
    )
    .unwrap();
    let readback_id = GpuReadbackId::allocate().unwrap();
    let readback = GpuReadbackOperation::new(region.into(), readback_id).unwrap();

    let mut builder = GpuWorkFragmentBuilder::new(
        label("native texture transfer"),
        provenance("native texture transfer"),
    );
    builder.declare_resource(texture.into()).unwrap();
    add_operation(
        &mut builder,
        "native texture upload",
        GpuWorkOperation::Upload(upload),
    );
    add_operation(
        &mut builder,
        "native texture readback",
        GpuWorkOperation::Readback(readback),
    );
    (
        GpuPreparedWorkGraph::prepare(
            label("native texture transfer graph"),
            [builder.finish().unwrap()],
        )
        .unwrap(),
        readback_id,
        expected,
    )
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by RunenGPU Native Conformance CI"]
fn native_texture_preparation_accounts_for_private_staging_and_drop_releases_capacity() {
    let (upload_graph, _, expected) = native_texture_round_trip_graph();
    let logical_bytes = u64::try_from(expected.len()).unwrap();
    let upload_policy = GpuExecutionPolicy::new(
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        logical_bytes,
        4096,
        2,
    );
    let upload_context = native_copy_context_with_policy(upload_policy);
    let error = pollster::block_on(upload_context.prepare_submission(upload_graph))
        .expect_err("physical padded upload staging must count against execution policy");
    assert_eq!(
        error.kind(),
        GpuSubmissionPreparationErrorKind::UploadDemandExceedsPolicy
    );
    assert_eq!(upload_context.execution_stats().prepared_submissions(), 0);

    let (readback_graph, _, _) = native_texture_round_trip_graph();
    let readback_policy = GpuExecutionPolicy::new(
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        4096,
        logical_bytes,
        2,
    );
    let readback_context = native_copy_context_with_policy(readback_policy);
    let error = pollster::block_on(readback_context.prepare_submission(readback_graph))
        .expect_err("physical padded readback staging must count against execution policy");
    assert_eq!(
        error.kind(),
        GpuSubmissionPreparationErrorKind::ReadbackDemandExceedsPolicy
    );
    assert_eq!(readback_context.execution_stats().prepared_submissions(), 0);

    let context = native_copy_context();
    let (graph, _, _) = native_texture_round_trip_graph();
    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    assert_eq!(context.execution_stats().prepared_submissions(), 1);
    drop(prepared);
    assert_eq!(context.execution_stats().prepared_submissions(), 0);
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by RunenGPU Native Conformance CI"]
fn native_texture_upload_readback_normalizes_private_row_padding() {
    let context = native_copy_context();
    let (graph, readback_id, expected) = native_texture_round_trip_graph();

    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    let submission = context.submit_prepared(prepared).unwrap();
    let readback = submission
        .readback(readback_id)
        .expect("accepted native texture readback must remain observable")
        .clone();
    let bytes = progress_to_readback(&context, &submission, &readback);

    assert_eq!(bytes.as_bytes(), expected.as_slice());
    assert_eq!(bytes.layout().byte_len(), expected.len() as u64);
    assert_eq!(bytes.texture_format(), Some(GpuTextureFormat::Rgba8Unorm));

    let stats = context.execution_stats();
    assert_eq!(stats.prepared_submissions(), 0);
    assert_eq!(stats.in_flight_submissions(), 0);
    assert_eq!(stats.upload_bytes_in_flight(), 0);
    assert_eq!(stats.readback_bytes_in_flight(), 0);
    assert_eq!(stats.pending_readbacks(), 0);
}

const DIRECT_COPY_WIDTH: u32 = 3;
const DIRECT_COPY_HEIGHT: u32 = 2;
const DIRECT_COPY_LAYERS: u32 = 2;
const DIRECT_COPY_ROW_BYTES: u32 = DIRECT_COPY_WIDTH * 4;
const DIRECT_COPY_ROWS: u32 = DIRECT_COPY_HEIGHT * DIRECT_COPY_LAYERS;

fn direct_copy_footprint(bytes_per_row: u32) -> u64 {
    u64::from(bytes_per_row) * u64::from(DIRECT_COPY_ROWS - 1) + u64::from(DIRECT_COPY_ROW_BYTES)
}

fn direct_copy_buffer(
    allocator: &mut GpuWorkResourceIdAllocator,
    name: &str,
    byte_len: u64,
) -> GpuBufferHandle {
    let resource_label = label(name);
    allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                common(name),
                byte_len,
                GpuBufferUsages::new(
                    &resource_label,
                    [GpuBufferUsage::CopySource, GpuBufferUsage::CopyDestination],
                )
                .unwrap(),
                GpuBufferInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap()
}

fn direct_copy_texture(allocator: &mut GpuWorkResourceIdAllocator, name: &str) -> GpuTextureHandle {
    let resource_label = label(name);
    allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::new(
                common(name),
                GpuTextureDimension::D2,
                GpuTextureExtent::new(
                    &resource_label,
                    GpuTextureDimension::D2,
                    DIRECT_COPY_WIDTH,
                    DIRECT_COPY_HEIGHT,
                    DIRECT_COPY_LAYERS,
                )
                .unwrap(),
                1,
                1,
                GpuTextureFormat::Rgba8Unorm,
                GpuTextureUsages::new(
                    &resource_label,
                    [
                        GpuTextureUsage::CopySource,
                        GpuTextureUsage::CopyDestination,
                    ],
                )
                .unwrap(),
                GpuTextureInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap()
}

fn direct_copy_region(texture: &GpuTextureHandle) -> GpuTextureCopyRegion {
    GpuTextureCopyRegion::new(
        texture,
        0,
        GpuTextureOrigin::new(0, 0, 0),
        GpuTextureAspect::Color,
        GpuCopyExtent::new(DIRECT_COPY_WIDTH, DIRECT_COPY_HEIGHT, DIRECT_COPY_LAYERS).unwrap(),
    )
    .unwrap()
}

fn direct_texture_copy_graph(
    bytes_per_row: u32,
) -> (GpuPreparedWorkGraph, Vec<GpuReadbackId>, Vec<Vec<u8>>, u64) {
    let footprint = direct_copy_footprint(bytes_per_row);
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let source_buffer = direct_copy_buffer(&mut allocator, "native direct-copy source", footprint);
    let destination_buffer =
        direct_copy_buffer(&mut allocator, "native direct-copy destination", footprint);
    let first_texture = direct_copy_texture(&mut allocator, "native direct-copy first texture");
    let second_texture = direct_copy_texture(&mut allocator, "native direct-copy second texture");
    let first_region = direct_copy_region(&first_texture);
    let second_region = direct_copy_region(&second_texture);

    let mut payload = vec![0xA5_u8; usize::try_from(footprint).unwrap()];
    let mut expected_rows = Vec::new();
    for row in 0..DIRECT_COPY_ROWS {
        let expected = (0..DIRECT_COPY_ROW_BYTES)
            .map(|column| u8::try_from((row * 37 + column * 11 + 3) % 251).unwrap())
            .collect::<Vec<_>>();
        let start = usize::try_from(u64::from(row) * u64::from(bytes_per_row)).unwrap();
        let end = start + expected.len();
        payload[start..end].copy_from_slice(&expected);
        expected_rows.push(expected);
    }

    let source_whole = GpuBufferRegion::new(
        &source_buffer,
        GpuBufferRange::whole(&source_buffer).unwrap(),
    )
    .unwrap();
    let upload = GpuUploadOperation::new(
        source_whole.into(),
        PreparedGpuData::<TransferData>::from_pod_transfer(
            "native direct-copy payload",
            payload.as_slice(),
            provenance("native direct-copy payload"),
        )
        .unwrap(),
    )
    .unwrap();
    let source_layout =
        GpuBufferTextureLayout::new(&source_buffer, 0, bytes_per_row, DIRECT_COPY_HEIGHT).unwrap();
    let destination_layout =
        GpuBufferTextureLayout::new(&destination_buffer, 0, bytes_per_row, DIRECT_COPY_HEIGHT)
            .unwrap();
    let buffer_to_texture =
        GpuCopyOperation::buffer_to_texture(source_layout, first_region.clone()).unwrap();
    let texture_to_texture =
        GpuCopyOperation::texture_to_texture(first_region.clone(), second_region.clone()).unwrap();
    let texture_to_buffer =
        GpuCopyOperation::texture_to_buffer(second_region, destination_layout).unwrap();

    let mut readback_ids = Vec::new();
    let mut readbacks = Vec::new();
    for row in 0..DIRECT_COPY_ROWS {
        let offset = u64::from(row) * u64::from(bytes_per_row);
        let region = GpuBufferRegion::new(
            &destination_buffer,
            GpuBufferRange::new(
                &destination_buffer,
                offset,
                u64::from(DIRECT_COPY_ROW_BYTES),
            )
            .unwrap(),
        )
        .unwrap();
        let id = GpuReadbackId::allocate().unwrap();
        readback_ids.push(id);
        readbacks.push(GpuReadbackOperation::new(region.into(), id).unwrap());
    }

    let name = "native direct texture copy";
    let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
    builder.declare_resource(source_buffer.into()).unwrap();
    builder.declare_resource(destination_buffer.into()).unwrap();
    builder.declare_resource(first_texture.into()).unwrap();
    builder.declare_resource(second_texture.into()).unwrap();
    add_operation(
        &mut builder,
        "native direct-copy upload",
        GpuWorkOperation::Upload(upload),
    );
    add_operation(
        &mut builder,
        "native buffer to texture copy",
        GpuWorkOperation::Copy(buffer_to_texture),
    );
    add_operation(
        &mut builder,
        "native texture to texture copy",
        GpuWorkOperation::Copy(texture_to_texture),
    );
    add_operation(
        &mut builder,
        "native texture to buffer copy",
        GpuWorkOperation::Copy(texture_to_buffer),
    );
    for (row, readback) in readbacks.into_iter().enumerate() {
        add_operation(
            &mut builder,
            &format!("native direct-copy row {row} readback"),
            GpuWorkOperation::Readback(readback),
        );
    }

    (
        GpuPreparedWorkGraph::prepare(
            label("native direct texture copy graph"),
            [builder.finish().unwrap()],
        )
        .unwrap(),
        readback_ids,
        expected_rows,
        footprint,
    )
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by RunenGPU Native Conformance CI"]
fn native_texture_copy_executes_all_directions_without_copy_scratch() {
    const BYTES_PER_ROW: u32 = 256;
    let (graph, readback_ids, expected_rows, footprint) = direct_texture_copy_graph(BYTES_PER_ROW);
    let policy = GpuExecutionPolicy::new(
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(1).unwrap(),
        footprint,
        u64::from(DIRECT_COPY_ROW_BYTES) * u64::from(DIRECT_COPY_ROWS),
        usize::try_from(DIRECT_COPY_ROWS).unwrap(),
    );
    let context = native_copy_context_with_policy(policy);

    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    let submission = context.submit_prepared(prepared).unwrap();
    for (readback_id, expected) in readback_ids.into_iter().zip(expected_rows) {
        let readback = submission
            .readback(readback_id)
            .expect("accepted direct-copy row readback must remain observable")
            .clone();
        let bytes = progress_to_readback(&context, &submission, &readback);
        assert_eq!(bytes.as_bytes(), expected.as_slice());
    }

    let stats = context.execution_stats();
    assert_eq!(stats.prepared_submissions(), 0);
    assert_eq!(stats.in_flight_submissions(), 0);
    assert_eq!(stats.upload_bytes_in_flight(), 0);
    assert_eq!(stats.readback_bytes_in_flight(), 0);
    assert_eq!(stats.pending_readbacks(), 0);
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by RunenGPU Native Conformance CI"]
fn native_texture_copy_rejects_nonencodable_logical_row_stride_before_acceptance() {
    let (graph, _, _, _) = direct_texture_copy_graph(DIRECT_COPY_ROW_BYTES);
    let context = native_copy_context();

    let error = pollster::block_on(context.prepare_submission(graph))
        .expect_err("logical row stride must be rejected before irreversible acceptance");
    assert_eq!(
        error.kind(),
        GpuSubmissionPreparationErrorKind::TransferAlignmentNotAdmitted
    );
    assert_eq!(context.execution_stats().prepared_submissions(), 0);
    assert_eq!(context.execution_stats().in_flight_submissions(), 0);
}
