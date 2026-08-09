use engine::plugins::gpu::{
    GpuBindingDeclaration, GpuBindingKey, GpuBindingKind, GpuBindingProvenance,
    GpuObservedBindingDeclaration, GpuObservedBindingKind, GpuObservedProgramInterface,
    GpuObservedSamplerClass, GpuObservedShaderStages, GpuObservedTextureSampleClass,
    GpuProgramContractCause, GpuProgramInterfaceDescriptor, GpuSamplerClass, GpuShaderStage,
    GpuShaderStages, GpuStorageBufferAccess, GpuStorageTextureAccess, GpuTextureFormat,
    GpuTextureSampleClass, GpuTextureViewDimension, compare_program_interfaces,
};
use std::num::{NonZeroU32, NonZeroU64};

fn key(group: u64, binding: u64) -> GpuBindingKey {
    GpuBindingKey::try_new(group, binding).expect("test key should fit u32")
}

fn expected(
    key: GpuBindingKey,
    visibility: GpuShaderStages,
    kind: GpuBindingKind,
    array_count: Option<NonZeroU32>,
    label: &str,
    detail: &str,
) -> GpuBindingDeclaration {
    GpuBindingDeclaration::new(
        key,
        visibility,
        kind,
        array_count,
        label,
        GpuBindingProvenance::new("program-interface-agreement-test", Some(detail.to_owned()))
            .expect("test provenance should be valid"),
    )
    .expect("test declaration should be valid")
}

fn observed(
    key: GpuBindingKey,
    kind: GpuObservedBindingKind,
    array_count: Option<NonZeroU32>,
    stages: impl IntoIterator<Item = GpuShaderStage>,
) -> GpuObservedBindingDeclaration {
    GpuObservedBindingDeclaration::new(key, kind, array_count, GpuObservedShaderStages::new(stages))
}

fn compare_rejects(expected: GpuProgramInterfaceDescriptor, observed: GpuObservedProgramInterface) {
    assert_eq!(
        compare_program_interfaces(&expected, &observed)
            .expect_err("incompatible observed resource facts must reject")
            .cause(),
        GpuProgramContractCause::ProgramInterfaceMismatch
    );
}

#[test]
fn equivalent_interfaces_agree_independent_of_insertion_order() {
    let expected_interface = GpuProgramInterfaceDescriptor::new([
        expected(
            key(1, 4),
            GpuShaderStages::one(GpuShaderStage::Compute),
            GpuBindingKind::sampler(GpuSamplerClass::Filtering),
            Some(NonZeroU32::new(2).unwrap()),
            "samplers",
            "expected sampler array",
        ),
        expected(
            key(0, 1),
            GpuShaderStages::one(GpuShaderStage::Compute),
            GpuBindingKind::storage_buffer(
                GpuStorageBufferAccess::ReadOnly,
                false,
                NonZeroU64::new(16),
            ),
            None,
            "input",
            "expected input",
        ),
    ])
    .unwrap();
    let observed_interface = GpuObservedProgramInterface::new([
        observed(
            key(0, 1),
            GpuObservedBindingKind::storage_buffer(
                GpuStorageBufferAccess::ReadOnly,
                NonZeroU64::new(16),
            ),
            None,
            [GpuShaderStage::Compute],
        ),
        observed(
            key(1, 4),
            GpuObservedBindingKind::sampler(GpuObservedSamplerClass::NonComparison),
            Some(NonZeroU32::new(2).unwrap()),
            [GpuShaderStage::Compute],
        ),
    ])
    .unwrap();

    compare_program_interfaces(&expected_interface, &observed_interface)
        .expect("equivalent normalized resource interfaces should agree");
}

#[test]
fn missing_or_additional_observed_binding_rejects() {
    let declaration = expected(
        key(0, 1),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::sampler(GpuSamplerClass::Filtering),
        None,
        "sampler",
        "expected",
    );
    compare_rejects(
        GpuProgramInterfaceDescriptor::new([declaration.clone()]).unwrap(),
        GpuObservedProgramInterface::new([]).unwrap(),
    );
    compare_rejects(
        GpuProgramInterfaceDescriptor::new([]).unwrap(),
        GpuObservedProgramInterface::new([observed(
            declaration.key(),
            GpuObservedBindingKind::sampler(GpuObservedSamplerClass::NonComparison),
            None,
            [GpuShaderStage::Compute],
        )])
        .unwrap(),
    );
}

#[test]
fn changed_group_or_binding_rejects() {
    compare_rejects(
        GpuProgramInterfaceDescriptor::new([expected(
            key(0, 1),
            GpuShaderStages::one(GpuShaderStage::Compute),
            GpuBindingKind::sampler(GpuSamplerClass::Filtering),
            None,
            "sampler",
            "expected",
        )])
        .unwrap(),
        GpuObservedProgramInterface::new([observed(
            key(0, 2),
            GpuObservedBindingKind::sampler(GpuObservedSamplerClass::NonComparison),
            None,
            [GpuShaderStage::Compute],
        )])
        .unwrap(),
    );
}

#[test]
fn resource_class_and_storage_access_mismatches_reject() {
    let expected_interface = GpuProgramInterfaceDescriptor::new([expected(
        key(0, 0),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::storage_buffer(GpuStorageBufferAccess::ReadOnly, false, None),
        None,
        "input",
        "expected",
    )])
    .unwrap();
    compare_rejects(
        expected_interface.clone(),
        GpuObservedProgramInterface::new([observed(
            key(0, 0),
            GpuObservedBindingKind::sampler(GpuObservedSamplerClass::NonComparison),
            None,
            [GpuShaderStage::Compute],
        )])
        .unwrap(),
    );
    compare_rejects(
        expected_interface,
        GpuObservedProgramInterface::new([observed(
            key(0, 0),
            GpuObservedBindingKind::storage_buffer(GpuStorageBufferAccess::ReadWrite, None),
            None,
            [GpuShaderStage::Compute],
        )])
        .unwrap(),
    );
}

#[test]
fn texture_dimension_type_and_format_mismatches_reject() {
    let sampled = GpuProgramInterfaceDescriptor::new([expected(
        key(0, 0),
        GpuShaderStages::one(GpuShaderStage::Fragment),
        GpuBindingKind::sampled_texture(
            GpuTextureSampleClass::FloatFilterable,
            GpuTextureViewDimension::D2,
            false,
        )
        .unwrap(),
        None,
        "sampled",
        "expected sampled",
    )])
    .unwrap();
    compare_rejects(
        sampled.clone(),
        GpuObservedProgramInterface::new([observed(
            key(0, 0),
            GpuObservedBindingKind::sampled_texture(
                GpuObservedTextureSampleClass::Float,
                GpuTextureViewDimension::D3,
                false,
            )
            .unwrap(),
            None,
            [GpuShaderStage::Fragment],
        )])
        .unwrap(),
    );
    compare_rejects(
        sampled,
        GpuObservedProgramInterface::new([observed(
            key(0, 0),
            GpuObservedBindingKind::sampled_texture(
                GpuObservedTextureSampleClass::Uint,
                GpuTextureViewDimension::D2,
                false,
            )
            .unwrap(),
            None,
            [GpuShaderStage::Fragment],
        )])
        .unwrap(),
    );

    let storage = GpuProgramInterfaceDescriptor::new([expected(
        key(0, 1),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::storage_texture(
            GpuStorageTextureAccess::WriteOnly,
            GpuTextureFormat::Rgba8Unorm,
            GpuTextureViewDimension::D2,
        )
        .unwrap(),
        None,
        "output",
        "expected storage",
    )])
    .unwrap();
    compare_rejects(
        storage,
        GpuObservedProgramInterface::new([observed(
            key(0, 1),
            GpuObservedBindingKind::storage_texture(
                GpuStorageTextureAccess::WriteOnly,
                GpuTextureFormat::R32Uint,
                GpuTextureViewDimension::D2,
            )
            .unwrap(),
            None,
            [GpuShaderStage::Compute],
        )])
        .unwrap(),
    );
}

#[test]
fn reflection_only_compares_sampler_and_float_texture_facts_it_can_observe() {
    let expected_interface = GpuProgramInterfaceDescriptor::new([
        expected(
            key(0, 0),
            GpuShaderStages::one(GpuShaderStage::Fragment),
            GpuBindingKind::sampled_texture(
                GpuTextureSampleClass::FloatUnfilterable,
                GpuTextureViewDimension::D2,
                false,
            )
            .unwrap(),
            None,
            "float texture",
            "explicit non-filtering layout policy",
        ),
        expected(
            key(0, 1),
            GpuShaderStages::one(GpuShaderStage::Fragment),
            GpuBindingKind::sampler(GpuSamplerClass::NonFiltering),
            None,
            "sampler",
            "explicit non-filtering layout policy",
        ),
    ])
    .unwrap();
    let observed_interface = GpuObservedProgramInterface::new([
        observed(
            key(0, 0),
            GpuObservedBindingKind::sampled_texture(
                GpuObservedTextureSampleClass::Float,
                GpuTextureViewDimension::D2,
                false,
            )
            .unwrap(),
            None,
            [GpuShaderStage::Fragment],
        ),
        observed(
            key(0, 1),
            GpuObservedBindingKind::sampler(GpuObservedSamplerClass::NonComparison),
            None,
            [GpuShaderStage::Fragment],
        ),
    ])
    .unwrap();

    compare_program_interfaces(&expected_interface, &observed_interface)
        .expect("WGSL reflection must not invent filtering policy absent from the shader");

    compare_rejects(
        GpuProgramInterfaceDescriptor::new([expected(
            key(0, 0),
            GpuShaderStages::one(GpuShaderStage::Fragment),
            GpuBindingKind::sampler(GpuSamplerClass::Comparison),
            None,
            "comparison sampler",
            "comparison semantics",
        )])
        .unwrap(),
        GpuObservedProgramInterface::new([observed(
            key(0, 0),
            GpuObservedBindingKind::sampler(GpuObservedSamplerClass::NonComparison),
            None,
            [GpuShaderStage::Fragment],
        )])
        .unwrap(),
    );
}

#[test]
fn fixed_array_cardinality_mismatch_rejects() {
    compare_rejects(
        GpuProgramInterfaceDescriptor::new([expected(
            key(0, 0),
            GpuShaderStages::one(GpuShaderStage::Compute),
            GpuBindingKind::sampler(GpuSamplerClass::Filtering),
            Some(NonZeroU32::new(2).unwrap()),
            "samplers",
            "expected",
        )])
        .unwrap(),
        GpuObservedProgramInterface::new([observed(
            key(0, 0),
            GpuObservedBindingKind::sampler(GpuObservedSamplerClass::NonComparison),
            Some(NonZeroU32::new(3).unwrap()),
            [GpuShaderStage::Compute],
        )])
        .unwrap(),
    );
}

#[test]
fn observed_stage_use_must_be_allowed_but_declared_superset_succeeds() {
    let expected_interface = GpuProgramInterfaceDescriptor::new([expected(
        key(0, 0),
        GpuShaderStages::new([GpuShaderStage::Vertex, GpuShaderStage::Fragment]).unwrap(),
        GpuBindingKind::uniform_buffer(false, None),
        None,
        "camera",
        "expected vertex-fragment visibility",
    )])
    .unwrap();
    let vertex_only = GpuObservedProgramInterface::new([observed(
        key(0, 0),
        GpuObservedBindingKind::uniform_buffer(None),
        None,
        [GpuShaderStage::Vertex],
    )])
    .unwrap();
    compare_program_interfaces(&expected_interface, &vertex_only)
        .expect("declared visibility may be a strict superset of observed static use");

    let outside_visibility = GpuObservedProgramInterface::new([observed(
        key(0, 0),
        GpuObservedBindingKind::uniform_buffer(None),
        None,
        [GpuShaderStage::Compute],
    )])
    .unwrap();
    compare_rejects(expected_interface, outside_visibility);
}

#[test]
fn declared_buffer_minimum_must_cover_observed_requirement_without_inference() {
    let expected_interface = GpuProgramInterfaceDescriptor::new([expected(
        key(0, 0),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::storage_buffer(
            GpuStorageBufferAccess::ReadOnly,
            false,
            NonZeroU64::new(16),
        ),
        None,
        "input",
        "declared 16 bytes",
    )])
    .unwrap();
    compare_rejects(
        expected_interface.clone(),
        GpuObservedProgramInterface::new([observed(
            key(0, 0),
            GpuObservedBindingKind::storage_buffer(
                GpuStorageBufferAccess::ReadOnly,
                NonZeroU64::new(32),
            ),
            None,
            [GpuShaderStage::Compute],
        )])
        .unwrap(),
    );
    compare_program_interfaces(
        &expected_interface,
        &GpuObservedProgramInterface::new([observed(
            key(0, 0),
            GpuObservedBindingKind::storage_buffer(GpuStorageBufferAccess::ReadOnly, None),
            None,
            [GpuShaderStage::Compute],
        )])
        .unwrap(),
    )
    .expect("an unavailable reflected minimum must not invent a mismatch");

    let deferred = GpuProgramInterfaceDescriptor::new([expected(
        key(0, 0),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::storage_buffer(GpuStorageBufferAccess::ReadOnly, false, None),
        None,
        "input",
        "intentionally deferred minimum",
    )])
    .unwrap();
    compare_program_interfaces(
        &deferred,
        &GpuObservedProgramInterface::new([observed(
            key(0, 0),
            GpuObservedBindingKind::storage_buffer(
                GpuStorageBufferAccess::ReadOnly,
                NonZeroU64::new(32),
            ),
            None,
            [GpuShaderStage::Compute],
        )])
        .unwrap(),
    )
    .expect("reflection evidence must not infer a deferred explicit minimum");
}

#[test]
fn diagnostics_do_not_affect_agreement_and_comparison_does_not_mutate_authority() {
    let left = GpuProgramInterfaceDescriptor::new([expected(
        key(0, 0),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::sampler(GpuSamplerClass::Filtering),
        None,
        "left label",
        "left provenance",
    )])
    .unwrap();
    let right = GpuProgramInterfaceDescriptor::new([expected(
        key(0, 0),
        GpuShaderStages::one(GpuShaderStage::Compute),
        GpuBindingKind::sampler(GpuSamplerClass::Filtering),
        None,
        "right label",
        "right provenance",
    )])
    .unwrap();
    let observed_interface = GpuObservedProgramInterface::new([observed(
        key(0, 0),
        GpuObservedBindingKind::sampler(GpuObservedSamplerClass::NonComparison),
        None,
        [GpuShaderStage::Compute],
    )])
    .unwrap();
    let expected_before = left.clone();
    let observed_before = observed_interface.clone();

    compare_program_interfaces(&left, &observed_interface)
        .expect("left diagnostics must not affect agreement");
    compare_program_interfaces(&right, &observed_interface)
        .expect("right diagnostics must not affect agreement");
    assert_eq!(left, right);
    assert_eq!(left, expected_before);
    assert_eq!(observed_interface, observed_before);
}
