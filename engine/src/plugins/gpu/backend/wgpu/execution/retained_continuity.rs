use crate::plugins::gpu::api::{initial_coverage_contains, initial_coverage_intersection};
use crate::plugins::gpu::{
    GpuContextAffinity, GpuInitialCoverage, GpuOpaqueContentContinuity, GpuPreparedWorkGraph,
    GpuResourceRef, GpuRetainedResourceContinuity, GpuSubmissionId, GpuSubmissionRejectionKind,
    GpuSubmissionRejectionReason, GpuWorkResourceId,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub(super) struct PreparedRetainedContinuity {
    resources: BTreeMap<GpuWorkResourceId, PreparedRetainedResource>,
}

#[derive(Debug, Clone)]
struct PreparedRetainedResource {
    resource: GpuResourceRef,
    consumed_seed: Option<GpuInitialCoverage>,
    initial: Option<GpuInitialCoverage>,
    final_coverage: Option<GpuInitialCoverage>,
    failure_preserved_coverage: Option<GpuInitialCoverage>,
}

impl PreparedRetainedContinuity {
    pub(super) fn from_graph(graph: &GpuPreparedWorkGraph) -> Self {
        let resources = graph
            .initialization()
            .iter()
            .filter(|summary| summary.resource().common().lifetime().is_retained())
            .map(|summary| {
                let resource = summary.resource().clone();
                let identity = resource.diagnostic_identity();
                let consumed_seed = graph.retained_seed().iter().find(|seed| {
                    summary
                        .initial()
                        .is_some_and(|initial| initial_coverage_contains(initial, seed))
                });
                (
                    identity,
                    PreparedRetainedResource {
                        resource,
                        consumed_seed: consumed_seed.cloned(),
                        initial: summary.initial().cloned(),
                        final_coverage: summary.final_coverage().cloned(),
                        failure_preserved_coverage: summary.failure_preserved_coverage().cloned(),
                    },
                )
            })
            .collect();
        Self { resources }
    }

    pub(super) fn contains(&self, resource: GpuWorkResourceId) -> bool {
        self.resources.contains_key(&resource)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

#[derive(Debug, Clone)]
struct RetainedContinuityRecord {
    resource: GpuResourceRef,
    initialized_coverage: Option<GpuInitialCoverage>,
    opaque_content: GpuOpaqueContentContinuity,
}

#[derive(Debug)]
pub(super) struct RetainedContinuityState {
    affinity: GpuContextAffinity,
    records: Mutex<BTreeMap<GpuWorkResourceId, RetainedContinuityRecord>>,
    reserved: Mutex<BTreeSet<GpuWorkResourceId>>,
}

impl RetainedContinuityState {
    pub(super) fn new(affinity: GpuContextAffinity) -> Self {
        Self {
            affinity,
            records: Mutex::new(BTreeMap::new()),
            reserved: Mutex::new(BTreeSet::new()),
        }
    }

    pub(super) fn coverage_seed(&self) -> Vec<GpuInitialCoverage> {
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .filter_map(|record| record.initialized_coverage.clone())
            .collect()
    }

    pub(super) fn snapshot(
        &self,
        resource: GpuWorkResourceId,
    ) -> Option<GpuRetainedResourceContinuity> {
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&resource)
            .map(|record| {
                GpuRetainedResourceContinuity::new(
                    self.affinity,
                    record.resource.clone(),
                    record.initialized_coverage.clone(),
                    record.opaque_content,
                )
            })
    }

    pub(super) fn validate_and_reserve(
        &self,
        transition: &PreparedRetainedContinuity,
    ) -> Result<(), GpuSubmissionRejectionReason> {
        if transition.is_empty() {
            return Ok(());
        }

        let records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut reserved = self
            .reserved
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        for (&identity, prepared) in &transition.resources {
            if reserved.contains(&identity) {
                return Err(continuity_changed(
                    identity,
                    "another accepted submission still owns retained-state transition authority for this resource",
                ));
            }
            let current = records.get(&identity);
            if let Some(current) = current
                && current.resource != prepared.resource
            {
                return Err(continuity_changed(
                    identity,
                    "the retained resource descriptor changed for the same logical identity",
                ));
            }
            if let Some(seed) = &prepared.consumed_seed {
                let Some(current_coverage) = current.and_then(|record| record.initialized_coverage.as_ref()) else {
                    return Err(continuity_changed(
                        identity,
                        "retained coverage used during graph preparation is no longer established",
                    ));
                };
                if !initial_coverage_contains(current_coverage, seed) {
                    return Err(continuity_changed(
                        identity,
                        "retained coverage used during graph preparation is no longer contained by current continuity",
                    ));
                }
            }
            if let Some(current_coverage) = current.and_then(|record| record.initialized_coverage.as_ref())
                && !prepared
                    .initial
                    .as_ref()
                    .is_some_and(|initial| initial_coverage_contains(initial, current_coverage))
            {
                return Err(continuity_changed(
                    identity,
                    "current retained coverage contains state not represented by the prepared graph entry state",
                ));
            }
        }

        reserved.extend(transition.resources.keys().copied());
        Ok(())
    }

    pub(super) fn complete(
        &self,
        submission: GpuSubmissionId,
        transition: &PreparedRetainedContinuity,
        submitted_writes: &BTreeSet<GpuWorkResourceId>,
    ) {
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (&identity, prepared) in &transition.resources {
            let previous_opaque = records
                .get(&identity)
                .map_or(GpuOpaqueContentContinuity::Unestablished, |record| {
                    record.opaque_content
                });
            records.insert(
                identity,
                RetainedContinuityRecord {
                    resource: prepared.resource.clone(),
                    initialized_coverage: prepared.final_coverage.clone(),
                    opaque_content: if submitted_writes.contains(&identity) {
                        GpuOpaqueContentContinuity::Established {
                            last_completed_write: submission,
                        }
                    } else {
                        previous_opaque
                    },
                },
            );
        }
        drop(records);
        self.release(transition);
    }

    pub(super) fn fail_after_acceptance(
        &self,
        transition: &PreparedRetainedContinuity,
        submitted_writes: &BTreeSet<GpuWorkResourceId>,
    ) {
        if !submitted_writes.is_empty() {
            let mut records = self
                .records
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for &identity in submitted_writes {
                let Some(prepared) = transition.resources.get(&identity) else {
                    continue;
                };
                let previous = records.get(&identity);
                let preserved = match (
                    previous.and_then(|record| record.initialized_coverage.as_ref()),
                    prepared.failure_preserved_coverage.as_ref(),
                ) {
                    (Some(current), Some(failure_preserved)) => {
                        initial_coverage_intersection(&prepared.resource, current, failure_preserved)
                    }
                    _ => None,
                };
                records.insert(
                    identity,
                    RetainedContinuityRecord {
                        resource: prepared.resource.clone(),
                        initialized_coverage: preserved,
                        opaque_content: GpuOpaqueContentContinuity::Unknown,
                    },
                );
            }
        }
        self.release(transition);
    }

    fn release(&self, transition: &PreparedRetainedContinuity) {
        let mut reserved = self
            .reserved
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for identity in transition.resources.keys() {
            reserved.remove(identity);
        }
    }
}

fn continuity_changed(
    resource: GpuWorkResourceId,
    detail: &'static str,
) -> GpuSubmissionRejectionReason {
    GpuSubmissionRejectionReason::new(
        GpuSubmissionRejectionKind::RetainedContinuityChanged,
        format!("retained resource {resource}: {detail}; prepare the work again against current continuity"),
    )
}
