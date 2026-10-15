//! Opt-in diagnostic of ordinary frame GPU submissions against existing G5 lifecycle handles.
//!
//! This uses only PUBLIC graph facts and intentionally is NOT a second retained-participation
//! or admission authority. Descriptor initial-content participation is partly GPU-private, so
//! these resource IDs are conservative trace context, never a correctness gate.
use runen_gpu::{
    GpuPreparedWorkGraph, GpuResourceRef, GpuSubmission, GpuSubmissionStatus, GpuWorkResourceId,
};
use std::collections::BTreeSet;

const TRACE_ENV: &str = "RUNENWERK_GPU_RETAINED_TRACE";

#[derive(Debug)]
struct AcceptedTrace {
    submission: GpuSubmission,
    resources: BTreeSet<GpuWorkResourceId>,
    frame: u64,
    surface: u64,
}

#[derive(Debug, Default)]
pub(in crate::plugins::render::renderer) struct RetainedSubmissionTrace {
    accepted: Vec<AcceptedTrace>,
}

impl RetainedSubmissionTrace {
    pub(super) fn before_submission(
        &mut self,
        graph: &GpuPreparedWorkGraph,
        frame: u64,
        surface: u64,
    ) -> Option<BTreeSet<GpuWorkResourceId>> {
        if std::env::var_os(TRACE_ENV).is_none() {
            return None;
        }
        let resources = observed_retained_accesses(graph);
        // Gfx has already invoked its one nonblocking RunenGPU progress point.
        // Record terminal transitions before retiring completed trace entries.
        self.accepted.retain(|prior| {
            let status = prior.submission.status();
            if status != GpuSubmissionStatus::Accepted {
                tracing::warn!(
                    target: "runenwerk.gpu.retained_trace",
                    prior_submission = %prior.submission.id(),
                    prior_frame = prior.frame,
                    prior_surface = prior.surface,
                    ?status,
                    "retained submission trace observed terminal transition"
                );
                false
            } else {
                true
            }
        });
        for prior in &self.accepted {
            let overlap = resources
                .intersection(&prior.resources)
                .copied()
                .collect::<Vec<_>>();
            tracing::warn!(
                target: "runenwerk.gpu.retained_trace",
                frame,
                surface,
                prior_submission = %prior.submission.id(),
                prior_frame = prior.frame,
                prior_surface = prior.surface,
                prior_status = ?prior.submission.status(),
                overlapping_observed_resources = ?overlap,
                "GPU frame about to prepare while another accepted submission is active"
            );
        }
        tracing::warn!(
            target: "runenwerk.gpu.retained_trace",
            frame,
            surface,
            observed_retained_resources = ?resources,
            active_predecessors = self.accepted.len(),
            "GPU frame preparing (observed public graph accesses, not G5 reservation proof)"
        );
        Some(resources)
    }

    pub(super) fn accepted(
        &mut self,
        submission: GpuSubmission,
        resources: Option<BTreeSet<GpuWorkResourceId>>,
        frame: u64,
        surface: u64,
    ) {
        let Some(resources) = resources else {
            return;
        };
        tracing::warn!(
            target: "runenwerk.gpu.retained_trace",
            frame,
            surface,
            submission = %submission.id(),
            ?resources,
            "GPU frame accepted"
        );
        // Diagnostic memory is bounded independently of execution. Eviction only
        // reduces tracing coverage; never authorizes a conflicting submission.
        if self.accepted.len() >= 64 {
            tracing::warn!(
                target: "runenwerk.gpu.retained_trace",
                "retained trace capacity exhausted; oldest diagnostic evicted"
            );
            self.accepted.remove(0);
        }
        self.accepted.push(AcceptedTrace {
            submission,
            resources,
            frame,
            surface,
        });
    }
}

fn observed_retained_accesses(graph: &GpuPreparedWorkGraph) -> BTreeSet<GpuWorkResourceId> {
    let accessed = graph
        .nodes()
        .iter()
        .flat_map(|node| node.node().accesses())
        .map(runen_gpu::GpuResourceAccess::resource_identity)
        .collect::<BTreeSet<_>>();
    graph
        .initialization()
        .iter()
        .filter_map(|summary| {
            let resource = summary.resource();
            if !resource.common().lifetime().is_retained()
                || !matches!(
                    resource,
                    GpuResourceRef::Buffer(_)
                        | GpuResourceRef::Texture(_)
                        | GpuResourceRef::QuerySet(_)
                )
            {
                return None;
            }
            let identity = resource.diagnostic_identity();
            accessed.contains(&identity).then_some(identity)
        })
        .collect()
}
