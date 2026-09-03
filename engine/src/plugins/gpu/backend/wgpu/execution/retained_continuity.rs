use super::WgpuExecutionState;
use crate::plugins::gpu::api::{
    initial_coverage_contains, initial_coverage_intersection, same_resource_descriptor,
};
use crate::plugins::gpu::{
    GpuContext, GpuContextAffinity, GpuContextDescriptor, GpuContextRequestError,
    GpuContextRequestErrorCategory, GpuExecutionPolicy, GpuInitialCoverage,
    GpuOpaqueContentContinuity, GpuPreparedWorkGraph, GpuRealizationPolicies, GpuReconstruction,
    GpuResourceLabel, GpuResourceRef, GpuRetainedInitializationSeed,
    GpuRetainedResourceContinuity, GpuSubmissionId, GpuSubmissionRejectionKind,
    GpuSubmissionRejectionReason, GpuWorkFragment, GpuWorkGraphError, GpuWorkResourceId,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

#[cfg(test)]
mod participation_tests;
#[cfg(test)]
mod storage_scope_tests;
#[cfg(test)]
mod terminal_boundary_tests;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(super) struct PreparedRetainedContinuity {
    resources: BTreeMap<GpuWorkResourceId, PreparedRetainedResource>,
}

#[derive(Debug, Clone)]
struct PreparedRetainedResource {
    resource: GpuResourceRef,
    consumed_lifecycle: bool,
    consumed_seed: Option<GpuInitialCoverage>,
    initial: Option<GpuInitialCoverage>,
    final_coverage: Option<GpuInitialCoverage>,
    failure_preserved_coverage: Option<GpuInitialCoverage>,
    reconstruction_target: bool,
    reconstruction_effect_proven: bool,
}

fn is_retained_storage_resource(resource: &GpuResourceRef) -> bool {
    resource.common().lifetime().is_retained()
        && matches!(
            resource,
            GpuResourceRef::Buffer(_) | GpuResourceRef::Texture(_) | GpuResourceRef::QuerySet(_)
        )
}

impl PreparedRetainedContinuity {
    pub(super) fn from_graph(graph: &GpuPreparedWorkGraph) -> Self {
        let participating = graph
            .nodes()
            .iter()
            .flat_map(|prepared| prepared.node().accesses().iter())
            .map(|access| access.resource_identity())
            .chain(
                graph
                    .initial_content()
                    .iter()
                    .map(|candidate| candidate.resource_identity()),
            )
            .collect::<BTreeSet<_>>();
        let writes = graph
            .nodes()
            .iter()
            .flat_map(|prepared| prepared.node().accesses().iter())
            .filter(|access| access.writes())
            .map(|access| access.resource_identity())
            .chain(
                graph
                    .initial_content()
                    .iter()
                    .map(|candidate| candidate.resource_identity()),
            )
            .collect::<BTreeSet<_>>();
        let resources = graph
            .initialization()
            .iter()
            .filter(|summary| {
                is_retained_storage_resource(summary.resource())
                    && participating.contains(&summary.resource().diagnostic_identity())
            })
            .map(|summary| {
                let resource = summary.resource().clone();
                let identity = resource.diagnostic_identity();
                let retained_seed = graph
                    .retained_seed()
                    .iter()
                    .find(|seed| seed.resource_identity() == identity);
                let consumed_lifecycle = retained_seed.is_some();
                let consumed_seed = retained_seed
                    .and_then(|seed| seed.initialized_coverage())
                    .cloned();
                let reconstruction_target = graph.reconstruction_targets().contains(&identity);
                (
                    identity,
                    PreparedRetainedResource {
                        resource,
                        consumed_lifecycle,
                        consumed_seed,
                        initial: summary.initial().cloned(),
                        final_coverage: summary.final_coverage().cloned(),
                        failure_preserved_coverage: graph
                            .failure_preserved_coverage(identity)
                            .cloned(),
                        reconstruction_target,
                        reconstruction_effect_proven: reconstruction_target
                            && (writes.contains(&identity)
                                || (!consumed_lifecycle && summary.initial().is_some())),
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
    write_pending: bool,
}

#[derive(Debug)]
pub(super) struct RetainedContinuityState {
    affinity: GpuContextAffinity,
    records: Mutex<BTreeMap<GpuWorkResourceId, RetainedContinuityRecord>>,
    reconstruction_required: Mutex<BTreeMap<GpuWorkResourceId, GpuResourceRef>>,
    reserved: Mutex<BTreeSet<GpuWorkResourceId>>,
}

impl RetainedContinuityState {
    pub(super) fn new(affinity: GpuContextAffinity) -> Self {
        Self::with_reconstruction_obligations(affinity, [])
    }

    pub(super) fn with_reconstruction_obligations(
        affinity: GpuContextAffinity,
        resources: impl IntoIterator<Item = GpuResourceRef>,
    ) -> Self {
        let reconstruction_required = resources
            .into_iter()
            .filter(is_retained_storage_resource)
            .map(|resource| (resource.diagnostic_identity(), resource))
            .collect();
        Self {
            affinity,
            records: Mutex::new(BTreeMap::new()),
            reconstruction_required: Mutex::new(reconstruction_required),
            reserved: Mutex::new(BTreeSet::new()),
        }
    }

    pub(super) fn reconstruction_seed(&self) -> Vec<GpuResourceRef> {
        let records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let obligations = self
            .reconstruction_required
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut resources = obligations.clone();
        for (&identity, record) in records.iter() {
            resources
                .entry(identity)
                .or_insert_with(|| record.resource.clone());
        }
        resources.into_values().collect()
    }

    pub(super) fn coverage_seed(&self) -> Vec<GpuRetainedInitializationSeed> {
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .map(|record| {
                GpuRetainedInitializationSeed::new(
                    record.resource.clone(),
                    record.initialized_coverage.clone(),
                )
            })
            .collect()
    }

    pub(super) fn snapshot(
        &self,
        resource: GpuWorkResourceId,
    ) -> Option<GpuRetainedResourceContinuity> {
        if let Some(record) = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&resource)
            .cloned()
        {
            return Some(GpuRetainedResourceContinuity::new(
                self.affinity,
                record.resource,
                record.initialized_coverage,
                if record.write_pending {
                    GpuOpaqueContentContinuity::Unknown
                } else {
                    record.opaque_content
                },
            ));
        }
        self.reconstruction_required
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&resource)
            .cloned()
            .map(|resource| {
                GpuRetainedResourceContinuity::new(
                    self.affinity,
                    resource,
                    None,
                    GpuOpaqueContentContinuity::Unknown,
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
        let reconstruction_required = self
            .reconstruction_required
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
                && !same_resource_descriptor(&current.resource, &prepared.resource)
            {
                return Err(continuity_changed(
                    identity,
                    "the retained resource descriptor changed for the same logical identity",
                ));
            }
            let reconstruction = reconstruction_required.get(&identity);
            if let Some(required) = reconstruction
                && !same_resource_descriptor(required, &prepared.resource)
            {
                return Err(continuity_changed(
                    identity,
                    "the reconstruction obligation names a different descriptor for this logical identity",
                ));
            }
            if reconstruction.is_some() && !prepared.reconstruction_target {
                return Err(reconstruction_required_reason(identity));
            }
            if prepared.reconstruction_target {
                if prepared.resource.common().reconstruction() == GpuReconstruction::NonReconstructable
                {
                    return Err(reconstruction_unavailable(identity));
                }
                if !prepared.reconstruction_effect_proven {
                    return Err(GpuSubmissionRejectionReason::new(
                        GpuSubmissionRejectionKind::RetainedReconstructionRequired,
                        format!(
                            "retained resource {identity}: declared reconstruction work has no accepted write or fresh-generation initialization effect establishing current contents"
                        ),
                    ));
                }
            }
            if current.is_some() != prepared.consumed_lifecycle {
                let detail = if current.is_some() {
                    "prepared work did not consume the current retained lifecycle state"
                } else {
                    "retained lifecycle state consumed during graph preparation is no longer current"
                };
                return Err(continuity_changed(identity, detail));
            }
            if let Some(seed) = &prepared.consumed_seed {
                let Some(current_coverage) =
                    current.and_then(|record| record.initialized_coverage.as_ref())
                else {
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
            if let Some(current_coverage) =
                current.and_then(|record| record.initialized_coverage.as_ref())
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

    pub(super) fn mark_may_execute(
        &self,
        transition: &PreparedRetainedContinuity,
        submitted_writes: &BTreeSet<GpuWorkResourceId>,
    ) {
        if submitted_writes.is_empty() {
            return;
        }

        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for &identity in submitted_writes {
            let Some(prepared) = transition.resources.get(&identity) else {
                continue;
            };
            let previous = records.get(&identity);
            let preserved = failure_safe_coverage(prepared);
            let previous_opaque = previous
                .map_or(GpuOpaqueContentContinuity::Unestablished, |record| {
                    record.opaque_content
                });
            records.insert(
                identity,
                RetainedContinuityRecord {
                    resource: prepared.resource.clone(),
                    initialized_coverage: preserved,
                    opaque_content: previous_opaque,
                    write_pending: true,
                },
            );
        }
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
        let mut reconstructed = BTreeSet::new();
        for (&identity, prepared) in &transition.resources {
            let previous_opaque = records
                .get(&identity)
                .map_or(GpuOpaqueContentContinuity::Unestablished, |record| {
                    record.opaque_content
                });
            let opaque_content = if prepared.reconstruction_target {
                reconstructed.insert(identity);
                GpuOpaqueContentContinuity::Established {
                    last_completed_write: submission,
                }
            } else if submitted_writes.contains(&identity) {
                match previous_opaque {
                    GpuOpaqueContentContinuity::Unknown => GpuOpaqueContentContinuity::Unknown,
                    GpuOpaqueContentContinuity::Unestablished
                    | GpuOpaqueContentContinuity::Established { .. } => {
                        GpuOpaqueContentContinuity::Established {
                            last_completed_write: submission,
                        }
                    }
                }
            } else {
                previous_opaque
            };
            records.insert(
                identity,
                RetainedContinuityRecord {
                    resource: prepared.resource.clone(),
                    initialized_coverage: prepared.final_coverage.clone(),
                    opaque_content,
                    write_pending: false,
                },
            );
        }
        drop(records);
        if !reconstructed.is_empty() {
            let mut obligations = self
                .reconstruction_required
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for identity in reconstructed {
                obligations.remove(&identity);
            }
        }
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
            let mut obligations = self
                .reconstruction_required
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for &identity in submitted_writes {
                let Some(prepared) = transition.resources.get(&identity) else {
                    continue;
                };
                let preserved = failure_safe_coverage(prepared);
                records.insert(
                    identity,
                    RetainedContinuityRecord {
                        resource: prepared.resource.clone(),
                        initialized_coverage: preserved,
                        opaque_content: GpuOpaqueContentContinuity::Unknown,
                        write_pending: false,
                    },
                );
                obligations.insert(identity, prepared.resource.clone());
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

impl WgpuExecutionState {
    pub(crate) fn new_with_reconstruction_obligations(
        affinity: GpuContextAffinity,
        policy: GpuExecutionPolicy,
        resources: impl IntoIterator<Item = GpuResourceRef>,
    ) -> Self {
        let mut state = Self::new(affinity, policy);
        state.retained = RetainedContinuityState::with_reconstruction_obligations(
            affinity,
            resources,
        );
        state
    }
}

impl GpuContext {
    /// Consumes this physical device generation and requests the next generation for the same
    /// process-local logical context identity.
    ///
    /// Realization/execution policies are preserved. The caller supplies a fresh explicit context
    /// descriptor so adapter/device admission remains policy owned by the caller. Surface state and
    /// physical realizations are not migrated. Retained resources that had lifecycle state become
    /// reconstruction obligations in the returned generation; their old initialized coverage and
    /// opaque continuity are never carried forward as current state.
    pub async fn replace_device_generation(
        self,
        descriptor: GpuContextDescriptor,
    ) -> Result<Self, GpuContextRequestError> {
        let id = self.id;
        let generation = self.generation.next().ok_or_else(|| {
            GpuContextRequestError::new(
                GpuContextRequestErrorCategory::IdentityExhausted,
                "device-generation identity space is exhausted for this logical context",
            )
        })?;
        let realization_policies = GpuRealizationPolicies::new(
            self.resource_realization_policy(),
            self.program_binding_realization_policy(),
        );
        let execution_policy = self.execution_policy();
        let reconstruction = self.backend.execution.retained.reconstruction_seed();
        drop(self);
        crate::plugins::gpu::backend::request_headless_generation(
            descriptor,
            realization_policies,
            execution_policy,
            id,
            generation,
            reconstruction,
        )
        .await
    }

    /// Prepares explicit current-state reconstruction work against this generation's canonical
    /// initialization authority.
    ///
    /// Targets remain ordinary retained resources in the one work graph. Successful completion may
    /// clear an outstanding reconstruction obligation only when the prepared graph proves a fresh
    /// initialization effect or contains a retained write for that target. The caller remains owner
    /// of the reconstruction recipe or external reimport data.
    pub fn prepare_reconstruction_work_graph(
        &self,
        label: GpuResourceLabel,
        fragments: impl IntoIterator<Item = GpuWorkFragment>,
        targets: impl IntoIterator<Item = GpuResourceRef>,
    ) -> Result<GpuPreparedWorkGraph, GpuWorkGraphError> {
        let retained = self.backend.execution.retained.coverage_seed();
        let targets = targets.into_iter().collect::<Vec<_>>();
        GpuPreparedWorkGraph::prepare_with_retained_coverage_and_reconstruction(
            label,
            fragments,
            &retained,
            &targets,
        )
    }
}

fn failure_safe_coverage(prepared: &PreparedRetainedResource) -> Option<GpuInitialCoverage> {
    let initial = prepared.initial.as_ref()?;
    let failure_preserved = prepared.failure_preserved_coverage.as_ref()?;
    initial_coverage_intersection(&prepared.resource, initial, failure_preserved)
}

fn continuity_changed(
    resource: GpuWorkResourceId,
    detail: &'static str,
) -> GpuSubmissionRejectionReason {
    GpuSubmissionRejectionReason::new(
        GpuSubmissionRejectionKind::RetainedContinuityChanged,
        format!(
            "retained resource {resource}: {detail}; prepare the work again against current continuity"
        ),
    )
}

fn reconstruction_required_reason(resource: GpuWorkResourceId) -> GpuSubmissionRejectionReason {
    GpuSubmissionRejectionReason::new(
        GpuSubmissionRejectionKind::RetainedReconstructionRequired,
        format!(
            "retained resource {resource}: current required contents belong to a lost or revoked state; submit explicit canonical reconstruction work before ordinary use"
        ),
    )
}

fn reconstruction_unavailable(resource: GpuWorkResourceId) -> GpuSubmissionRejectionReason {
    GpuSubmissionRejectionReason::new(
        GpuSubmissionRejectionKind::RetainedReconstructionUnavailable,
        format!(
            "retained resource {resource}: its NonReconstructable contract cannot certify recovery of lost current contents; allocate a new logical state identity for an explicit reset"
        ),
    )
}
