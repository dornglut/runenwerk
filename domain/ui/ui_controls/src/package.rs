//! File: domain/ui/ui_controls/src/package.rs
//! Crate: ui_controls

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use ui_program::{RouteCapability, RouteId, RouteSchemaVersion};
use ui_schema::{UiSchemaRef, UiSchemaShape};

use crate::diagnostics::{
    ControlDiagnosticDescriptor, ControlDiagnosticId, ControlDiagnosticKind, ControlDiagnosticScope,
    ControlDiagnosticSeverity,
};
use crate::kernel::{ControlKernelDescriptor, ControlKernelId, ControlKernelKind, ControlKernelSet};
use crate::migration::{ControlDeprecationStatus, ControlMigrationHook, ControlMigrationId};
use crate::schema::{ControlSchemaDescriptor, ControlSchemaRole};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlPackageId(String);

impl ControlPackageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control package IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "package")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlKindId(String);

impl ControlKindId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control kind IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "kind")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlFixtureId(String);

impl ControlFixtureId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control fixture IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "fixture")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlStoryId(String);

impl ControlStoryId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control story IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "story")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlRenderEvidenceId(String);

impl ControlRenderEvidenceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control render evidence IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "render evidence")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlBudgetEvidenceId(String);

impl ControlBudgetEvidenceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control budget evidence IDs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "budget evidence")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ControlTargetProfileRef(String);

impl ControlTargetProfileRef {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("control target profile refs must be namespaced")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, ControlPackageContractError> {
        let value = value.into();
        validate_control_id(&value, "target profile")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ControlPackageVersion(u32);

impl ControlPackageVersion {
    pub const fn new(value: u32) -> Self {
        assert!(value > 0);
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlPackageCategory(pub String);

impl Default for ControlPackageCategory {
    fn default() -> Self {
        Self("uncategorized".to_owned())
    }
}

impl ControlPackageCategory {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTag(pub String);

impl ControlTag {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlCompatibilityFlags {
    pub supports_story_proof: bool,
    pub supports_gallery_inspection: bool,
    pub supports_workbench_consumption: bool,
    pub supports_designer_consumption: bool,
    pub supports_runtime_mount: bool,
}

impl ControlCompatibilityFlags {
    pub const fn descriptor_only() -> Self {
        Self {
            supports_story_proof: true,
            supports_gallery_inspection: true,
            supports_workbench_consumption: true,
            supports_designer_consumption: true,
            supports_runtime_mount: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlCatalogMetadata {
    pub sort_key: String,
    pub group: String,
    #[serde(default)]
    pub discoverable: bool,
}

impl ControlCatalogMetadata {
    pub fn new(sort_key: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            sort_key: sort_key.into(),
            group: group.into(),
            discoverable: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlFixtureDescriptor {
    pub fixture_id: ControlFixtureId,
    pub description: String,
}

impl ControlFixtureDescriptor {
    pub fn new(fixture_id: ControlFixtureId, description: impl Into<String>) -> Self {
        Self {
            fixture_id,
            description: description.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlStoryDescriptor {
    pub story_id: ControlStoryId,
    pub description: String,
    #[serde(default)]
    pub requires_runtime_evidence: bool,
}

impl ControlStoryDescriptor {
    pub fn new(story_id: ControlStoryId, description: impl Into<String>) -> Self {
        Self {
            story_id,
            description: description.into(),
            requires_runtime_evidence: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlRenderEvidenceRequirement {
    pub evidence_id: ControlRenderEvidenceId,
    pub description: String,
}

impl ControlRenderEvidenceRequirement {
    pub fn new(evidence_id: ControlRenderEvidenceId, description: impl Into<String>) -> Self {
        Self {
            evidence_id,
            description: description.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlBudgetEvidenceRequirement {
    pub evidence_id: ControlBudgetEvidenceId,
    pub description: String,
}

impl ControlBudgetEvidenceRequirement {
    pub fn new(evidence_id: ControlBudgetEvidenceId, description: impl Into<String>) -> Self {
        Self {
            evidence_id,
            description: description.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlRequirement {
    pub requirement_id: String,
    pub description: String,
}

impl ControlRequirement {
    pub fn new(requirement_id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            requirement_id: requirement_id.into(),
            description: description.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlRouteRequirement {
    pub route_id: RouteId,
    pub schema_version: RouteSchemaVersion,
    #[serde(default)]
    pub capabilities: Vec<RouteCapability>,
}

impl ControlRouteRequirement {
    pub fn new(route_id: RouteId, schema_version: RouteSchemaVersion) -> Self {
        Self {
            route_id,
            schema_version,
            capabilities: Vec::new(),
        }
    }

    pub fn with_capability(mut self, capability: RouteCapability) -> Self {
        self.capabilities.push(capability);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMountEligibility {
    NotEligible { reason: String },
    RequiresEvidence {
        story_ids: Vec<ControlStoryId>,
        render_evidence_ids: Vec<ControlRenderEvidenceId>,
        budget_evidence_ids: Vec<ControlBudgetEvidenceId>,
    },
}

impl Default for ControlMountEligibility {
    fn default() -> Self {
        Self::NotEligible {
            reason: "story proof and runtime evidence are not attached yet".to_owned(),
        }
    }
}

impl ControlMountEligibility {
    pub fn not_eligible(reason: impl Into<String>) -> Self {
        Self::NotEligible {
            reason: reason.into(),
        }
    }

    pub fn requires_evidence(
        story_ids: impl IntoIterator<Item = ControlStoryId>,
        render_evidence_ids: impl IntoIterator<Item = ControlRenderEvidenceId>,
        budget_evidence_ids: impl IntoIterator<Item = ControlBudgetEvidenceId>,
    ) -> Self {
        Self::RequiresEvidence {
            story_ids: story_ids.into_iter().collect(),
            render_evidence_ids: render_evidence_ids.into_iter().collect(),
            budget_evidence_ids: budget_evidence_ids.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlKindDescriptor {
    pub control_kind_id: ControlKindId,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: ControlPackageCategory,
    #[serde(default)]
    pub tags: Vec<ControlTag>,
    #[serde(default)]
    pub target_profiles: Vec<ControlTargetProfileRef>,
    #[serde(default)]
    pub compatibility: ControlCompatibilityFlags,
    pub property_schema: UiSchemaRef,
    pub state_schema: UiSchemaRef,
    pub event_payload_schema: UiSchemaRef,
    pub kernels: ControlKernelSet,
    #[serde(default)]
    pub required_capabilities: Vec<RouteCapability>,
    #[serde(default)]
    pub route_requirements: Vec<ControlRouteRequirement>,
    #[serde(default)]
    pub fixture_ids: Vec<ControlFixtureId>,
    #[serde(default)]
    pub diagnostic_ids: Vec<ControlDiagnosticId>,
    #[serde(default)]
    pub migration_ids: Vec<ControlMigrationId>,
    #[serde(default)]
    pub story_ids: Vec<ControlStoryId>,
    #[serde(default)]
    pub mount_eligibility: ControlMountEligibility,
    #[serde(default)]
    pub binding_requirements: Vec<ControlRequirement>,
    #[serde(default)]
    pub theme_token_requirements: Vec<ControlRequirement>,
    #[serde(default)]
    pub accessibility_requirements: Vec<ControlRequirement>,
    #[serde(default)]
    pub render_evidence_requirements: Vec<ControlRenderEvidenceRequirement>,
    #[serde(default)]
    pub budget_evidence_requirements: Vec<ControlBudgetEvidenceRequirement>,
    #[serde(default)]
    pub deprecation: ControlDeprecationStatus,
}

impl ControlKindDescriptor {
    pub fn new(
        control_kind_id: ControlKindId,
        display_name: impl Into<String>,
        property_schema: UiSchemaRef,
        state_schema: UiSchemaRef,
        event_payload_schema: UiSchemaRef,
        kernels: ControlKernelSet,
    ) -> Self {
        Self {
            control_kind_id,
            display_name: display_name.into(),
            description: String::new(),
            category: ControlPackageCategory::default(),
            tags: Vec::new(),
            target_profiles: Vec::new(),
            compatibility: ControlCompatibilityFlags::descriptor_only(),
            property_schema,
            state_schema,
            event_payload_schema,
            kernels,
            required_capabilities: Vec::new(),
            route_requirements: Vec::new(),
            fixture_ids: Vec::new(),
            diagnostic_ids: Vec::new(),
            migration_ids: Vec::new(),
            story_ids: Vec::new(),
            mount_eligibility: ControlMountEligibility::default(),
            binding_requirements: Vec::new(),
            theme_token_requirements: Vec::new(),
            accessibility_requirements: Vec::new(),
            render_evidence_requirements: Vec::new(),
            budget_evidence_requirements: Vec::new(),
            deprecation: ControlDeprecationStatus::Active,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = ControlPackageCategory::new(category);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(ControlTag::new(tag));
        self
    }

    pub fn with_target_profile(mut self, target_profile: ControlTargetProfileRef) -> Self {
        self.target_profiles.push(target_profile);
        self
    }

    pub fn with_required_capability(mut self, capability: RouteCapability) -> Self {
        self.required_capabilities.push(capability);
        self
    }

    pub fn with_route_requirement(mut self, route_requirement: ControlRouteRequirement) -> Self {
        self.route_requirements.push(route_requirement);
        self
    }

    pub fn with_fixture(mut self, fixture_id: ControlFixtureId) -> Self {
        self.fixture_ids.push(fixture_id);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic_id: ControlDiagnosticId) -> Self {
        self.diagnostic_ids.push(diagnostic_id);
        self
    }

    pub fn with_migration(mut self, migration_id: ControlMigrationId) -> Self {
        self.migration_ids.push(migration_id);
        self
    }

    pub fn with_story(mut self, story_id: ControlStoryId) -> Self {
        self.story_ids.push(story_id);
        self
    }

    pub fn with_mount_eligibility(mut self, mount_eligibility: ControlMountEligibility) -> Self {
        self.mount_eligibility = mount_eligibility;
        self
    }

    pub fn with_binding_requirement(mut self, requirement: ControlRequirement) -> Self {
        self.binding_requirements.push(requirement);
        self
    }

    pub fn with_theme_token_requirement(mut self, requirement: ControlRequirement) -> Self {
        self.theme_token_requirements.push(requirement);
        self
    }

    pub fn with_accessibility_requirement(mut self, requirement: ControlRequirement) -> Self {
        self.accessibility_requirements.push(requirement);
        self
    }

    pub fn with_render_evidence_requirement(
        mut self,
        requirement: ControlRenderEvidenceRequirement,
    ) -> Self {
        self.render_evidence_requirements.push(requirement);
        self
    }

    pub fn with_budget_evidence_requirement(
        mut self,
        requirement: ControlBudgetEvidenceRequirement,
    ) -> Self {
        self.budget_evidence_requirements.push(requirement);
        self
    }

    pub fn with_deprecation(mut self, deprecation: ControlDeprecationStatus) -> Self {
        self.deprecation = deprecation;
        self
    }

    pub fn validate_contract(&self) -> ControlPackageValidationReport {
        let mut report = ControlPackageValidationReport::new();
        if self.display_name.trim().is_empty() {
            report.push(ControlPackageValidationDiagnostic::kind(
                self.control_kind_id.clone(),
                ControlPackageValidationReason::MissingKindMetadata,
                "control kind display name must not be empty",
            ));
        }
        if self.description.trim().is_empty() {
            report.push(ControlPackageValidationDiagnostic::kind(
                self.control_kind_id.clone(),
                ControlPackageValidationReason::MissingKindMetadata,
                "control kind description must not be empty",
            ));
        }
        if self.target_profiles.is_empty() {
            report.push(ControlPackageValidationDiagnostic::kind(
                self.control_kind_id.clone(),
                ControlPackageValidationReason::MissingTargetProfile,
                "control kind must name at least one target profile",
            ));
        }
        if self.diagnostic_ids.is_empty() {
            report.push(ControlPackageValidationDiagnostic::kind(
                self.control_kind_id.clone(),
                ControlPackageValidationReason::MissingDiagnostic,
                "control kind must name at least one contract diagnostic",
            ));
        }
        if self.fixture_ids.is_empty() {
            report.push(ControlPackageValidationDiagnostic::kind(
                self.control_kind_id.clone(),
                ControlPackageValidationReason::MissingFixture,
                "control kind must name at least one fixture",
            ));
        }
        if self.route_requirements.is_empty() {
            report.push(ControlPackageValidationDiagnostic::kind(
                self.control_kind_id.clone(),
                ControlPackageValidationReason::MissingRoute,
                "control kind must name at least one host-intent route requirement",
            ));
        }
        for diagnostic in duplicate_values(
            "target_profile",
            self.target_profiles.iter().map(|profile| profile.as_str().to_owned()),
            &self.control_kind_id,
            ControlPackageValidationReason::UnsupportedTargetProfile,
        ) {
            report.push(diagnostic);
        }
        for diagnostic in duplicate_values(
            "route",
            self.route_requirements
                .iter()
                .map(|route| route.route_id.as_str().to_owned()),
            &self.control_kind_id,
            ControlPackageValidationReason::DuplicateRouteRequirement,
        ) {
            report.push(diagnostic);
        }
        for diagnostic in duplicate_values(
            "story",
            self.story_ids.iter().map(|story| story.as_str().to_owned()),
            &self.control_kind_id,
            ControlPackageValidationReason::DuplicateStoryId,
        ) {
            report.push(diagnostic);
        }
        if let ControlMountEligibility::RequiresEvidence {
            story_ids,
            render_evidence_ids,
            budget_evidence_ids,
        } = &self.mount_eligibility
        {
            if story_ids.is_empty() {
                report.push(ControlPackageValidationDiagnostic::kind(
                    self.control_kind_id.clone(),
                    ControlPackageValidationReason::MissingMountEvidence,
                    "mount eligibility requires at least one story id",
                ));
            }
            if render_evidence_ids.is_empty() {
                report.push(ControlPackageValidationDiagnostic::kind(
                    self.control_kind_id.clone(),
                    ControlPackageValidationReason::RenderEvidenceMissing,
                    "mount eligibility requires render evidence",
                ));
            }
            if budget_evidence_ids.is_empty() {
                report.push(ControlPackageValidationDiagnostic::kind(
                    self.control_kind_id.clone(),
                    ControlPackageValidationReason::BudgetEvidenceMissing,
                    "mount eligibility requires budget evidence",
                ));
            }
        }
        report
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlModuleDescriptor {
    pub kind: ControlKindDescriptor,
    #[serde(default)]
    pub schemas: Vec<ControlSchemaDescriptor>,
    #[serde(default)]
    pub kernels: Vec<ControlKernelDescriptor>,
    #[serde(default)]
    pub fixtures: Vec<ControlFixtureDescriptor>,
    #[serde(default)]
    pub diagnostics: Vec<ControlDiagnosticDescriptor>,
    #[serde(default)]
    pub migrations: Vec<ControlMigrationHook>,
    #[serde(default)]
    pub stories: Vec<ControlStoryDescriptor>,
}

impl ControlModuleDescriptor {
    pub fn new(kind: ControlKindDescriptor) -> Self {
        Self {
            kind,
            schemas: Vec::new(),
            kernels: Vec::new(),
            fixtures: Vec::new(),
            diagnostics: Vec::new(),
            migrations: Vec::new(),
            stories: Vec::new(),
        }
    }

    pub fn with_schema(mut self, schema: ControlSchemaDescriptor) -> Self {
        self.schemas.push(schema);
        self
    }

    pub fn with_kernel(mut self, kernel: ControlKernelDescriptor) -> Self {
        self.kernels.push(kernel);
        self
    }

    pub fn with_fixture(mut self, fixture: ControlFixtureDescriptor) -> Self {
        self.fixtures.push(fixture);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: ControlDiagnosticDescriptor) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    pub fn with_migration(mut self, migration: ControlMigrationHook) -> Self {
        self.migrations.push(migration);
        self
    }

    pub fn with_story(mut self, story: ControlStoryDescriptor) -> Self {
        self.stories.push(story);
        self
    }

    pub fn validate_contract(&self) -> ControlPackageValidationReport {
        ControlPackageDescriptor::from_modules(
            ControlPackageId::new("runenwerk.validation.module"),
            ControlPackageVersion::new(1),
            [self.clone()],
        )
        .with_display_name("Module validation")
        .with_description("Synthetic package for validating one control module descriptor.")
        .with_category("validation")
        .with_target_profile(ControlTargetProfileRef::new("runenwerk.validation.profile"))
        .validate_contract()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlPackageDescriptor {
    pub package_id: ControlPackageId,
    pub version: ControlPackageVersion,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: ControlPackageCategory,
    #[serde(default)]
    pub tags: Vec<ControlTag>,
    #[serde(default)]
    pub target_profiles: Vec<ControlTargetProfileRef>,
    #[serde(default)]
    pub compatibility: ControlCompatibilityFlags,
    #[serde(default)]
    pub catalog_metadata: ControlCatalogMetadata,
    #[serde(default)]
    pub control_kinds: Vec<ControlKindDescriptor>,
    #[serde(default)]
    pub property_schemas: Vec<ControlSchemaDescriptor>,
    #[serde(default)]
    pub state_schemas: Vec<ControlSchemaDescriptor>,
    #[serde(default)]
    pub event_payload_schemas: Vec<ControlSchemaDescriptor>,
    #[serde(default)]
    pub required_capabilities: Vec<RouteCapability>,
    #[serde(default)]
    pub route_requirements: Vec<ControlRouteRequirement>,
    #[serde(default)]
    pub kernels: Vec<ControlKernelDescriptor>,
    #[serde(default)]
    pub kernel_ids: Vec<ControlKernelId>,
    #[serde(default)]
    pub fixture_ids: Vec<ControlFixtureId>,
    #[serde(default)]
    pub fixtures: Vec<ControlFixtureDescriptor>,
    #[serde(default)]
    pub diagnostics: Vec<ControlDiagnosticDescriptor>,
    #[serde(default)]
    pub diagnostic_ids: Vec<ControlDiagnosticId>,
    #[serde(default)]
    pub migrations: Vec<ControlMigrationHook>,
    #[serde(default)]
    pub migration_ids: Vec<ControlMigrationId>,
    #[serde(default)]
    pub story_ids: Vec<ControlStoryId>,
    #[serde(default)]
    pub stories: Vec<ControlStoryDescriptor>,
    #[serde(default)]
    pub mount_eligibility: ControlMountEligibility,
    #[serde(default)]
    pub binding_requirements: Vec<ControlRequirement>,
    #[serde(default)]
    pub theme_token_requirements: Vec<ControlRequirement>,
    #[serde(default)]
    pub accessibility_requirements: Vec<ControlRequirement>,
    #[serde(default)]
    pub render_evidence_requirements: Vec<ControlRenderEvidenceRequirement>,
    #[serde(default)]
    pub budget_evidence_requirements: Vec<ControlBudgetEvidenceRequirement>,
    #[serde(default)]
    pub deprecation: ControlDeprecationStatus,
}

impl ControlPackageDescriptor {
    pub fn new(package_id: ControlPackageId, version: ControlPackageVersion) -> Self {
        Self {
            package_id,
            version,
            display_name: String::new(),
            description: String::new(),
            category: ControlPackageCategory::default(),
            tags: Vec::new(),
            target_profiles: Vec::new(),
            compatibility: ControlCompatibilityFlags::descriptor_only(),
            catalog_metadata: ControlCatalogMetadata::default(),
            control_kinds: Vec::new(),
            property_schemas: Vec::new(),
            state_schemas: Vec::new(),
            event_payload_schemas: Vec::new(),
            required_capabilities: Vec::new(),
            route_requirements: Vec::new(),
            kernels: Vec::new(),
            kernel_ids: Vec::new(),
            fixture_ids: Vec::new(),
            fixtures: Vec::new(),
            diagnostics: Vec::new(),
            diagnostic_ids: Vec::new(),
            migrations: Vec::new(),
            migration_ids: Vec::new(),
            story_ids: Vec::new(),
            stories: Vec::new(),
            mount_eligibility: ControlMountEligibility::default(),
            binding_requirements: Vec::new(),
            theme_token_requirements: Vec::new(),
            accessibility_requirements: Vec::new(),
            render_evidence_requirements: Vec::new(),
            budget_evidence_requirements: Vec::new(),
            deprecation: ControlDeprecationStatus::Active,
        }
    }

    pub fn from_modules(
        package_id: ControlPackageId,
        version: ControlPackageVersion,
        modules: impl IntoIterator<Item = ControlModuleDescriptor>,
    ) -> Self {
        let mut package = Self::new(package_id, version);
        for module in modules {
            package = package.with_module(module);
        }
        package
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = ControlPackageCategory::new(category);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(ControlTag::new(tag));
        self
    }

    pub fn with_target_profile(mut self, target_profile: ControlTargetProfileRef) -> Self {
        self.target_profiles.push(target_profile);
        self
    }

    pub fn with_catalog_metadata(mut self, catalog_metadata: ControlCatalogMetadata) -> Self {
        self.catalog_metadata = catalog_metadata;
        self
    }

    pub fn with_mount_eligibility(mut self, mount_eligibility: ControlMountEligibility) -> Self {
        self.mount_eligibility = mount_eligibility;
        self
    }

    pub fn with_module(mut self, module: ControlModuleDescriptor) -> Self {
        let ControlModuleDescriptor {
            kind,
            schemas,
            kernels,
            fixtures,
            diagnostics,
            migrations,
            stories,
        } = module;

        self.required_capabilities
            .extend(kind.required_capabilities.iter().cloned());
        self.route_requirements
            .extend(kind.route_requirements.iter().cloned());
        self.fixture_ids.extend(kind.fixture_ids.iter().cloned());
        self.diagnostic_ids
            .extend(kind.diagnostic_ids.iter().cloned());
        self.migration_ids
            .extend(kind.migration_ids.iter().cloned());
        self.story_ids.extend(kind.story_ids.iter().cloned());
        self.binding_requirements
            .extend(kind.binding_requirements.iter().cloned());
        self.theme_token_requirements
            .extend(kind.theme_token_requirements.iter().cloned());
        self.accessibility_requirements
            .extend(kind.accessibility_requirements.iter().cloned());
        self.render_evidence_requirements
            .extend(kind.render_evidence_requirements.iter().cloned());
        self.budget_evidence_requirements
            .extend(kind.budget_evidence_requirements.iter().cloned());

        for kernel_id in kind.kernels.ids() {
            self.kernel_ids.push(kernel_id.clone());
        }

        for schema in schemas {
            match schema.role {
                ControlSchemaRole::Properties => self.property_schemas.push(schema),
                ControlSchemaRole::State => self.state_schemas.push(schema),
                ControlSchemaRole::EventPayload => self.event_payload_schemas.push(schema),
            }
        }

        self.kernels.extend(kernels);
        self.fixtures.extend(fixtures);
        self.diagnostics.extend(diagnostics);
        self.migrations.extend(migrations);
        self.stories.extend(stories);
        self.control_kinds.push(kind);
        self
    }

    pub fn control_kind(&self, id: &ControlKindId) -> Option<&ControlKindDescriptor> {
        self.control_kinds
            .iter()
            .find(|kind| &kind.control_kind_id == id)
    }

    pub fn validated(self) -> Result<Self, ControlPackageValidationReport> {
        let report = self.validate_contract();
        if report.is_valid() {
            Ok(self)
        } else {
            Err(report)
        }
    }

    pub fn validate_contract(&self) -> ControlPackageValidationReport {
        let mut report = ControlPackageValidationReport::new();
        if self.display_name.trim().is_empty() {
            report.push(ControlPackageValidationDiagnostic::package(
                self.package_id.clone(),
                ControlPackageValidationReason::MissingPackageMetadata,
                "control package display name must not be empty",
            ));
        }
        if self.description.trim().is_empty() {
            report.push(ControlPackageValidationDiagnostic::package(
                self.package_id.clone(),
                ControlPackageValidationReason::MissingPackageMetadata,
                "control package description must not be empty",
            ));
        }
        if self.target_profiles.is_empty() {
            report.push(ControlPackageValidationDiagnostic::package(
                self.package_id.clone(),
                ControlPackageValidationReason::MissingTargetProfile,
                "control package must name at least one target profile",
            ));
        }
        if self.control_kinds.is_empty() {
            report.push(ControlPackageValidationDiagnostic::package(
                self.package_id.clone(),
                ControlPackageValidationReason::MissingKindMetadata,
                "control package must contain at least one control kind",
            ));
        }

        self.validate_duplicate_package_lists(&mut report);
        self.validate_deprecation(&mut report);

        let property_schemas = schema_index(&self.property_schemas);
        let state_schemas = schema_index(&self.state_schemas);
        let event_payload_schemas = schema_index(&self.event_payload_schemas);
        let kernels = kernel_index(&self.kernels);
        let fixture_ids = self
            .fixtures
            .iter()
            .map(|fixture| fixture.fixture_id.as_str().to_owned())
            .collect::<BTreeSet<_>>();
        let diagnostic_ids = self
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.diagnostic_id.as_str().to_owned())
            .collect::<BTreeSet<_>>();
        let migration_ids = self
            .migrations
            .iter()
            .map(|migration| migration.migration_id.as_str().to_owned())
            .collect::<BTreeSet<_>>();
        let story_ids = self
            .stories
            .iter()
            .map(|story| story.story_id.as_str().to_owned())
            .collect::<BTreeSet<_>>();
        let render_evidence_ids = self
            .render_evidence_requirements
            .iter()
            .map(|evidence| evidence.evidence_id.as_str().to_owned())
            .collect::<BTreeSet<_>>();
        let budget_evidence_ids = self
            .budget_evidence_requirements
            .iter()
            .map(|evidence| evidence.evidence_id.as_str().to_owned())
            .collect::<BTreeSet<_>>();

        for kind in &self.control_kinds {
            report.extend(kind.validate_contract());
            self.validate_kind_references(
                kind,
                &property_schemas,
                &state_schemas,
                &event_payload_schemas,
                &kernels,
                &fixture_ids,
                &diagnostic_ids,
                &migration_ids,
                &story_ids,
                &render_evidence_ids,
                &budget_evidence_ids,
                &mut report,
            );
        }

        for migration in &self.migrations {
            if let Err(message) = migration.validate_contract(&self.package_id) {
                report.push(ControlPackageValidationDiagnostic::package(
                    self.package_id.clone(),
                    ControlPackageValidationReason::InvalidMigrationVersion,
                    message,
                ));
            }
        }
        for diagnostic in &self.diagnostics {
            if diagnostic.message_template.trim().is_empty() {
                report.push(ControlPackageValidationDiagnostic::package(
                    self.package_id.clone(),
                    ControlPackageValidationReason::MissingDiagnostic,
                    "control diagnostic message template must not be empty",
                ));
            }
        }
        report
    }

    fn validate_duplicate_package_lists(&self, report: &mut ControlPackageValidationReport) {
        push_duplicate_package_values(
            report,
            &self.package_id,
            "control_kind",
            self.control_kinds
                .iter()
                .map(|kind| kind.control_kind_id.as_str().to_owned()),
            ControlPackageValidationReason::DuplicateControlKindId,
        );
        push_duplicate_package_values(
            report,
            &self.package_id,
            "property_schema",
            self.property_schemas.iter().map(|schema| schema_ref_key(schema.schema_ref())),
            ControlPackageValidationReason::DuplicateSchemaRef,
        );
        push_duplicate_package_values(
            report,
            &self.package_id,
            "state_schema",
            self.state_schemas.iter().map(|schema| schema_ref_key(schema.schema_ref())),
            ControlPackageValidationReason::DuplicateSchemaRef,
        );
        push_duplicate_package_values(
            report,
            &self.package_id,
            "event_payload_schema",
            self.event_payload_schemas
                .iter()
                .map(|schema| schema_ref_key(schema.schema_ref())),
            ControlPackageValidationReason::DuplicateSchemaRef,
        );
        push_duplicate_package_values(
            report,
            &self.package_id,
            "kernel",
            self.kernels.iter().map(|kernel| kernel.kernel_id.as_str().to_owned()),
            ControlPackageValidationReason::DuplicateKernelId,
        );
        push_duplicate_package_values(
            report,
            &self.package_id,
            "fixture",
            self.fixtures.iter().map(|fixture| fixture.fixture_id.as_str().to_owned()),
            ControlPackageValidationReason::DuplicateFixtureId,
        );
        push_duplicate_package_values(
            report,
            &self.package_id,
            "diagnostic",
            self.diagnostics
                .iter()
                .map(|diagnostic| diagnostic.diagnostic_id.as_str().to_owned()),
            ControlPackageValidationReason::DuplicateDiagnosticId,
        );
        push_duplicate_package_values(
            report,
            &self.package_id,
            "migration",
            self.migrations
                .iter()
                .map(|migration| migration.migration_id.as_str().to_owned()),
            ControlPackageValidationReason::DuplicateMigrationId,
        );
        push_duplicate_package_values(
            report,
            &self.package_id,
            "story",
            self.stories.iter().map(|story| story.story_id.as_str().to_owned()),
            ControlPackageValidationReason::DuplicateStoryId,
        );
    }

    fn validate_deprecation(&self, report: &mut ControlPackageValidationReport) {
        if self.deprecation.replacement_package_id() == Some(&self.package_id) {
            report.push(ControlPackageValidationDiagnostic::package(
                self.package_id.clone(),
                ControlPackageValidationReason::InvalidDeprecation,
                "control package deprecation replacement must not point at itself",
            ));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_kind_references(
        &self,
        kind: &ControlKindDescriptor,
        property_schemas: &BTreeMap<String, &ControlSchemaDescriptor>,
        state_schemas: &BTreeMap<String, &ControlSchemaDescriptor>,
        event_payload_schemas: &BTreeMap<String, &ControlSchemaDescriptor>,
        kernels: &BTreeMap<String, &ControlKernelDescriptor>,
        fixture_ids: &BTreeSet<String>,
        diagnostic_ids: &BTreeSet<String>,
        migration_ids: &BTreeSet<String>,
        story_ids: &BTreeSet<String>,
        render_evidence_ids: &BTreeSet<String>,
        budget_evidence_ids: &BTreeSet<String>,
        report: &mut ControlPackageValidationReport,
    ) {
        if !property_schemas.contains_key(&schema_ref_key(&kind.property_schema)) {
            report.push(ControlPackageValidationDiagnostic::kind(
                kind.control_kind_id.clone(),
                ControlPackageValidationReason::MissingSchema,
                "control kind property schema ref does not resolve to a properties schema descriptor",
            ));
        }
        if !state_schemas.contains_key(&schema_ref_key(&kind.state_schema)) {
            report.push(ControlPackageValidationDiagnostic::kind(
                kind.control_kind_id.clone(),
                ControlPackageValidationReason::MissingSchema,
                "control kind state schema ref does not resolve to a state schema descriptor",
            ));
        }
        let event_schema = event_payload_schemas.get(&schema_ref_key(&kind.event_payload_schema));
        if event_schema.is_none() {
            report.push(ControlPackageValidationDiagnostic::kind(
                kind.control_kind_id.clone(),
                ControlPackageValidationReason::MissingSchema,
                "control kind event payload schema ref does not resolve to an event payload schema descriptor",
            ));
        }
        for (kernel_id, expected_kind) in [
            (&kind.kernels.layout, ControlKernelKind::Layout),
            (&kind.kernels.interaction, ControlKernelKind::Interaction),
            (&kind.kernels.visual, ControlKernelKind::Visual),
            (&kind.kernels.accessibility, ControlKernelKind::Accessibility),
            (&kind.kernels.inspection, ControlKernelKind::Inspection),
        ] {
            match kernels.get(kernel_id.as_str()) {
                Some(kernel) if kernel.kind == expected_kind => {}
                Some(_) => report.push(ControlPackageValidationDiagnostic::kind(
                    kind.control_kind_id.clone(),
                    ControlPackageValidationReason::MissingKernel,
                    format!(
                        "control kind kernel {} exists but does not match expected role {:?}",
                        kernel_id.as_str(),
                        expected_kind
                    ),
                )),
                None => report.push(ControlPackageValidationDiagnostic::kind(
                    kind.control_kind_id.clone(),
                    ControlPackageValidationReason::MissingKernel,
                    format!(
                        "control kind kernel {} does not resolve to a kernel descriptor",
                        kernel_id.as_str()
                    ),
                )),
            }
        }
        for fixture_id in &kind.fixture_ids {
            if !fixture_ids.contains(fixture_id.as_str()) {
                report.push(ControlPackageValidationDiagnostic::kind(
                    kind.control_kind_id.clone(),
                    ControlPackageValidationReason::MissingFixture,
                    format!(
                        "control kind fixture {} does not resolve to a fixture descriptor",
                        fixture_id.as_str()
                    ),
                ));
            }
        }
        for diagnostic_id in &kind.diagnostic_ids {
            if !diagnostic_ids.contains(diagnostic_id.as_str()) {
                report.push(ControlPackageValidationDiagnostic::kind(
                    kind.control_kind_id.clone(),
                    ControlPackageValidationReason::MissingDiagnostic,
                    format!(
                        "control kind diagnostic {} does not resolve to a diagnostic descriptor",
                        diagnostic_id.as_str()
                    ),
                ));
            }
        }
        for migration_id in &kind.migration_ids {
            if !migration_ids.contains(migration_id.as_str()) {
                report.push(ControlPackageValidationDiagnostic::kind(
                    kind.control_kind_id.clone(),
                    ControlPackageValidationReason::MissingMigration,
                    format!(
                        "control kind migration {} does not resolve to a migration hook",
                        migration_id.as_str()
                    ),
                ));
            }
        }
        for story_id in &kind.story_ids {
            if !story_ids.contains(story_id.as_str()) {
                report.push(ControlPackageValidationDiagnostic::kind(
                    kind.control_kind_id.clone(),
                    ControlPackageValidationReason::MissingStory,
                    format!(
                        "control kind story {} does not resolve to a story descriptor",
                        story_id.as_str()
                    ),
                ));
            }
        }
        if let ControlMountEligibility::RequiresEvidence {
            story_ids: mount_story_ids,
            render_evidence_ids: mount_render_evidence_ids,
            budget_evidence_ids: mount_budget_evidence_ids,
        } = &kind.mount_eligibility
        {
            for story_id in mount_story_ids {
                if !story_ids.contains(story_id.as_str()) {
                    report.push(ControlPackageValidationDiagnostic::kind(
                        kind.control_kind_id.clone(),
                        ControlPackageValidationReason::MissingStory,
                        format!(
                            "mount eligibility story {} does not resolve to a story descriptor",
                            story_id.as_str()
                        ),
                    ));
                }
            }
            for evidence_id in mount_render_evidence_ids {
                if !render_evidence_ids.contains(evidence_id.as_str()) {
                    report.push(ControlPackageValidationDiagnostic::kind(
                        kind.control_kind_id.clone(),
                        ControlPackageValidationReason::RenderEvidenceMissing,
                        format!(
                            "mount eligibility render evidence {} does not resolve to a render evidence requirement",
                            evidence_id.as_str()
                        ),
                    ));
                }
            }
            for evidence_id in mount_budget_evidence_ids {
                if !budget_evidence_ids.contains(evidence_id.as_str()) {
                    report.push(ControlPackageValidationDiagnostic::kind(
                        kind.control_kind_id.clone(),
                        ControlPackageValidationReason::BudgetEvidenceMissing,
                        format!(
                            "mount eligibility budget evidence {} does not resolve to a budget evidence requirement",
                            evidence_id.as_str()
                        ),
                    ));
                }
            }
        }
        if kind
            .deprecation
            .replacement_control_kind_id()
            .is_some_and(|replacement| replacement == &kind.control_kind_id)
        {
            report.push(ControlPackageValidationDiagnostic::kind(
                kind.control_kind_id.clone(),
                ControlPackageValidationReason::InvalidDeprecation,
                "control kind deprecation replacement must not point at itself",
            ));
        }
        if event_schema
            .is_some_and(|schema| schema_contains_route_ref(&schema.schema))
            && kind.route_requirements.is_empty()
        {
            report.push(ControlPackageValidationDiagnostic::kind(
                kind.control_kind_id.clone(),
                ControlPackageValidationReason::MissingRoute,
                "event payload schema contains a route ref but the control kind declares no route requirement",
            ));
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlPackageValidationReport {
    pub diagnostics: Vec<ControlPackageValidationDiagnostic>,
}

impl ControlPackageValidationReport {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == ControlPackageValidationSeverity::Error)
    }

    pub fn push(&mut self, diagnostic: ControlPackageValidationDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend(&mut self, report: ControlPackageValidationReport) {
        self.diagnostics.extend(report.diagnostics);
    }

    pub fn has_reason(&self, reason: ControlPackageValidationReason) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.reason == reason)
    }
}

impl Default for ControlPackageValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlPackageValidationDiagnostic {
    pub diagnostic_id: ControlDiagnosticId,
    pub severity: ControlPackageValidationSeverity,
    pub scope: ControlPackageValidationScope,
    pub path: ControlPackageValidationPath,
    pub reason: ControlPackageValidationReason,
    pub message: String,
}

impl ControlPackageValidationDiagnostic {
    pub fn package(
        package_id: ControlPackageId,
        reason: ControlPackageValidationReason,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            ControlPackageValidationScope::Package { package_id },
            ControlPackageValidationPath::new("package"),
            reason,
            message,
        )
    }

    pub fn kind(
        control_kind_id: ControlKindId,
        reason: ControlPackageValidationReason,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            ControlPackageValidationScope::Kind { control_kind_id },
            ControlPackageValidationPath::new("control_kind"),
            reason,
            message,
        )
    }

    pub fn new(
        scope: ControlPackageValidationScope,
        path: ControlPackageValidationPath,
        reason: ControlPackageValidationReason,
        message: impl Into<String>,
    ) -> Self {
        Self {
            diagnostic_id: ControlDiagnosticId::new(format!(
                "runenwerk.ui.controls.validation.{}",
                reason.as_snake_case()
            )),
            severity: ControlPackageValidationSeverity::Error,
            scope,
            path,
            reason,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlPackageValidationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlPackageValidationScope {
    Package { package_id: ControlPackageId },
    Kind { control_kind_id: ControlKindId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlPackageValidationPath {
    pub segments: Vec<String>,
}

impl ControlPackageValidationPath {
    pub fn new(segment: impl Into<String>) -> Self {
        Self {
            segments: vec![segment.into()],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlPackageValidationReason {
    MissingPackageMetadata,
    MissingKindMetadata,
    MissingSchema,
    MissingKernel,
    MissingFixture,
    MissingDiagnostic,
    MissingMigration,
    MissingStory,
    MissingRoute,
    MissingCapability,
    MissingTargetProfile,
    MissingMountEvidence,
    DuplicatePackageId,
    DuplicateControlKindId,
    DuplicateSchemaRef,
    DuplicateKernelId,
    DuplicateFixtureId,
    DuplicateDiagnosticId,
    DuplicateMigrationId,
    DuplicateStoryId,
    DuplicateRouteRequirement,
    InvalidDeprecation,
    InvalidMigrationVersion,
    InvalidMountEligibility,
    UnsupportedTargetProfile,
    UnresolvedReference,
    RenderEvidenceMissing,
    BudgetEvidenceMissing,
}

impl ControlPackageValidationReason {
    pub const fn as_snake_case(self) -> &'static str {
        match self {
            Self::MissingPackageMetadata => "missing_package_metadata",
            Self::MissingKindMetadata => "missing_kind_metadata",
            Self::MissingSchema => "missing_schema",
            Self::MissingKernel => "missing_kernel",
            Self::MissingFixture => "missing_fixture",
            Self::MissingDiagnostic => "missing_diagnostic",
            Self::MissingMigration => "missing_migration",
            Self::MissingStory => "missing_story",
            Self::MissingRoute => "missing_route",
            Self::MissingCapability => "missing_capability",
            Self::MissingTargetProfile => "missing_target_profile",
            Self::MissingMountEvidence => "missing_mount_evidence",
            Self::DuplicatePackageId => "duplicate_package_id",
            Self::DuplicateControlKindId => "duplicate_control_kind_id",
            Self::DuplicateSchemaRef => "duplicate_schema_ref",
            Self::DuplicateKernelId => "duplicate_kernel_id",
            Self::DuplicateFixtureId => "duplicate_fixture_id",
            Self::DuplicateDiagnosticId => "duplicate_diagnostic_id",
            Self::DuplicateMigrationId => "duplicate_migration_id",
            Self::DuplicateStoryId => "duplicate_story_id",
            Self::DuplicateRouteRequirement => "duplicate_route_requirement",
            Self::InvalidDeprecation => "invalid_deprecation",
            Self::InvalidMigrationVersion => "invalid_migration_version",
            Self::InvalidMountEligibility => "invalid_mount_eligibility",
            Self::UnsupportedTargetProfile => "unsupported_target_profile",
            Self::UnresolvedReference => "unresolved_reference",
            Self::RenderEvidenceMissing => "render_evidence_missing",
            Self::BudgetEvidenceMissing => "budget_evidence_missing",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlPackageContractError {
    EmptyId { kind: &'static str },
    UnnamespacedId { kind: &'static str, value: String },
    InvalidIdCharacter { kind: &'static str, value: String },
}

impl fmt::Display for ControlPackageContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId { kind } => write!(formatter, "empty control {kind} id"),
            Self::UnnamespacedId { kind, value } => {
                write!(formatter, "control {kind} id {value} is not namespaced")
            }
            Self::InvalidIdCharacter { kind, value } => write!(
                formatter,
                "control {kind} id {value} contains an invalid character"
            ),
        }
    }
}

impl std::error::Error for ControlPackageContractError {}

fn validate_control_id(value: &str, kind: &'static str) -> Result<(), ControlPackageContractError> {
    if value.is_empty() {
        return Err(ControlPackageContractError::EmptyId { kind });
    }
    if !value.contains('.') {
        return Err(ControlPackageContractError::UnnamespacedId {
            kind,
            value: value.to_owned(),
        });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Err(ControlPackageContractError::InvalidIdCharacter {
            kind,
            value: value.to_owned(),
        });
    }
    Ok(())
}

fn schema_ref_key(schema_ref: &UiSchemaRef) -> String {
    format!("{}@{}", schema_ref.id.as_str(), schema_ref.version.value())
}

fn schema_index(schemas: &[ControlSchemaDescriptor]) -> BTreeMap<String, &ControlSchemaDescriptor> {
    schemas
        .iter()
        .map(|schema| (schema_ref_key(schema.schema_ref()), schema))
        .collect()
}

fn kernel_index(kernels: &[ControlKernelDescriptor]) -> BTreeMap<String, &ControlKernelDescriptor> {
    kernels
        .iter()
        .map(|kernel| (kernel.kernel_id.as_str().to_owned(), kernel))
        .collect()
}

fn schema_contains_route_ref(schema: &ui_schema::UiSchema) -> bool {
    schema
        .fields
        .values()
        .any(|field| shape_contains_route_ref(&field.shape))
}

fn shape_contains_route_ref(shape: &UiSchemaShape) -> bool {
    match shape {
        UiSchemaShape::RouteRef => true,
        UiSchemaShape::List(inner) | UiSchemaShape::Nullable(inner) => shape_contains_route_ref(inner),
        _ => false,
    }
}

fn push_duplicate_package_values(
    report: &mut ControlPackageValidationReport,
    package_id: &ControlPackageId,
    label: &'static str,
    values: impl IntoIterator<Item = String>,
    reason: ControlPackageValidationReason,
) {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value.clone()) {
            report.push(ControlPackageValidationDiagnostic::package(
                package_id.clone(),
                reason,
                format!("duplicate {label} id {value}"),
            ));
        }
    }
}

fn duplicate_values(
    label: &'static str,
    values: impl IntoIterator<Item = String>,
    control_kind_id: &ControlKindId,
    reason: ControlPackageValidationReason,
) -> Vec<ControlPackageValidationDiagnostic> {
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for value in values {
        if !seen.insert(value.clone()) {
            diagnostics.push(ControlPackageValidationDiagnostic::kind(
                control_kind_id.clone(),
                reason,
                format!("duplicate {label} value {value}"),
            ));
        }
    }
    diagnostics
}

impl ControlDiagnosticKind {
    pub(crate) fn control_package_validation() -> Self {
        Self::ContractValidation
    }
}

impl ControlDiagnosticScope {
    pub(crate) fn control_kind_contract(control_kind_id: ControlKindId) -> Self {
        Self::ControlKind {
            control_kind_id: control_kind_id.as_str().to_owned(),
        }
    }
}

impl ControlDiagnosticDescriptor {
    pub(crate) fn contract(
        diagnostic_id: ControlDiagnosticId,
        control_kind_id: ControlKindId,
        message_template: impl Into<String>,
    ) -> Self {
        Self::new(diagnostic_id, message_template)
            .with_kind(ControlDiagnosticKind::control_package_validation())
            .with_scope(ControlDiagnosticScope::control_kind_contract(control_kind_id))
            .with_severity(ControlDiagnosticSeverity::Error)
    }
}
