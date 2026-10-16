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
    GpuDispatchSize, GpuExecutionPreference, GpuExplicitOrder, GpuReadbackId, GpuReadbackOperation,
    GpuReadbackStatus, GpuReconstruction, GpuResourceLabel, GpuResourceLifetime,
    GpuResourceProvenance, GpuRuntimeBindingResource, GpuRuntimeBindingSet,
    GpuRuntimeBindingValue, GpuRuntimeBufferBinding, GpuSubmission, GpuSubmissionStatus,
    GpuWorkFragment, GpuWorkFragmentBuilder, GpuWorkOperation, GpuWorkResourceIdAllocator,
    admit_static_wgsl_sources,
};
use std::collections::VecDeque;
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

/// A test-only immutable batch. Retained-resource reservation is not mirrored here.
struct AcceptedBatch {
    submission: GpuSubmission,
    last_tick: u64,
    expected_counter: u32,
    readback: GpuReadbackId,
}

/// A *semantic step* ledger, not a copy of G5's internal GPU resource table.
/// Work is removed from the pending queue only when G5 actually accepts it.
struct StepLedger {
    next_tick: u64,
    completed_tick: u64,
    completed_counter: u32,
    backlog: VecDeque<Step>,
    accepted: Option<AcceptedBatch>,
    limit: usize,
}

impl StepLedger {
    fn new(limit: usize) -> Self {
        Self {
            next_tick: 1,
            completed_tick: 0,
            completed_counter: 0,
            backlog: VecDeque::new(),
            accepted: None,
            limit,
        }
    }

    fn requested_tick(&self) -> u64 {
        self.next_tick - 1
    }

    fn outstanding(&self) -> usize {
        self.backlog.len() + self.accepted.as_ref().map_or(0, |batch| {
            usize::try_from(batch.last_tick - self.completed_tick)
                .expect("bounded GPU step interval")
        })
    }

    /// Reject excess demand *atomically*. A real producer must choose and
    /// document its domain-specific wait, backpressure, or resync policy.
    fn request(&mut self, increments: &[u8]) -> Result<(), &'static str> {
        if increments.iter().any(|increment| *increment == 0 || *increment > 16) {
            return Err("invalid captured step increment");
        }
        let next_outstanding = self
            .outstanding()
            .checked_add(increments.len())
            .ok_or("step backlog overflow")?;
        if next_outstanding > self.limit {
            return Err("step backlog limit");
        }
        let next_tick = self
            .next_tick
            .checked_add(u64::try_from(increments.len()).map_err(|_| "step count overflow")?)
            .ok_or("step identity exhausted")?;
        for (index, increment) in increments.iter().copied().enumerate() {
            let tick = self.next_tick + u64::try_from(index).expect("within checked count");
            self.backlog.push_back(Step { tick, increments: increment });
        }
        self.next_tick = next_tick;
        Ok(())
    }

    /// Called without waiting from host frames. Does not prepare a graph while
    /// this producer has accepted unfinished state mutation.
    fn admit(
        &mut self,
        context: &GpuContext,
        state: &GpuBufferHandle,
        pipeline: &GpuComputePipelineDescriptor,
    ) -> bool {
        if self.accepted.is_some() || self.backlog.is_empty() {
            return false;
        }
        let snapshot = self.backlog.iter().copied().take(3).collect::<Vec<_>>();
        assert_eq!(
            snapshot.first().expect("nonempty").tick,
            self.completed_tick + 1
        );
        for window in snapshot.windows(2) {
            assert_eq!(window[1].tick, window[0].tick + 1);
        }
        let (fragment, readback) = batch_fragment(state, pipeline, &snapshot);
        let submission = pollster::block_on(context.submit_work(
            "runenwerk.fixed-step.fresh-batch",
            [fragment],
        ))
        .expect("test producer is uncontended and re-prepares after terminal completion");
        assert_eq!(submission.status(), GpuSubmissionStatus::Accepted);
        let expected_counter = self.completed_counter
            + snapshot.iter().map(|step| u32::from(step.increments)).sum::<u32>();
        let last_tick = snapshot.last().expect("nonempty").tick;
        for expected in &snapshot {
            assert_eq!(self.backlog.pop_front(), Some(*expected));
        }
        self.accepted = Some(AcceptedBatch {
            submission,
            last_tick,
            expected_counter,
            readback,
        });
        true
    }

    fn reconcile(&mut self, context: &GpuContext) {
        let Some(accepted) = self.accepted.as_ref() else {
            return;
        };
        assert_eq!(
            accepted.submission.affinity(),
            context.affinity(),
            "generation replacement requires semantic reconstruction, not replay"
        );
        match accepted.submission.status() {
            GpuSubmissionStatus::Accepted => {}
            GpuSubmissionStatus::Completed => {
                let result = accepted.submission.readback(accepted.readback)
                    .expect("required proof readback").status();
                let GpuReadbackStatus::Ready(bytes) = result else {
                    panic!("completed GPU step batch must have ready readback: {result:?}");
                };
                let observed = u32::from_le_bytes(
                    bytes.as_bytes()[0..4].try_into().expect("four-byte readback")
                );
                assert_eq!(observed, accepted.expected_counter);
                self.completed_counter = observed;
                self.completed_tick = accepted.last_tick;
                self.accepted = None;
            }
            GpuSubmissionStatus::Failed(failure) => {
                panic!("accepted state mutation failed; content needs explicit reconstruction: {failure:?}")
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
    let mut preceding = None;
    for step in steps {
        for ordinal in 0..step.increments {
            let node = builder
                .compute(
                    format!("step.{}.increment.{ordinal}", step.tick),
                    increment_op(pipeline, state),
                )
                .expect("bounded semantic step compute node");
            if let Some(prev) = preceding {
                builder
                    .add_explicit_order(
                        GpuExplicitOrder::new(&prev, &node, "ordered steps within one GPU batch")
                            .expect("valid canonical intra-batch ordering"),
                    )
                    .expect("no cycle in forward step sequence");
            }
            preceding = Some(node);
        }
    }
    let readback = GpuReadbackId::allocate().expect("unique observation readback");
    let op = GpuReadbackOperation::new(
        GpuBufferRegion::whole(state).expect("whole state").into(),
        readback,
    )
    .expect("readback operation");
    let tail = builder
        .add_node(
            label("fixed.step.batch.terminal.readback"),
            GpuWorkOperation::Readback(op),
            [],
            runen_gpu::GpuCapabilityRequirements::new(),
            GpuExecutionPreference::TransferPreferred,
            provenance(name),
        )
        .expect("readback node");
    if let Some(prev) = preceding {
        builder
            .add_explicit_order(
                GpuExplicitOrder::new(&prev, &tail, "read after all accepted step effects")
                    .expect("valid final ordering"),
            )
            .expect("valid terminal observation");
    }
    (builder.finish().expect("complete GPU step graph"), readback)
}

fn progress_to_terminal(context: &GpuContext, submission: &GpuSubmission) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while submission.status() == GpuSubmissionStatus::Accepted {
        assert!(Instant::now() < deadline, "GPU submission did not become terminal");
        // Separate native *test harness* progress; not an Engine Gfx progress point.
        context.progress();
        std::thread::yield_now();
    }
}

#[test]
fn delayed_gpu_batches_consume_all_captured_steps_once_without_global_queue_lock() {
    let Some(context) = context() else {
        return;
    };
    let buffer = state_buffer();
    let pipeline = pipeline();
    let mut ledger = StepLedger::new(5);

    // CPU frame 1: one step accepted by G5, not yet claimed completed.
    ledger.request(&[1]).expect("first captured step");
    assert!(ledger.admit(&context, &buffer, &pipeline));
    assert_eq!(ledger.completed_tick, 0);
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
    assert_eq!(ledger.completed_tick, 0);
    assert_eq!(ledger.outstanding(), 4);

    // Capacity does not silently discard an earlier event or advance time.
    assert!(ledger.request(&[1, 1]).is_err());
    assert_eq!(ledger.requested_tick(), 4);
    assert_eq!(ledger.outstanding(), 4);

    let first = ledger.accepted.as_ref().expect("first submission").submission.clone();
    progress_to_terminal(&context, &first);
    ledger.reconcile(&context);
    assert_eq!((ledger.completed_tick, ledger.completed_counter), (1, 1));

    // Freshly formed graph batches *captured* steps 2..4, including their
    // differing work weights, not a replacement latest-frame step count.
    assert!(ledger.admit(&context, &buffer, &pipeline));
    let second = ledger.accepted.as_ref().expect("second submission").submission.clone();
    assert_eq!(ledger.completed_tick, 1);
    progress_to_terminal(&context, &second);
    ledger.reconcile(&context);
    assert_eq!((ledger.requested_tick(), ledger.completed_tick), (4, 4));
    assert_eq!(ledger.completed_counter, 7);
    assert_eq!(ledger.outstanding(), 0);
    assert!(!ledger.admit(&context, &buffer, &pipeline));
}
