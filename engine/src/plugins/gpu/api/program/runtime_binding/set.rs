use super::{
    GpuRuntimeBindingDeviceFacts, GpuRuntimeBindingValue, GpuValidatedBindGroupBindings,
};
use crate::plugins::gpu::{
    GpuBindingClass, GpuPipelineLayoutDescriptor, GpuProgramContractCause, GpuProgramContractError,
};
use std::collections::BTreeMap;

/// Complete logical runtime binding use for one pipeline invocation.
///
/// This owner is pipeline-layout shaped. Per-group resource compatibility remains owned by
/// [`GpuValidatedBindGroupBindings`]; this type owns complete group coverage and the admitted
/// pipeline-wide dynamic-buffer counts. Dynamic offsets remain in the retained runtime values and
/// therefore stay per-use logical state rather than physical bind-group identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRuntimeBindingSet {
    layout: GpuPipelineLayoutDescriptor,
    groups: Vec<GpuValidatedBindGroupBindings>,
}

impl GpuRuntimeBindingSet {
    pub fn new(
        layout: GpuPipelineLayoutDescriptor,
        values: impl IntoIterator<Item = GpuRuntimeBindingValue>,
        device_facts: &GpuRuntimeBindingDeviceFacts,
    ) -> Result<Self, GpuProgramContractError> {
        validate_dynamic_binding_counts(&layout, device_facts)?;

        let mut values_by_group = BTreeMap::<u32, Vec<GpuRuntimeBindingValue>>::new();
        for value in values {
            values_by_group
                .entry(value.key().group())
                .or_default()
                .push(value);
        }

        let mut groups = Vec::with_capacity(layout.groups().len());
        for group in layout.groups() {
            let values = values_by_group.remove(&group.group()).unwrap_or_default();
            groups.push(GpuValidatedBindGroupBindings::new(
                group.clone(),
                values,
                device_facts,
            )?);
        }

        if let Some((&group, _)) = values_by_group.first_key_value() {
            return Err(incompatible(
                format!("group {group}"),
                "provide runtime values only for groups declared by the exact pipeline layout",
            ));
        }

        Ok(Self { layout, groups })
    }

    pub fn layout(&self) -> &GpuPipelineLayoutDescriptor {
        &self.layout
    }

    pub fn groups(&self) -> &[GpuValidatedBindGroupBindings] {
        &self.groups
    }

    pub fn group(&self, group: u32) -> Option<&GpuValidatedBindGroupBindings> {
        self.groups
            .binary_search_by_key(&group, |bindings| bindings.layout().group())
            .ok()
            .map(|index| &self.groups[index])
    }

    pub fn values(&self) -> impl Iterator<Item = &GpuRuntimeBindingValue> {
        self.groups.iter().flat_map(GpuValidatedBindGroupBindings::values)
    }
}

fn validate_dynamic_binding_counts(
    layout: &GpuPipelineLayoutDescriptor,
    device_facts: &GpuRuntimeBindingDeviceFacts,
) -> Result<(), GpuProgramContractError> {
    let mut dynamic_uniform_buffers = 0_u64;
    let mut dynamic_storage_buffers = 0_u64;

    for group in layout.groups() {
        for declaration in group.bindings() {
            if !declaration.kind().uses_dynamic_offset() {
                continue;
            }
            match declaration.kind().class() {
                GpuBindingClass::UniformBuffer => dynamic_uniform_buffers += 1,
                GpuBindingClass::StorageBuffer => dynamic_storage_buffers += 1,
                GpuBindingClass::SampledTexture
                | GpuBindingClass::StorageTexture
                | GpuBindingClass::Sampler => {
                    return Err(incompatible(
                        declaration.key().to_string(),
                        "use dynamic offsets only with uniform or storage buffer declarations",
                    ));
                }
            }
        }
    }

    if dynamic_uniform_buffers
        > u64::from(device_facts.max_dynamic_uniform_buffers_per_pipeline_layout())
    {
        return Err(incompatible(
            "dynamic uniform buffers",
            "reduce dynamic uniform-buffer declarations to the admitted pipeline-layout limit",
        ));
    }
    if dynamic_storage_buffers
        > u64::from(device_facts.max_dynamic_storage_buffers_per_pipeline_layout())
    {
        return Err(incompatible(
            "dynamic storage buffers",
            "reduce dynamic storage-buffer declarations to the admitted pipeline-layout limit",
        ));
    }
    Ok(())
}

fn incompatible(label: impl Into<String>, correction: &'static str) -> GpuProgramContractError {
    GpuProgramContractError::invalid(
        "construct runtime GPU binding set",
        label,
        GpuProgramContractCause::RuntimeBindingIncompatible,
        correction,
    )
}
