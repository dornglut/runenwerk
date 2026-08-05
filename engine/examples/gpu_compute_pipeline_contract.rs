use engine::plugins::gpu;
use std::num::NonZeroU64;

const WGSL: &str = r#"
@group(0) @binding(0)
var<storage, read> input_values: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output_values: array<u32>;

@compute @workgroup_size(64)
fn compute_main(@builtin(global_invocation_id) id: vec3<u32>) {
    output_values[id.x] = input_values[id.x];
}
"#;

fn main() {
    let owner = gpu::GpuProgramSourceOwnerId::allocate().expect("source owner should allocate");
    let identity = gpu::GpuProgramSourceIdentity::new(
        owner,
        gpu::GpuProgramSourceKey::new("examples.compute.copy").expect("source key should be valid"),
        gpu::GpuProgramSourceRevision::try_from_raw(1).expect("source revision should be nonzero"),
    );
    let mut registry =
        gpu::GpuProgramSourceRegistry::new(4, 16 * 1024).expect("registry should construct");
    let source = registry
        .admit_wgsl(identity.clone(), WGSL, provenance("initial admission"))
        .expect("source should admit");
    let repeated = registry
        .admit_wgsl(identity, WGSL, provenance("idempotent rediscovery"))
        .expect("identical source should remain idempotent");
    assert!(source.is_same_record(&repeated));

    let visibility = gpu::GpuShaderStages::one(gpu::GpuShaderStage::Compute);
    let input = binding(
        0,
        visibility,
        gpu::GpuStorageBufferAccess::ReadOnly,
        "input-values",
    );
    let output = binding(
        1,
        visibility,
        gpu::GpuStorageBufferAccess::ReadWrite,
        "output-values",
    );
    let interface = gpu::GpuProgramInterfaceDescriptor::new([input, output])
        .expect("interface should construct");
    let program = gpu::GpuProgramDescriptor::new(
        source,
        interface.clone(),
        [gpu::GpuEntryPointDescriptor::new(
            entry_point("compute_main"),
            gpu::GpuShaderStage::Compute,
            interface.clone(),
        )],
    )
    .expect("program should construct");
    let layout = gpu::GpuPipelineLayoutDescriptor::from_interface(&interface)
        .expect("layout should derive from the interface");
    let pipeline = gpu::GpuComputePipelineDescriptor::new(
        program,
        entry_point("compute_main"),
        layout,
        empty_specialization(),
        gpu::GpuCapabilityRequirements::new(),
    )
    .expect("compute pipeline should construct");

    assert!(matches!(
        pipeline
            .requirements()
            .get(gpu::GpuCapabilityFeature::Compute),
        Some(gpu::GpuCapabilityRequirement::Required(
            gpu::GpuCapabilityFeature::Compute
        ))
    ));
}

fn binding(
    binding: u64,
    visibility: gpu::GpuShaderStages,
    access: gpu::GpuStorageBufferAccess,
    label: &str,
) -> gpu::GpuBindingDeclaration {
    gpu::GpuBindingDeclaration::new(
        gpu::GpuBindingKey::try_new(0, binding).expect("binding key should fit u32"),
        visibility,
        gpu::GpuBindingKind::storage_buffer(access, false, NonZeroU64::new(4)),
        None,
        label,
        gpu::GpuBindingProvenance::new("gpu-compute-contract-example", None)
            .expect("binding provenance should be valid"),
    )
    .expect("binding declaration should construct")
}

fn provenance(detail: &str) -> gpu::GpuProgramSourceProvenance {
    gpu::GpuProgramSourceProvenance::new("gpu-compute-contract-example", Some(detail.to_owned()))
        .expect("source provenance should be valid")
}

fn entry_point(name: &str) -> gpu::GpuEntryPointName {
    gpu::GpuEntryPointName::new(name).expect("entry-point name should be valid")
}

fn empty_specialization() -> gpu::GpuSpecializationValueSet {
    let schema =
        gpu::GpuSpecializationSchema::new([]).expect("empty specialization schema should be valid");
    gpu::GpuSpecializationValueSet::new(schema, [])
        .expect("empty specialization value set should be valid")
}
