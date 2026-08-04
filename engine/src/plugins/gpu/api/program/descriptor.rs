use super::contract_diagnostics::{GpuProgramContractCause, GpuProgramContractError};
use super::entry_point::{GpuEntryPointDescriptor, GpuEntryPointName};
use super::interface::{GpuBindingClass, GpuProgramInterfaceDescriptor, GpuShaderStage};
use super::requirement_identity::hash_capability_requirements;
use super::source::GpuAdmittedProgramSource;
use crate::plugins::gpu::{
    GpuCapabilityFeature, GpuCapabilityRequirement, GpuCapabilityRequirements,
};
use core::hash::Hash;
use std::sync::Arc;

#[derive(Debug)]
struct GpuProgramDescriptorInner {
    source: GpuAdmittedProgramSource,
    interface: GpuProgramInterfaceDescriptor,
    entry_points: Vec<GpuEntryPointDescriptor>,
    requirements: GpuCapabilityRequirements,
}

/// Immutable admitted program contract retained by later realization records.
#[derive(Debug, Clone)]
pub struct GpuProgramDescriptor(Arc<GpuProgramDescriptorInner>);

impl GpuProgramDescriptor {
    pub fn new(
        source: GpuAdmittedProgramSource,
        interface: GpuProgramInterfaceDescriptor,
        entry_points: impl IntoIterator<Item = GpuEntryPointDescriptor>,
    ) -> Result<Self, GpuProgramContractError> {
        let mut entry_points = entry_points.into_iter().collect::<Vec<_>>();
        if entry_points.is_empty() {
            return Err(GpuProgramContractError::invalid(
                "construct admitted GPU program",
                source.identity().diagnostic_label(),
                GpuProgramContractCause::EntryPointMissing,
                "declare at least one typed entry point",
            ));
        }

        entry_points.sort_by(|left, right| {
            left.stage()
                .cmp(&right.stage())
                .then_with(|| left.name().cmp(right.name()))
        });

        if let Some(duplicate) = entry_points
            .windows(2)
            .find(|pair| pair[0].stage() == pair[1].stage() && pair[0].name() == pair[1].name())
        {
            return Err(GpuProgramContractError::invalid(
                "construct admitted GPU program",
                format!("{:?}:{}", duplicate[0].stage(), duplicate[0].name()),
                GpuProgramContractCause::DuplicateEntryPoint,
                "declare each stage and entry-point name pair exactly once",
            ));
        }

        if let Some(mismatch) = entry_points
            .iter()
            .find(|entry_point| entry_point.interface() != &interface)
        {
            return Err(GpuProgramContractError::invalid(
                "construct admitted GPU program",
                format!("{:?}:{}", mismatch.stage(), mismatch.name()),
                GpuProgramContractCause::ProgramInterfaceMismatch,
                "bind every entry point to the program's one explicit resource interface",
            ));
        }

        for binding in interface.bindings() {
            for visible_stage in binding.visibility().iter() {
                if !entry_points
                    .iter()
                    .any(|entry_point| entry_point.stage() == visible_stage)
                {
                    return Err(GpuProgramContractError::invalid(
                        "construct admitted GPU program",
                        format!("binding {} visible to {visible_stage:?}", binding.key()),
                        GpuProgramContractCause::ProgramInterfaceMismatch,
                        "declare an entry point for every shader stage named by binding visibility",
                    ));
                }
            }
        }

        let mut requirements = GpuCapabilityRequirements::new();
        for binding in interface.bindings() {
            if binding.kind().class() == GpuBindingClass::StorageTexture {
                requirements
                    .insert(GpuCapabilityRequirement::Required(
                        GpuCapabilityFeature::StorageTexture,
                    ))
                    .map_err(|error| {
                        GpuProgramContractError::invalid(
                            "construct admitted GPU program",
                            format!("{}: {error}", source.identity().diagnostic_label()),
                            GpuProgramContractCause::ProgramInterfaceMismatch,
                            "remove conflicting capability requirements implied by the program interface",
                        )
                    })?;
            }
        }

        Ok(Self(Arc::new(GpuProgramDescriptorInner {
            source,
            interface,
            entry_points,
            requirements,
        })))
    }

    pub fn source(&self) -> &GpuAdmittedProgramSource {
        &self.0.source
    }

    pub fn interface(&self) -> &GpuProgramInterfaceDescriptor {
        &self.0.interface
    }

    pub fn entry_points(&self) -> impl ExactSizeIterator<Item = &GpuEntryPointDescriptor> {
        self.0.entry_points.iter()
    }

    pub fn entry_point(
        &self,
        stage: GpuShaderStage,
        name: &GpuEntryPointName,
    ) -> Option<&GpuEntryPointDescriptor> {
        self.0
            .entry_points
            .binary_search_by(|entry_point| {
                entry_point
                    .stage()
                    .cmp(&stage)
                    .then_with(|| entry_point.name().cmp(name))
            })
            .ok()
            .map(|index| &self.0.entry_points[index])
    }

    pub fn requirements(&self) -> &GpuCapabilityRequirements {
        &self.0.requirements
    }

    pub fn is_same_record(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl PartialEq for GpuProgramDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.0.source == other.0.source
            && self.0.interface == other.0.interface
            && self.0.entry_points == other.0.entry_points
            && self.0.requirements == other.0.requirements
    }
}

impl Eq for GpuProgramDescriptor {}

impl Hash for GpuProgramDescriptor {
    fn hash<State: core::hash::Hasher>(&self, state: &mut State) {
        self.0.source.hash(state);
        self.0.interface.hash(state);
        self.0.entry_points.hash(state);
        hash_capability_requirements(&self.0.requirements, state);
    }
}
