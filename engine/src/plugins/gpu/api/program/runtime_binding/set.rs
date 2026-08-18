use super::{
    GpuRuntimeBindingDeviceFacts, GpuRuntimeBindingResource, GpuRuntimeBindingValue,
    GpuValidatedBindGroupBindings,
};
use crate::plugins::gpu::{
    GpuBindingClass, GpuBindingDeclaration, GpuBufferAccess, GpuBufferAccessKind, GpuBufferRange,
    GpuPipelineLayoutDescriptor, GpuProgramContractCause, GpuProgramContractError,
    GpuResourceAccess, GpuSamplerUse, GpuStorageBufferAccess, GpuStorageTextureAccess,
    GpuTextureAccess, GpuTextureAccessKind, GpuTextureAccessResource,
};
use std::collections::BTreeMap;

/// Complete logical runtime binding use for one pipeline invocation.
///
/// This owner is pipeline-layout shaped. Per-group resource compatibility remains owned by
/// [`GpuValidatedBindGroupBindings`]; this type owns complete group coverage, admitted pipeline-wide
/// dynamic-buffer counts, and the exact G3 resource accesses after per-use dynamic offsets are
/// applied. Dynamic offsets remain in the retained runtime values and therefore stay per-use logical
/// state rather than physical bind-group identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRuntimeBindingSet {
    layout: GpuPipelineLayoutDescriptor,
    groups: Vec<GpuValidatedBindGroupBindings>,
    accesses: Vec<GpuResourceAccess>,
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

        let accesses = derive_effective_accesses(&groups)?;
        Ok(Self {
            layout,
            groups,
            accesses,
        })
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
        self.groups
            .iter()
            .flat_map(GpuValidatedBindGroupBindings::values)
    }

    pub fn accesses(&self) -> &[GpuResourceAccess] {
        &self.accesses
    }
}

fn derive_effective_accesses(
    groups: &[GpuValidatedBindGroupBindings],
) -> Result<Vec<GpuResourceAccess>, GpuProgramContractError> {
    let mut accesses = Vec::new();
    for group in groups {
        for declaration in group.layout().bindings() {
            let value = group
                .value(declaration.key().binding())
                .ok_or_else(|| effective_access_error(declaration))?;
            for resource in value.resources() {
                accesses.push(derive_effective_access(declaration, resource)?);
            }
        }
    }
    Ok(accesses)
}

fn derive_effective_access(
    declaration: &GpuBindingDeclaration,
    resource: &GpuRuntimeBindingResource,
) -> Result<GpuResourceAccess, GpuProgramContractError> {
    match (declaration.kind().class(), resource) {
        (GpuBindingClass::UniformBuffer, GpuRuntimeBindingResource::Buffer(binding)) => {
            derive_buffer_access(declaration, binding, GpuBufferAccessKind::UniformRead)
        }
        (GpuBindingClass::StorageBuffer, GpuRuntimeBindingResource::Buffer(binding)) => {
            let kind = match declaration.kind().storage_buffer_access() {
                Some(GpuStorageBufferAccess::ReadOnly) => GpuBufferAccessKind::StorageRead,
                Some(GpuStorageBufferAccess::ReadWrite) => GpuBufferAccessKind::StorageReadWrite,
                None => return Err(effective_access_error(declaration)),
            };
            derive_buffer_access(declaration, binding, kind)
        }
        (GpuBindingClass::SampledTexture, GpuRuntimeBindingResource::TextureView(binding)) => {
            derive_texture_access(declaration, binding, GpuTextureAccessKind::SampledRead)
        }
        (GpuBindingClass::StorageTexture, GpuRuntimeBindingResource::TextureView(binding)) => {
            let kind = match declaration.kind().storage_texture_access() {
                Some(GpuStorageTextureAccess::ReadOnly) => GpuTextureAccessKind::StorageRead,
                Some(GpuStorageTextureAccess::WriteOnly) => GpuTextureAccessKind::StorageWrite,
                Some(GpuStorageTextureAccess::ReadWrite) => GpuTextureAccessKind::StorageReadWrite,
                None => return Err(effective_access_error(declaration)),
            };
            derive_texture_access(declaration, binding, kind)
        }
        (GpuBindingClass::Sampler, GpuRuntimeBindingResource::Sampler(handle)) => {
            Ok(GpuResourceAccess::Sampler(GpuSamplerUse::new(handle)))
        }
        _ => Err(effective_access_error(declaration)),
    }
}

fn derive_buffer_access(
    declaration: &GpuBindingDeclaration,
    binding: &super::GpuRuntimeBufferBinding,
    kind: GpuBufferAccessKind,
) -> Result<GpuResourceAccess, GpuProgramContractError> {
    let effective_offset = binding
        .checked_effective_offset()
        .ok_or_else(|| effective_access_error(declaration))?;
    let range = GpuBufferRange::new(binding.handle(), effective_offset, binding.size().get())
        .map_err(|_| effective_access_error(declaration))?;
    let access = GpuBufferAccess::new(binding.handle(), range, kind)
        .map_err(|_| effective_access_error(declaration))?;
    Ok(GpuResourceAccess::Buffer(access))
}

fn derive_texture_access(
    declaration: &GpuBindingDeclaration,
    binding: &super::GpuRuntimeTextureViewBinding,
    kind: GpuTextureAccessKind,
) -> Result<GpuResourceAccess, GpuProgramContractError> {
    let handle = binding.handle();
    let access = GpuTextureAccess::new(
        GpuTextureAccessResource::TextureView(handle.clone()),
        handle.descriptor().subresources(),
        kind,
    )
    .map_err(|_| effective_access_error(declaration))?;
    Ok(GpuResourceAccess::Texture(access))
}

fn effective_access_error(declaration: &GpuBindingDeclaration) -> GpuProgramContractError {
    incompatible(
        declaration.key().to_string(),
        "keep effective resource access consistent with the validated runtime binding declaration",
    )
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
