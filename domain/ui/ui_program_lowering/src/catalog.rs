//! File: domain/ui/ui_program_lowering/src/catalog.rs
//! Crate: ui_program_lowering
//!
//! Control package metadata adapted into semantic UiProgram formation contracts.

use std::collections::BTreeMap;

use ui_controls::{
    ControlKernelDescriptor, ControlKernelId, ControlKernelKind, ControlPackageDescriptor,
    ControlSchemaDescriptor,
};
use ui_program::{ControlKernelRef, RouteCapability, UiProgramDiagnostic, UiSchemaRef};
use ui_schema::UiSchema;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiProgramFormationControlCatalog {
    control_kinds: BTreeMap<String, UiProgramFormationControlContract>,
}

impl UiProgramFormationControlCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_control_kind(mut self, contract: UiProgramFormationControlContract) -> Self {
        self.control_kinds
            .insert(contract.kind_id.clone(), contract);
        self
    }

    pub fn control_kind(&self, kind_id: &str) -> Option<&UiProgramFormationControlContract> {
        self.control_kinds.get(kind_id)
    }

    pub fn contains_control_kind(&self, kind_id: &str) -> bool {
        self.control_kinds.contains_key(kind_id)
    }

    pub fn derive_from_control_package_registry_snapshot(
        snapshot: &ui_controls::ControlPackageRegistrySnapshot,
    ) -> UiProgramFormationCatalogReport {
        let mut catalog = Self::new();
        let mut diagnostics = Vec::new();
        let mut skipped_control_kinds = Vec::new();

        for package in &snapshot.packages {
            for kind in &package.control_kinds {
                let package_id = package.package_id.as_str();
                let control_kind_id = kind.control_kind_id.as_str();
                let mut kind_diagnostics = Vec::new();

                let activation_capability = if let Some(capability) =
                    kind.required_capabilities.first().cloned()
                {
                    Some(capability)
                } else {
                    kind_diagnostics.push(UiProgramDiagnostic::new(
                        "ui.program.catalog.control_kind_missing_activation_capability",
                        format!(
                            "package {package_id} control kind {control_kind_id} contract part activation_capability has no required capability; expected at least one route capability"
                        ),
                    ));
                    None
                };

                let mut validation = ControlContractValidationContext {
                    package,
                    control_kind_id,
                    diagnostics: &mut kind_diagnostics,
                };
                let property_schema = validation.validate_schema_ref(
                    "property_schema",
                    &kind.property_schema,
                    "Properties",
                    "ui.program.catalog.missing_property_schema",
                    &package.property_schemas,
                );
                validation.validate_schema_ref(
                    "state_schema",
                    &kind.state_schema,
                    "State",
                    "ui.program.catalog.missing_state_schema",
                    &package.state_schemas,
                );
                validation.validate_schema_ref(
                    "event_payload_schema",
                    &kind.event_payload_schema,
                    "EventPayload",
                    "ui.program.catalog.missing_event_payload_schema",
                    &package.event_payload_schemas,
                );

                validation.validate_kernel_ref(
                    "layout_kernel",
                    &kind.kernels.layout,
                    ControlKernelKind::Layout,
                    "ui.program.catalog.missing_layout_kernel",
                );
                validation.validate_kernel_ref(
                    "interaction_kernel",
                    &kind.kernels.interaction,
                    ControlKernelKind::Interaction,
                    "ui.program.catalog.missing_interaction_kernel",
                );
                validation.validate_kernel_ref(
                    "visual_kernel",
                    &kind.kernels.visual,
                    ControlKernelKind::Visual,
                    "ui.program.catalog.missing_visual_kernel",
                );
                validation.validate_kernel_ref(
                    "accessibility_kernel",
                    &kind.kernels.accessibility,
                    ControlKernelKind::Accessibility,
                    "ui.program.catalog.missing_accessibility_kernel",
                );
                validation.validate_kernel_ref(
                    "inspection_kernel",
                    &kind.kernels.inspection,
                    ControlKernelKind::Inspection,
                    "ui.program.catalog.missing_inspection_kernel",
                );

                let Some(activation_capability) = activation_capability else {
                    diagnostics.extend(kind_diagnostics);
                    skipped_control_kinds.push(control_kind_id.to_owned());
                    continue;
                };

                if !kind_diagnostics.is_empty() {
                    diagnostics.extend(kind_diagnostics);
                    skipped_control_kinds.push(control_kind_id.to_owned());
                    continue;
                }

                catalog = catalog.with_control_kind(UiProgramFormationControlContract {
                    kind_id: control_kind_id.to_owned(),
                    package_id: package_id.to_owned(),
                    display_name: kind.display_name.clone(),
                    property_schema: property_schema
                        .expect("property schema was validated before catalog insert"),
                    state_schema: kind.state_schema.clone(),
                    event_payload_schema: kind.event_payload_schema.clone(),
                    layout_kernel: ControlKernelRef::new(kind.kernels.layout.as_str()),
                    visual_kernel: ControlKernelRef::new(kind.kernels.visual.as_str()),
                    activation_capability,
                });
            }
        }

        UiProgramFormationCatalogReport {
            catalog,
            diagnostics,
            skipped_control_kinds,
        }
    }
}

struct ControlContractValidationContext<'a> {
    package: &'a ControlPackageDescriptor,
    control_kind_id: &'a str,
    diagnostics: &'a mut Vec<UiProgramDiagnostic>,
}

impl ControlContractValidationContext<'_> {
    fn validate_schema_ref(
        &mut self,
        contract_part: &str,
        expected_schema: &UiSchemaRef,
        expected_role: &str,
        diagnostic_code: &'static str,
        schemas: &[ControlSchemaDescriptor],
    ) -> Option<UiSchema> {
        let schema = schemas
            .iter()
            .find(|schema| schema.schema_ref() == expected_schema);

        if let Some(schema) = schema {
            return Some(schema.schema.clone());
        }

        self.diagnostics.push(UiProgramDiagnostic::new(
            diagnostic_code,
            format!(
                "package {} control kind {} contract part {} references schema {}@{}; expected schema role {}",
                self.package.package_id.as_str(),
                self.control_kind_id,
                contract_part,
                expected_schema.id.as_str(),
                expected_schema.version.value(),
                expected_role
            ),
        ));
        None
    }

    fn validate_kernel_ref(
        &mut self,
        contract_part: &str,
        expected_kernel: &ControlKernelId,
        expected_kind: ControlKernelKind,
        missing_diagnostic_code: &'static str,
    ) {
        let Some(kernel) = self
            .package
            .kernels
            .iter()
            .find(|kernel| &kernel.kernel_id == expected_kernel)
        else {
            self.diagnostics.push(UiProgramDiagnostic::new(
                missing_diagnostic_code,
                format!(
                    "package {} control kind {} contract part {} references kernel {}; expected kernel kind {:?}",
                    self.package.package_id.as_str(),
                    self.control_kind_id,
                    contract_part,
                    expected_kernel.as_str(),
                    expected_kind
                ),
            ));
            return;
        };

        self.validate_kernel_kind(contract_part, kernel, expected_kind);
    }

    fn validate_kernel_kind(
        &mut self,
        contract_part: &str,
        kernel: &ControlKernelDescriptor,
        expected_kind: ControlKernelKind,
    ) {
        if kernel.kind == expected_kind {
            return;
        }

        self.diagnostics.push(UiProgramDiagnostic::new(
            "ui.program.catalog.kernel_kind_mismatch",
            format!(
                "package {} control kind {} contract part {} references kernel {}; expected kernel kind {:?}, found {:?}",
                self.package.package_id.as_str(),
                self.control_kind_id,
                contract_part,
                kernel.kernel_id.as_str(),
                expected_kind,
                kernel.kind
            ),
        ));
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiProgramFormationControlContract {
    pub kind_id: String,
    pub package_id: String,
    pub display_name: String,
    pub property_schema: UiSchema,
    pub state_schema: UiSchemaRef,
    pub event_payload_schema: UiSchemaRef,
    pub layout_kernel: ControlKernelRef,
    pub visual_kernel: ControlKernelRef,
    pub activation_capability: RouteCapability,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiProgramFormationCatalogReport {
    pub catalog: UiProgramFormationControlCatalog,
    pub diagnostics: Vec<UiProgramDiagnostic>,
    pub skipped_control_kinds: Vec<String>,
}

impl UiProgramFormationCatalogReport {
    pub fn passed(&self) -> bool {
        self.diagnostics.is_empty() && self.skipped_control_kinds.is_empty()
    }

    pub fn has_diagnostic(&self, code: &str) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code)
    }
}
