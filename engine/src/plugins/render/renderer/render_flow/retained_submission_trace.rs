//! Opt-in diagnostic of ordinary frame GPU submissions against existing G5 lifecycle handles.
//!
//! This uses only PUBLIC graph facts and intentionally is NOT a second retained-participation
//! or admission authority. Descriptor initial-content participation is partly GPU-private, so
//! these resource IDs are conservative trace context, never a correctness gate.
use runen_gpu::{
    GpuPreparedWorkGraph, GpuResourceRef, GpuSubmission, GpuSubmissionRejectionReason,
    GpuSubmissionStatus, GpuWorkResourceId,
};
use std::collections::BTreeSet;

const TRACE_ENV: &str = "RUNENWERK_GPU_RETAINED_TRACE";

/// Publicly observed candidate identities, not GPU-owned reservation participants.
#[derive(Debug)]
pub(super) struct PublicRetainedObservations {
    accessed: BTreeSet<GpuWorkResourceId>,
    descriptor_only: BTreeSet<GpuWorkResourceId>,
}

impl PublicRetainedObservations {
    fn candidates(&self) -> BTreeSet<GpuWorkResourceId> {
        self.accessed
            .union(&self.descriptor_only)
            .copied()
            .collect()
    }
}

#[derive(Debug)]
struct AcceptedTrace {
    submission: GpuSubmission,
    resources: PublicRetainedObservations,
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
    ) -> Option<PublicRetainedObservations> {
        std::env::var_os(TRACE_ENV)?;
        let resources = observed_retained_candidates(graph);
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
        let current_candidates = resources.candidates();
        for prior in &self.accepted {
            let previous_candidates = prior.resources.candidates();
            let overlap = current_candidates
                .intersection(&previous_candidates)
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
                overlapping_public_retained_candidates = ?overlap,
                "GPU frame about to prepare while another accepted submission is active"
            );
        }
        tracing::warn!(
            target: "runenwerk.gpu.retained_trace",
            frame,
            surface,
            observed_retained_accesses = ?resources.accessed,
            observed_descriptor_only_candidates = ?resources.descriptor_only,
            active_predecessors = self.accepted.len(),
            "GPU frame preparing (public graph candidates, not G5 reservation proof)"
        );
        Some(resources)
    }

    pub(super) fn accepted(
        &mut self,
        submission: GpuSubmission,
        resources: Option<PublicRetainedObservations>,
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

    /// A G5 rejection is distinct from graph preparation failure and from
    /// terminal failure of a previously accepted submission.
    pub(super) fn rejected(
        &self,
        resources: Option<&PublicRetainedObservations>,
        frame: u64,
        surface: u64,
        reason: &GpuSubmissionRejectionReason,
    ) {
        let Some(resources) = resources else {
            return;
        };
        tracing::warn!(
            target: "runenwerk.gpu.retained_trace",
            frame,
            surface,
            kind = ?reason.kind(),
            detail = reason.detail(),
            public_retained_candidates = ?resources.candidates(),
            "RunenGPU rejected prepared frame; no automatic retry or presentation"
        );
    }
}

fn partition_public_candidates(
    accessed: BTreeSet<GpuWorkResourceId>,
    retained_descriptors: BTreeSet<GpuWorkResourceId>,
) -> PublicRetainedObservations {
    // Descriptor-only candidates must not be lost merely because the node
    // access list omits them. Conversely transient accesses are not retained.
    let retained_accesses = accessed
        .intersection(&retained_descriptors)
        .copied()
        .collect();
    let descriptor_only = retained_descriptors
        .difference(&accessed)
        .copied()
        .collect();
    PublicRetainedObservations {
        accessed: retained_accesses,
        descriptor_only,
    }
}

fn observed_retained_candidates(graph: &GpuPreparedWorkGraph) -> PublicRetainedObservations {
    let accessed = graph
        .nodes()
        .iter()
        .flat_map(|node| node.node().accesses())
        .map(runen_gpu::GpuResourceAccess::resource_identity)
        .collect::<BTreeSet<_>>();
    let retained_descriptors = graph
        .initialization()
        .iter()
        .filter_map(|summary| {
            let resource = summary.resource();
            (resource.common().lifetime().is_retained()
                && matches!(
                    resource,
                    GpuResourceRef::Buffer(_)
                        | GpuResourceRef::Texture(_)
                        | GpuResourceRef::QuerySet(_)
                ))
            .then_some(resource.diagnostic_identity())
        })
        .collect::<BTreeSet<_>>();
    partition_public_candidates(accessed, retained_descriptors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use runen_gpu::{
        GpuBufferDescriptor, GpuBufferInitialization, GpuBufferUsage, GpuReconstruction,
        GpuResourceLifetime, GpuWorkResourceIdAllocator,
    };

    #[test]
    fn trace_retains_descriptor_only_candidates_without_promoting_transient_accesses() {
        let mut allocator = GpuWorkResourceIdAllocator::new();
        let mut allocate = |name, lifetime| {
            allocator
                .allocate_buffer_handle(
                    GpuBufferDescriptor::ordinary_owned(
                        name,
                        lifetime,
                        GpuReconstruction::SourceBacked,
                        64,
                        [GpuBufferUsage::CopySource],
                        GpuBufferInitialization::Zeroed,
                    )
                    .expect("valid typed resource"),
                )
                .expect("unique identity")
                .diagnostic_identity()
        };
        let accessed_retained = allocate("trace.accessed", GpuResourceLifetime::Retained);
        let only_descriptor = allocate("trace.descriptor", GpuResourceLifetime::Retained);
        let transient = allocate("trace.transient", GpuResourceLifetime::Transient);
        let observations = partition_public_candidates(
            BTreeSet::from([accessed_retained, transient]),
            BTreeSet::from([accessed_retained, only_descriptor]),
        );
        assert_eq!(observations.accessed, BTreeSet::from([accessed_retained]));
        assert_eq!(
            observations.descriptor_only,
            BTreeSet::from([only_descriptor])
        );
        assert_eq!(
            observations.candidates(),
            BTreeSet::from([accessed_retained, only_descriptor])
        );
    }
}
