use ui_controls::{
    ControlKernelId, ControlKernelSet, ControlKindDescriptor, ControlKindId,
    ControlPackageDescriptor, ControlPackageId, ControlPackageRegistry, ControlPackageVersion,
    runenwerk_control_package,
};
use ui_program::UiSchemaRef;
use ui_program_lowering::UiProgramFormationControlCatalog;

#[test]
fn runenwerk_control_package_derives_catalog_without_diagnostics() {
    let registry = ControlPackageRegistry::new()
        .with_package(runenwerk_control_package())
        .expect("runenwerk controls package should register");

    let report = UiProgramFormationControlCatalog::derive_from_control_package_registry_snapshot(
        &registry.snapshot(),
    );

    assert!(report.passed(), "{:?}", report.diagnostics);
    assert!(report.diagnostics.is_empty());
    assert!(report.skipped_control_kinds.is_empty());
}

#[test]
fn catalog_derivation_reports_control_kind_without_activation_capability() {
    let control_kind_id = "fixture.ui.controls.no-capability";

    let kind = ControlKindDescriptor::new(
        ControlKindId::new(control_kind_id),
        "NoCapability",
        UiSchemaRef::new("fixture.ui.controls.no_capability.properties", 1),
        UiSchemaRef::new("fixture.ui.controls.no_capability.state", 1),
        UiSchemaRef::new("fixture.ui.controls.no_capability.event", 1),
        ControlKernelSet::new(
            ControlKernelId::new("fixture.ui.controls.no_capability.layout"),
            ControlKernelId::new("fixture.ui.controls.no_capability.interaction"),
            ControlKernelId::new("fixture.ui.controls.no_capability.visual"),
            ControlKernelId::new("fixture.ui.controls.no_capability.accessibility"),
            ControlKernelId::new("fixture.ui.controls.no_capability.inspection"),
        ),
    );

    let mut package = ControlPackageDescriptor::new(
        ControlPackageId::new("fixture.ui.controls"),
        ControlPackageVersion::new(1),
    );
    package.control_kinds.push(kind);

    let registry = ControlPackageRegistry::new()
        .with_package(package)
        .expect("fixture package should register");

    let report = UiProgramFormationControlCatalog::derive_from_control_package_registry_snapshot(
        &registry.snapshot(),
    );

    assert!(!report.passed());
    assert!(report.has_diagnostic("ui.program.catalog.control_kind_missing_activation_capability"));
    assert_eq!(report.skipped_control_kinds, [control_kind_id.to_owned()]);
}
