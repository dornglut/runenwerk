//! Backend-neutral context admission contracts.
//!
//! This module is the public namespace only.  Each admission concern has one
//! implementation owner below it; it is not a compatibility layer.

use super::{GpuRealizationPolicies, GpuRuntimeBindingDeviceFacts};
use core::num::NonZeroU64;

mod admission;
mod descriptor;
mod diagnostics;
mod facts;
mod identity;
mod selection;

pub use admission::{
    GpuAdmissionContract, GpuCandidateAdmissionReport, GpuDegradationRecord,
    GpuRejectedCandidateReport,
};
pub use descriptor::{
    GpuAdapterClass, GpuAlignmentKind, GpuBackendFamily, GpuContextDescriptor, GpuFormatRole,
    GpuLimitConstraint, GpuLimitKind, GpuPortabilityPolicy, GpuPowerPreference,
    GpuSoftwareFallbackPolicy,
};
pub use diagnostics::{GpuContextRequestError, GpuContextRequestErrorCategory};
pub use facts::{
    GpuAdapterFacts, GpuAdapterLimits, GpuAdmittedDeviceFacts, GpuAlignmentFacts, GpuDeviceLimits,
    GpuDeviceRequestProfile, GpuFallbackStatus, GpuPortabilityClass, GpuPortabilityEvidence,
    GpuPortabilityReason, GpuSoftwareStatus, GpuWorkloadBudget,
};
pub use identity::{
    GpuContextAffinity, GpuContextAffinityError, GpuContextId, GpuDeviceGeneration,
};
pub use selection::{
    GpuCandidateDisposition, GpuCandidateId, GpuCandidateRankEvidence,
    GpuCandidateSelectionEvidence, GpuCandidateSelectionKind, GpuContextAdmissionReport,
};

pub(crate) use admission::{
    GpuCandidateEnvironmentEvidence, admitted_device_facts, validate_descriptor,
};
pub(crate) use diagnostics::sanitized_diagnostic;
pub(crate) use identity::allocate_context_id;
#[cfg(test)]
pub(crate) use selection::select_candidate_with_host_evidence;
pub(crate) use selection::{
    GpuCandidateInput, GpuCandidateSelection, canonical_candidate_input_key,
    select_candidate_inputs,
};

#[derive(Debug)]
pub struct GpuContext {
    pub(crate) id: GpuContextId,
    pub(crate) generation: GpuDeviceGeneration,
    pub(crate) adapter: GpuAdapterFacts,
    pub(crate) device: GpuAdmittedDeviceFacts,
    pub(crate) report: GpuContextAdmissionReport,
    pub(crate) backend: crate::plugins::gpu::backend::WgpuContextState,
}

impl GpuContext {
    /// Requests an asynchronous, headless-first context admission.
    pub async fn request(descriptor: GpuContextDescriptor) -> Result<Self, GpuContextRequestError> {
        Self::request_with_realization_policies(descriptor, GpuRealizationPolicies::default()).await
    }

    /// Requests a context with explicit G4C1 and G4C2 realization-record bounds.
    ///
    /// Policies do not alter adapter selection, device admission, or retry identity.
    pub async fn request_with_realization_policies(
        descriptor: GpuContextDescriptor,
        realization_policies: GpuRealizationPolicies,
    ) -> Result<Self, GpuContextRequestError> {
        crate::plugins::gpu::backend::request_headless(descriptor, realization_policies).await
    }

    pub const fn id(&self) -> GpuContextId {
        self.id
    }

    pub const fn generation(&self) -> GpuDeviceGeneration {
        self.generation
    }

    pub const fn affinity(&self) -> GpuContextAffinity {
        GpuContextAffinity {
            context: self.id,
            generation: self.generation,
        }
    }

    pub fn adapter_facts(&self) -> &GpuAdapterFacts {
        &self.adapter
    }

    pub fn device_facts(&self) -> &GpuAdmittedDeviceFacts {
        &self.device
    }

    /// Projects the admitted device facts needed by backend-neutral runtime binding validation.
    ///
    /// This is derived entirely from accepted RunenGPU context facts. It does not expose private
    /// backend objects or re-query mutable backend state.
    pub fn runtime_binding_device_facts(&self) -> Option<GpuRuntimeBindingDeviceFacts> {
        let device_limits = self.device.device_limits();
        let alignments = device_limits.alignments();
        let uniform_buffer_offset_alignment = NonZeroU64::new(alignments.uniform_dynamic_offset?)?;
        let storage_buffer_offset_alignment = NonZeroU64::new(alignments.storage_dynamic_offset?)?;
        let limits = device_limits.values();
        Some(GpuRuntimeBindingDeviceFacts::new(
            uniform_buffer_offset_alignment,
            storage_buffer_offset_alignment,
            limits.max_dynamic_uniform_buffers_per_pipeline_layout(),
            limits.max_dynamic_storage_buffers_per_pipeline_layout(),
            self.adapter.supported().formats(),
        ))
    }

    pub fn admission_report(&self) -> &GpuContextAdmissionReport {
        &self.report
    }

    pub fn validate_affinity(
        &self,
        affinity: GpuContextAffinity,
    ) -> Result<(), GpuContextAffinityError> {
        identity::validate_affinity(self.affinity(), affinity)
    }
}
