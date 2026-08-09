use core::num::NonZeroU32;
use engine::plugins::gpu::{
    GpuAdmittedProgramSource, GpuBindingDeclaration, GpuBindingKey, GpuBindingKind,
    GpuBindingProvenance, GpuBlendMode, GpuCapabilityFeature, GpuCapabilityRequirement,
    GpuCapabilityRequirements, GpuColorTargetStateDescriptor, GpuColorWriteMask,
    GpuComputePipelineDescriptor, GpuEntryPointDescriptor, GpuEntryPointName,
    GpuFragmentOutputStateDescriptor, GpuMultisampleStateDescriptor, GpuPipelineLayoutDescriptor,
    GpuPrimitiveStateDescriptor, GpuProgramDescriptor, GpuProgramInterfaceDescriptor,
    GpuProgramSourceIdentity, GpuProgramSourceKey, GpuProgramSourceOwnerId,
    GpuProgramSourceProvenance, GpuProgramSourceRegistry, GpuProgramSourceRevision,
    GpuRenderEntryPoints, GpuRenderPipelineDescriptor, GpuRenderPipelineStateDescriptor,
    GpuSamplerClass, GpuShaderStage, GpuShaderStages, GpuSpecializationSchema,
    GpuSpecializationValueSet, GpuStorageBufferAccess, GpuStorageTextureAccess, GpuTextureFormat,
    GpuTextureSampleClass, GpuTextureViewDimension, GpuVertexInputStateDescriptor,
};

fn admitted_source(key: &str) -> (GpuProgramSourceRegistry, GpuAdmittedProgramSource) {
    let identity = GpuProgramSourceIdentity::new(
        GpuProgramSourceOwnerId::allocate().expect("test source owner should allocate"),
        GpuProgramSourceKey::new(key).expect("test source key should be valid"),
        GpuProgramSourceRevision::try_from_raw(1).expect("test source revision should be nonzero"),
    );
    let mut registry =
        GpuProgramSourceRegistry::new(4, 4096).expect("test source registry should construct");
    let source = registry
        .admit_wgsl(
            identity,
            "@compute @workgroup_size(1) fn compute_main() {}\n@vertex fn vertex_main() -> @builtin(position) vec4f { return vec4f(); }\n@fragment fn fragment_main() -> @location(0) vec4f { return vec4f(); }",
            GpuProgramSourceProvenance::new("gpu-program-requirement-test", None)
                .expect("test source provenance should be valid"),
        )
        .expect("test source should admit");
    (registry, source)
}

fn fixed_array_declaration(
    binding: u64,
    stages: GpuShaderStages,
    kind: GpuBindingKind,
    label: &str,
) -> GpuBindingDeclaration {
    GpuBindingDeclaration::new(
        GpuBindingKey::try_new(0, binding).unwrap(),
        stages,
        kind,
        NonZeroU32::new(2),
        label,
        GpuBindingProvenance::new("gpu-program-requirement-test", None).unwrap(),
    )
    .unwrap()
}

fn fixed_array_interface(stages: GpuShaderStages) -> GpuProgramInterfaceDescriptor {
    GpuProgramInterfaceDescriptor::new([
        fixed_array_declaration(
            0,
            stages,
            GpuBindingKind::uniform_buffer(false, None),
            "uniform-array",
        ),
        fixed_array_declaration(
            1,
            stages,
            GpuBindingKind::storage_buffer(GpuStorageBufferAccess::ReadOnly, false, None),
            "storage-buffer-array",
        ),
        fixed_array_declaration(
            2,
            stages,
            GpuBindingKind::sampled_texture(
                GpuTextureSampleClass::FloatFilterable,
                GpuTextureViewDimension::D2,
                false,
            )
            .unwrap(),
            "sampled-texture-array",
        ),
        fixed_array_declaration(
            3,
            stages,
            GpuBindingKind::storage_texture(
                GpuStorageTextureAccess::WriteOnly,
                GpuTextureFormat::Rgba8Unorm,
                GpuTextureViewDimension::D2,
            )
            .unwrap(),
            "storage-texture-array",
        ),
        fixed_array_declaration(
            4,
            stages,
            GpuBindingKind::sampler(GpuSamplerClass::Filtering),
            "sampler-array",
        ),
    ])
    .unwrap()
}

fn entry_point(name: &str) -> GpuEntryPointName {
    GpuEntryPointName::new(name).expect("test entry-point name should be valid")
}

fn entry(
    name: &str,
    stage: GpuShaderStage,
    interface: GpuProgramInterfaceDescriptor,
) -> GpuEntryPointDescriptor {
    GpuEntryPointDescriptor::new(entry_point(name), stage, interface)
}

fn specialization() -> GpuSpecializationValueSet {
    GpuSpecializationValueSet::new(GpuSpecializationSchema::new([]).unwrap(), []).unwrap()
}

fn assert_required(requirements: &GpuCapabilityRequirements, feature: GpuCapabilityFeature) {
    assert_eq!(
        requirements.get(feature),
        Some(GpuCapabilityRequirement::Required(feature))
    );
}

fn assert_fixed_array_requirements(requirements: &GpuCapabilityRequirements) {
    for feature in [
        GpuCapabilityFeature::StorageTexture,
        GpuCapabilityFeature::TextureBindingArray,
        GpuCapabilityFeature::BufferBindingArray,
        GpuCapabilityFeature::StorageResourceBindingArray,
        GpuCapabilityFeature::UniformBufferBindingArray,
    ] {
        assert_required(requirements, feature);
    }
}

#[test]
fn compute_pipeline_inherits_program_interface_requirements() {
    let (_registry, source) = admitted_source("compute.program-requirements");
    let interface = fixed_array_interface(GpuShaderStages::one(GpuShaderStage::Compute));
    let program = GpuProgramDescriptor::new(
        source,
        interface.clone(),
        [entry(
            "compute_main",
            GpuShaderStage::Compute,
            interface.clone(),
        )],
    )
    .unwrap();
    assert_fixed_array_requirements(program.requirements());

    let pipeline = GpuComputePipelineDescriptor::new(
        program,
        entry_point("compute_main"),
        GpuPipelineLayoutDescriptor::from_interface(&interface).unwrap(),
        specialization(),
        GpuCapabilityRequirements::new(),
    )
    .unwrap();
    assert_fixed_array_requirements(pipeline.requirements());
}

#[test]
fn render_pipeline_inherits_program_interface_requirements() {
    let (_registry, source) = admitted_source("render.program-requirements");
    let interface = fixed_array_interface(GpuShaderStages::one(GpuShaderStage::Fragment));
    let program = GpuProgramDescriptor::new(
        source,
        interface.clone(),
        [
            entry("vertex_main", GpuShaderStage::Vertex, interface.clone()),
            entry("fragment_main", GpuShaderStage::Fragment, interface.clone()),
        ],
    )
    .unwrap();
    assert_fixed_array_requirements(program.requirements());

    let color_target = GpuColorTargetStateDescriptor::new(
        GpuTextureFormat::Rgba8Unorm,
        GpuBlendMode::Replace,
        GpuColorWriteMask::ALL,
    )
    .unwrap();
    let state = GpuRenderPipelineStateDescriptor::new(
        GpuVertexInputStateDescriptor::new([]).unwrap(),
        Some(GpuFragmentOutputStateDescriptor::new([color_target])),
        GpuPrimitiveStateDescriptor::default(),
        None,
        GpuMultisampleStateDescriptor::default(),
    )
    .unwrap();
    let pipeline = GpuRenderPipelineDescriptor::new(
        program,
        GpuRenderEntryPoints::new(
            entry_point("vertex_main"),
            Some(entry_point("fragment_main")),
        ),
        state,
        GpuPipelineLayoutDescriptor::from_interface(&interface).unwrap(),
        specialization(),
        GpuCapabilityRequirements::new(),
    )
    .unwrap();
    assert_fixed_array_requirements(pipeline.requirements());
}
