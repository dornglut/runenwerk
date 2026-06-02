//! File: domain/ui/ui_controls/src/lib.rs
//! Crate: ui_controls

pub mod action_prompt;
pub mod button;
pub mod color_picker;
pub mod diagnostics;
pub mod inspector_field;
pub mod kernel;
pub mod label;
pub mod list_view;
pub mod migration;
pub mod package;
pub mod registry;
pub mod schema;
pub mod table_view;
pub mod tree_view;

pub use action_prompt::ACTION_PROMPT_CONTROL_KIND_ID;
pub use button::BUTTON_CONTROL_KIND_ID;
pub use color_picker::COLOR_PICKER_CONTROL_KIND_ID;
pub use diagnostics::*;
pub use inspector_field::INSPECTOR_FIELD_CONTROL_KIND_ID;
pub use kernel::*;
pub use label::LABEL_CONTROL_KIND_ID;
pub use list_view::LIST_VIEW_CONTROL_KIND_ID;
pub use migration::*;
pub use package::*;
pub use registry::*;
pub use schema::*;
pub use table_view::TABLE_VIEW_CONTROL_KIND_ID;
pub use tree_view::TREE_VIEW_CONTROL_KIND_ID;

use ui_program::RouteCapability;
use ui_schema::UiSchema;

pub const RUNENWERK_CONTROL_PACKAGE_ID: &str = "runenwerk.ui.controls";

pub fn runenwerk_control_package() -> ControlPackageDescriptor {
    ControlPackageDescriptor::from_modules(
        ControlPackageId::new(RUNENWERK_CONTROL_PACKAGE_ID),
        ControlPackageVersion::new(1),
        [
            label::control_module(),
            button::control_module(),
            inspector_field::control_module(),
            color_picker::control_module(),
            action_prompt::control_module(),
            list_view::control_module(),
            tree_view::control_module(),
            table_view::control_module(),
        ],
    )
}

pub(crate) fn control_module_contract(
    kind_suffix: &str,
    display_name: &str,
    property_schema: UiSchema,
    state_schema: UiSchema,
    event_payload_schema: UiSchema,
    capability: RouteCapability,
) -> ControlModuleDescriptor {
    let property_schema = ControlSchemaDescriptor::properties(property_schema);
    let state_schema = ControlSchemaDescriptor::state(state_schema);
    let event_payload_schema = ControlSchemaDescriptor::event_payload(event_payload_schema);

    let layout = ControlKernelDescriptor::new(
        ControlKernelId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.layout"
        )),
        ControlKernelKind::Layout,
    );
    let interaction = ControlKernelDescriptor::new(
        ControlKernelId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.interaction"
        )),
        ControlKernelKind::Interaction,
    );
    let visual = ControlKernelDescriptor::new(
        ControlKernelId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.visual"
        )),
        ControlKernelKind::Visual,
    );
    let accessibility = ControlKernelDescriptor::new(
        ControlKernelId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.accessibility"
        )),
        ControlKernelKind::Accessibility,
    );
    let inspection = ControlKernelDescriptor::new(
        ControlKernelId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.inspection"
        )),
        ControlKernelKind::Inspection,
    );
    let kernels = ControlKernelSet::new(
        layout.kernel_id.clone(),
        interaction.kernel_id.clone(),
        visual.kernel_id.clone(),
        accessibility.kernel_id.clone(),
        inspection.kernel_id.clone(),
    );
    let fixture_id = ControlFixtureId::new(format!(
        "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.fixture.default"
    ));
    let diagnostic = ControlDiagnosticDescriptor::new(
        ControlDiagnosticId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.diagnostic.contract"
        )),
        format!("{display_name} control package contract violation"),
    );
    let migration = ControlMigrationHook::initial(
        ControlMigrationId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.migration.initial"
        )),
        ControlPackageVersion::new(1),
    );
    let kind = ControlKindDescriptor::new(
        ControlKindId::new(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}")),
        display_name,
        property_schema.schema_ref().clone(),
        state_schema.schema_ref().clone(),
        event_payload_schema.schema_ref().clone(),
        kernels,
    )
    .with_required_capability(capability)
    .with_fixture(fixture_id)
    .with_diagnostic(diagnostic.diagnostic_id.clone())
    .with_migration(migration.migration_id.clone());

    ControlModuleDescriptor::new(kind)
        .with_schema(property_schema)
        .with_schema(state_schema)
        .with_schema(event_payload_schema)
        .with_kernel(layout)
        .with_kernel(interaction)
        .with_kernel(visual)
        .with_kernel(accessibility)
        .with_kernel(inspection)
        .with_diagnostic(diagnostic)
        .with_migration(migration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_package_records_schemas_kernels_diagnostics_and_migrations() {
        let package = runenwerk_control_package();

        assert_eq!(package.package_id.as_str(), RUNENWERK_CONTROL_PACKAGE_ID);
        assert_eq!(package.version.value(), 1);
        assert_eq!(package.control_kinds.len(), 8);
        assert_eq!(package.property_schemas.len(), 8);
        assert_eq!(package.state_schemas.len(), 8);
        assert_eq!(package.event_payload_schemas.len(), 8);
        assert_eq!(package.kernels.len(), 40);
        assert_eq!(package.kernel_ids.len(), 40);
        assert_eq!(package.diagnostics.len(), 8);
        assert_eq!(package.migrations.len(), 8);
        assert!(
            package
                .control_kind(&ControlKindId::new(COLOR_PICKER_CONTROL_KIND_ID))
                .is_some()
        );
        assert!(
            package
                .kernel_ids
                .iter()
                .any(|kernel_id| kernel_id.as_str() == "runenwerk.ui.controls.color-picker.visual")
        );
        assert!(
            package
                .migrations
                .iter()
                .all(|migration| migration.preserves_source_maps)
        );
    }

    #[test]
    fn control_package_registry_uses_explicit_snapshot() {
        let package = runenwerk_control_package();
        let registry = ControlPackageRegistry::new()
            .with_package(package)
            .expect("package should register once");
        let snapshot = registry.snapshot();

        assert_eq!(snapshot.packages.len(), 1);
        assert_eq!(snapshot.control_kinds.len(), 8);
        assert!(registry.contains_kind(&ControlKindId::new(BUTTON_CONTROL_KIND_ID)));
        assert!(
            registry
                .package(&ControlPackageId::new(RUNENWERK_CONTROL_PACKAGE_ID))
                .is_some()
        );
    }
}
