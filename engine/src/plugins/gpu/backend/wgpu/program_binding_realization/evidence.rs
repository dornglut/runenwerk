//! The one private direct-Naga evidence path for G4C2.
//!
//! Naga IR is parsed, validated, normalized, and dropped in this module. The original admitted
//! canonical WGSL remains the sole text sent to WGPU by the realization owner.

use crate::plugins::gpu::{
    GpuBindingKey, GpuEntryPointName, GpuFragmentOutputBuiltin, GpuObservedBindingDeclaration,
    GpuObservedBindingKind, GpuObservedFragmentOutputSignature, GpuObservedProgramInterface,
    GpuObservedSamplerClass, GpuObservedShaderStages, GpuObservedTextureSampleClass,
    GpuObservedVertexInputSignature, GpuProgramBindingRealizationError,
    GpuProgramBindingRealizationErrorCategory, GpuProgramDescriptor, GpuShaderIoLocation,
    GpuShaderIoScalarClass, GpuShaderIoValueType, GpuShaderStage, GpuStorageBufferAccess,
    GpuStorageTextureAccess, GpuTextureFormat, GpuTextureViewDimension, GpuVertexInputBuiltin,
    compare_program_interfaces,
};
use core::num::NonZeroU32;
use naga::{
    AddressSpace, ArraySize, Binding, BuiltIn, ImageClass, ImageDimension, ScalarKind, ShaderStage,
    StorageAccess, StorageFormat, TypeInner, VectorSize,
};

pub(super) const G4C2_NAGA_VALIDATION_PROFILE_REVISION: u32 = 2;
pub(super) const G4C2_WGPU_REALIZATION_COMPATIBILITY_REVISION: u32 = 2;

#[derive(Debug)]
pub(super) struct ProgramEvidence {
    pub(super) observed_interface: GpuObservedProgramInterface,
    pub(super) vertex_inputs: Vec<GpuObservedVertexInputSignature>,
    pub(super) fragment_outputs: Vec<GpuObservedFragmentOutputSignature>,
}

pub(super) fn validate_and_normalize(
    descriptor: &GpuProgramDescriptor,
) -> Result<ProgramEvidence, GpuProgramBindingRealizationError> {
    let source = descriptor.source().canonical_wgsl();
    let module = naga::front::wgsl::parse_str(source).map_err(|error| {
        failure(
            GpuProgramBindingRealizationErrorCategory::WgslParseOrValidationFailed,
            descriptor,
            format!("Naga WGSL parse failed: {error}"),
        )
    })?;
    let module_info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::default(),
    )
    .validate(&module)
    .map_err(|error| {
        failure(
            GpuProgramBindingRealizationErrorCategory::WgslParseOrValidationFailed,
            descriptor,
            format!("Naga WGSL validation failed: {error}"),
        )
    })?;

    let selected_entry_points = descriptor.entry_points().collect::<Vec<_>>();
    let mut selected_indices = Vec::with_capacity(selected_entry_points.len());
    for declared in &selected_entry_points {
        let stage = naga_stage(declared.stage());
        let index = module
            .entry_points
            .iter()
            .position(|entry| entry.stage == stage && entry.name == declared.name().as_str())
            .ok_or_else(|| {
                failure(
                    GpuProgramBindingRealizationErrorCategory::ProgramInterfaceMismatch,
                    descriptor,
                    format!(
                        "canonical WGSL has no declared {:?} entry point '{}'",
                        declared.stage(),
                        declared.name()
                    ),
                )
            })?;
        selected_indices.push((declared.stage(), index, declared.name().clone()));
    }

    let mut observed_bindings = Vec::new();
    for (global_handle, global) in module.global_variables.iter() {
        let Some(binding) = global.binding else {
            continue;
        };
        let key = GpuBindingKey::try_new(binding.group as u64, binding.binding as u64).map_err(
            |error| {
                failure(
                    GpuProgramBindingRealizationErrorCategory::ProgramInterfaceMismatch,
                    descriptor,
                    error.to_string(),
                )
            },
        )?;
        let (base_type, array_count) =
            binding_array_type(&module, global.ty).map_err(|detail| {
                failure(
                    GpuProgramBindingRealizationErrorCategory::ProgramInterfaceMismatch,
                    descriptor,
                    format!("binding {key}: {detail}"),
                )
            })?;
        let kind = observed_binding_kind(&module, global.space, base_type).map_err(|detail| {
            failure(
                GpuProgramBindingRealizationErrorCategory::ProgramInterfaceMismatch,
                descriptor,
                format!("binding {key}: {detail}"),
            )
        })?;
        let used_stages = selected_indices
            .iter()
            .filter_map(|(stage, index, _)| {
                (!module_info.get_entry_point(*index)[global_handle].is_empty()).then_some(*stage)
            })
            .collect::<Vec<_>>();
        observed_bindings.push(GpuObservedBindingDeclaration::new(
            key,
            kind,
            array_count,
            GpuObservedShaderStages::new(used_stages),
        ));
    }
    let observed_interface =
        GpuObservedProgramInterface::new(observed_bindings).map_err(|error| {
            failure(
                GpuProgramBindingRealizationErrorCategory::ProgramInterfaceMismatch,
                descriptor,
                error.to_string(),
            )
        })?;
    compare_program_interfaces(descriptor.interface(), &observed_interface).map_err(|error| {
        failure(
            GpuProgramBindingRealizationErrorCategory::ProgramInterfaceMismatch,
            descriptor,
            error.to_string(),
        )
    })?;

    let mut vertex_inputs = Vec::new();
    let mut fragment_outputs = Vec::new();
    for (stage, index, name) in selected_indices {
        let entry = &module.entry_points[index];
        match stage {
            GpuShaderStage::Vertex => vertex_inputs.push(normalize_vertex_input(
                &module,
                &entry.function,
                name,
                descriptor,
            )?),
            GpuShaderStage::Fragment => fragment_outputs.push(normalize_fragment_output(
                &module,
                &entry.function,
                name,
                descriptor,
            )?),
            GpuShaderStage::Compute => {}
        }
    }

    Ok(ProgramEvidence {
        observed_interface,
        vertex_inputs,
        fragment_outputs,
    })
}

fn binding_array_type(
    module: &naga::Module,
    ty: naga::Handle<naga::Type>,
) -> Result<(naga::Handle<naga::Type>, Option<NonZeroU32>), &'static str> {
    match module.types[ty].inner {
        TypeInner::BindingArray { base, size } => match size {
            ArraySize::Constant(count) => Ok((base, Some(count))),
            ArraySize::Pending(_) | ArraySize::Dynamic => {
                Err("binding arrays require one fixed nonzero cardinality")
            }
        },
        _ => Ok((ty, None)),
    }
}

fn observed_binding_kind(
    module: &naga::Module,
    space: AddressSpace,
    ty: naga::Handle<naga::Type>,
) -> Result<GpuObservedBindingKind, &'static str> {
    match space {
        AddressSpace::Uniform => Ok(GpuObservedBindingKind::uniform_buffer(None)),
        AddressSpace::Storage { access } => Ok(GpuObservedBindingKind::storage_buffer(
            if access.intersects(StorageAccess::STORE | StorageAccess::ATOMIC) {
                GpuStorageBufferAccess::ReadWrite
            } else {
                GpuStorageBufferAccess::ReadOnly
            },
            None,
        )),
        AddressSpace::Handle => match module.types[ty].inner {
            TypeInner::Sampler { comparison } => {
                Ok(GpuObservedBindingKind::sampler(if comparison {
                    GpuObservedSamplerClass::Comparison
                } else {
                    GpuObservedSamplerClass::NonComparison
                }))
            }
            TypeInner::Image {
                dim,
                arrayed,
                class,
            } => match class {
                ImageClass::Sampled { kind, multi } => GpuObservedBindingKind::sampled_texture(
                    observed_sample_class(kind)?,
                    texture_view_dimension(dim, arrayed)?,
                    multi,
                )
                .map_err(|_| "sampled texture evidence is structurally invalid"),
                ImageClass::Depth { multi } => GpuObservedBindingKind::sampled_texture(
                    GpuObservedTextureSampleClass::Depth,
                    texture_view_dimension(dim, arrayed)?,
                    multi,
                )
                .map_err(|_| "depth texture evidence is structurally invalid"),
                ImageClass::Storage { format, access } => GpuObservedBindingKind::storage_texture(
                    storage_texture_access(access)?,
                    storage_texture_format(format)?,
                    texture_view_dimension(dim, arrayed)?,
                )
                .map_err(|_| "storage texture evidence is structurally invalid"),
                ImageClass::External => {
                    Err("external textures are outside the admitted G4C2 binding vocabulary")
                }
            },
            _ => Err("handle-space binding has an unsupported WGSL resource type"),
        },
        _ => Err("only uniform, storage, sampler, and texture globals are admitted resources"),
    }
}

fn observed_sample_class(kind: ScalarKind) -> Result<GpuObservedTextureSampleClass, &'static str> {
    match kind {
        ScalarKind::Float => Ok(GpuObservedTextureSampleClass::Float),
        ScalarKind::Sint => Ok(GpuObservedTextureSampleClass::Sint),
        ScalarKind::Uint => Ok(GpuObservedTextureSampleClass::Uint),
        ScalarKind::Bool | ScalarKind::AbstractInt | ScalarKind::AbstractFloat => {
            Err("sampled texture has an unsupported scalar class")
        }
    }
}

fn texture_view_dimension(
    dimension: ImageDimension,
    arrayed: bool,
) -> Result<GpuTextureViewDimension, &'static str> {
    match (dimension, arrayed) {
        (ImageDimension::D1, false) => Ok(GpuTextureViewDimension::D1),
        (ImageDimension::D2, false) => Ok(GpuTextureViewDimension::D2),
        (ImageDimension::D2, true) => Ok(GpuTextureViewDimension::D2Array),
        (ImageDimension::D3, false) => Ok(GpuTextureViewDimension::D3),
        (ImageDimension::Cube, false) => Ok(GpuTextureViewDimension::Cube),
        (ImageDimension::Cube, true) => Ok(GpuTextureViewDimension::CubeArray),
        _ => Err("WGSL image dimension/array combination is outside G4C2 vocabulary"),
    }
}

fn storage_texture_access(access: StorageAccess) -> Result<GpuStorageTextureAccess, &'static str> {
    match (
        access.contains(StorageAccess::LOAD),
        access.intersects(StorageAccess::STORE | StorageAccess::ATOMIC),
    ) {
        (true, false) => Ok(GpuStorageTextureAccess::ReadOnly),
        (false, true) => Ok(GpuStorageTextureAccess::WriteOnly),
        (true, true) => Ok(GpuStorageTextureAccess::ReadWrite),
        (false, false) => Err("storage texture declares no load/store access"),
    }
}

fn storage_texture_format(format: StorageFormat) -> Result<GpuTextureFormat, &'static str> {
    match format {
        StorageFormat::R8Unorm => Ok(GpuTextureFormat::R8Unorm),
        StorageFormat::Rgba8Unorm => Ok(GpuTextureFormat::Rgba8Unorm),
        StorageFormat::Bgra8Unorm => Ok(GpuTextureFormat::Bgra8Unorm),
        StorageFormat::R32Uint => Ok(GpuTextureFormat::R32Uint),
        _ => Err("storage texture format is outside the accepted RunenGPU format vocabulary"),
    }
}

fn normalize_vertex_input(
    module: &naga::Module,
    function: &naga::Function,
    entry_point: GpuEntryPointName,
    descriptor: &GpuProgramDescriptor,
) -> Result<GpuObservedVertexInputSignature, GpuProgramBindingRealizationError> {
    let mut locations = Vec::new();
    let mut builtins = Vec::new();
    for argument in &function.arguments {
        collect_io(
            module,
            argument.ty,
            argument.binding.as_ref(),
            &mut locations,
            &mut |builtin| match builtin {
                BuiltIn::VertexIndex => Ok(GpuVertexInputBuiltin::VertexIndex),
                BuiltIn::InstanceIndex => Ok(GpuVertexInputBuiltin::InstanceIndex),
                _ => Err("vertex input uses an unsupported builtin"),
            },
            &mut builtins,
        )
        .map_err(|detail| observed_io_failure(descriptor, detail))?;
    }
    GpuObservedVertexInputSignature::new(entry_point, locations, builtins)
        .map_err(|error| observed_io_failure(descriptor, error.to_string()))
}

fn normalize_fragment_output(
    module: &naga::Module,
    function: &naga::Function,
    entry_point: GpuEntryPointName,
    descriptor: &GpuProgramDescriptor,
) -> Result<GpuObservedFragmentOutputSignature, GpuProgramBindingRealizationError> {
    let mut locations = Vec::new();
    let mut builtins = Vec::new();
    if let Some(result) = &function.result {
        collect_io(
            module,
            result.ty,
            result.binding.as_ref(),
            &mut locations,
            &mut |builtin| match builtin {
                BuiltIn::FragDepth => Ok(GpuFragmentOutputBuiltin::FragDepth),
                BuiltIn::SampleMask => Ok(GpuFragmentOutputBuiltin::SampleMask),
                _ => Err("fragment output uses an unsupported builtin"),
            },
            &mut builtins,
        )
        .map_err(|detail| observed_io_failure(descriptor, detail))?;
    }
    GpuObservedFragmentOutputSignature::new(entry_point, locations, builtins)
        .map_err(|error| observed_io_failure(descriptor, error.to_string()))
}

fn collect_io<B>(
    module: &naga::Module,
    ty: naga::Handle<naga::Type>,
    binding: Option<&Binding>,
    locations: &mut Vec<GpuShaderIoLocation>,
    map_builtin: &mut impl FnMut(BuiltIn) -> Result<B, &'static str>,
    builtins: &mut Vec<B>,
) -> Result<(), String> {
    if binding.is_none()
        && let TypeInner::Struct { members, .. } = &module.types[ty].inner
    {
        for member in members {
            collect_io(
                module,
                member.ty,
                member.binding.as_ref(),
                locations,
                map_builtin,
                builtins,
            )?;
        }
        return Ok(());
    }
    match binding {
        Some(Binding::Location {
            location,
            blend_src: None,
            ..
        }) => {
            locations.push(GpuShaderIoLocation::new(
                *location,
                io_value_type(module, ty)?,
            ));
            Ok(())
        }
        Some(Binding::Location {
            blend_src: Some(_), ..
        }) => Err(
            "dual-source blend output is outside the accepted G4C2 stage-IO vocabulary".to_string(),
        ),
        Some(Binding::BuiltIn(builtin)) => {
            builtins.push(map_builtin(*builtin).map_err(str::to_string)?);
            Ok(())
        }
        None => Err(
            "entry-point IO lacks an explicit location/builtin or a typed struct member"
                .to_string(),
        ),
    }
}

fn io_value_type(
    module: &naga::Module,
    ty: naga::Handle<naga::Type>,
) -> Result<GpuShaderIoValueType, String> {
    let (scalar, width) = match module.types[ty].inner {
        TypeInner::Scalar(scalar) => (scalar, 1),
        TypeInner::Vector { size, scalar } => (scalar, vector_width(size)),
        _ => return Err("entry-point IO uses an unsupported non-scalar/vector type".to_string()),
    };
    let scalar_class = match scalar.kind {
        ScalarKind::Float => GpuShaderIoScalarClass::Float,
        ScalarKind::Sint => GpuShaderIoScalarClass::Sint,
        ScalarKind::Uint => GpuShaderIoScalarClass::Uint,
        _ => return Err("entry-point IO uses an unsupported scalar class".to_string()),
    };
    GpuShaderIoValueType::try_new(scalar_class, width).map_err(|error| error.to_string())
}

const fn vector_width(size: VectorSize) -> u8 {
    size as u8
}

const fn naga_stage(stage: GpuShaderStage) -> ShaderStage {
    match stage {
        GpuShaderStage::Compute => ShaderStage::Compute,
        GpuShaderStage::Vertex => ShaderStage::Vertex,
        GpuShaderStage::Fragment => ShaderStage::Fragment,
    }
}

fn failure(
    category: GpuProgramBindingRealizationErrorCategory,
    descriptor: &GpuProgramDescriptor,
    detail: impl Into<String>,
) -> GpuProgramBindingRealizationError {
    GpuProgramBindingRealizationError::new(
        category,
        descriptor.source().identity().diagnostic_label(),
        detail,
    )
}

fn observed_io_failure(
    descriptor: &GpuProgramDescriptor,
    detail: impl Into<String>,
) -> GpuProgramBindingRealizationError {
    failure(
        GpuProgramBindingRealizationErrorCategory::ObservedStageIoInvalid,
        descriptor,
        detail,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_profile_is_the_exact_refreshed_profile() {
        assert_eq!(G4C2_NAGA_VALIDATION_PROFILE_REVISION, 2);
        assert_eq!(G4C2_WGPU_REALIZATION_COMPATIBILITY_REVISION, 2);
        let _validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::default(),
        );
    }
}
