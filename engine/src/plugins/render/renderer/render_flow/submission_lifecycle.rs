//! Renderer-owned lifecycle of accepted GPU frame submissions.
//!
//! RunenGPU alone owns retained-resource participation, admission, and physical
//! execution. This integration layer retains exact public submission handles
//! independently of optional timing, capture, or UI presentation observations.
//! It neither performs GPU progress nor attempts to reserve retained resources.
use runen_gpu::{GpuContext, GpuExecutionLifecycleState, GpuSubmission, GpuSubmissionStatus};

#[derive(Debug)]
struct AcceptedFrameSubmission {
    submission: GpuSubmission,
    frame: u64,
    surface: u64,
}

#[derive(Debug, Default)]
pub(in crate::plugins::render::renderer) struct RendererSubmissionLifecycle {
    accepted: Vec<AcceptedFrameSubmission>,
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
    if !current_affinity || context_lifecycle == GpuExecutionLifecycleState::Closed {
        return SubmissionDisposition::Invalidated;
    }
    match status {
        GpuSubmissionStatus::Accepted => SubmissionDisposition::Pending,
        GpuSubmissionStatus::Completed => SubmissionDisposition::Completed,
        GpuSubmissionStatus::Failed(_) => SubmissionDisposition::Failed,
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

    pub(in crate::plugins::render::renderer) fn reconcile(&mut self, context: &GpuContext) {
        let affinity = context.affinity();
        let lifecycle = context.execution_lifecycle_state();
        self.accepted.retain(|accepted| {
            let status = accepted.submission.status();
            match classify_submission(
                accepted.submission.affinity() == affinity,
                lifecycle,
                &status,
            ) {
                SubmissionDisposition::Pending => true,
                SubmissionDisposition::Completed => {
                    tracing::trace!(
                        target: "runenwerk.gpu.submission",
                        submission = %accepted.submission.id(),
                        frame = accepted.frame,
                        surface = accepted.surface,
                        "GPU submission terminally completed"
                    );
                    false
                }
                SubmissionDisposition::Failed => {
                    if let GpuSubmissionStatus::Failed(failure) = status {
                        tracing::error!(
                            target: "runenwerk.gpu.submission",
                            submission = %accepted.submission.id(),
                            frame = accepted.frame,
                            surface = accepted.surface,
                            kind = ?failure.kind(),
                            detail = failure.detail(),
                            "GPU submission terminally failed"
                        );
                    }
                    false
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
                    false
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
