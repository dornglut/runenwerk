use super::device_request::{enforce_runengpu_instance_flags, request_with_instance};
use crate::plugins::gpu::*;
use std::time::{Duration, Instant};
use wgpu::{Backends, Instance, InstanceDescriptor, NoopBackendOptions};

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

fn buffer(
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

fn round_trip_graph(name: &str, values: &[u32]) -> (GpuPreparedWorkGraph, GpuReadbackId) {
    let byte_len = u64::try_from(std::mem::size_of_val(values)).unwrap();
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let source = buffer(&mut allocator, &format!("{name} source"), byte_len);
    let destination = buffer(&mut allocator, &format!("{name} destination"), byte_len);
    let source_region =
        GpuBufferRegion::new(&source, GpuBufferRange::whole(&source).unwrap()).unwrap();
    let destination_region =
        GpuBufferRegion::new(&destination, GpuBufferRange::whole(&destination).unwrap()).unwrap();
    let payload = PreparedGpuData::<TransferData>::from_pod_transfer(
        format!("{name} payload"),
        values,
        provenance(&format!("{name} payload")),
    )
    .unwrap();
    let upload = GpuUploadOperation::new(source_region.clone().into(), payload).unwrap();
    let copy =
        GpuCopyOperation::buffer_to_buffer(source_region, destination_region.clone()).unwrap();
    let readback_id = GpuReadbackId::allocate().unwrap();
    let readback = GpuReadbackOperation::new(destination_region.into(), readback_id).unwrap();

    let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
    builder.declare_resource(source.into()).unwrap();
    builder.declare_resource(destination.into()).unwrap();
    add_operation(
        &mut builder,
        &format!("{name} upload"),
        GpuWorkOperation::Upload(upload),
    );
    add_operation(
        &mut builder,
        &format!("{name} copy"),
        GpuWorkOperation::Copy(copy),
    );
    add_operation(
        &mut builder,
        &format!("{name} readback"),
        GpuWorkOperation::Readback(readback),
    );
    (
        GpuPreparedWorkGraph::prepare(label(&format!("{name} graph")), [builder.finish().unwrap()])
            .unwrap(),
        readback_id,
    )
}

fn noop_context() -> GpuContext {
    let mut requirements = GpuCapabilityRequirements::new();
    requirements
        .insert(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::Copy,
        ))
        .unwrap();
    let descriptor = GpuContextDescriptor::new(requirements).with_label("G5B noop buffer proof");

    let mut instance_descriptor = InstanceDescriptor::new_without_display_handle();
    instance_descriptor.backends = Backends::NOOP;
    instance_descriptor.backend_options.noop = NoopBackendOptions::enabled();
    let instance = Instance::new(enforce_runengpu_instance_flags(instance_descriptor));

    pollster::block_on(request_with_instance(
        instance,
        descriptor,
        None,
        GpuRealizationPolicies::default(),
        GpuExecutionPolicy::default(),
    ))
    .expect("explicitly enabled WGPU noop backend must admit the buffer-only G5B test context")
}

#[test]
fn noop_backend_buffer_round_trip_proves_first_g5b_checkpoint_runtime() {
    let context = noop_context();
    let values = [0x0102_0304_u32, 17, 29, u32::MAX];
    let (graph, readback_id) = round_trip_graph("noop round trip", &values);

    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    let submission = context.submit_prepared(prepared).unwrap();
    let readback = submission
        .readback(readback_id)
        .expect("accepted readback must remain observable")
        .clone();

    let deadline = Instant::now() + Duration::from_secs(5);
    let bytes = loop {
        context.progress();
        match readback.status() {
            GpuReadbackStatus::Ready(bytes) => break bytes,
            GpuReadbackStatus::Failed(failure) => {
                panic!("noop G5B readback failed: {failure:?}")
            }
            GpuReadbackStatus::Pending => {}
        }
        if let GpuSubmissionStatus::Failed(failure) = submission.status() {
            panic!("noop G5B submission failed before readback: {failure:?}");
        }
        assert!(
            Instant::now() < deadline,
            "noop G5B readback did not materialize"
        );
        std::thread::yield_now();
    };

    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Completed => break,
            GpuSubmissionStatus::Failed(failure) => {
                panic!("noop G5B submission failed: {failure:?}")
            }
            GpuSubmissionStatus::Accepted => {}
        }
        assert!(
            Instant::now() < deadline,
            "noop G5B submission did not terminalize"
        );
        std::thread::yield_now();
    }

    let expected = values
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect::<Vec<_>>();
    assert_eq!(bytes.as_bytes(), expected.as_slice());
    assert_eq!(bytes.layout().byte_len(), expected.len() as u64);
    assert_eq!(bytes.texture_format(), None);

    let stats = context.execution_stats();
    assert_eq!(stats.prepared_submissions(), 0);
    assert_eq!(stats.in_flight_submissions(), 0);
    assert_eq!(stats.upload_bytes_in_flight(), 0);
    assert_eq!(stats.readback_bytes_in_flight(), 0);
    assert_eq!(stats.pending_readbacks(), 0);
}
