use engine::plugins::gpu::*;
use std::time::{Duration, Instant};

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

fn native_copy_context() -> GpuContext {
    let mut requirements = GpuCapabilityRequirements::new();
    requirements
        .insert(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::Copy,
        ))
        .unwrap();
    let descriptor = GpuContextDescriptor::new(requirements)
        .with_fallback_policy(GpuSoftwareFallbackPolicy::Require)
        .with_allowed_backends([GpuBackendFamily::Vulkan])
        .with_label("G5R native prepared initial-content proof");
    let context = pollster::block_on(GpuContext::request(descriptor))
        .expect("native conformance environment must provide a Vulkan fallback adapter");
    assert_eq!(context.adapter_facts().backend(), GpuBackendFamily::Vulkan);
    assert_eq!(
        context.adapter_facts().fallback(),
        GpuFallbackStatus::ConfirmedFallback
    );
    context
}

fn add_operation(builder: &mut GpuWorkFragmentBuilder, name: &str, operation: GpuWorkOperation) {
    builder
        .add_node(
            label(name),
            operation,
            [],
            GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance(name),
        )
        .unwrap();
}

fn prepared_buffer(
    allocator: &mut GpuWorkResourceIdAllocator,
    initial: &[u8],
) -> GpuBufferHandle {
    let resource_label = label("native G5R prepared buffer");
    allocator
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                common("native G5R prepared buffer"),
                initial.len() as u64,
                GpuBufferUsages::new(
                    &resource_label,
                    [GpuBufferUsage::CopySource, GpuBufferUsage::CopyDestination],
                )
                .unwrap(),
                GpuBufferInitialization::Prepared(
                    PreparedGpuData::<TransferData>::from_pod_transfer(
                        "native G5R initial bytes",
                        initial,
                        provenance("native G5R initial bytes"),
                    )
                    .unwrap(),
                ),
            )
            .unwrap(),
        )
        .unwrap()
}

fn readback_graph(buffer: &GpuBufferHandle) -> (GpuPreparedWorkGraph, GpuReadbackId) {
    let region = GpuBufferRegion::new(buffer, GpuBufferRange::whole(buffer).unwrap()).unwrap();
    let id = GpuReadbackId::allocate().unwrap();
    let readback = GpuReadbackOperation::new(region.into(), id).unwrap();
    let mut builder = GpuWorkFragmentBuilder::new(
        label("native G5R prepared readback"),
        provenance("native G5R prepared readback"),
    );
    builder.declare_resource(buffer.clone().into()).unwrap();
    add_operation(
        &mut builder,
        "read prepared bytes",
        GpuWorkOperation::Readback(readback),
    );
    (
        GpuPreparedWorkGraph::prepare(
            label("native G5R prepared readback graph"),
            [builder.finish().unwrap()],
        )
        .unwrap(),
        id,
    )
}

fn upload_and_readback_graph(
    buffer: &GpuBufferHandle,
    replacement: &[u8],
) -> (GpuPreparedWorkGraph, GpuReadbackId) {
    let region = GpuBufferRegion::new(buffer, GpuBufferRange::whole(buffer).unwrap()).unwrap();
    let upload = GpuUploadOperation::new(
        region.clone().into(),
        PreparedGpuData::<TransferData>::from_pod_transfer(
            "native G5R replacement bytes",
            replacement,
            provenance("native G5R replacement bytes"),
        )
        .unwrap(),
    )
    .unwrap();
    let id = GpuReadbackId::allocate().unwrap();
    let readback = GpuReadbackOperation::new(region.into(), id).unwrap();
    let mut builder = GpuWorkFragmentBuilder::new(
        label("native G5R explicit replacement"),
        provenance("native G5R explicit replacement"),
    );
    builder.declare_resource(buffer.clone().into()).unwrap();
    add_operation(
        &mut builder,
        "replace prepared bytes",
        GpuWorkOperation::Upload(upload),
    );
    add_operation(
        &mut builder,
        "read replaced bytes",
        GpuWorkOperation::Readback(readback),
    );
    (
        GpuPreparedWorkGraph::prepare(
            label("native G5R explicit replacement graph"),
            [builder.finish().unwrap()],
        )
        .unwrap(),
        id,
    )
}

fn submit_and_readback(
    context: &GpuContext,
    graph: GpuPreparedWorkGraph,
    readback_id: GpuReadbackId,
) -> GpuReadbackBytes {
    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    let submission = context.submit_prepared(prepared).unwrap();
    let readback = submission
        .readback(readback_id)
        .expect("accepted G5R readback must remain observable")
        .clone();
    let deadline = Instant::now() + Duration::from_secs(15);
    let bytes = loop {
        context.progress();
        match readback.status() {
            GpuReadbackStatus::Ready(bytes) => break bytes,
            GpuReadbackStatus::Failed(failure) => panic!("G5R readback failed: {failure:?}"),
            GpuReadbackStatus::Pending => {}
        }
        if let GpuSubmissionStatus::Failed(failure) = submission.status() {
            panic!("G5R submission failed before readback: {failure:?}");
        }
        assert!(Instant::now() < deadline, "G5R readback timed out");
        std::thread::yield_now();
    };
    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Completed => break,
            GpuSubmissionStatus::Failed(failure) => panic!("G5R submission failed: {failure:?}"),
            GpuSubmissionStatus::Accepted => {}
        }
        assert!(Instant::now() < deadline, "G5R submission did not terminalize");
        std::thread::yield_now();
    }
    bytes
}

#[test]
#[ignore = "requires a real Vulkan fallback adapter; executed by RunenGPU Native Conformance CI"]
fn prepared_buffer_materializes_once_per_physical_realization_and_explicit_upload_wins() {
    let context = native_copy_context();
    let initial = (0_u8..64).collect::<Vec<_>>();
    let replacement = (0_u8..64).map(|value| 255 - value).collect::<Vec<_>>();
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let buffer = prepared_buffer(&mut allocator, &initial);

    let (first_graph, first_id) = readback_graph(&buffer);
    let first = submit_and_readback(&context, first_graph, first_id);
    assert_eq!(first.as_bytes(), initial.as_slice());

    let (replacement_graph, replacement_id) = upload_and_readback_graph(&buffer, &replacement);
    let replaced = submit_and_readback(&context, replacement_graph, replacement_id);
    assert_eq!(
        replaced.as_bytes(),
        replacement.as_slice(),
        "an explicit upload on the same realization must remain unconditional and must not be overwritten by Prepared metadata"
    );

    let (later_graph, later_id) = readback_graph(&buffer);
    let later = submit_and_readback(&context, later_graph, later_id);
    assert_eq!(
        later.as_bytes(),
        replacement.as_slice(),
        "later submissions on the same physical realization must not replay Prepared initial content"
    );
}
