//! File: domain/ui/ui_program_lowering/src/catalog.rs
//! Crate: ui_program_lowering
//!
//! Control package metadata adapted into semantic UiProgram formation contracts.

use std::collections::BTreeMap;

use ui_program::{ControlKernelRef, RouteCapability, UiProgramDiagnostic, UiSchemaRef};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

    pub(crate) fn control_kind(&self, kind_id: &str) -> Option<&UiProgramFormationControlContract> {
        self.control_kinds.get(kind_id)
    }

    pub fn derive_from_control_package_registry_snapshot(
        snapshot: &ui_controls::ControlPackageRegistrySnapshot,
    ) -> UiProgramFormationCatalogReport {
        let mut catalog = Self::new();
        let mut diagnostics = Vec::new();
        let mut skipped_control_kinds = Vec::new();

        for package in &snapshot.packages {
            for kind in &package.control_kinds {
                let Some(activation_capability) = kind.required_capabilities.first().cloned()
                else {
                    let control_kind_id = kind.control_kind_id.as_str().to_owned();

                    diagnostics.push(UiProgramDiagnostic::new(
                        "ui.program.catalog.control_kind_missing_activation_capability",
                        format!(
                            "control kind {control_kind_id} has no activation capability and cannot be used for UiProgram formation"
                        ),
                    ));

                    skipped_control_kinds.push(control_kind_id);
                    continue;
                };

                catalog = catalog.with_control_kind(UiProgramFormationControlContract::new(
                    kind.control_kind_id.as_str(),
                    package.package_id.as_str(),
                    kind.display_name.clone(),
                    kind.property_schema.clone(),
                    kind.state_schema.clone(),
                    kind.event_payload_schema.clone(),
                    ControlKernelRef::new(kind.kernels.layout.as_str()),
                    ControlKernelRef::new(kind.kernels.visual.as_str()),
                    activation_capability,
                ));
            }
        }

        UiProgramFormationCatalogReport {
            catalog,
            diagnostics,
            skipped_control_kinds,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiProgramFormationControlContract {
    pub kind_id: String,
    pub package_id: String,
    pub display_name: String,
    pub property_schema: UiSchemaRef,
    pub state_schema: UiSchemaRef,
    pub event_payload_schema: UiSchemaRef,
    pub layout_kernel: ControlKernelRef,
    pub visual_kernel: ControlKernelRef,
    pub activation_capability: RouteCapability,
}

impl UiProgramFormationControlContract {
    pub fn new(
        kind_id: impl Into<String>,
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        property_schema: UiSchemaRef,
        state_schema: UiSchemaRef,
        event_payload_schema: UiSchemaRef,
        layout_kernel: ControlKernelRef,
        visual_kernel: ControlKernelRef,
        activation_capability: RouteCapability,
    ) -> Self {
        Self {
            kind_id: kind_id.into(),
            package_id: package_id.into(),
            display_name: display_name.into(),
            event_payload_schema,
            layout_kernel,
            state_schema,
            activation_capability,
            property_schema,
            visual_kernel,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
