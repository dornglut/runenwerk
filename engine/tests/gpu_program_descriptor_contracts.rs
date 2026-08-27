use engine::plugins::gpu::{
    GpuAdmittedProgramSource, GpuBindingKey, GpuBindingLayoutRefinement, GpuEntryPointName,
    GpuProgramContractCause, GpuProgramDescriptor, GpuProgramSourceIdentity, GpuProgramSourceKey,
    GpuProgramSourceOwnerId, GpuProgramSourceProvenance, GpuProgramSourceRegistry,
    GpuProgramSourceRevision, GpuShaderStage, GpuShaderStages, GpuStorageBufferAccess,
};

const PROGRAM_WGSL: &str = r#"
@group(0) @binding(0)
var<storage, read> input_values: array<u32>;

@group(0) @binding(1)
var<storage, read_write> output_values: array<u32>;

@group(0) @binding(2)
var<storage, read> unused_values: array<u32>;

@compute @workgroup_size(1)
fn copy_values() {
    output_values[0] = input_values[0];
}

@compute @workgroup_size(1)
fn reduce_values() {
    output_values[0] = input_values[0] + 1u;
}
"#;

fn admitted_source_from(source_text: &str) -> (GpuProgramSourceRegistry, GpuAdmittedProgramSource) {
    let owner = GpuProgramSourceOwnerId::allocate().expect("source owner should allocate");
    let identity = GpuProgramSourceIdentity::new(
        owner,
        GpuProgramSourceKey::new("compute.program-descriptor")
            .expect("test source key should be valid"),
        GpuProgramSourceRevision::try_from_raw(1).expect("test source revision should be nonzero"),
    );
    let mut registry = GpuProgramSourceRegistry::new(4, 16 * 1024)
        .expect("bounded source registry should construct");
    let source = registry
        .admit_wgsl(
            identity,
            source_text,
            GpuProgramSourceProvenance::new("gpu-program-descriptor-test", None)
                .expect("test source provenance should be valid"),
        )
        .expect("source registry admission owns identity/content bounds, not WGSL semantics");
    (registry, source)
}

fn admitted_source() -> (GpuProgramSourceRegistry, GpuAdmittedProgramSource) {
    admitted_source_from(PROGRAM_WGSL)
}

fn entry(name: &str) -> GpuEntryPointName {
    GpuEntryPointName::new(name).expect("test entry-point name should be valid")
}

fn key(binding: u64) -> GpuBindingKey {
    GpuBindingKey::try_new(0, binding).expect("test binding key should fit u32")
}

#[test]
fn admitted_program_derives_entries_interface_and_static_visibility() {
    let (mut registry, source) = admitted_source();
    let program = GpuProgramDescriptor::new(
        source,
        [entry("reduce_values"), entry("copy_values")],
        std::iter::empty::<GpuBindingLayoutRefinement>(),
    )
    .expect("canonical WGSL should derive the selected program contract");

    assert_eq!(
        program
            .entry_points()
            .map(|entry_point| (entry_point.name().as_str(), entry_point.stage()))
            .collect::<Vec<_>>(),
        [
            ("copy_values", GpuShaderStage::Compute),
            ("reduce_values", GpuShaderStage::Compute),
        ]
    );
    assert!(
        program
            .entry_point(GpuShaderStage::Compute, &entry("copy_values"))
            .is_some()
    );

    let bindings = program.interface().bindings().collect::<Vec<_>>();
    assert_eq!(
        bindings.len(),
        2,
        "unused bound globals are not program-interface members"
    );
    assert_eq!(bindings[0].key(), key(0));
    assert_eq!(bindings[1].key(), key(1));
    assert_eq!(
        bindings[0].kind().storage_buffer_access(),
        Some(GpuStorageBufferAccess::ReadOnly)
    );
    assert_eq!(
        bindings[1].kind().storage_buffer_access(),
        Some(GpuStorageBufferAccess::ReadWrite)
    );
    assert_eq!(
        bindings[0].visibility(),
        GpuShaderStages::one(GpuShaderStage::Compute)
    );
    assert!(program.interface().binding(key(2)).is_none());
    assert!(program.is_same_record(&program.clone()));
    assert_eq!(registry.collect_unretained(), 0);

    drop(program);
    assert_eq!(registry.collect_unretained(), 1);
}

#[test]
fn admitted_program_rejects_duplicate_selected_entry_names() {
    let (_registry, source) = admitted_source();
    let duplicate = entry("copy_values");

    let error = GpuProgramDescriptor::new(
        source,
        [duplicate.clone(), duplicate],
        std::iter::empty::<GpuBindingLayoutRefinement>(),
    )
    .expect_err("duplicate selected names must be rejected before backend realization");

    assert_eq!(error.cause(), GpuProgramContractCause::DuplicateEntryPoint);
}

#[test]
fn admitted_program_rejects_missing_selected_entry() {
    let (_registry, source) = admitted_source();

    let error = GpuProgramDescriptor::new(
        source,
        [entry("missing")],
        std::iter::empty::<GpuBindingLayoutRefinement>(),
    )
    .expect_err("missing selected entries must reject during program admission");

    assert_eq!(error.cause(), GpuProgramContractCause::EntryPointMissing);
}

#[test]
fn admitted_program_rejects_malformed_canonical_wgsl() {
    let (_registry, source) = admitted_source_from("@compute fn broken(");

    let error = GpuProgramDescriptor::new(
        source,
        [entry("broken")],
        std::iter::empty::<GpuBindingLayoutRefinement>(),
    )
    .expect_err("WGSL syntax must be validated before backend realization");

    assert_eq!(error.cause(), GpuProgramContractCause::CanonicalWgslInvalid);
}

#[test]
fn refinement_cannot_resurrect_an_unused_shader_binding() {
    let (_registry, source) = admitted_source();
    let refinement = GpuBindingLayoutRefinement::new(key(2)).with_dynamic_offset(true);

    let error = GpuProgramDescriptor::new(source, [entry("copy_values")], [refinement])
        .expect_err("refinements must target effective selected-program bindings only");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::BindingRefinementInvalid
    );
}

#[test]
fn buffer_refinement_changes_only_host_layout_policy() {
    let (_registry, source) = admitted_source();
    let refinement = GpuBindingLayoutRefinement::new(key(0)).with_dynamic_offset(true);

    let program = GpuProgramDescriptor::new(source, [entry("copy_values")], [refinement])
        .expect("dynamic offset is valid host policy for a storage buffer");
    let binding = program.interface().binding(key(0)).unwrap();

    assert!(binding.kind().uses_dynamic_offset());
    assert_eq!(
        binding.kind().storage_buffer_access(),
        Some(GpuStorageBufferAccess::ReadOnly)
    );
    assert_eq!(
        binding.visibility(),
        GpuShaderStages::one(GpuShaderStage::Compute)
    );
}

#[test]
fn visibility_refinement_cannot_invent_an_unselected_stage() {
    let (_registry, source) = admitted_source();
    let visibility = GpuShaderStages::new([GpuShaderStage::Compute, GpuShaderStage::Fragment])
        .expect("test visibility should be nonempty");
    let refinement = GpuBindingLayoutRefinement::new(key(0)).with_visibility(visibility);

    let error = GpuProgramDescriptor::new(source, [entry("copy_values")], [refinement])
        .expect_err("visibility may widen only within stages selected by this program");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::BindingRefinementInvalid
    );
}
