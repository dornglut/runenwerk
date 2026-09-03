use super::{GpuContextAffinity, GpuResourceRef};

/// Outstanding current-state reconstruction requirement for one retained logical storage resource.
///
/// This is lifecycle evidence, not another reconstruction-policy taxonomy. The durable owner
/// contract remains [`super::GpuReconstruction`] on the resource descriptor. Presence means the
/// resource's current-generation contents have not yet been re-established after loss/revocation.
/// It deliberately grants neither initialized coverage nor opaque content continuity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRetainedReconstructionRequirement {
    affinity: GpuContextAffinity,
    resource: GpuResourceRef,
    descriptor_initial_state_matches_required_contents: bool,
}

impl GpuRetainedReconstructionRequirement {
    pub(crate) fn new(
        affinity: GpuContextAffinity,
        resource: GpuResourceRef,
        descriptor_initial_state_matches_required_contents: bool,
    ) -> Self {
        Self {
            affinity,
            resource,
            descriptor_initial_state_matches_required_contents,
        }
    }

    /// Context/device generation in which this requirement must be satisfied.
    pub const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    /// Retained logical storage resource whose current required contents are unavailable.
    pub fn resource(&self) -> &GpuResourceRef {
        &self.resource
    }

    /// Whether accepted prior lifecycle history proves that the required contents at loss still
    /// matched the descriptor's initial-state request.
    ///
    /// `true` only says that descriptor-backed source/materialization may be a valid reconstruction
    /// source. It does not establish bytes, initialized coverage, or continuity in this generation;
    /// reconstruction must still complete through canonical work/materialization authority.
    pub const fn descriptor_initial_state_matches_required_contents(&self) -> bool {
        self.descriptor_initial_state_matches_required_contents
    }
}
