use super::super::contract_diagnostics::{GpuProgramContractCause, GpuProgramContractError};
use super::{GpuBindingDeclaration, GpuObservedBindingDeclaration, GpuObservedProgramInterface};
use super::{GpuProgramInterfaceDescriptor, GpuShaderStage};

/// Compares explicit resource-interface authority with normalized reflection evidence.
///
/// The explicit descriptor is never changed or replaced. Observed statically-used
/// stages must be a subset of declared visibility because visibility is an
/// accessibility allowance, not a claim that every stage consumes the resource.
pub fn compare_program_interfaces(
    expected: &GpuProgramInterfaceDescriptor,
    observed: &GpuObservedProgramInterface,
) -> Result<(), GpuProgramContractError> {
    let mut expected_bindings = expected.bindings().peekable();
    let mut observed_bindings = observed.bindings().peekable();

    loop {
        match (
            expected_bindings.peek().copied(),
            observed_bindings.peek().copied(),
        ) {
            (None, None) => return Ok(()),
            (Some(expected), None) => {
                return missing_or_additional(
                    expected.key().to_string(),
                    "missing observed binding",
                );
            }
            (None, Some(observed)) => {
                return missing_or_additional(
                    observed.key().to_string(),
                    "unexpected observed binding",
                );
            }
            (Some(expected), Some(observed)) => match expected.key().cmp(&observed.key()) {
                core::cmp::Ordering::Less => {
                    return missing_or_additional(
                        expected.key().to_string(),
                        "missing observed binding",
                    );
                }
                core::cmp::Ordering::Greater => {
                    return missing_or_additional(
                        observed.key().to_string(),
                        "unexpected observed binding",
                    );
                }
                core::cmp::Ordering::Equal => {
                    compare_binding(expected, observed)?;
                    expected_bindings.next();
                    observed_bindings.next();
                }
            },
        }
    }
}

fn compare_binding(
    expected: &GpuBindingDeclaration,
    observed: &GpuObservedBindingDeclaration,
) -> Result<(), GpuProgramContractError> {
    let key = expected.key();
    if expected.kind().class() != observed.kind().class() {
        return mismatch(
            key,
            format!(
                "resource class expected={:?} observed={:?}",
                expected.kind().class(),
                observed.kind().class()
            ),
            "make the reflected resource class match the explicit declaration",
        );
    }
    if expected.array_count() != observed.array_count() {
        return mismatch(
            key,
            format!(
                "array cardinality expected={:?} observed={:?}",
                expected.array_count(),
                observed.array_count()
            ),
            "make the reflected fixed array cardinality match the explicit declaration",
        );
    }

    compare_kind(expected, observed)?;
    for stage in observed.statically_used_stages().iter() {
        if !expected.visibility().contains(stage) {
            return visibility_mismatch(key, stage);
        }
    }
    Ok(())
}

fn compare_kind(
    expected: &GpuBindingDeclaration,
    observed: &GpuObservedBindingDeclaration,
) -> Result<(), GpuProgramContractError> {
    let expected_kind = expected.kind();
    let observed_kind = observed.kind();
    let key = expected.key();

    compare_minimum_buffer_size(
        key,
        expected_kind.minimum_buffer_size(),
        observed_kind.minimum_buffer_size(),
    )?;
    compare_exact(
        key,
        "storage-buffer access",
        expected_kind.storage_buffer_access(),
        observed_kind.storage_buffer_access(),
    )?;
    compare_exact(
        key,
        "texture view dimension",
        expected_kind.texture_view_dimension(),
        observed_kind.texture_view_dimension(),
    )?;
    compare_sampled_texture_class(
        key,
        expected_kind.texture_sample_class(),
        observed_kind.texture_sample_class(),
    )?;
    compare_exact(
        key,
        "sampled texture multisample state",
        expected_kind.is_multisampled_texture(),
        observed_kind.is_multisampled_texture(),
    )?;
    compare_exact(
        key,
        "storage texture access",
        expected_kind.storage_texture_access(),
        observed_kind.storage_texture_access(),
    )?;
    compare_exact(
        key,
        "storage texture format",
        expected_kind.storage_texture_format(),
        observed_kind.storage_texture_format(),
    )?;
    compare_sampler_class(
        key,
        expected_kind.sampler_class(),
        observed_kind.sampler_class(),
    )
}

fn compare_sampled_texture_class(
    key: super::GpuBindingKey,
    expected: Option<super::GpuTextureSampleClass>,
    observed: Option<super::GpuObservedTextureSampleClass>,
) -> Result<(), GpuProgramContractError> {
    let compatible = matches!(
        (expected, observed),
        (None, None)
            | (
                Some(
                    super::GpuTextureSampleClass::FloatFilterable
                        | super::GpuTextureSampleClass::FloatUnfilterable
                ),
                Some(super::GpuObservedTextureSampleClass::Float)
            )
            | (
                Some(super::GpuTextureSampleClass::Depth),
                Some(super::GpuObservedTextureSampleClass::Depth)
            )
            | (
                Some(super::GpuTextureSampleClass::Sint),
                Some(super::GpuObservedTextureSampleClass::Sint)
            )
            | (
                Some(super::GpuTextureSampleClass::Uint),
                Some(super::GpuObservedTextureSampleClass::Uint)
            )
    );
    if compatible {
        return Ok(());
    }
    mismatch(
        key,
        format!("sampled texture class expected={expected:?} observed={observed:?}"),
        "make the reflected sampled texture numeric class match the explicit declaration",
    )
}

fn compare_sampler_class(
    key: super::GpuBindingKey,
    expected: Option<super::GpuSamplerClass>,
    observed: Option<super::GpuObservedSamplerClass>,
) -> Result<(), GpuProgramContractError> {
    let compatible = matches!(
        (expected, observed),
        (None, None)
            | (
                Some(super::GpuSamplerClass::Filtering | super::GpuSamplerClass::NonFiltering),
                Some(super::GpuObservedSamplerClass::NonComparison)
            )
            | (
                Some(super::GpuSamplerClass::Comparison),
                Some(super::GpuObservedSamplerClass::Comparison)
            )
    );
    if compatible {
        return Ok(());
    }
    mismatch(
        key,
        format!("sampler comparison semantics expected={expected:?} observed={observed:?}"),
        "make the reflected sampler comparison semantics match the explicit declaration",
    )
}

fn compare_minimum_buffer_size(
    key: super::GpuBindingKey,
    declared: Option<core::num::NonZeroU64>,
    observed: Option<core::num::NonZeroU64>,
) -> Result<(), GpuProgramContractError> {
    if let (Some(declared), Some(observed)) = (declared, observed)
        && declared.get() < observed.get()
    {
        return mismatch(
            key,
            format!(
                "minimum buffer size declared={} observed_required={}",
                declared, observed
            ),
            "make the explicit minimum binding size sufficient for the reflected shader requirement",
        );
    }
    Ok(())
}

fn compare_exact<T>(
    key: super::GpuBindingKey,
    fact: &'static str,
    expected: T,
    observed: T,
) -> Result<(), GpuProgramContractError>
where
    T: core::fmt::Debug + PartialEq,
{
    if expected == observed {
        return Ok(());
    }
    mismatch(
        key,
        format!("{fact} expected={expected:?} observed={observed:?}"),
        "make the reflected resource facts match the explicit declaration",
    )
}

fn visibility_mismatch(
    key: super::GpuBindingKey,
    observed_stage: GpuShaderStage,
) -> Result<(), GpuProgramContractError> {
    mismatch(
        key,
        format!("observed static use outside declared visibility stage={observed_stage:?}"),
        "include every reflected statically-used stage in explicit binding visibility",
    )
}

fn missing_or_additional(key: String, reason: &'static str) -> Result<(), GpuProgramContractError> {
    Err(GpuProgramContractError::invalid(
        "compare GPU program resource interfaces",
        format!("binding {key}: {reason}"),
        GpuProgramContractCause::ProgramInterfaceMismatch,
        "make explicit and observed resource binding identities agree exactly",
    ))
}

fn mismatch(
    key: super::GpuBindingKey,
    reason: String,
    correction: &'static str,
) -> Result<(), GpuProgramContractError> {
    Err(GpuProgramContractError::invalid(
        "compare GPU program resource interfaces",
        format!("binding {key}: {reason}"),
        GpuProgramContractCause::ProgramInterfaceMismatch,
        correction,
    ))
}
