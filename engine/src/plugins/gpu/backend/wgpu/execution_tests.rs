use super::device_request::{enforce_runengpu_instance_flags, request_with_instance};
use crate::plugins::gpu::*;
use std::num::NonZeroU64;
use std::time::{Duration, Instant};
use wgpu::{Backends, Instance, InstanceDescriptor, NoopBackendOptions};

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

fn noop_instance() -> Instance {
    let mut instance_descriptor = InstanceDescriptor::new_without_display_handle();
    instance_descriptor.backends = Backends::NOOP;
    instance_descriptor.backend_options.noop = NoopBackendOptions::enabled();
    Instance::new(enforce_runengpu_instance_flags(instance_descriptor))
}

fn assert_noop_test_context_truth(context: &GpuContext) {
    assert_eq!(
        context.adapter_facts().backend(),
        GpuBackendFamily::UnknownBackend,
        "the deterministic runtime proof must not masquerade WGPU noop as a production backend"
    );
    assert_eq!(
        context.admission_report().candidate().portability(),
        GpuPortabilityClass::Unsupported,
        "the test-only seam must preserve production portability truth"
    );
}

fn noop_context() -> GpuContext {
    let mut requirements = GpuCapabilityRequirements::new();
    requirements
        .insert(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::Copy,
        ))
        .unwrap();
    let descriptor = GpuContextDescriptor::new(requirements)
        .with_allowed_backends([GpuBackendFamily::UnknownBackend])
        .with_label("G5B noop buffer proof");

    let context = pollster::block_on(request_with_instance(
        noop_instance(),
        descriptor,
        None,
        GpuRealizationPolicies::default(),
        GpuExecutionPolicy::default(),
    ))
    .expect("explicitly enabled WGPU noop backend must admit the buffer-only G5B test context");
    assert_noop_test_context_truth(&context);
    context
}

fn noop_compute_context() -> GpuContext {
    let mut requirements = GpuCapabilityProfile::ComputeBaseline.requirements();
    requirements
        .insert(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::Copy,
        ))
        .unwrap();
    let descriptor = GpuContextDescriptor::new(requirements)
        .with_allowed_backends([GpuBackendFamily::UnknownBackend])
        .with_label("G5B noop direct compute proof");

    let context = pollster::block_on(request_with_instance(
        noop_instance(),
        descriptor,
        None,
        GpuRealizationPolicies::default(),
        GpuExecutionPolicy::default(),
    ))
    .expect("explicitly enabled WGPU noop backend must admit the direct-compute G5B test context");
    assert_noop_test_context_truth(&context);
    context
}

fn admitted_compute_source() -> GpuAdmittedProgramSource {
    let owner = GpuProgramSourceOwnerId::allocate().expect("test source owner should allocate");
    let identity = GpuProgramSourceIdentity::new(
        owner,
        GpuProgramSourceKey::new("g5b.noop.dynamic-compute").unwrap(),
        GpuProgramSourceRevision::try_from_raw(1).unwrap(),
    );
    let mut sources = GpuProgramSourceRegistry::new(4, 16 * 1024).unwrap();
    sources
        .admit_wgsl(
            identity,
            DYNAMIC_COMPUTE_WGSL,
            GpuProgramSourceProvenance::new("g5b-noop-direct-compute", None).unwrap(),
        )
        .unwrap()
}

fn dynamic_compute_pipeline() -> GpuComputePipelineDescriptor {
    let binding = GpuBindingDeclaration::new(
        GpuBindingKey::try_new(0, 0).unwrap(),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::storage_buffer(
            GpuStorageBufferAccess::ReadWrite,
            true,
            NonZeroU64::new(4),
        ),
        None,
        "values",
        GpuBindingProvenance::new("g5b-noop-direct-compute", None).unwrap(),
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

fn dynamic_compute_operation(
    context: &GpuContext,
    pipeline: &GpuComputePipelineDescriptor,
    buffer: &GpuBufferHandle,
    dynamic_offset: u64,
    x: u32,
) -> GpuComputeOperation {
    let binding = GpuRuntimeBindingValue::new(
        GpuBindingKey::try_new(0, 0).unwrap(),
        [GpuRuntimeBindingResource::Buffer(GpuRuntimeBufferBinding::new(
            buffer.clone(),
            0,
            NonZeroU64::new(4).unwrap(),
            Some(dynamic_offset),
        ))],
    )
    .unwrap();
    let facts = context
        .runtime_binding_device_facts()
        .expect("admitted compute context must publish dynamic binding facts");
    let bindings = GpuRuntimeBindingSet::new(pipeline.layout().clone(), [binding], &facts).unwrap();
    let dispatch = GpuDispatchIntent::direct(
        GpuDispatchSize::new(x, 1, 1).unwrap(),
        context.device_facts().workload_budget().limits(),
    )
    .unwrap();
    GpuComputeOperation::new(pipeline.clone(), bindings, dispatch).unwrap()
}

fn dynamic_compute_graph(
    context: &GpuContext,
) -> (GpuPreparedWorkGraph, GpuReadbackId, Vec<u32>) {
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
    assert!(storage_alignment.is_multiple_of(4));
    assert!(storage_alignment.is_multiple_of(copy_alignment));

    let byte_len = storage_alignment
        .checked_mul(2)
        .expect("test buffer size must fit u64");
    let value_count = usize::try_from(byte_len / 4).unwrap();
    let second_index = usize::try_from(storage_alignment / 4).unwrap();
    let mut values = vec![0_u32; value_count];
    values[0] = 10;
    values[second_index] = 20;

    let mut allocator = GpuWorkResourceIdAllocator::new();
    let values_buffer = compute_buffer(&mut allocator, "dynamic compute values", byte_len);
    let whole = GpuBufferRegion::new(
        &values_buffer,
        GpuBufferRange::whole(&values_buffer).unwrap(),
    )
    .unwrap();
    let payload = PreparedGpuData::<TransferData>::from_pod_transfer(
        "dynamic compute payload",
        &values,
        provenance("dynamic compute payload"),
    )
    .unwrap();
    let upload = GpuUploadOperation::new(whole.clone().into(), payload).unwrap();
    let readback_id = GpuReadbackId::allocate().unwrap();
    let readback = GpuReadbackOperation::new(whole.into(), readback_id).unwrap();

    let pipeline = dynamic_compute_pipeline();
    let first = dynamic_compute_operation(context, &pipeline, &values_buffer, 0, 1);
    let second =
        dynamic_compute_operation(context, &pipeline, &values_buffer, storage_alignment, 1);
    let zero = dynamic_compute_operation(context, &pipeline, &values_buffer, 0, 0);

    let name = "noop dynamic compute";
    let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
    builder.declare_resource(values_buffer.into()).unwrap();
    add_operation(
        &mut builder,
        "dynamic compute upload",
        GpuWorkOperation::Upload(upload),
    );
    let first_id = builder.compute("dynamic compute first", first).unwrap();
    let second_id = builder.compute("dynamic compute second", second).unwrap();
    let zero_id = builder.compute("dynamic compute zero", zero).unwrap();
    builder
        .add_explicit_order(
            GpuExplicitOrder::new(&first_id, &second_id, "prove ordered dynamic-offset reuse")
                .unwrap(),
        )
        .unwrap();
    builder
        .add_explicit_order(
            GpuExplicitOrder::new(&second_id, &zero_id, "prove zero-dispatch execution semantics")
                .unwrap(),
        )
        .unwrap();
    add_operation(
        &mut builder,
        "dynamic compute readback",
        GpuWorkOperation::Readback(readback),
    );

    (
        GpuPreparedWorkGraph::prepare(label("noop dynamic compute graph"), [
            builder.finish().unwrap(),
        ])
        .unwrap(),
        readback_id,
        values,
    )
}

fn progress_to_readback(
    context: &GpuContext,
    submission: &GpuSubmission,
    readback: &GpuReadback,
) -> GpuReadbackBytes {
    let deadline = Instant::now() + Duration::from_secs(5);
    let bytes = loop {
        context.progress();
        match readback.status() {
            GpuReadbackStatus::Ready(bytes) => break bytes,
            GpuReadbackStatus::Failed(failure) => panic!("G5B readback failed: {failure:?}"),
            GpuReadbackStatus::Pending => {}
        }
        if let GpuSubmissionStatus::Failed(failure) = submission.status() {
            panic!("G5B submission failed before readback: {failure:?}");
        }
        assert!(Instant::now() < deadline, "G5B readback did not materialize");
        std::thread::yield_now();
    };

    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Completed => break,
            GpuSubmissionStatus::Failed(failure) => panic!("G5B submission failed: {failure:?}"),
            GpuSubmissionStatus::Accepted => {}
        }
        assert!(Instant::now() < deadline, "G5B submission did not terminalize");
        std::thread::yield_now();
    }
    bytes
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
    let bytes = progress_to_readback(&context, &submission, &readback);

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

#[test]
fn noop_backend_proves_direct_compute_encoding_and_dynamic_bind_group_reuse() {
    let context = noop_compute_context();
    let (graph, readback_id, expected) = dynamic_compute_graph(&context);

    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    assert_eq!(
        context.program_binding_realization_stats().bind_groups(),
        1,
        "per-use dynamic offsets must not split physical bind-group identity"
    );
    let submission = context.submit_prepared(prepared).unwrap();
    let readback = submission
        .readback(readback_id)
        .expect("accepted compute readback must remain observable")
        .clone();
    let bytes = progress_to_readback(&context, &submission, &readback);

    let expected_bytes = expected
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect::<Vec<_>>();
    assert_eq!(
        bytes.as_bytes(),
        expected_bytes.as_slice(),
        "WGPU Noop performs no computation; unchanged bytes prove only the real G5B preparation/encoding/submission/readback path, not shader execution"
    );

    let stats = context.execution_stats();
    assert_eq!(stats.prepared_submissions(), 0);
    assert_eq!(stats.in_flight_submissions(), 0);
    assert_eq!(stats.upload_bytes_in_flight(), 0);
    assert_eq!(stats.readback_bytes_in_flight(), 0);
    assert_eq!(stats.pending_readbacks(), 0);
}
