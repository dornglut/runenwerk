use super::super::contract_diagnostics::{GpuProgramContractCause, GpuProgramContractError};
use super::super::{
    GpuBindingClass, GpuEntryPointName, GpuPipelineLayoutDescriptor, GpuProgramDescriptor,
    GpuShaderStage, GpuSpecializationValueSet,
};
use crate::plugins::gpu::{
    GpuCapabilityFeature, GpuCapabilityRequirement, GpuCapabilityRequirements,
};
use core::hash::Hash;
use std::sync::Arc;

#[derive(Debug)]
struct GpuComputePipelineDescriptorInner {
    program: GpuProgramDescriptor,
    entry_point: GpuEntryPointName,
    layout: GpuPipelineLayoutDescriptor,
    specialization: GpuSpecializationValueSet,
    requirements: GpuCapabilityRequirements,
}

/// Complete backend-neutral compute pipeline contract.
#[derive(Debug, Clone)]
pub struct GpuComputePipelineDescriptor(Arc<GpuComputePipelineDescriptorInner>);

impl GpuComputePipelineDescriptor {
    pub fn new(
        program: GpuProgramDescriptor,
        entry_point: GpuEntryPointName,
        layout: GpuPipelineLayoutDescriptor,
        specialization: GpuSpecializationValueSet,
        additional_requirements: GpuCapabilityRequirements,
    ) -> Result<Self, GpuProgramContractError> {
        if program
            .entry_point(GpuShaderStage::Compute, &entry_point)
            .is_none()
        {
            return Err(GpuProgramContractError::invalid(
                "construct GPU compute pipeline descriptor",
                entry_point.to_string(),
                GpuProgramContractCause::PipelineDescriptorInvalid,
                "select a compute entry point declared by the admitted program",
            ));
        }

        let expected_layout = GpuPipelineLayoutDescriptor::from_interface(program.interface())?;
        if layout != expected_layout {
            return Err(GpuProgramContractError::invalid(
                "construct GPU compute pipeline descriptor",
                entry_point.to_string(),
                GpuProgramContractCause::PipelineDescriptorInvalid,
                "use the pipeline layout derived from the admitted program interface",
            ));
        }

        let mut requirements = additional_requirements;
        insert_requirement(
            &mut requirements,
            GpuCapabilityRequirement::Required(GpuCapabilityFeature::Compute),
            &entry_point,
        )?;
        for requirement in specialization.requirements().iter() {
            insert_requirement(&mut requirements, requirement, &entry_point)?;
        }
        for binding in program.interface().bindings() {
            if binding.kind().class() == GpuBindingClass::StorageTexture {
                insert_requirement(
                    &mut requirements,
                    GpuCapabilityRequirement::Required(GpuCapabilityFeature::StorageTexture),
                    &entry_point,
                )?;
            }
        }

        Ok(Self(Arc::new(GpuComputePipelineDescriptorInner {
            program,
            entry_point,
            layout,
            specialization,
            requirements,
        })))
    }

    pub fn program(&self) -> &GpuProgramDescriptor {
        &self.0.program
    }

    pub fn entry_point(&self) -> &GpuEntryPointName {
        &self.0.entry_point
    }

    pub fn layout(&self) -> &GpuPipelineLayoutDescriptor {
        &self.0.layout
    }

    pub fn specialization(&self) -> &GpuSpecializationValueSet {
        &self.0.specialization
    }

    pub fn requirements(&self) -> &GpuCapabilityRequirements {
        &self.0.requirements
    }

    pub fn is_same_record(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl PartialEq for GpuComputePipelineDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.0.program == other.0.program
            && self.0.entry_point == other.0.entry_point
            && self.0.layout == other.0.layout
            && self.0.specialization == other.0.specialization
            && self.0.requirements == other.0.requirements
    }
}

impl Eq for GpuComputePipelineDescriptor {}

impl Hash for GpuComputePipelineDescriptor {
    fn hash<State: core::hash::Hasher>(&self, state: &mut State) {
        self.0.program.hash(state);
        self.0.entry_point.hash(state);
        self.0.layout.hash(state);
        self.0.specialization.hash(state);
        hash_requirements(&self.0.requirements, state);
    }
}

fn insert_requirement(
    requirements: &mut GpuCapabilityRequirements,
    requirement: GpuCapabilityRequirement,
    entry_point: &GpuEntryPointName,
) -> Result<(), GpuProgramContractError> {
    requirements.insert(requirement).map_err(|error| {
        GpuProgramContractError::invalid(
            "construct GPU compute pipeline descriptor",
            format!("{entry_point}: {error}"),
            GpuProgramContractCause::PipelineDescriptorInvalid,
            "remove conflicting capability requirements from the pipeline inputs",
        )
    })
}

fn hash_requirements<State: core::hash::Hasher>(
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
