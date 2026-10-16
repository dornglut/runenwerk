//! Bounded producer-authored semantic step demand across asynchronous GPU frames.
//!
//! This is a Runenwerk transport for *immutable producer input*, not a clock,
//! GPU resource reservation system, or a replacement for RunenGPU's G3/G5.
//! Only the producer can certify that a prepared GPU graph faithfully lowers
//! every captured step. An accepted submission is not a completed simulation.
use runen_gpu::{GpuContextAffinity, GpuSubmission, GpuSubmissionStatus};
use std::collections::VecDeque;

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
    EmptyBatch,
    BatchExceedsPending,
    PredecessorInFlight,
    NoAcceptedBatch,
    SubmissionPending,
    SubmissionFailed,
    DeviceGenerationChanged,
}

impl std::fmt::Display for RenderGpuStepDemandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RenderGpuStepDemandError {}

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

    /// Caller MUST invoke this only with the exact accepted submission whose
    /// graph includes all and only the selected candidate's semantic steps.
    /// No implicit G5 rejection retry and no graph/resource participation
    /// inference are performed here.
    pub fn accept(
        &mut self,
        submission: GpuSubmission,
        selected_steps: usize,
    ) -> Result<(), RenderGpuStepDemandError> {
        if self.accepted.is_some() {
            return Err(RenderGpuStepDemandError::PredecessorInFlight);
        }
        if selected_steps == 0 {
            return Err(RenderGpuStepDemandError::EmptyBatch);
        }
        if selected_steps > self.pending.len() {
            return Err(RenderGpuStepDemandError::BatchExceedsPending);
        }
        // A public handle can become terminally failed immediately after G5
        // accepts it. Even then its write may have partly executed: retain
        // the historical batch and require producer-owned reconstruction.
        let steps = self.pending.drain(..selected_steps).collect();
        self.accepted = Some(AcceptedSteps { submission, steps });
        Ok(())
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

    /// The semantic producer calls this only after confirming the exact
    /// accepted submission faithfully represented its captured step batch.
    /// On failed/unknown work or device change the whole batch remains held;
    /// the producer must reconstruct state and deliberately create a new
    /// demand stream instead of blindly replaying possibly executed writes.
    pub fn commit_completed(
        &mut self,
        current_affinity: GpuContextAffinity,
    ) -> Result<u64, RenderGpuStepDemandError> {
        let batch = self
            .accepted
            .as_ref()
            .ok_or(RenderGpuStepDemandError::NoAcceptedBatch)?;
        if batch.submission.affinity() != current_affinity {
            return Err(RenderGpuStepDemandError::DeviceGenerationChanged);
        }
        match batch.submission.status() {
            GpuSubmissionStatus::Accepted => {
                return Err(RenderGpuStepDemandError::SubmissionPending);
            }
            GpuSubmissionStatus::Failed(_) => {
                return Err(RenderGpuStepDemandError::SubmissionFailed);
            }
            GpuSubmissionStatus::Completed => {}
        }
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
