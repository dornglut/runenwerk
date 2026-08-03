use engine::plugins::gpu::{
    GpuAdmittedProgramSource, GpuBindingDeclaration, GpuBindingKey, GpuBindingKind,
    GpuBindingProvenance, GpuEntryPointDescriptor, GpuEntryPointName, GpuProgramContractCause,
    GpuProgramDescriptor, GpuProgramInterfaceDescriptor, GpuProgramSourceIdentity,
    GpuProgramSourceKey, GpuProgramSourceOwnerId, GpuProgramSourceProvenance,
    GpuProgramSourceRegistry, GpuProgramSourceRevision, GpuShaderStage, GpuShaderStages,
    GpuStorageBufferAccess,
};

const PROGRAM_WGSL: &str = r#"
@compute @workgroup_size(1)
fn copy_values() {}

@compute @workgroup_size(1)
fn reduce_values() {}
"#;

fn admitted_source() -> (GpuProgramSourceRegistry, GpuAdmittedProgramSource) {
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
            PROGRAM_WGSL,
            GpuProgramSourceProvenance::new("gpu-program-descriptor-test", None)
                .expect("test source provenance should be valid"),
        )
        .expect("test source should admit");
    (registry, source)
}

fn interface(stages: GpuShaderStages, binding: u64) -> GpuProgramInterfaceDescriptor {
    let declaration = GpuBindingDeclaration::new(
        GpuBindingKey::try_new(0, binding).expect("test binding key should fit u32"),
        stages,
        GpuBindingKind::storage_buffer(GpuStorageBufferAccess::ReadWrite, false, None),
        None,
        format!("storage-{binding}"),
        GpuBindingProvenance::new("gpu-program-descriptor-test", None)
            .expect("test binding provenance should be valid"),
    )
    .expect("test binding declaration should be valid");
    GpuProgramInterfaceDescriptor::new([declaration])
        .expect("test program interface should be valid")
}

fn entry(
    name: &str,
    stage: GpuShaderStage,
    interface: GpuProgramInterfaceDescriptor,
) -> GpuEntryPointDescriptor {
    GpuEntryPointDescriptor::new(
        GpuEntryPointName::new(name).expect("test entry-point name should be valid"),
        stage,
        interface,
    )
}

#[test]
fn admitted_program_normalizes_entries_and_retains_its_source() {
    let (mut registry, source) = admitted_source();
    let interface = interface(GpuShaderStages::one(GpuShaderStage::Compute), 0);
    let program = GpuProgramDescriptor::new(
        source,
        interface.clone(),
        [
            entry("reduce_values", GpuShaderStage::Compute, interface.clone()),
            entry("copy_values", GpuShaderStage::Compute, interface.clone()),
        ],
    )
    .expect("program descriptor should normalize valid entries");

    assert_eq!(
        program
            .entry_points()
            .map(|entry_point| entry_point.name().as_str())
            .collect::<Vec<_>>(),
        ["copy_values", "reduce_values"]
    );
    assert!(
        program
            .entry_point(
                GpuShaderStage::Compute,
                &GpuEntryPointName::new("copy_values").unwrap()
            )
            .is_some()
    );
    assert_eq!(program.interface(), &interface);
    assert!(program.is_same_record(&program.clone()));
    assert_eq!(registry.collect_unretained(), 0);

    drop(program);
    assert_eq!(registry.collect_unretained(), 1);
}

#[test]
fn admitted_program_rejects_duplicate_stage_and_name_pairs() {
    let (_registry, source) = admitted_source();
    let interface = interface(GpuShaderStages::one(GpuShaderStage::Compute), 0);
    let duplicate = entry("copy_values", GpuShaderStage::Compute, interface.clone());

    let error = GpuProgramDescriptor::new(source, interface, [duplicate.clone(), duplicate])
        .expect_err("duplicate stage and name pairs must be rejected");

    assert_eq!(error.cause(), GpuProgramContractCause::DuplicateEntryPoint);
}

#[test]
fn admitted_program_rejects_entry_interface_disagreement() {
    let (_registry, source) = admitted_source();
    let program_interface = interface(GpuShaderStages::one(GpuShaderStage::Compute), 0);
    let entry_interface = interface(GpuShaderStages::one(GpuShaderStage::Compute), 1);

    let error = GpuProgramDescriptor::new(
        source,
        program_interface,
        [entry(
            "copy_values",
            GpuShaderStage::Compute,
            entry_interface,
        )],
    )
    .expect_err("entry points must use the program's one explicit interface");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::ProgramInterfaceMismatch
    );
}

#[test]
fn admitted_program_rejects_visibility_for_an_undeclared_stage() {
    let (_registry, source) = admitted_source();
    let stages = GpuShaderStages::new([GpuShaderStage::Compute, GpuShaderStage::Fragment])
        .expect("test stage visibility should be nonempty");
    let interface = interface(stages, 0);

    let error = GpuProgramDescriptor::new(
        source,
        interface.clone(),
        [entry("copy_values", GpuShaderStage::Compute, interface)],
    )
    .expect_err("binding visibility must reference declared entry stages");

    assert_eq!(
        error.cause(),
        GpuProgramContractCause::ProgramInterfaceMismatch
    );
}
