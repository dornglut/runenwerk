use super::super::contract_diagnostics::{GpuProgramContractCause, GpuProgramContractError};
use crate::plugins::gpu::{GpuCapabilityRequirement, GpuCapabilityRequirements};
use core::hash::Hash;

pub(super) fn insert_pipeline_requirement(
    operation: &'static str,
    label: impl Into<String>,
    requirements: &mut GpuCapabilityRequirements,
    requirement: GpuCapabilityRequirement,
) -> Result<(), GpuProgramContractError> {
    requirements.insert(requirement).map_err(|error| {
        GpuProgramContractError::invalid(
            operation,
            format!("{}: {error}", label.into()),
            GpuProgramContractCause::PipelineDescriptorInvalid,
            "remove conflicting capability requirements from the pipeline inputs",
        )
    })
}

pub(super) fn hash_requirements<State: core::hash::Hasher>(
    requirements: &GpuCapabilityRequirements,
    state: &mut State,
) {
    requirements.iter().len().hash(state);
    for requirement in requirements.iter() {
        match requirement {
            GpuCapabilityRequirement::Required(feature) => {
                0u8.hash(state);
                feature.hash(state);
            }
            GpuCapabilityRequirement::Preferred { feature, fallback } => {
                1u8.hash(state);
                feature.hash(state);
                fallback.hash(state);
            }
            GpuCapabilityRequirement::Disabled(feature) => {
                2u8.hash(state);
                feature.hash(state);
            }
        }
    }
}
