use engine::plugins::gpu::{
    GpuAdmittedProgramSource, GpuBindGroupLayoutDescriptor, GpuCapabilityFeature,
    GpuCapabilityRequirement, GpuCapabilityRequirements, GpuComputePipelineDescriptor,
    GpuEntryPointDescriptor, GpuEntryPointName, GpuPipelineLayoutDescriptor,
    GpuProgramContractCause, GpuProgramDescriptor, GpuProgramInterfaceDescriptor,
    GpuProgramSourceIdentity, GpuProgramSourceKey, GpuProgramSourceOwnerId,
    GpuProgramSourceProvenance, GpuProgramSourceRegistry, GpuProgramSourceRevision, GpuShaderStage,
    GpuSpecializationDeclaration, GpuSpecializationKey, GpuSpecializationSchema,
    GpuSpecializationValue, GpuSpecializationValueSet, GpuSpecializationValueType,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn source() -> (GpuProgramSourceRegistry, GpuAdmittedProgramSource) {
    let identity = GpuProgramSourceIdentity::new(
        GpuProgramSourceOwnerId::allocate().expect("test source owner should allocate"),
        GpuProgramSourceKey::new("compute.pipeline").expect("test source key should be valid"),
        GpuProgramSourceRevision::try_from_raw(1).expect("test source revision should be nonzero"),
    );
    let mut registry =
        GpuProgramSourceRegistry::new(4, 4096).expect("test source registry should construct");
    let source = registry
        .admit_wgsl(
            identity,
            "@compute @workgroup_size(1) fn main() {}",
            GpuProgramSourceProvenance::new("gpu-compute-pipeline-test", None)
                .expect("test provenance should be valid"),
        )
        .expect("test source should admit");
    (registry, source)
}

fn program(stage: GpuShaderStage, name: &str) -> GpuProgramDescriptor {
    let (_registry, source) = source();
    let interface =
        GpuProgramInterfaceDescriptor::new([]).expect("empty test interface should be valid");
    GpuProgramDescriptor::new(
        source,
        interface.clone(),
        [GpuEntryPointDescriptor::new(
            GpuEntryPointName::new(name).expect("test entry-point name should be valid"),
            stage,
            interface,
        )],
    )
    .expect("test program should be valid")
}

fn specialization(requirements: GpuCapabilityRequirements) -> GpuSpecializationValueSet {
    let declaration = GpuSpecializationDeclaration::new(
        GpuSpecializationKey::new("iterations").expect("test specialization key should be valid"),
        GpuSpecializationValueType::U32,
        Some(GpuSpecializationValue::U32(1)),
        requirements,
    )
    .expect("test specialization declaration should be valid");
    let schema = GpuSpecializationSchema::new([declaration])
        .expect("test specialization schema should be valid");
    GpuSpecializationValueSet::new(schema, [])
        .expect("test specialization values should use the default")
}

fn hash_of(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn compute_pipeline_binds_program_layout_specialization_and_requirements() {
    let program = program(GpuShaderStage::Compute, "main");
    let layout = GpuPipelineLayoutDescriptor::from_interface(program.interface())
        .expect("test pipeline layout should derive");
    let mut specialization_requirements = GpuCapabilityRequirements::new();
    specialization_requirements
        .insert(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::TimestampQuery,
        ))
        .unwrap();
    let specialization = specialization(specialization_requirements);

    let descriptor = GpuComputePipelineDescriptor::new(
        program.clone(),
        GpuEntryPointName::new("main").unwrap(),
        layout.clone(),
        specialization.clone(),
        GpuCapabilityRequirements::new(),
    )
    .expect("valid compute pipeline descriptor should construct");
    let equivalent = GpuComputePipelineDescriptor::new(
        program,
        GpuEntryPointName::new("main").unwrap(),
        layout,
        specialization,
        GpuCapabilityRequirements::new(),
    )
    .unwrap();

    assert_eq!(descriptor, equivalent);
    assert_eq!(hash_of(&descriptor), hash_of(&equivalent));
    assert!(descriptor.is_same_record(&descriptor.clone()));
    assert!(matches!(
        descriptor.requirements().get(GpuCapabilityFeature::Compute),
        Some(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::Compute
        ))
    ));
    assert!(matches!(
        descriptor
            .requirements()
            .get(GpuCapabilityFeature::TimestampQuery),
        Some(GpuCapabilityRequirement::Required(
            GpuCapabilityFeature::TimestampQuery
        ))
    ));
}

#[test]
fn compute_pipeline_rejects_a_non_compute_or_missing_entry_point() {
    let program = program(GpuShaderStage::Vertex, "vertex_main");
    let layout = GpuPipelineLayoutDescriptor::from_interface(program.interface()).unwrap();
    let error = GpuComputePipelineDescriptor::new(
        program,
        GpuEntryPointName::new("vertex_main").unwrap(),
        layout,
        specialization(GpuCapabilityRequirements::new()),
        GpuCapabilityRequirements::new(),
    )
    .expect_err("compute pipelines must select a declared compute entry point");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::PipelineDescriptorInvalid
    );
}

#[test]
fn compute_pipeline_rejects_a_layout_not_derived_from_the_program_interface() {
    let program = program(GpuShaderStage::Compute, "main");
    let mismatched_group =
        GpuBindGroupLayoutDescriptor::new(0, []).expect("empty mismatched group should construct");
    let mismatched_layout = GpuPipelineLayoutDescriptor::new([mismatched_group])
        .expect("mismatched test layout should construct");
    let error = GpuComputePipelineDescriptor::new(
        program,
        GpuEntryPointName::new("main").unwrap(),
        mismatched_layout,
        specialization(GpuCapabilityRequirements::new()),
        GpuCapabilityRequirements::new(),
    )
    .expect_err("pipeline layout must match the program interface");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::PipelineDescriptorInvalid
    );
}

#[test]
fn compute_pipeline_rejects_conflicting_capability_requirements() {
    let program = program(GpuShaderStage::Compute, "main");
    let layout = GpuPipelineLayoutDescriptor::from_interface(program.interface()).unwrap();
    let mut requirements = GpuCapabilityRequirements::new();
    requirements
        .insert(GpuCapabilityRequirement::Disabled(
            GpuCapabilityFeature::Compute,
        ))
        .unwrap();

    let error = GpuComputePipelineDescriptor::new(
        program,
        GpuEntryPointName::new("main").unwrap(),
        layout,
        specialization(GpuCapabilityRequirements::new()),
        requirements,
    )
    .expect_err("compute cannot be disabled for a compute pipeline");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::PipelineDescriptorInvalid
    );
}
