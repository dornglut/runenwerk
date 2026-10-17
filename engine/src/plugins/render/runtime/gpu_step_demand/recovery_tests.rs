//! Actual RunenGPU public submission/reconstruction/producer recovery proof.
//! Kept apart from pure immutable-step transport tests because it needs a GPU.
use super::*;

// This exercises the *public* RunenGPU lifecycle with real buffer contents.
// It is not a synthetic status transition, CPU-only counter, or claim that
// the engine automatically knows how to recover an arbitrary simulation.
#[test]
fn accepted_unknown_write_requires_verified_checkpoint_reimport_before_replay() {
    use runen_gpu::{
        GpuBufferDescriptor, GpuBufferHandle, GpuBufferInitialization, GpuBufferRange,
        GpuBufferRegion, GpuBufferUsage, GpuBufferUsages, GpuCapabilityFeature,
        GpuCapabilityRequirement, GpuCapabilityRequirements, GpuContext,
        GpuContextDescriptor, GpuContextRequestErrorCategory, GpuExecutionPreference,
        GpuMemoryIntent, GpuReadbackOperation, GpuReconstruction, GpuResourceCommon,
        GpuResourceLabel, GpuResourceLifetime, GpuResourceProvenance, GpuUploadOperation,
        GpuWorkFragment, GpuWorkFragmentBuilder, GpuWorkNodeId, GpuWorkOperation,
        GpuWorkResourceIdAllocator, PreparedGpuData, TransferData,
    };
    use std::time::{Duration, Instant};

    fn label(value: &str) -> GpuResourceLabel {
        GpuResourceLabel::new(value).expect("GPU resource label")
    }
    fn provenance(value: &str) -> GpuResourceProvenance {
        GpuResourceProvenance::new(label(value), None, None)
    }
    fn request_context(name: &str) -> Option<GpuContext> {
        let mut requirements = GpuCapabilityRequirements::new();
        requirements
            .insert(GpuCapabilityRequirement::Required(GpuCapabilityFeature::Copy))
            .expect("copy capability");
        let descriptor = GpuContextDescriptor::new(requirements).with_label(name);
        match pollster::block_on(GpuContext::request(descriptor)) {
            Ok(context) => Some(context),
            Err(error)
                if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable =>
            {
                assert_ne!(
                    std::env::var("RUNENWERK_G5_REQUIRE_GPU").ok().as_deref(),
                    Some("1"),
                    "GPU-required validation must not silently skip recovery"
                );
                return None;
            }
            Err(error) => panic!("recovery GPU context creation: {error}"),
        }
    }
    fn make_work(
        target: &GpuBufferHandle,
        values: &[u32; 2],
        name: &str,
    ) -> (GpuWorkFragment, GpuReadbackId, GpuWorkNodeId) {
        let region =
            GpuBufferRegion::new(target, GpuBufferRange::whole(target).unwrap()).unwrap();
        let payload = PreparedGpuData::<TransferData>::from_pod_transfer(
            format!("{name} checkpoint bytes"),
            values,
            provenance(name),
        )
        .unwrap();
        let upload = GpuUploadOperation::new(region.clone().into(), payload).unwrap();
        let id = GpuReadbackId::allocate().unwrap();
        let readback = GpuReadbackOperation::new(region.into(), id).unwrap();
        let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
        builder.declare_resource(target.clone().into()).unwrap();
        let upload_node = builder
            .add_node(
                label(&format!("{name} upload")),
                GpuWorkOperation::Upload(upload),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::Automatic,
                provenance(name),
            )
            .unwrap();
        builder
            .add_node(
                label(&format!("{name} readback")),
                GpuWorkOperation::Readback(readback),
                [],
                GpuCapabilityRequirements::new(),
                GpuExecutionPreference::Automatic,
                provenance(name),
            )
            .unwrap();
        (builder.finish().unwrap(), id, upload_node)
    }
    fn wait(context: &GpuContext, submission: &GpuSubmission) {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            context.progress();
            match submission.status() {
                GpuSubmissionStatus::Completed => return,
                GpuSubmissionStatus::Failed(failure) => {
                    panic!("recovery GPU work failed: {failure:?}")
                }
                GpuSubmissionStatus::Accepted => {}
            }
            assert!(Instant::now() < deadline, "recovery GPU progress timed out");
            std::thread::yield_now();
        }
    }
    fn bytes_equal(
        submission: &GpuSubmission,
        readback_id: GpuReadbackId,
        values: &[u32; 2],
    ) -> bool {
        let expected = values
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect::<Vec<_>>();
        submission.readback(readback_id).is_some_and(|readback| {
            matches!(
                readback.status(),
                GpuReadbackStatus::Ready(ref data)
                    if data.as_bytes() == expected.as_slice()
            )
        })
    }

    let Some(old_context) = request_context("Runenwerk unknown stateful write") else {
        return;
    };
    // The producer's LAST VERIFIED source state is distinct from the
    // accepted, unverified write. Zero-initialization is not restoration.
    let verified_checkpoint = [0x1020_3040_u32, 0x1122_3344_u32];
    let unverified_write = [5_u32, 7_u32];
    let name = "producer-owned retained simulation state";
    let resource_label = label(name);
    let target = GpuWorkResourceIdAllocator::new()
        .allocate_buffer_handle(
            GpuBufferDescriptor::new(
                GpuResourceCommon::owned(
                    resource_label.clone(),
                    GpuResourceLifetime::Retained,
                    GpuMemoryIntent::Device,
                    GpuReconstruction::SourceBacked,
                    provenance(name),
                )
                .unwrap(),
                8,
                GpuBufferUsages::new(
                    &resource_label,
                    [GpuBufferUsage::CopySource, GpuBufferUsage::CopyDestination],
                )
                .unwrap(),
                GpuBufferInitialization::Uninitialized,
            )
            .unwrap(),
        )
        .unwrap();

    // Establish actual GPU-resident checkpoint contents for the
    // producer's initial verified tick, rather than assuming descriptor
    // initialization or a CPU prediction is already a GPU fact.
    let (initial_checkpoint_work, initial_readback, initial_node) =
        make_work(&target, &verified_checkpoint, "initial verified checkpoint");
    let initial = pollster::block_on(old_context.submit_work(
        "initial producer state upload and readback",
        [initial_checkpoint_work],
    ))
    .unwrap();
    wait(&old_context, &initial);
    assert!(initial.contains_work_node(&initial_node));
    assert!(bytes_equal(
        &initial,
        initial_readback,
        &verified_checkpoint
    ));

    let mut producer = RenderGpuStepDemand::<u32>::new(0, 4);
    producer.capture_batch(vec![5, 7]).unwrap();
    let (initial, _, _) = make_work(&target, &unverified_write, "accepted unknown write");
    let accepted = producer
        .submit_with(2, |_| {
            let submission = pollster::block_on(
                old_context.submit_work("accepted unknown source write", [initial]),
            )
            .unwrap();
            Ok::<_, ()>((submission, ()))
        })
        .unwrap();
    assert_eq!(accepted, Some(()));
    assert_eq!(producer.completed_tick(), 0);
    producer.capture(9).unwrap();
    let old_receipt = producer.accepted_submission().unwrap().clone();
    // Even a real, completed earlier checkpoint does NOT permit replay
    // while this accepted write may still be modifying retained state.
    assert!(matches!(
        producer.requeue_verified_after_reconstruction(
            old_context.affinity(),
            &initial,
            |_, _, _| Ok::<(), ()>(())
        ),
        Err(RenderGpuStepCommitError::Submission(
            RenderGpuStepDemandError::SubmissionPending
        ))
    ));
    // Shutdown/loss makes an accepted write's contents untrustworthy,
    // whether zero, some, or all physical operations ran before loss.
    drop(old_context);
    assert!(matches!(
        old_receipt.status(),
        GpuSubmissionStatus::Failed(_)
    ));
    assert!(matches!(
        producer.commit_verified(old_receipt.affinity(), |_, _| Ok::<(), ()>(())),
        Err(RenderGpuStepCommitError::Submission(
            RenderGpuStepDemandError::SubmissionFailed
        ))
    ));
    assert_eq!(
        (producer.requested_tick(), producer.completed_tick()),
        (3, 0)
    );

    let new_context = request_context("Runenwerk source-checkpoint restoration")
        .expect("GPU available for initial admission must support reconstruction request");
    let invalid_checkpoint = [0xdead_beef_u32, 0xabab_cdcd_u32];
    let (wrong_work, wrong_readback, _) =
        make_work(&target, &invalid_checkpoint, "incorrect checkpoint bytes");
    let wrong_restore = pollster::block_on(
        new_context.submit_work("wrong producer checkpoint restoration", [wrong_work]),
    )
    .unwrap();
    wait(&new_context, &wrong_restore);
    assert!(bytes_equal(
        &wrong_restore,
        wrong_readback,
        &invalid_checkpoint
    ));
    // A completed G5 upload is still not evidence of the producer's
    // semantic checkpoint. The actual terminal bytes are wrong.
    let refused = producer.requeue_verified_after_reconstruction(
        new_context.affinity(),
        &wrong_restore,
        |reconstruction, _, _| {
            if bytes_equal(reconstruction, wrong_readback, &verified_checkpoint) {
                Ok(())
            } else {
                Err("GPU reconstruction did not reproduce source checkpoint")
            }
        },
    );
    assert!(matches!(
        refused,
        Err(RenderGpuStepCommitError::Verification(_))
    ));
    assert_eq!(producer.accepted_steps().len(), 2);
    assert_eq!(producer.pending_len(), 1);

    let (restore_work, restore_readback, restore_node) =
        make_work(&target, &verified_checkpoint, "explicit source checkpoint");
    let restore = pollster::block_on(
        new_context.submit_work("producer checkpoint upload and readback", [restore_work]),
    )
    .unwrap();
    // An outstanding reconstruction is not sufficient for replay, even
    // if the producer has the original inputs and the right CPU tick.
    assert!(matches!(
        producer.requeue_verified_after_reconstruction(
            new_context.affinity(),
            &restore,
            |_, _, _| Ok::<(), ()>(())
        ),
        Err(RenderGpuStepCommitError::Submission(
            RenderGpuStepDemandError::SubmissionPending
        ))
    ));
    wait(&new_context, &restore);
    assert!(restore.contains_work_node(&restore_node));
    producer
        .requeue_verified_after_reconstruction(
            new_context.affinity(),
            &restore,
            |reconstruction, prior, old_steps| {
                assert_eq!(prior.id(), old_receipt.id());
                assert_eq!(
                    old_steps.iter().map(|step| step.tick()).collect::<Vec<_>>(),
                    vec![1, 2]
                );
                assert!(reconstruction.contains_work_node(&restore_node));
                assert!(bytes_equal(
                    reconstruction,
                    restore_readback,
                    &verified_checkpoint
                ));
                Ok::<(), ()>(())
            },
        )
        .unwrap();
    assert_eq!(producer.completed_tick(), 0);
    assert_eq!(producer.requested_tick(), 3);
    assert_eq!(producer.accepted_steps().len(), 0);
    assert_eq!(
        producer
            .candidate(3)
            .iter()
            .map(|v| (v.tick(), *v.input()))
            .collect::<Vec<_>>(),
        vec![(1, 5), (2, 7), (3, 9)]
    );

    // The original accepted prefix is newly admitted once, with real G5
    // bytes and the original immutable source inputs, NOT latest CPU state.
    let (replay_work, replay_readback, _) =
        make_work(&target, &unverified_write, "verified historical replay");
    producer
        .submit_with(2, |steps| {
            assert_eq!(
                steps
                    .iter()
                    .map(|s| (s.tick(), *s.input()))
                    .collect::<Vec<_>>(),
                vec![(1, 5), (2, 7)]
            );
            let submission = pollster::block_on(
                new_context.submit_work("producer historical replay", [replay_work]),
            )
            .unwrap();
            Ok::<_, ()>((submission, ()))
        })
        .unwrap();
    let replay_receipt = producer.accepted_submission().unwrap().clone();
    wait(&new_context, &replay_receipt);
    let committed = producer
        .commit_verified(new_context.affinity(), |submission, steps| {
            assert_eq!(submission.id(), replay_receipt.id());
            assert_eq!(
                steps.iter().map(|s| s.tick()).collect::<Vec<_>>(),
                vec![1, 2]
            );
            assert!(bytes_equal(submission, replay_readback, &unverified_write));
            Ok::<(), ()>(())
        })
        .unwrap();
    assert_eq!(committed, 2);
    assert_eq!(
        (producer.requested_tick(), producer.completed_tick()),
        (3, 2)
    );
    assert_eq!(
        producer
            .candidate(3)
            .iter()
            .map(|s| s.tick())
            .collect::<Vec<_>>(),
        vec![3]
    );
}
