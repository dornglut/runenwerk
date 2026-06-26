//! File: domain/ui/ui_controls/src/lib.rs
//! Crate: ui_controls

pub mod action_prompt;
pub mod button;
pub mod color_picker;
pub mod diagnostics;
pub mod input;
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
pub use input::*;
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

use ui_program::{RouteCapability, RouteId, RouteSchemaVersion};
use ui_schema::UiSchema;

pub const RUNENWERK_CONTROL_PACKAGE_ID: &str = "runenwerk.ui.controls";
pub const RUNENWERK_CONTROL_TARGET_EDITOR: &str = "runenwerk.ui.target.editor";

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
    .with_display_name("Runenwerk base UI controls")
    .with_description("Reusable descriptor package for Runenwerk base controls. Runtime mount eligibility remains disabled until story, render, and budget evidence are attached.")
    .with_category("base-controls")
    .with_tag("control-package")
    .with_tag("descriptor-only")
    .with_target_profile(ControlTargetProfileRef::new(RUNENWERK_CONTROL_TARGET_EDITOR))
    .with_catalog_metadata(ControlCatalogMetadata::new(RUNENWERK_CONTROL_PACKAGE_ID, "Base Controls"))
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
    let diagnostic_id = ControlDiagnosticId::new(format!(
        "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.diagnostic.contract"
    ));
    let migration = ControlMigrationHook::initial(
        ControlMigrationId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.migration.initial"
        )),
        ControlPackageVersion::new(1),
    );
    let story_id = ControlStoryId::new(format!(
        "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.story.contract"
    ));
    let render_evidence_id = ControlRenderEvidenceId::new(format!(
        "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.evidence.render.contract"
    ));
    let budget_evidence_id = ControlBudgetEvidenceId::new(format!(
        "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.evidence.budget.contract"
    ));
    let control_kind_id =
        ControlKindId::new(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}"));
    let route_requirement = ControlRouteRequirement::new(
        RouteId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.intent"
        )),
        RouteSchemaVersion::new(1),
    )
    .with_capability(capability.clone());
    let kind = ControlKindDescriptor::new(control_kind_id.clone(), display_name, property_schema.schema_ref().clone(), state_schema.schema_ref().clone(), event_payload_schema.schema_ref().clone(), kernels)
        .with_description(format!("{display_name} reusable control descriptor with schemas, kernels, diagnostics, fixture, story, host-intent route metadata, and explicit non-mount eligibility until story proof is attached."))
        .with_category("base-control")
        .with_tag(kind_suffix)
        .with_tag("descriptor-only")
        .with_target_profile(ControlTargetProfileRef::new(RUNENWERK_CONTROL_TARGET_EDITOR))
        .with_required_capability(capability)
        .with_route_requirement(route_requirement)
        .with_fixture(fixture_id.clone())
        .with_diagnostic(diagnostic_id.clone())
        .with_migration(migration.migration_id.clone())
        .with_story(story_id.clone())
        .with_mount_eligibility(ControlMountEligibility::not_eligible("runtime mount eligibility requires future story, render, and budget evidence"))
        .with_binding_requirement(ControlRequirement::new(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.binding.contract"), "Binding behavior must be declared through host-owned state and route contracts."))
        .with_theme_token_requirement(ControlRequirement::new(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.theme.contract"), "Visual states must use theme/token metadata rather than renderer-owned semantics."))
        .with_accessibility_requirement(ControlRequirement::new(format!("{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.accessibility.contract"), "Accessibility role, label, focus, and inspection facts must be explicit before mount eligibility."))
        .with_render_evidence_requirement(ControlRenderEvidenceRequirement::new(render_evidence_id, "Renderer-neutral primitive evidence required before runtime mount eligibility."))
        .with_budget_evidence_requirement(ControlBudgetEvidenceRequirement::new(budget_evidence_id, "Layout, interaction, text, and render budget evidence required before runtime mount eligibility."));

    ControlModuleDescriptor::new(kind)
        .with_schema(property_schema)
        .with_schema(state_schema)
        .with_schema(event_payload_schema)
        .with_kernel(layout)
        .with_kernel(interaction)
        .with_kernel(visual)
        .with_kernel(accessibility)
        .with_kernel(inspection)
        .with_fixture(ControlFixtureDescriptor::new(
            fixture_id,
            format!("Default {display_name} descriptor fixture"),
        ))
        .with_diagnostic(ControlDiagnosticDescriptor::contract(
            diagnostic_id,
            control_kind_id,
            format!("{display_name} control package contract violation"),
        ))
        .with_migration(migration)
        .with_story(ControlStoryDescriptor::new(
            story_id,
            format!("{display_name} descriptor contract story placeholder"),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_package_complete_contract_validates() {
        let package = runenwerk_control_package();
        let report = package.validate_contract();
        assert!(report.is_valid(), "{:?}", report.diagnostics);
        assert_eq!(package.control_kinds.len(), 8);
        assert_eq!(package.property_schemas.len(), 8);
        assert_eq!(package.state_schemas.len(), 8);
        assert_eq!(package.event_payload_schemas.len(), 8);
        assert_eq!(package.kernels.len(), 40);
        assert_eq!(package.fixtures.len(), 8);
        assert_eq!(package.diagnostics.len(), 8);
        assert_eq!(package.migrations.len(), 8);
        assert_eq!(package.stories.len(), 8);
    }

    #[test]
    fn control_package_rejects_missing_property_schema() {
        let mut package = runenwerk_control_package();
        package.property_schemas.clear();
        assert_has_reason(package, ControlPackageValidationReason::MissingSchema);
    }
    #[test]
    fn control_package_rejects_missing_state_schema() {
        let mut package = runenwerk_control_package();
        package.state_schemas.clear();
        assert_has_reason(package, ControlPackageValidationReason::MissingSchema);
    }
    #[test]
    fn control_package_rejects_missing_event_payload_schema() {
        let mut package = runenwerk_control_package();
        package.event_payload_schemas.clear();
        assert_has_reason(package, ControlPackageValidationReason::MissingSchema);
    }
    #[test]
    fn control_package_rejects_missing_layout_kernel() {
        let mut package = runenwerk_control_package();
        let missing = package.control_kinds[0].kernels.layout.clone();
        package.kernels.retain(|kernel| kernel.kernel_id != missing);
        assert_has_reason(package, ControlPackageValidationReason::MissingKernel);
    }
    #[test]
    fn control_package_rejects_missing_interaction_kernel() {
        let mut package = runenwerk_control_package();
        let missing = package.control_kinds[0].kernels.interaction.clone();
        package.kernels.retain(|kernel| kernel.kernel_id != missing);
        assert_has_reason(package, ControlPackageValidationReason::MissingKernel);
    }
    #[test]
    fn control_package_rejects_missing_visual_kernel() {
        let mut package = runenwerk_control_package();
        let missing = package.control_kinds[0].kernels.visual.clone();
        package.kernels.retain(|kernel| kernel.kernel_id != missing);
        assert_has_reason(package, ControlPackageValidationReason::MissingKernel);
    }
    #[test]
    fn control_package_rejects_missing_accessibility_kernel() {
        let mut package = runenwerk_control_package();
        let missing = package.control_kinds[0].kernels.accessibility.clone();
        package.kernels.retain(|kernel| kernel.kernel_id != missing);
        assert_has_reason(package, ControlPackageValidationReason::MissingKernel);
    }
    #[test]
    fn control_package_rejects_missing_inspection_kernel() {
        let mut package = runenwerk_control_package();
        let missing = package.control_kinds[0].kernels.inspection.clone();
        package.kernels.retain(|kernel| kernel.kernel_id != missing);
        assert_has_reason(package, ControlPackageValidationReason::MissingKernel);
    }
    #[test]
    fn control_package_rejects_duplicate_schema_ref() {
        let mut package = runenwerk_control_package();
        package
            .property_schemas
            .push(package.property_schemas[0].clone());
        assert_has_reason(package, ControlPackageValidationReason::DuplicateSchemaRef);
    }
    #[test]
    fn control_package_rejects_duplicate_kernel_id() {
        let mut package = runenwerk_control_package();
        package.kernels.push(package.kernels[0].clone());
        assert_has_reason(package, ControlPackageValidationReason::DuplicateKernelId);
    }
    #[test]
    fn control_package_rejects_duplicate_fixture_id() {
        let mut package = runenwerk_control_package();
        package.fixtures.push(package.fixtures[0].clone());
        assert_has_reason(package, ControlPackageValidationReason::DuplicateFixtureId);
    }
    #[test]
    fn control_package_rejects_duplicate_diagnostic_id() {
        let mut package = runenwerk_control_package();
        package.diagnostics.push(package.diagnostics[0].clone());
        assert_has_reason(
            package,
            ControlPackageValidationReason::DuplicateDiagnosticId,
        );
    }
    #[test]
    fn control_package_rejects_duplicate_migration_id() {
        let mut package = runenwerk_control_package();
        package.migrations.push(package.migrations[0].clone());
        assert_has_reason(
            package,
            ControlPackageValidationReason::DuplicateMigrationId,
        );
    }
    #[test]
    fn control_package_rejects_duplicate_story_id() {
        let mut package = runenwerk_control_package();
        package.stories.push(package.stories[0].clone());
        assert_has_reason(package, ControlPackageValidationReason::DuplicateStoryId);
    }
    #[test]
    fn runenwerk_control_package_validates() {
        assert!(runenwerk_control_package().validate_contract().is_valid());
    }

    fn assert_has_reason(
        package: ControlPackageDescriptor,
        reason: ControlPackageValidationReason,
    ) {
        let report = package.validate_contract();
        assert!(!report.is_valid(), "package unexpectedly valid");
        assert!(
            report.has_reason(reason),
            "expected reason {:?}, got {:?}",
            reason,
            report.diagnostics
        );
    }
}
