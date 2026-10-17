//! Bounded producer-authored semantic step demand across asynchronous GPU frames.
//!
//! This is a Runenwerk transport for *immutable producer input*, not a clock,
//! GPU resource reservation system, or a replacement for RunenGPU's G3/G5.
//! Only the producer can certify that a prepared GPU graph faithfully lowers
//! every captured step. An accepted submission is not a completed simulation.
use crate::plugins::render::{
    PreparedRenderFrame, RenderFixedStepIterationUniform, RenderFlowId,
};
use crate::plugins::render::inspect::{
    RenderCaptureIdentity, RenderCaptureSubmissionReceipt,
};
use runen_gpu::{
    GpuContextAffinity, GpuReadbackId, GpuReadbackStatus, GpuSubmission, GpuSubmissionStatus,
    GpuTextureFormat, GpuWorkResourceId,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A copied or otherwise producer-owned immutable step input with its sequence
/// identity. Do not retain references to mutable ECS state in the payload.
#[derive(Debug)]
pub struct RenderGpuStepIntent<T> {
    tick: u64,
    input: T,
}

impl<T> RenderGpuStepIntent<T> {
    pub const fn tick(&self) -> u64 {
        self.tick
    }

    pub fn input(&self) -> &T {
        &self.input
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderGpuStepDemandError {
    CapacityExceeded,
    TickIdentityExhausted,
    NoAcceptedBatch,
    SubmissionPending,
    SubmissionFailed,
    DeviceGenerationChanged,
    ReconstructionReceiptIsPriorWork,
}

impl std::fmt::Display for RenderGpuStepDemandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RenderGpuStepDemandError {}

/// A completed GPU submission is not a verified semantic simulation result.
/// Rejected verification leaves the exact accepted submission and immutable
/// inputs owned by the producer, without advancing its completed tick.
#[derive(Debug, PartialEq, Eq)]
pub enum RenderGpuStepCommitError<E> {
    Submission(RenderGpuStepDemandError),
    Verification(E),
}

impl<E: std::fmt::Display> std::fmt::Display for RenderGpuStepCommitError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Submission(reason) => write!(f, "GPU step commit rejected: {reason}"),
            Self::Verification(reason) => write!(f, "GPU step verification rejected: {reason}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for RenderGpuStepCommitError<E> {}

#[derive(Debug)]
struct AcceptedSteps<T> {
    submission: GpuSubmission,
    steps: Vec<RenderGpuStepIntent<T>>,
}

/// Explicit producer demand; never consumes an input merely because CPU time
/// advanced or a GPU graph was prepared. The producer decides whether to pause
/// simulation, reject new demand or reconstruct after a terminal failure.
#[derive(Debug)]
pub struct RenderGpuStepDemand<T> {
    next_tick: u64,
    completed_tick: u64,
    maximum_outstanding: usize,
    pending: VecDeque<RenderGpuStepIntent<T>>,
    accepted: Option<AcceptedSteps<T>>,
}

impl<T> RenderGpuStepDemand<T> {
    /// The initial completed tick MUST reflect producer-verified GPU contents.
    /// A fresh zeroed state starts at zero; reconstruction is producer-owned.
    pub fn new(completed_tick: u64, maximum_outstanding: usize) -> Self {
        Self {
            next_tick: completed_tick,
            completed_tick,
            maximum_outstanding,
            pending: VecDeque::new(),
            accepted: None,
        }
    }

    pub const fn requested_tick(&self) -> u64 {
        self.next_tick
    }

    pub const fn completed_tick(&self) -> u64 {
        self.completed_tick
    }

    pub fn outstanding(&self) -> usize {
        self.pending.len() + self.accepted.as_ref().map_or(0, |a| a.steps.len())
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Atomically capture a producer-owned immutable historical interval.
    /// Rejected inputs are returned to the producer, not silently destroyed;
    /// the producer MUST decide whether to pause, retry or resynchronize.
    pub fn capture_batch(
        &mut self,
        inputs: Vec<T>,
    ) -> Result<(u64, u64), (RenderGpuStepDemandError, Vec<T>)> {
        let Some(requested) = self.outstanding().checked_add(inputs.len()) else {
            return Err((RenderGpuStepDemandError::CapacityExceeded, inputs));
        };
        if requested > self.maximum_outstanding {
            return Err((RenderGpuStepDemandError::CapacityExceeded, inputs));
        }
        let Ok(count) = u64::try_from(inputs.len()) else {
            return Err((RenderGpuStepDemandError::TickIdentityExhausted, inputs));
        };
        let Some(last_tick) = self.next_tick.checked_add(count) else {
            return Err((RenderGpuStepDemandError::TickIdentityExhausted, inputs));
        };
        let start_tick = self.next_tick;
        for (offset, input) in (1..=count).zip(inputs) {
            self.pending.push_back(RenderGpuStepIntent {
                tick: start_tick + offset,
                input,
            });
        }
        self.next_tick = last_tick;
        Ok((start_tick, last_tick))
    }

    /// Capture one producer-owned step. On rejection the unconsumed input
    /// is returned in the error alongside its reason.
    pub fn capture(&mut self, input: T) -> Result<u64, (RenderGpuStepDemandError, T)> {
        self.capture_batch(vec![input])
            .map(|(_, last_tick)| last_tick)
            .map_err(|(reason, mut inputs)| (reason, inputs.remove(0)))
    }

    /// Returns only the captured historical prefix for a *fresh* GPU graph.
    /// A candidate does not consume work; RunenGPU admission does.
    pub fn candidate(&self, max_steps: usize) -> Vec<&RenderGpuStepIntent<T>> {
        if self.accepted.is_some() || max_steps == 0 {
            return Vec::new();
        }
        self.pending.iter().take(max_steps).collect()
    }

    /// Atomically lower a freshly borrowed immutable prefix through the
    /// producer's normal RunenGPU admission path. A rejected submission leaves
    /// every input pending, with its original order and identity intact.
    ///
    /// The closure MUST submit GPU work faithfully representing precisely the
    /// supplied inputs and return its actual accepted GpuSubmission handle.
    /// Upon success the matching prefix moves to in-flight *infallibly*: a
    /// caller cannot accidentally submit first and then reject the count.
    /// This is producer backpressure, not a RunenGPU G5 hazard shadow.
    pub fn submit_with<R, E>(
        &mut self,
        max_steps: usize,
        submit: impl FnOnce(&[&RenderGpuStepIntent<T>]) -> Result<(GpuSubmission, R), E>,
    ) -> Result<Option<R>, E> {
        let candidate = self.candidate(max_steps);
        if candidate.is_empty() {
            return Ok(None);
        }
        let count = candidate.len();
        let (submission, result) = submit(&candidate)?;
        drop(candidate);
        // An accepted submission may have failed before the returned handle
        // is observed. Even then writes can be partially executed; holding
        // the exact historical input batch forces explicit reconstruction.
        let steps = self.pending.drain(..count).collect();
        self.accepted = Some(AcceptedSteps { submission, steps });
        Ok(Some(result))
    }

    /// Requeue exactly the accepted historical prefix after the producer has
    /// independently reconstructed its *previous verified* semantic state.
    ///
    /// Recovery is deliberately an explicit producer decision. The earlier
    /// submission must have terminalized: an Accepted write may still execute
    /// and race reconstruction. The distinct reconstruction receipt must be
    /// Completed on the current GPU generation; the callback must additionally
    /// verify that its effects actually restore the state represented by
    /// `completed_tick` (for example by correlating a canonical restore work
    /// node and checking its GPU readback).
    ///
    /// An ordinary zero-initialized descriptor or a different generation is not
    /// itself reconstruction evidence. In particular, this method does not
    /// authorize replay of a potentially partial write without an exact,
    /// application-verified replacement of its effects.
    ///
    /// On any rejection, all original historical inputs and the accepted
    /// submission remain owned by the producer. After successful verification,
    /// no CPU tick is reissued: only the original accepted prefix moves back
    /// before later queued steps for *fresh* preparation and G5 admission.
    pub fn requeue_verified_after_reconstruction<E>(
        &mut self,
        current_affinity: GpuContextAffinity,
        reconstruction: &GpuSubmission,
        verify: impl FnOnce(&GpuSubmission, &GpuSubmission, &[RenderGpuStepIntent<T>]) -> Result<(), E>,
    ) -> Result<(), RenderGpuStepCommitError<E>> {
        let batch = self
            .accepted
            .as_ref()
            .ok_or(RenderGpuStepCommitError::Submission(
                RenderGpuStepDemandError::NoAcceptedBatch,
            ))?;
        if matches!(batch.submission.status(), GpuSubmissionStatus::Accepted) {
            return Err(RenderGpuStepCommitError::Submission(
                RenderGpuStepDemandError::SubmissionPending,
            ));
        }
        if reconstruction.affinity() != current_affinity {
            return Err(RenderGpuStepCommitError::Submission(
                RenderGpuStepDemandError::DeviceGenerationChanged,
            ));
        }
        if reconstruction.affinity() == batch.submission.affinity()
            && reconstruction.id() == batch.submission.id()
        {
            return Err(RenderGpuStepCommitError::Submission(
                RenderGpuStepDemandError::ReconstructionReceiptIsPriorWork,
            ));
        }
        match reconstruction.status() {
            GpuSubmissionStatus::Accepted => {
                return Err(RenderGpuStepCommitError::Submission(
                    RenderGpuStepDemandError::SubmissionPending,
                ));
            }
            GpuSubmissionStatus::Failed(_) => {
                return Err(RenderGpuStepCommitError::Submission(
                    RenderGpuStepDemandError::SubmissionFailed,
                ));
            }
            GpuSubmissionStatus::Completed => {}
        }
        verify(reconstruction, &batch.submission, &batch.steps)
            .map_err(RenderGpuStepCommitError::Verification)?;

        // Allocate before transferring ownership: subsequent work only moves
        // already-captured values, without reconstructing or cloning inputs.
        let mut requeued = VecDeque::with_capacity(self.outstanding());
        let batch = self.accepted.take().expect("verified accepted batch");
        requeued.extend(batch.steps);
        requeued.append(&mut self.pending);
        self.pending = requeued;
        Ok(())
    }

    // The renderer calls this only immediately after a successful G5 admission.
    // All source snapshots were compared to the pending prefix *before* G5;
    // no code able to change this producer runs between that check and here.
    fn accept_prevalidated(&mut self, count: usize, submission: GpuSubmission) {
        debug_assert!(self.accepted.is_none());
        debug_assert!(count > 0 && count <= self.pending.len());
        let steps = self.pending.drain(..count).collect();
        self.accepted = Some(AcceptedSteps { submission, steps });
    }

    pub fn accepted_submission(&self) -> Option<&GpuSubmission> {
        self.accepted.as_ref().map(|batch| &batch.submission)
    }

    /// Preserve the admitted historical payloads until their producer has
    /// explicitly checked its desired completion evidence.
    pub fn accepted_steps(&self) -> &[RenderGpuStepIntent<T>] {
        self.accepted
            .as_ref()
            .map_or(&[], |batch| batch.steps.as_slice())
    }

    /// Verify the *exact* terminal GPU submission against the producer's
    /// immutable authored steps as one transaction with semantic completion.
    /// The provided callback MUST inspect the producer's required result
    /// evidence (for example, GPU readback). A failed callback cannot consume
    /// source inputs or advance the completed producer tick.
    ///
    /// This guards against accidentally committing merely because a physical
    /// GPU submission became Completed. RunenGPU owns physical status; the
    /// producer owns the meaning and sufficiency of verification.
    ///
    /// Failure or generation change keeps the original admitted history
    /// quarantined. Reconstruction is deliberately not automatic.
    pub fn commit_verified<E>(
        &mut self,
        current_affinity: GpuContextAffinity,
        verify: impl FnOnce(&GpuSubmission, &[RenderGpuStepIntent<T>]) -> Result<(), E>,
    ) -> Result<u64, RenderGpuStepCommitError<E>> {
        let batch = self
            .accepted
            .as_ref()
            .ok_or(RenderGpuStepCommitError::Submission(
                RenderGpuStepDemandError::NoAcceptedBatch,
            ))?;
        if batch.submission.affinity() != current_affinity {
            return Err(RenderGpuStepCommitError::Submission(
                RenderGpuStepDemandError::DeviceGenerationChanged,
            ));
        }
        match batch.submission.status() {
            GpuSubmissionStatus::Accepted => {
                return Err(RenderGpuStepCommitError::Submission(
                    RenderGpuStepDemandError::SubmissionPending,
                ));
            }
            GpuSubmissionStatus::Failed(_) => {
                return Err(RenderGpuStepCommitError::Submission(
                    RenderGpuStepDemandError::SubmissionFailed,
                ));
            }
            GpuSubmissionStatus::Completed => {}
        }
        verify(&batch.submission, &batch.steps).map_err(RenderGpuStepCommitError::Verification)?;
        let last_tick = batch.steps.last().expect("accepted batch nonempty").tick;
        debug_assert_eq!(batch.steps[0].tick, self.completed_tick + 1);
        self.accepted = None;
        self.completed_tick = last_tick;
        Ok(last_tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_capture_preserves_all_historical_inputs_without_premature_commit() {
        let mut steps = RenderGpuStepDemand::new(7, 3);
        assert_eq!(steps.capture(String::from("at-8")), Ok(8));
        assert_eq!(steps.capture(String::from("at-9")), Ok(9));
        assert_eq!(steps.capture(String::from("at-10")), Ok(10));
        assert_eq!(
            steps.capture(String::from("lost")),
            Err((
                RenderGpuStepDemandError::CapacityExceeded,
                String::from("lost")
            ))
        );
        assert_eq!(
            (
                steps.requested_tick(),
                steps.completed_tick(),
                steps.outstanding()
            ),
            (10, 7, 3)
        );
        assert_eq!(
            steps
                .candidate(2)
                .iter()
                .map(|v| v.input().as_str())
                .collect::<Vec<_>>(),
            vec!["at-8", "at-9"]
        );
        assert!(steps.accepted_submission().is_none());
        assert!(steps.accepted_steps().is_empty());
    }

    #[test]
    fn rejected_bulk_capture_is_atomic_and_returns_immutable_inputs() {
        let mut steps = RenderGpuStepDemand::new(0, 3);
        assert_eq!(steps.capture(7u8), Ok(1));
        assert_eq!(
            steps.capture_batch(vec![8, 9, 10]),
            Err((RenderGpuStepDemandError::CapacityExceeded, vec![8, 9, 10]))
        );
        assert_eq!(steps.requested_tick(), 1);
        assert_eq!(steps.outstanding(), 1);
        assert_eq!(steps.capture_batch(vec![8, 9]), Ok((1, 3)));
        assert_eq!(steps.requested_tick(), 3);
        assert_eq!(steps.pending_len(), 3);
    }

    #[test]
    fn rejected_gpu_admission_preserves_the_entire_prepared_candidate() {
        let mut demand = RenderGpuStepDemand::new(0, 3);
        demand
            .capture_batch(vec![11_u8, 22])
            .expect("bounded capture");
        let rejected = demand.submit_with(2, |candidate| {
            assert_eq!(candidate.len(), 2);
            assert_eq!(candidate[0].tick(), 1);
            assert_eq!(*candidate[1].input(), 22);
            Err::<(GpuSubmission, ()), &'static str>("rejected before G5 acceptance")
        });
        assert_eq!(rejected, Err("rejected before G5 acceptance"));
        assert_eq!(demand.requested_tick(), 2);
        assert_eq!(demand.completed_tick(), 0);
        assert_eq!(demand.outstanding(), 2);
        assert!(demand.accepted_submission().is_none());
        assert_eq!(
            demand
                .candidate(2)
                .iter()
                .map(|step| *step.input())
                .collect::<Vec<_>>(),
            vec![11, 22]
        );
    }

    #[test]
    fn tick_identity_exhaustion_does_not_consume_any_input() {
        let mut steps = RenderGpuStepDemand::<u8>::new(u64::MAX, 2);
        assert_eq!(
            steps.capture(1),
            Err((RenderGpuStepDemandError::TickIdentityExhausted, 1))
        );
        assert_eq!(steps.outstanding(), 0);
        assert_eq!(steps.requested_tick(), u64::MAX);
    }
}

/// Immutable bytes authored at each semantic tick for uniforms used by one
/// fixed-step RenderFlow region. The producer owns what these values mean.
/// Runenwerk only transports them into the selected pass's actual GPU upload.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderGpuFixedStepInput {
    pub uniforms: BTreeMap<GpuWorkResourceId, Vec<u8>>,
    /// Immutable source-time control. The renderer supplies only the actual
    /// selected batch count/index, never a replacement latest-frame CPU time.
    pub iteration: RenderFixedStepIterationUniform,
}

/// Immutable capture metadata paired with the exact producer-owned G5
/// submission. The actual readback bytes remain in RunenGPU, not a bounded
/// diagnostic ECS view or another Runenwerk byte queue.
#[derive(Debug, Clone)]
pub struct RenderGpuAcceptedCaptureReadback {
    pub identity: RenderCaptureIdentity,
    pub receipt: RenderCaptureSubmissionReceipt,
    pub width: u32,
    pub height: u32,
    pub source_format: GpuTextureFormat,
}

impl RenderGpuAcceptedCaptureReadback {
    pub fn sample_center_rgba8(&self, submission: &GpuSubmission) -> Option<[u8; 4]> {
        if !self.receipt.matches_completed_readback(submission) {
            return None;
        }
        let readback = submission.readback(self.receipt.readback_id())?;
        let GpuReadbackStatus::Ready(bytes) = readback.status() else {
            return None;
        };
        if bytes.texture_format() != Some(self.source_format) {
            return None;
        }
        let width = usize::try_from(self.width).ok()?;
        let height = usize::try_from(self.height).ok()?;
        let expected_len = width.checked_mul(height)?.checked_mul(4)?;
        if width == 0 || height == 0 || bytes.as_bytes().len() != expected_len {
            return None;
        }
        let pixel_offset = (height / 2)
            .checked_mul(width)?
            .checked_add(width / 2)?
            .checked_mul(4)?;
        let pixel = bytes.as_bytes().get(pixel_offset..pixel_offset.checked_add(4)?)?;
        let mut rgba = [pixel[0], pixel[1], pixel[2], pixel[3]];
        match self.source_format {
            GpuTextureFormat::Bgra8Unorm | GpuTextureFormat::Bgra8UnormSrgb => rgba.swap(0, 2),
            GpuTextureFormat::Rgba8Unorm | GpuTextureFormat::Rgba8UnormSrgb => {}
            _ => return None,
        }
        Some(rgba)
    }
}

/// A producer cannot treat missing or truncated capture references as
/// successful verification. A full G5 submission stays retained either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderGpuAcceptedCaptureError {
    NoAcceptedBatch,
    CapacityExceeded,
}

impl std::fmt::Display for RenderGpuAcceptedCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RenderGpuAcceptedCaptureError {}

const MAX_ACCEPTED_PRODUCER_READBACKS: usize = 64;

#[derive(Debug, Default)]
struct AcceptedCaptureReadbacks {
    references: Vec<RenderGpuAcceptedCaptureReadback>,
    truncated: bool,
}

/// An opt-in, producer-owned set of streams, one per ordinary RenderFlow.
/// Existing CPU-driven flows never consult this registry. Producers capture
/// snapshots before irreversibly advancing their own domain state and verify
/// GPU results using the exact accepted submission retained in each stream.
#[derive(Debug, Default, runen_ecs::Resource)]
pub struct RenderGpuFlowStepStreams {
    streams: BTreeMap<RenderFlowId, RenderGpuStepDemand<RenderGpuFixedStepInput>>,
    accepted_capture_readbacks: BTreeMap<RenderFlowId, AcceptedCaptureReadbacks>,
}

impl RenderGpuFlowStepStreams {
    pub fn register(
        &mut self,
        flow: RenderFlowId,
        verified_tick: u64,
        maximum_outstanding: usize,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            maximum_outstanding > 0 && !self.streams.contains_key(&flow),
            "GPU step stream must have positive capacity and unique flow identity"
        );
        self.streams.insert(
            flow,
            RenderGpuStepDemand::new(verified_tick, maximum_outstanding),
        );
        Ok(())
    }

    pub fn stream_mut(
        &mut self,
        flow: RenderFlowId,
    ) -> Option<&mut RenderGpuStepDemand<RenderGpuFixedStepInput>> {
        self.streams.get_mut(&flow)
    }

    pub fn stream(
        &self,
        flow: RenderFlowId,
    ) -> Option<&RenderGpuStepDemand<RenderGpuFixedStepInput>> {
        self.streams.get(&flow)
    }

    /// The capture readback IDs of the currently accepted producer interval.
    /// Unlike derived ECS pixel images, these descriptors are not evicted by
    /// diagnostic history pressure. Consumers must verify G5's terminal
    /// readback and their own source semantics before committing any tick.
    pub fn accepted_capture_readbacks(
        &self,
        flow: RenderFlowId,
    ) -> Result<&[RenderGpuAcceptedCaptureReadback], RenderGpuAcceptedCaptureError> {
        let Some(stream) = self.streams.get(&flow) else {
            return Err(RenderGpuAcceptedCaptureError::NoAcceptedBatch);
        };
        let Some(submission) = stream.accepted_submission() else {
            return Err(RenderGpuAcceptedCaptureError::NoAcceptedBatch);
        };
        let Some(associated) = self.accepted_capture_readbacks.get(&flow) else {
            return Ok(&[]);
        };
        if associated.truncated {
            return Err(RenderGpuAcceptedCaptureError::CapacityExceeded);
        }
        debug_assert!(associated.references.iter().all(|reference| {
            reference.receipt.submission_id() == submission.id()
                && reference.receipt.affinity() == submission.affinity()
        }));
        Ok(&associated.references)
    }

    /// Associate only canonical readback IDs of this exact accepted G5 graph.
    /// This does not produce GPU completion, duplicate bytes, or decide whether
    /// a producer's simulator has semantically advanced.
    pub(crate) fn associate_accepted_capture(
        &mut self,
        submission: &GpuSubmission,
        identity: RenderCaptureIdentity,
        readback_id: GpuReadbackId,
        width: u32,
        height: u32,
        source_format: GpuTextureFormat,
    ) {
        if submission.readback(readback_id).is_none() {
            return;
        }
        for (flow, stream) in &self.streams {
            if identity.flow_id() != flow.to_string()
                || !stream.accepted_submission().is_some_and(|accepted| {
                    accepted.id() == submission.id()
                        && accepted.affinity() == submission.affinity()
                })
            {
                continue;
            }
            let entry = self.accepted_capture_readbacks.entry(*flow).or_default();
            if entry.references.len() >= MAX_ACCEPTED_PRODUCER_READBACKS {
                entry.truncated = true;
                return;
            }
            entry.references.push(RenderGpuAcceptedCaptureReadback {
                identity,
                receipt: RenderCaptureSubmissionReceipt::accepted(submission, readback_id),
                width,
                height,
                source_format,
            });
            return;
        }
    }

    /// Called before surface acquisition. One accepted semantic producer
    /// interval forbids a second frame using its retained flow, even if its
    /// earlier GPU receipt is physically terminal but unverified. This is
    /// source-owned pacing, not a duplicate G5 resource reservation table.
    pub(crate) fn should_defer(&self, frame: &PreparedRenderFrame) -> bool {
        frame.flow_invocations.iter().any(|invocation| {
            self.streams
                .get(&invocation.flow_id)
                .is_some_and(|stream| stream.accepted_submission().is_some())
        })
    }

    /// Exact source/history comparison before composing the G5 submission.
    /// Reject stale prepared snapshots rather than falling back to latest ECS.
    pub(crate) fn validate_prepared(&self, frame: &PreparedRenderFrame) -> anyhow::Result<()> {
        let mut seen = BTreeSet::new();
        for invocation in &frame.flow_invocations {
            let Some(stream) = self.streams.get(&invocation.flow_id) else {
                anyhow::ensure!(
                    invocation.inputs.fixed_step_history.is_none(),
                    "prepared GPU history has no live semantic producer for flow {:?}",
                    invocation.flow_id
                );
                continue;
            };
            anyhow::ensure!(
                seen.insert(invocation.flow_id),
                "GPU step stream for flow {:?} is invoked more than once in a composed submission",
                invocation.flow_id
            );
            anyhow::ensure!(
                stream.accepted_submission().is_none(),
                "GPU step stream for flow {:?} still has unverified accepted work",
                invocation.flow_id
            );
            let history = invocation
                .inputs
                .fixed_step_history
                .as_ref()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "GPU step stream for flow {:?} lacks source-authored history",
                        invocation.flow_id
                    )
                })?;
            anyhow::ensure!(
                !history.steps.is_empty() || stream.pending_len() == 0,
                "prepared GPU step history predates new producer work for flow {:?}",
                invocation.flow_id
            );
            let pending = stream.candidate(history.steps.len());
            anyhow::ensure!(
                history.steps.len() == pending.len()
                    && history
                        .steps
                        .iter()
                        .zip(pending)
                        .all(|(prepared, original)| {
                            prepared.tick == original.tick() && prepared.input == *original.input()
                        }),
                "prepared GPU step history is stale for flow {:?}; fresh frame preparation required",
                invocation.flow_id
            );
        }
        Ok(())
    }

    /// Only called after validate_prepared and an actual G5 success, before
    /// any fallible retained-session association, observation or provenance.
    pub(crate) fn accepted(&mut self, frame: &PreparedRenderFrame, submission: &GpuSubmission) {
        for invocation in &frame.flow_invocations {
            let Some(stream) = self.streams.get_mut(&invocation.flow_id) else {
                continue;
            };
            let history = invocation
                .inputs
                .fixed_step_history
                .as_ref()
                .expect("prevalidated producer history");
            if !history.steps.is_empty() {
                self.accepted_capture_readbacks.remove(&invocation.flow_id);
                stream.accept_prevalidated(history.steps.len(), submission.clone());
            }
        }
    }
}
