//! Renderer-owned lifecycle of accepted GPU frame submissions.
//!
//! RunenGPU alone owns retained-resource participation, admission, and physical
//! execution. This integration layer retains exact public submission handles
//! independently of optional timing, capture, or UI presentation observations.
//! It neither performs GPU progress nor attempts to reserve retained resources.
use runen_gpu::{
    GpuContext, GpuContextAffinity, GpuExecutionLifecycleState, GpuSubmission,
    GpuSubmissionFailure, GpuSubmissionId, GpuSubmissionStatus,
};
use std::collections::VecDeque;

#[derive(Debug)]
struct AcceptedFrameSubmission {
    submission: GpuSubmission,
    frame: u64,
    surface: u64,
}

// This is a bounded *observation delivery* buffer, not a G5 reservation table
// or a domain simulation cursor. Consumers MUST regard any reported gap as
// incomplete evidence and requalify their own state before continuing.
const MAX_TERMINAL_OUTCOMES: usize = 1024;

/// Terminal outcome of a previously accepted ordinary composed frame.
/// Completion establishes GPU execution, not domain-state publication,
/// reconstructed contents in a later generation, or visible presentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderGpuFrameTerminalDisposition {
    Completed,
    Failed(GpuSubmissionFailure),
    /// An accepted submission no longer belongs to the current GPU generation
    /// or the context closed before completion could be established.
    Invalidated,
}

#[derive(Debug, Clone)]
pub struct RenderGpuFrameTerminalOutcome {
    frame: u64,
    surface: u64,
    submission: GpuSubmissionId,
    affinity: GpuContextAffinity,
    disposition: RenderGpuFrameTerminalDisposition,
}

impl RenderGpuFrameTerminalOutcome {
    pub const fn frame(&self) -> u64 {
        self.frame
    }

    pub const fn surface(&self) -> u64 {
        self.surface
    }

    pub const fn submission_id(&self) -> GpuSubmissionId {
        self.submission
    }

    pub const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub fn disposition(&self) -> &RenderGpuFrameTerminalDisposition {
        &self.disposition
    }
}

/// Exactly-once drain of renderer-observed terminal outcomes.
/// Any lost entries invalidate completeness of this delivery stream.
/// RunenGPU's original submission handles remain the execution truth.
#[derive(Debug, Default)]
pub struct RenderGpuFrameTerminalBatch {
    outcomes: Vec<RenderGpuFrameTerminalOutcome>,
    lost_outcomes: u64,
}

impl RenderGpuFrameTerminalBatch {
    pub fn outcomes(&self) -> &[RenderGpuFrameTerminalOutcome] {
        &self.outcomes
    }

    pub const fn lost_outcomes(&self) -> u64 {
        self.lost_outcomes
    }

    pub fn is_complete(&self) -> bool {
        self.lost_outcomes == 0
    }
}

/// Correlates a renderer frame and surface with the exact public RunenGPU
/// status handle. This is not permission to retry a conflicting submission.
#[derive(Debug, Clone)]
pub struct RenderGpuFrameSubmissionReceipt {
    frame: u64,
    surface: u64,
    submission: GpuSubmission,
}

impl RenderGpuFrameSubmissionReceipt {
    pub const fn frame(&self) -> u64 {
        self.frame
    }

    pub const fn surface(&self) -> u64 {
        self.surface
    }

    pub fn submission(&self) -> &GpuSubmission {
        &self.submission
    }
}

#[derive(Debug, Default)]
pub(in crate::plugins::render::renderer) struct RendererSubmissionLifecycle {
    accepted: Vec<AcceptedFrameSubmission>,
    terminal: VecDeque<RenderGpuFrameTerminalOutcome>,
    lost_terminal: u64,
}

/// Oldest-first overflow is explicit and countable. It is never safe for a
/// semantic producer to advance a completed-step cursor across a delivery gap.
fn push_bounded<T>(queue: &mut VecDeque<T>, lost: &mut u64, limit: usize, item: T) {
    if queue.len() >= limit {
        queue.pop_front();
        *lost = lost.saturating_add(1);
    }
    queue.push_back(item);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubmissionDisposition {
    Pending,
    Completed,
    Failed,
    Invalidated,
}

fn classify_submission(
    current_affinity: bool,
    context_lifecycle: GpuExecutionLifecycleState,
    status: &GpuSubmissionStatus,
) -> SubmissionDisposition {
    // A terminal receipt remains a historical fact even when its originating
    // device generation is no longer current. It does not, by itself, establish
    // that the old retained contents are usable in the replacement generation.
    match status {
        GpuSubmissionStatus::Completed => SubmissionDisposition::Completed,
        GpuSubmissionStatus::Failed(_) => SubmissionDisposition::Failed,
        GpuSubmissionStatus::Accepted
            if !current_affinity || context_lifecycle == GpuExecutionLifecycleState::Closed =>
        {
            SubmissionDisposition::Invalidated
        }
        GpuSubmissionStatus::Accepted => SubmissionDisposition::Pending,
    }
}

impl RendererSubmissionLifecycle {
    pub(in crate::plugins::render::renderer) fn accept(
        &mut self,
        context: &GpuContext,
        submission: GpuSubmission,
        frame: u64,
        surface: u64,
    ) {
        // Reconcile public status only; the enclosing Gfx already owns the one
        // nonblocking GPU progress call. RunenGPU enforces physical capacity.
        self.reconcile(context);
        if self.accepted.len() >= context.execution_policy().max_in_flight_submissions().get() {
            tracing::warn!(
                target: "runenwerk.gpu.submission",
                tracked_submissions = self.accepted.len(),
                "renderer GPU submission tracking exceeds configured in-flight capacity"
            );
        }
        tracing::trace!(
            target: "runenwerk.gpu.submission",
            submission = %submission.id(),
            frame,
            surface,
            affinity = ?submission.affinity(),
            "GPU frame submission accepted; completion unproven"
        );
        self.accepted.push(AcceptedFrameSubmission {
            submission,
            frame,
            surface,
        });
    }

    pub(in crate::plugins::render::renderer) fn pending_receipts(
        &self,
    ) -> Vec<RenderGpuFrameSubmissionReceipt> {
        self.accepted
            .iter()
            .map(|entry| RenderGpuFrameSubmissionReceipt {
                frame: entry.frame,
                surface: entry.surface,
                submission: entry.submission.clone(),
            })
            .collect()
    }

    /// Drain terminal facts once, even if the next frame's rendering later
    /// fails. This method never advances the GPU or infers domain publication.
    pub(in crate::plugins::render::renderer) fn take_terminal(
        &mut self,
    ) -> RenderGpuFrameTerminalBatch {
        RenderGpuFrameTerminalBatch {
            outcomes: self.terminal.drain(..).collect(),
            lost_outcomes: std::mem::take(&mut self.lost_terminal),
        }
    }

    pub(in crate::plugins::render::renderer) fn reconcile(&mut self, context: &GpuContext) {
        let affinity = context.affinity();
        let lifecycle = context.execution_lifecycle_state();
        let terminal = &mut self.terminal;
        let lost = &mut self.lost_terminal;
        self.accepted.retain(|accepted| {
            let status = accepted.submission.status();
            let disposition = classify_submission(
                accepted.submission.affinity() == affinity,
                lifecycle,
                &status,
            );
            let outcome = match disposition {
                SubmissionDisposition::Pending => return true,
                SubmissionDisposition::Completed => {
                    tracing::trace!(
                        target: "runenwerk.gpu.submission",
                        submission = %accepted.submission.id(),
                        frame = accepted.frame,
                        surface = accepted.surface,
                        "GPU submission terminally completed"
                    );
                    RenderGpuFrameTerminalDisposition::Completed
                }
                SubmissionDisposition::Failed => {
                    let GpuSubmissionStatus::Failed(failure) = status else {
                        unreachable!("failed disposition requires typed failure")
                    };
                    tracing::error!(
                        target: "runenwerk.gpu.submission",
                        submission = %accepted.submission.id(),
                        frame = accepted.frame,
                        surface = accepted.surface,
                        kind = ?failure.kind(),
                        detail = failure.detail(),
                        "GPU submission terminally failed"
                    );
                    RenderGpuFrameTerminalDisposition::Failed(failure)
                }
                SubmissionDisposition::Invalidated => {
                    tracing::warn!(
                        target: "runenwerk.gpu.submission",
                        submission = %accepted.submission.id(),
                        frame = accepted.frame,
                        surface = accepted.surface,
                        old_affinity = ?accepted.submission.affinity(),
                        current_affinity = ?affinity,
                        ?lifecycle,
                        "GPU submission context closed or device generation changed"
                    );
                    RenderGpuFrameTerminalDisposition::Invalidated
                }
            };
            push_bounded(
                terminal,
                lost,
                MAX_TERMINAL_OUTCOMES,
                RenderGpuFrameTerminalOutcome {
                    frame: accepted.frame,
                    surface: accepted.surface,
                    submission: accepted.submission.id(),
                    affinity: accepted.submission.affinity(),
                    disposition: outcome,
                },
            );
            false
        });
        if *lost > 0 {
            tracing::error!(
                target: "runenwerk.gpu.submission",
                undelivered_terminal_outcomes = *lost,
                retained_terminal_capacity = MAX_TERMINAL_OUTCOMES,
                "GPU submission terminal outcome delivery overflow; consumer must not infer complete simulation continuity"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_delivery_bounds_memory_and_exposes_sequence_gaps() {
        let mut queue = VecDeque::new();
        let mut lost = 0;
        for ordinal in 0..7 {
            push_bounded(&mut queue, &mut lost, 3, ordinal);
        }
        assert_eq!(lost, 4);
        assert_eq!(queue.into_iter().collect::<Vec<_>>(), vec![4, 5, 6]);
    }

    #[test]
    fn terminal_disposition_never_conflates_accepted_with_completed() {
        for life in [
            GpuExecutionLifecycleState::Running,
            GpuExecutionLifecycleState::ShuttingDown,
        ] {
            assert_eq!(
                classify_submission(true, life, &GpuSubmissionStatus::Accepted),
                SubmissionDisposition::Pending,
            );
        }
        assert_eq!(
            classify_submission(
                false,
                GpuExecutionLifecycleState::Running,
                &GpuSubmissionStatus::Accepted,
            ),
            SubmissionDisposition::Invalidated,
        );
    }

    #[test]
    fn observation_free_accepted_work_remains_pending() {
        assert_eq!(
            classify_submission(
                true,
                GpuExecutionLifecycleState::Running,
                &GpuSubmissionStatus::Accepted,
            ),
            SubmissionDisposition::Pending,
        );
    }

    #[test]
    fn only_terminal_success_establishes_completion() {
        assert_eq!(
            classify_submission(
                true,
                GpuExecutionLifecycleState::Running,
                &GpuSubmissionStatus::Completed,
            ),
            SubmissionDisposition::Completed,
        );
    }

    #[test]
    fn changed_generation_or_closed_context_invalidates_pending_submission() {
        assert_eq!(
            classify_submission(
                false,
                GpuExecutionLifecycleState::Running,
                &GpuSubmissionStatus::Accepted,
            ),
            SubmissionDisposition::Invalidated,
        );
        assert_eq!(
            classify_submission(
                true,
                GpuExecutionLifecycleState::Closed,
                &GpuSubmissionStatus::Accepted,
            ),
            SubmissionDisposition::Invalidated,
        );
    }

    #[test]
    fn completed_receipt_remains_historical_success_after_generation_change_or_closure() {
        // Terminal evidence is not revoked by the next device generation.
        // Reconstructing contents for that generation remains a separate G5 obligation.
        assert_eq!(
            classify_submission(
                false,
                GpuExecutionLifecycleState::Running,
                &GpuSubmissionStatus::Completed,
            ),
            SubmissionDisposition::Completed,
        );
        assert_eq!(
            classify_submission(
                true,
                GpuExecutionLifecycleState::Closed,
                &GpuSubmissionStatus::Completed,
            ),
            SubmissionDisposition::Completed,
        );
    }

    #[test]
    fn graceful_shutdown_does_not_falsely_terminalize_still_accepted_work() {
        assert_eq!(
            classify_submission(
                true,
                GpuExecutionLifecycleState::ShuttingDown,
                &GpuSubmissionStatus::Accepted,
            ),
            SubmissionDisposition::Pending,
        );
    }
}
