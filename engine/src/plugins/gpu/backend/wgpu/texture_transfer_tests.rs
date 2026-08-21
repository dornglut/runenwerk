use super::device_request::{enforce_runengpu_instance_flags, request_with_instance};
use crate::plugins::gpu::*;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};
use wgpu::{Backends, Instance, InstanceDescriptor, NoopBackendOptions};

const WIDTH: u32 = 3;
const HEIGHT: u32 = 2;
const LAYERS: u32 = 2;
const BYTES_PER_TEXEL: u32 = 4;
const LOGICAL_BYTE_LEN: u64 =
    u64::from(WIDTH) * u64::from(HEIGHT) * u64::from(LAYERS) * u64::from(BYTES_PER_TEXEL);
const PHYSICAL_STAGING_BYTE_LEN: u64 = 780;

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

fn texture(allocator: &mut GpuWorkResourceIdAllocator, name: &str) -> GpuTextureHandle {
    let resource_label = label(name);
    allocator
        .allocate_texture_handle(
            GpuTextureDescriptor::new(
                common(name),
                GpuTextureDimension::D2,
                GpuTextureExtent::new(
                    &resource_label,
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
                    &resource_label,
                    [GpuTextureUsage::CopySource, GpuTextureUsage::CopyDestination],
                )
                .unwrap(),
                GpuTextureInitialization::Uninitialized,
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

fn payload() -> Vec<u8> {
    (0..LOGICAL_BYTE_LEN)
        .map(|value| u8::try_from((value * 13 + 7) % 251).unwrap())
        .collect()
}

fn texture_round_trip_graph(name: &str) -> (GpuPreparedWorkGraph, GpuReadbackId, Vec<u8>) {
    let mut allocator = GpuWorkResourceIdAllocator::new();
    let texture = texture(&mut allocator, &format!("{name} texture"));
    let region = GpuTextureCopyRegion::new(
        &texture,
        0,
        GpuTextureOrigin::new(0, 0, 0),
        GpuTextureAspect::Color,
        GpuCopyExtent::new(WIDTH, HEIGHT, LAYERS).unwrap(),
    )
    .unwrap();
    let expected = payload();
    let upload = GpuUploadOperation::new(
        region.clone().into(),
        PreparedGpuData::<TransferData>::from_pod_transfer(
            format!("{name} payload"),
            expected.as_slice(),
            provenance(&format!("{name} payload")),
        )
        .unwrap(),
    )
    .unwrap();
    let readback_id = GpuReadbackId::allocate().unwrap();
    let readback = GpuReadbackOperation::new(region.into(), readback_id).unwrap();

    let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
    builder.declare_resource(texture.into()).unwrap();
    add_operation(
        &mut builder,
        &format!("{name} upload"),
        GpuWorkOperation::Upload(upload),
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
        expected,
    )
}

fn noop_instance() -> Instance {
    let mut descriptor = InstanceDescriptor::new_without_display_handle();
    descriptor.backends = Backends::NOOP;
    descriptor.backend_options.noop = NoopBackendOptions::enabled();
    Instance::new(enforce_runengpu_instance_flags(descriptor))
}

fn context(policy: GpuExecutionPolicy, name: &str) -> GpuContext {
    let mut requirements = GpuCapabilityRequirements::new();
    requirements
        .insert(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::Copy,
        ))
        .unwrap();
    let descriptor = GpuContextDescriptor::new(requirements)
        .with_allowed_backends([GpuBackendFamily::UnknownBackend])
        .with_label(name);
    let context = pollster::block_on(request_with_instance(
        noop_instance(),
        descriptor,
        None,
        GpuRealizationPolicies::default(),
        policy,
    ))
    .expect("explicit WGPU Noop must admit the texture-transfer proof context");
    assert_eq!(context.adapter_facts().backend(), GpuBackendFamily::UnknownBackend);
    assert_eq!(
        context.admission_report().candidate().portability(),
        GpuPortabilityClass::Unsupported,
        "test-only WGPU Noop must not masquerade as production portability evidence"
    );
    context
}

fn policy(upload_bytes: u64, readback_bytes: u64) -> GpuExecutionPolicy {
    GpuExecutionPolicy::new(
        NonZeroUsize::new(4).unwrap(),
        NonZeroUsize::new(2).unwrap(),
        upload_bytes,
        readback_bytes,
        4,
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
            GpuReadbackStatus::Failed(failure) => {
                panic!("texture readback failed: {failure:?}")
            }
            GpuReadbackStatus::Pending => {}
        }
        if let GpuSubmissionStatus::Failed(failure) = submission.status() {
            panic!("texture submission failed before readback: {failure:?}");
        }
        assert!(Instant::now() < deadline, "texture readback did not materialize");
        std::thread::yield_now();
    };

    loop {
        context.progress();
        match submission.status() {
            GpuSubmissionStatus::Completed => break,
            GpuSubmissionStatus::Failed(failure) => {
                panic!("texture submission failed: {failure:?}")
            }
            GpuSubmissionStatus::Accepted => {}
        }
        assert!(
            Instant::now() < deadline,
            "texture submission did not terminalize"
        );
        std::thread::yield_now();
    }
    bytes
}

#[test]
fn texture_upload_and_readback_remove_private_row_padding() {
    let context = context(policy(4096, 4096), "G5B noop texture round trip");
    let (graph, readback_id, expected) = texture_round_trip_graph("noop texture round trip");

    let prepared = pollster::block_on(context.prepare_submission(graph)).unwrap();
    let submission = context.submit_prepared(prepared).unwrap();
    let readback = submission
        .readback(readback_id)
        .expect("accepted texture readback must remain observable")
        .clone();
    let bytes = progress_to_readback(&context, &submission, &readback);

    assert_eq!(bytes.as_bytes(), expected.as_slice());
    assert_eq!(bytes.layout().byte_len(), LOGICAL_BYTE_LEN);
    assert_eq!(bytes.layout().stride(), u64::from(WIDTH * BYTES_PER_TEXEL));
    assert_eq!(bytes.layout().element_count(), u64::from(HEIGHT * LAYERS));
    assert_eq!(bytes.texture_format(), Some(GpuTextureFormat::Rgba8Unorm));

    let stats = context.execution_stats();
    assert_eq!(stats.prepared_submissions(), 0);
    assert_eq!(stats.in_flight_submissions(), 0);
    assert_eq!(stats.upload_bytes_in_flight(), 0);
    assert_eq!(stats.readback_bytes_in_flight(), 0);
    assert_eq!(stats.pending_readbacks(), 0);
}

#[test]
fn texture_preparation_accounts_for_private_padded_staging_bytes() {
    assert!(LOGICAL_BYTE_LEN < PHYSICAL_STAGING_BYTE_LEN);
    let context = context(
        policy(LOGICAL_BYTE_LEN, 4096),
        "G5B padded texture staging pressure",
    );
    let (graph, _, _) = texture_round_trip_graph("padded texture staging pressure");

    let error = pollster::block_on(context.prepare_submission(graph))
        .expect_err("physical padded upload staging must count against execution policy");
    assert_eq!(
        error.kind(),
        GpuSubmissionPreparationErrorKind::UploadDemandExceedsPolicy
    );
    assert_eq!(context.execution_stats().prepared_submissions(), 0);
}
