//! Host-owned, public-only stateful GPU conformance for Runenwerk #1291.
//!
//! This is a bounded *test-owned* reference execution, not the shipping
//! RenderFlow scheduler. It proves that immutable CPU step intents can outlive
//! multiple deferred host frames, and that RunenGPU can prepare an exact new
//! batch after the prior accepted retained submission has terminally completed.
//! It deliberately never interprets RetainedContinuityChanged as retryable.
use runen_gpu::{
    GpuBindingKey, GpuBufferDescriptor, GpuBufferHandle, GpuBufferInitialization, GpuBufferRegion,
    GpuBufferUsage, GpuCapabilityProfile, GpuComputeOperation, GpuComputePipelineDescriptor,
    GpuContext, GpuContextDescriptor, GpuContextRequestErrorCategory, GpuDispatchIntent,
    GpuDispatchSize, GpuExecutionPreference, GpuReadbackId, GpuReadbackOperation,
    GpuReadbackStatus, GpuReconstruction, GpuResourceLabel, GpuResourceLifetime,
    GpuResourceProvenance, GpuRuntimeBindingResource, GpuRuntimeBindingSet, GpuRuntimeBindingValue,
    GpuRuntimeBufferBinding, GpuSubmission, GpuSubmissionRejectionKind, GpuSubmissionStatus,
    GpuWorkFragment, GpuWorkFragmentBuilder, GpuWorkOperation, GpuWorkResourceIdAllocator,
    GpuWorkSubmissionError, admit_static_wgsl_sources,
};
use std::num::NonZeroU64;
use std::time::{Duration, Instant};

const INCREMENT: &str = r#"
@group(0) @binding(0)
var<storage, read_write> value: array<u32>;

@compute @workgroup_size(1)
fn cs_main() {
    value[0] = value[0] + 1u;
}
"#;

fn label(value: &str) -> GpuResourceLabel {
    GpuResourceLabel::new(value).expect("nonempty GPU label")
}

fn provenance(value: &str) -> GpuResourceProvenance {
    GpuResourceProvenance::new(label(value), None, None)
}

fn context() -> Option<GpuContext> {
    let request = GpuContextDescriptor::new(GpuCapabilityProfile::ComputeBaseline.requirements())
        .with_label("Runenwerk fixed GPU step continuity proof");
    match pollster::block_on(GpuContext::request(request)) {
        Ok(context) => Some(context),
        Err(error) if error.category() == GpuContextRequestErrorCategory::NoAdapterAvailable => {
            assert_ne!(
                std::env::var("RUNENWERK_G5_REQUIRE_GPU").ok().as_deref(),
                Some("1"),
                "native CI requires a usable public GPU context"
            );
            None
        }
        Err(error) => panic!("GPU context request failed: {error}"),
    }
}

fn state_buffer() -> GpuBufferHandle {
    GpuWorkResourceIdAllocator::new()
        .allocate_buffer_handle(
            GpuBufferDescriptor::ordinary_owned(
                "fixed.step.retained.state",
                GpuResourceLifetime::Retained,
                GpuReconstruction::SourceBacked,
                64,
                [GpuBufferUsage::Storage, GpuBufferUsage::CopySource],
                GpuBufferInitialization::Zeroed,
            )
            .expect("zeroed retained source-backed state descriptor"),
        )
        .expect("unique state resource")
}

fn pipeline() -> GpuComputePipelineDescriptor {
    let [source] = admit_static_wgsl_sources([("runenwerk.fixed_step.counter", 1, INCREMENT)])
        .expect("canonical public WGSL admission");
    GpuComputePipelineDescriptor::ordinary(source, "cs_main")
        .expect("ordinary public compute pipeline")
}

fn increment_op(
    pipeline: &GpuComputePipelineDescriptor,
    state: &GpuBufferHandle,
) -> GpuComputeOperation {
    let binding = GpuRuntimeBindingValue::new(
        GpuBindingKey::try_new(0, 0).expect("binding slot"),
        [GpuRuntimeBindingResource::Buffer(
            GpuRuntimeBufferBinding::new(
                state.clone(),
                0,
                NonZeroU64::new(4).expect("four-byte storage binding"),
                None,
            ),
        )],
    )
    .expect("read-write state binding");
    let bindings = GpuRuntimeBindingSet::new(pipeline.layout().clone(), [binding])
        .expect("compatible compute pipeline binding set");
    GpuComputeOperation::new(
        pipeline.clone(),
        bindings,
        GpuDispatchIntent::direct(GpuDispatchSize::new(1, 1, 1)),
    )
    .expect("one counter increment")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Step {
    tick: u64,
    increments: u8,
}

/// GPU readback-specific proof attached to the exact admitted semantic
/// batch. Immutable step capture/ownership is the shipping producer transport.
struct AcceptedBatch {
    expected_counter: u32,
    readback: GpuReadbackId,
}

struct StepLedger {
    history: engine::plugins::render::RenderGpuStepDemand<Step>,
    completed_counter: u32,
    accepted: Option<AcceptedBatch>,
}

impl StepLedger {
    fn new(limit: usize) -> Self {
        Self {
            history: engine::plugins::render::RenderGpuStepDemand::new(0, limit),
            completed_counter: 0,
            accepted: None,
        }
    }

    fn requested_tick(&self) -> u64 {
        self.history.requested_tick()
    }

    fn completed_tick(&self) -> u64 {
        self.history.completed_tick()
    }

    fn outstanding(&self) -> usize {
        self.history.outstanding()
    }

    fn request(&mut self, increments: &[u8]) -> Result<(), &'static str> {
        if increments
            .iter()
            .any(|increment| *increment == 0 || *increment > 16)
        {
            return Err("invalid captured step increment");
        }
        let snapshots = increments
            .iter()
            .enumerate()
            .map(|(index, increment)| {
                let offset = u64::try_from(index).map_err(|_| "step count overflow")?;
                let tick = self
                    .history
                    .requested_tick()
                    .checked_add(offset + 1)
                    .ok_or("tick identity exhausted")?;
                Ok(Step {
                    tick,
                    increments: *increment,
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?;
        self.history
            .capture_batch(snapshots)
            .map_err(|_| "step demand capacity or identity exhausted")?;
        Ok(())
    }

    fn admit(
        &mut self,
        context: &GpuContext,
        state: &GpuBufferHandle,
        pipeline: &GpuComputePipelineDescriptor,
    ) -> bool {
        let completed_tick = self.history.completed_tick();
        let completed_counter = self.completed_counter;
        let proof = self
            .history
            .submit_with(3, |candidate| {
                let snapshots = candidate
                    .iter()
                    .map(|entry| *entry.input())
                    .collect::<Vec<_>>();
                assert_eq!(snapshots[0].tick, completed_tick + 1);
                for (intent, snapshot) in candidate.iter().zip(&snapshots) {
                    assert_eq!(
                        intent.tick(),
                        snapshot.tick,
                        "captured and authored step identity agree"
                    );
                }
                for window in snapshots.windows(2) {
                    assert_eq!(window[1].tick, window[0].tick + 1);
                }
                let (fragment, readback) = batch_fragment(state, pipeline, &snapshots);
                let submission = pollster::block_on(
                    context.submit_work("runenwerk.fixed-step.fresh-batch", [fragment]),
                )
                .expect("uncontended test producer re-prepares after terminal");
                let expected_counter = completed_counter
                    + snapshots
                        .iter()
                        .map(|step| u32::from(step.increments))
                        .sum::<u32>();
                Ok::<_, &'static str>((
                    submission,
                    AcceptedBatch {
                        expected_counter,
                        readback,
                    },
                ))
            })
            .expect("the test's fresh graph must be accepted");
        let Some(accepted) = proof else {
            return false;
        };
        self.accepted = Some(accepted);
        true
    }

    fn reconcile(&mut self, context: &GpuContext) {
        let Some(accepted) = self.accepted.as_ref() else {
            return;
        };
        let submission = self
            .history
            .accepted_submission()
            .expect("accepted GPU batch");
        assert_eq!(
            submission.affinity(),
            context.affinity(),
            "generation replacement requires semantic reconstruction, not replay"
        );
        match submission.status() {
            GpuSubmissionStatus::Accepted => {}
            GpuSubmissionStatus::Completed => {
                let result = submission
                    .readback(accepted.readback)
                    .expect("required proof readback")
                    .status();
                let GpuReadbackStatus::Ready(bytes) = result else {
                    panic!("completed GPU step batch must have ready readback: {result:?}");
                };
                let observed = u32::from_le_bytes(
                    bytes.as_bytes()[0..4]
                        .try_into()
                        .expect("four-byte readback"),
                );
                assert_eq!(observed, accepted.expected_counter);
                self.history
                    .commit_completed(context.affinity())
                    .expect("producer verified exact completed batch");
                self.completed_counter = observed;
                self.accepted = None;
            }
            GpuSubmissionStatus::Failed(failure) => {
                panic!(
                    "accepted state mutation failed; content needs explicit reconstruction: {failure:?}"
                )
            }
        }
    }
}

fn batch_fragment(
    state: &GpuBufferHandle,
    pipeline: &GpuComputePipelineDescriptor,
    steps: &[Step],
) -> (GpuWorkFragment, GpuReadbackId) {
    let name = "runenwerk.fixed-step.batch";
    let mut builder = GpuWorkFragmentBuilder::new(label(name), provenance(name));
    builder
        .declare_resource(state.clone().into())
        .expect("declare canonical state");
    // G3 owns WAW and RAW dependencies for this shared retained storage.
    // Explicit duplicate edges are rejected as RedundantExplicitDataOrder.
    for step in steps {
        for ordinal in 0..step.increments {
            builder
                .compute(
                    format!("step.{}.increment.{ordinal}", step.tick),
                    increment_op(pipeline, state),
                )
                .expect("bounded semantic step compute node");
        }
    }
    let readback = GpuReadbackId::allocate().expect("unique observation readback");
    let op = GpuReadbackOperation::new(
        GpuBufferRegion::whole(state).expect("whole state").into(),
        readback,
    )
    .expect("readback operation");
    builder
        .add_node(
            label("fixed.step.batch.terminal.readback"),
            GpuWorkOperation::Readback(op),
            [],
            runen_gpu::GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance(name),
        )
        .expect("readback follows typed write/read graph hazards");
    (builder.finish().expect("complete GPU step graph"), readback)
}

fn progress_to_terminal(context: &GpuContext, submission: &GpuSubmission) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while submission.status() == GpuSubmissionStatus::Accepted {
        assert!(
            Instant::now() < deadline,
            "GPU submission did not become terminal"
        );
        // Separate native *test harness* progress; not an Engine Gfx progress point.
        context.progress();
        std::thread::yield_now();
    }
}

#[test]
fn delayed_gpu_batches_consume_all_captured_steps_once() {
    let Some(context) = context() else {
        return;
    };
    let buffer = state_buffer();
    let pipeline = pipeline();
    let mut ledger = StepLedger::new(5);

    // CPU frame 1: one step accepted by G5, not yet claimed completed.
    ledger.request(&[1]).expect("first captured step");
    assert!(ledger.admit(&context, &buffer, &pipeline));
    assert_eq!(ledger.completed_tick(), 0);
    assert_eq!(ledger.outstanding(), 1);

    // CPU frames 2-4: no step, then two, then one; GPU predecessor
    // deliberately remains un-reconciled across all of them.
    ledger.request(&[]).expect("zero-step frame");
    assert!(!ledger.admit(&context, &buffer, &pipeline));
    ledger.request(&[2, 3]).expect("two-step frame");
    assert!(!ledger.admit(&context, &buffer, &pipeline));
    ledger.request(&[1]).expect("one-step frame");
    assert!(!ledger.admit(&context, &buffer, &pipeline));
    assert_eq!(ledger.requested_tick(), 4);
    assert_eq!(ledger.completed_tick(), 0);
    assert_eq!(ledger.outstanding(), 4);

    // Capacity does not silently discard an earlier event or advance time.
    assert!(ledger.request(&[1, 1]).is_err());
    assert_eq!(ledger.requested_tick(), 4);
    assert_eq!(ledger.outstanding(), 4);

    let first = ledger
        .history
        .accepted_submission()
        .expect("first submission")
        .clone();
    progress_to_terminal(&context, &first);
    ledger.reconcile(&context);
    assert_eq!((ledger.completed_tick(), ledger.completed_counter), (1, 1));

    // Freshly formed graph batches *captured* steps 2..4, including their
    // differing work weights, not a replacement latest-frame step count.
    assert!(ledger.admit(&context, &buffer, &pipeline));
    let second = ledger
        .history
        .accepted_submission()
        .expect("second submission")
        .clone();
    assert_eq!(ledger.completed_tick(), 1);
    progress_to_terminal(&context, &second);
    ledger.reconcile(&context);
    assert_eq!((ledger.requested_tick(), ledger.completed_tick()), (4, 4));
    assert_eq!(ledger.completed_counter, 7);
    assert_eq!(ledger.outstanding(), 0);
    assert!(!ledger.admit(&context, &buffer, &pipeline));
}

#[test]
fn captured_producer_steps_survive_real_g5_rejection_and_fresh_reprepare() {
    let Some(context) = context() else {
        return;
    };
    let buffer = state_buffer();
    let pipeline = pipeline();

    // External accepted retained work reserves the same actual RunenGPU
    // resource. This is a deliberately controlled G5 admission conflict.
    let (blocking_fragment, _) = batch_fragment(
        &buffer,
        &pipeline,
        &[Step {
            tick: 99,
            increments: 1,
        }],
    );
    let blocker =
        pollster::block_on(context.submit_work("fixed.step.g5.blocker", [blocking_fragment]))
            .expect("first retained GPU admission");
    assert_eq!(blocker.status(), GpuSubmissionStatus::Accepted);

    let mut producer = engine::plugins::render::RenderGpuStepDemand::new(0, 3);
    let snapshots = [
        Step {
            tick: 1,
            increments: 2,
        },
        Step {
            tick: 2,
            increments: 3,
        },
    ];
    producer
        .capture_batch(snapshots.to_vec())
        .expect("capture both producer-authored historical inputs");

    let rejected = producer.submit_with(2, |candidate| {
        let captured = candidate
            .iter()
            .map(|entry| *entry.input())
            .collect::<Vec<_>>();
        let (graph, _) = batch_fragment(&buffer, &pipeline, &captured);
        pollster::block_on(context.submit_work("fixed.step.g5.conflict", [graph]))
            .map(|submission| (submission, ()))
    });
    let rejection = match rejected {
        Err(GpuWorkSubmissionError::SubmissionRejected(reason)) => reason,
        Err(error) => panic!("expected RunenGPU G5 retained rejection, got {error:?}"),
        Ok(_) => panic!("overlapping retained writer was incorrectly accepted"),
    };
    assert_eq!(
        rejection.kind(),
        GpuSubmissionRejectionKind::RetainedContinuityChanged,
    );
    assert_eq!(producer.requested_tick(), 2);
    assert_eq!(producer.completed_tick(), 0);
    assert_eq!(producer.pending_len(), 2);
    assert_eq!(producer.outstanding(), 2);
    assert!(producer.accepted_submission().is_none());
    assert_eq!(
        producer
            .candidate(2)
            .iter()
            .map(|intent| (intent.tick(), intent.input().increments))
            .collect::<Vec<_>>(),
        vec![(1, 2), (2, 3)]
    );

    // This test *knows* its transient predecessor and waits for that exact
    // public receipt to finish. Production MUST NOT infer retryability from
    // RetainedContinuityChanged alone: #168 distinguishes permanent faults.
    progress_to_terminal(&context, &blocker);
    let readback = producer
        .submit_with(2, |candidate| {
            let captured = candidate
            .iter()
            .map(|entry| *entry.input())
            .collect::<Vec<_>>();
            let (fresh_graph, readback) = batch_fragment(&buffer, &pipeline, &captured);
            pollster::block_on(
                context.submit_work("fixed.step.g5.fresh-after-terminal", [fresh_graph]),
            )
            .map(|submission| (submission, readback))
        })
        .expect("fresh graph after exact predecessor termination")
        .expect("both historical steps are still pending");
    let accepted = producer
        .accepted_submission()
        .expect("exact accepted G5 receipt")
        .clone();
    assert_eq!(producer.pending_len(), 0);
    assert_eq!(producer.outstanding(), 2);
    assert_eq!(producer.completed_tick(), 0);
    progress_to_terminal(&context, &accepted);
    let readback = accepted
        .readback(readback)
        .expect("completed producer readback");
    let GpuReadbackStatus::Ready(bytes) = readback.status() else {
        panic!("completed producer readback must be ready");
    };
    assert_eq!(
        u32::from_le_bytes(bytes.as_bytes()[..4].try_into().expect("counter bytes")),
        6,
        "one blocking increment and then historical producer increments 2+3",
    );
    assert_eq!(producer.commit_completed(context.affinity()), Ok(2));
    assert_eq!(producer.outstanding(), 0);
}
