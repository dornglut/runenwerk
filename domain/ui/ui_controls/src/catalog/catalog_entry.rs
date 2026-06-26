//! File: domain/ui/ui_controls/src/catalog_entry.rs
//! Crate: ui_controls

use serde::{Deserialize, Serialize};

use crate::package::descriptor::{ControlKindDescriptor, ControlPackageDescriptor};
use crate::package::metadata::ControlMountEligibility;

use super::{ControlCatalogDeprecationStatus, ControlCompatibilitySummary};

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
        let tags = sorted_unique(
            package
                .tags
                .iter()
                .map(|tag| tag.as_str().to_owned())
                .chain(kind.tags.iter().map(|tag| tag.as_str().to_owned()))
                .collect(),
        );
        let target_profiles = sorted_unique(
            package
                .target_profiles
                .iter()
                .map(|target| target.as_str().to_owned())
                .chain(
                    kind.target_profiles
                        .iter()
                        .map(|target| target.as_str().to_owned()),
                )
                .collect(),
        );
        let capabilities = sorted_unique(
            package
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
                .collect(),
        );
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

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
