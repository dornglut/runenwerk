//! File: domain/ui/ui_controls/src/catalog.rs
//! Crate: ui_controls

use serde::{Deserialize, Serialize};

use super::descriptor::{ControlKindDescriptor, ControlPackageDescriptor};
use super::metadata::ControlMountEligibility;
use super::story_proof::{ControlStoryProofSummary, ControlStoryProofVerdict};
use crate::migration::ControlDeprecationStatus;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlCatalogIndex {
    pub entries: Vec<ControlCatalogEntryDescriptor>,
}

impl ControlCatalogIndex {
    pub fn from_packages<'a>(
        packages: impl IntoIterator<Item = &'a ControlPackageDescriptor>,
    ) -> Self {
        let mut entries = packages
            .into_iter()
            .flat_map(|package| {
                package.control_kinds.iter().map(move |kind| {
                    ControlCatalogEntryDescriptor::from_control_kind(package, kind)
                })
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.package_id
                .cmp(&right.package_id)
                .then_with(|| left.control_kind_id.cmp(&right.control_kind_id))
        });
        Self { entries }
    }

    pub fn query(&self, query: &ControlCatalogQuery) -> ControlCatalogQueryResult {
        ControlCatalogQueryResult {
            entries: self
                .entries
                .iter()
                .filter(|entry| query.matches(entry))
                .cloned()
                .collect(),
        }
    }

    pub fn entry(&self, control_kind_id: &str) -> Option<&ControlCatalogEntryDescriptor> {
        self.entries
            .iter()
            .find(|entry| entry.control_kind_id == control_kind_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlCatalogEntryDescriptor {
    pub package_id: String,
    pub control_kind_id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub target_profiles: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub route_ids: Vec<String>,
    #[serde(default)]
    pub fixture_ids: Vec<String>,
    #[serde(default)]
    pub story_ids: Vec<String>,
    #[serde(default)]
    pub diagnostic_ids: Vec<String>,
    pub story_required: bool,
    pub mount_eligible: bool,
    pub has_diagnostics: bool,
    pub mount_explanation: String,
    pub compatibility: ControlCompatibilitySummary,
    pub deprecation: ControlCatalogDeprecationStatus,
}

impl ControlCatalogEntryDescriptor {
    pub fn from_control_kind(
        package: &ControlPackageDescriptor,
        kind: &ControlKindDescriptor,
    ) -> Self {
        let mut tags = package
            .tags
            .iter()
            .map(|tag| tag.as_str().to_owned())
            .chain(kind.tags.iter().map(|tag| tag.as_str().to_owned()))
            .collect::<Vec<_>>();
        tags = sorted_unique(tags);

        let mut target_profiles = package
            .target_profiles
            .iter()
            .map(|target| target.as_str().to_owned())
            .chain(
                kind.target_profiles
                    .iter()
                    .map(|target| target.as_str().to_owned()),
            )
            .collect::<Vec<_>>();
        target_profiles = sorted_unique(target_profiles);

        let mut capabilities = package
            .required_capabilities
            .iter()
            .map(|capability| capability.as_str().to_owned())
            .chain(
                kind.required_capabilities
                    .iter()
                    .map(|capability| capability.as_str().to_owned()),
            )
            .chain(kind.route_requirements.iter().flat_map(|route| {
                route
                    .capabilities
                    .iter()
                    .map(|capability| capability.as_str().to_owned())
            }))
            .collect::<Vec<_>>();
        capabilities = sorted_unique(capabilities);

        let story_ids = kind
            .story_ids
            .iter()
            .map(|story_id| story_id.as_str().to_owned())
            .collect::<Vec<_>>();
        let diagnostic_ids = kind
            .diagnostic_ids
            .iter()
            .map(|diagnostic_id| diagnostic_id.as_str().to_owned())
            .collect::<Vec<_>>();
        let (mount_eligible, mount_explanation) = mount_status(kind);

        Self {
            package_id: package.package_id.as_str().to_owned(),
            control_kind_id: kind.control_kind_id.as_str().to_owned(),
            display_name: kind.display_name.to_owned(),
            description: kind.description.to_owned(),
            category: kind.category.as_str().to_owned(),
            tags,
            target_profiles,
            capabilities,
            route_ids: kind
                .route_requirements
                .iter()
                .map(|route| route.route_id.as_str().to_owned())
                .collect(),
            fixture_ids: kind
                .fixture_ids
                .iter()
                .map(|fixture_id| fixture_id.as_str().to_owned())
                .collect(),
            story_required: !story_ids.is_empty(),
            story_ids,
            has_diagnostics: !diagnostic_ids.is_empty(),
            diagnostic_ids,
            mount_eligible,
            mount_explanation,
            compatibility: ControlCompatibilitySummary::from_control_kind(kind),
            deprecation: ControlCatalogDeprecationStatus::from_status(&kind.deprecation),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlCatalogDeprecationStatus {
    Active,
    Deprecated,
    Removed,
}

impl ControlCatalogDeprecationStatus {
    pub fn from_status(status: &ControlDeprecationStatus) -> Self {
        match status {
            ControlDeprecationStatus::Active => Self::Active,
            ControlDeprecationStatus::Deprecated { .. } => Self::Deprecated,
            ControlDeprecationStatus::Removed { .. } => Self::Removed,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlCatalogQuery {
    #[serde(default)]
    pub package_id: Option<String>,
    #[serde(default)]
    pub control_kind_id: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub target_profile: Option<String>,
    #[serde(default)]
    pub capability: Option<String>,
    #[serde(default)]
    pub story_required: Option<bool>,
    #[serde(default)]
    pub mount_eligible: Option<bool>,
    #[serde(default)]
    pub has_diagnostics: Option<bool>,
}

impl ControlCatalogQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_package_id(mut self, package_id: impl Into<String>) -> Self {
        self.package_id = Some(package_id.into());
        self
    }

    pub fn with_control_kind_id(mut self, control_kind_id: impl Into<String>) -> Self {
        self.control_kind_id = Some(control_kind_id.into());
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    pub fn with_target_profile(mut self, target_profile: impl Into<String>) -> Self {
        self.target_profile = Some(target_profile.into());
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capability = Some(capability.into());
        self
    }

    pub fn with_story_required(mut self, story_required: bool) -> Self {
        self.story_required = Some(story_required);
        self
    }

    pub fn with_mount_eligible(mut self, mount_eligible: bool) -> Self {
        self.mount_eligible = Some(mount_eligible);
        self
    }

    pub fn with_has_diagnostics(mut self, has_diagnostics: bool) -> Self {
        self.has_diagnostics = Some(has_diagnostics);
        self
    }

    pub fn matches(&self, entry: &ControlCatalogEntryDescriptor) -> bool {
        self.package_id
            .as_deref()
            .map_or(true, |value| entry.package_id == value)
            && self
                .control_kind_id
                .as_deref()
                .map_or(true, |value| entry.control_kind_id == value)
            && self
                .category
                .as_deref()
                .map_or(true, |value| entry.category == value)
            && self
                .tag
                .as_deref()
                .map_or(true, |value| entry.tags.iter().any(|tag| tag == value))
            && self.target_profile.as_deref().map_or(true, |value| {
                entry.target_profiles.iter().any(|target| target == value)
            })
            && self.capability.as_deref().map_or(true, |value| {
                entry
                    .capabilities
                    .iter()
                    .any(|capability| capability == value)
            })
            && self
                .story_required
                .map_or(true, |value| entry.story_required == value)
            && self
                .mount_eligible
                .map_or(true, |value| entry.mount_eligible == value)
            && self
                .has_diagnostics
                .map_or(true, |value| entry.has_diagnostics == value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlCatalogFilter {
    pub query: ControlCatalogQuery,
}

impl ControlCatalogFilter {
    pub fn new(query: ControlCatalogQuery) -> Self {
        Self { query }
    }

    pub fn matches(&self, entry: &ControlCatalogEntryDescriptor) -> bool {
        self.query.matches(entry)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlCatalogQueryResult {
    pub entries: Vec<ControlCatalogEntryDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlInspectionDescriptor {
    pub package_id: String,
    pub control_kind_id: String,
    pub display_name: String,
    pub facts: Vec<ControlInspectionFact>,
    pub diagnostic_badges: Vec<ControlDiagnosticBadge>,
    #[serde(default)]
    pub story_proof_badge: Option<ControlStoryProofBadge>,
}

impl ControlInspectionDescriptor {
    pub fn from_control_kind(
        package: &ControlPackageDescriptor,
        kind: &ControlKindDescriptor,
    ) -> Self {
        let entry = ControlCatalogEntryDescriptor::from_control_kind(package, kind);
        let mut facts = Vec::new();
        push_fact(
            &mut facts,
            ControlInspectionSection::Identity,
            "package_id",
            &entry.package_id,
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Identity,
            "control_kind_id",
            &entry.control_kind_id,
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Identity,
            "display_name",
            &entry.display_name,
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Metadata,
            "category",
            &entry.category,
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Metadata,
            "tags",
            &entry.tags.join(","),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Metadata,
            "target_profiles",
            &entry.target_profiles.join(","),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Compatibility,
            "supports_story_proof",
            bool_string(entry.compatibility.supports_story_proof),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Compatibility,
            "supports_runtime_mount",
            bool_string(entry.compatibility.supports_runtime_mount),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Schemas,
            "properties",
            &schema_ref(&kind.property_schema),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Schemas,
            "state",
            &schema_ref(&kind.state_schema),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Schemas,
            "event_payload",
            &schema_ref(&kind.event_payload_schema),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Kernels,
            "layout",
            kind.kernels.layout.as_str(),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Kernels,
            "interaction",
            kind.kernels.interaction.as_str(),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Kernels,
            "visual",
            kind.kernels.visual.as_str(),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Kernels,
            "accessibility",
            kind.kernels.accessibility.as_str(),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Kernels,
            "inspection",
            kind.kernels.inspection.as_str(),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Routes,
            "routes",
            &entry.route_ids.join(","),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Routes,
            "capabilities",
            &entry.capabilities.join(","),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Fixtures,
            "fixtures",
            &entry.fixture_ids.join(","),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Stories,
            "stories",
            &entry.story_ids.join(","),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::StoryProof,
            "story_required",
            bool_string(entry.story_required),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::Diagnostics,
            "diagnostics",
            &entry.diagnostic_ids.join(","),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::MountEligibility,
            "eligible",
            bool_string(entry.mount_eligible),
        );
        push_fact(
            &mut facts,
            ControlInspectionSection::MountEligibility,
            "explanation",
            &entry.mount_explanation,
        );

        Self {
            package_id: entry.package_id,
            control_kind_id: entry.control_kind_id,
            display_name: entry.display_name,
            facts,
            diagnostic_badges: diagnostic_badges(package, kind),
            story_proof_badge: None,
        }
    }

    pub fn with_story_proof_summary(mut self, summary: &ControlStoryProofSummary) -> Self {
        self.story_proof_badge = Some(ControlStoryProofBadge::from_summary(summary));
        self
    }

    pub fn fact(&self, section: ControlInspectionSection, key: &str) -> Option<&str> {
        self.facts
            .iter()
            .find(|fact| fact.section == section && fact.key == key)
            .map(|fact| fact.value.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlInspectionSection {
    Identity,
    Metadata,
    Compatibility,
    Schemas,
    Kernels,
    Routes,
    Fixtures,
    Stories,
    StoryProof,
    Diagnostics,
    MountEligibility,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlInspectionFact {
    pub section: ControlInspectionSection,
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlDiagnosticBadge {
    pub diagnostic_id: String,
    pub severity: String,
    pub message_template: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlCompatibilitySummary {
    pub supports_story_proof: bool,
    pub supports_gallery_inspection: bool,
    pub supports_workbench_consumption: bool,
    pub supports_designer_consumption: bool,
    pub supports_runtime_mount: bool,
}

impl ControlCompatibilitySummary {
    pub fn from_control_kind(kind: &ControlKindDescriptor) -> Self {
        Self {
            supports_story_proof: kind.compatibility.supports_story_proof,
            supports_gallery_inspection: kind.compatibility.supports_gallery_inspection,
            supports_workbench_consumption: kind.compatibility.supports_workbench_consumption,
            supports_designer_consumption: kind.compatibility.supports_designer_consumption,
            supports_runtime_mount: kind.compatibility.supports_runtime_mount,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlStoryProofBadge {
    pub verdict: ControlStoryProofVerdict,
    #[serde(default)]
    pub first_unsatisfied_story_id: Option<String>,
    #[serde(default)]
    pub first_blocking_message: Option<String>,
}

impl ControlStoryProofBadge {
    pub fn from_summary(summary: &ControlStoryProofSummary) -> Self {
        Self {
            verdict: summary.verdict,
            first_unsatisfied_story_id: summary
                .first_unsatisfied_requirement
                .as_ref()
                .map(|requirement| requirement.story_id.as_str().to_owned()),
            first_blocking_message: summary
                .first_blocking_diagnostic
                .as_ref()
                .map(|diagnostic| diagnostic.message.to_owned()),
        }
    }
}

fn diagnostic_badges(
    package: &ControlPackageDescriptor,
    kind: &ControlKindDescriptor,
) -> Vec<ControlDiagnosticBadge> {
    kind.diagnostic_ids
        .iter()
        .map(|diagnostic_id| {
            let descriptor = package
                .diagnostics
                .iter()
                .find(|descriptor| descriptor.diagnostic_id == *diagnostic_id);
            ControlDiagnosticBadge {
                diagnostic_id: diagnostic_id.as_str().to_owned(),
                severity: descriptor
                    .map(|descriptor| format!("{:?}", descriptor.severity))
                    .unwrap_or_else(|| "Unknown".to_owned()),
                message_template: descriptor
                    .map(|descriptor| descriptor.message_template.to_owned())
                    .unwrap_or_else(|| "diagnostic descriptor is not attached".to_owned()),
            }
        })
        .collect()
}

fn mount_status(kind: &ControlKindDescriptor) -> (bool, String) {
    match &kind.mount_eligibility {
        ControlMountEligibility::NotEligible { reason } => (false, reason.to_owned()),
        ControlMountEligibility::RequiresEvidence {
            story_ids,
            render_evidence_ids,
            budget_evidence_ids,
        } => (
            kind.compatibility.supports_runtime_mount,
            format!(
                "runtime mount requires {} story, {} render, and {} budget evidence item(s)",
                story_ids.len(),
                render_evidence_ids.len(),
                budget_evidence_ids.len()
            ),
        ),
    }
}

fn push_fact(
    facts: &mut Vec<ControlInspectionFact>,
    section: ControlInspectionSection,
    key: &'static str,
    value: &str,
) {
    facts.push(ControlInspectionFact {
        section,
        key: key.to_owned(),
        value: value.to_owned(),
    });
}

fn bool_string(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn schema_ref(schema: &ui_schema::UiSchemaRef) -> String {
    format!("{}@{}", schema.id.as_str(), schema.version.value())
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
