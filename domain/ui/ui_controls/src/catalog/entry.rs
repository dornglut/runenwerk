//! File: domain/ui/ui_controls/src/catalog/entry.rs
//! Crate: ui_controls

use serde::{Deserialize, Serialize};

use crate::interaction::ControlInteractionSupportSummary;
use crate::package::descriptor::{ControlKindDescriptor, ControlPackageDescriptor};
use crate::package::metadata::ControlMountEligibility;

use super::{ControlCatalogDeprecationStatus, ControlCompatibilitySummary};

/// Read-only catalog entry for one package control kind.
///
/// Phase 12 interaction fields are descriptor projections only. They make
/// reusable interaction visible to catalog/inspection consumers without giving
/// controls command, product mutation, overlay, or text-editing authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlCatalogEntryDescriptor {
    /// Owning package id.
    pub package_id: String,

    /// Control kind id.
    pub control_kind_id: String,

    /// Human-readable display name.
    pub display_name: String,

    /// Human-readable description.
    pub description: String,

    /// Catalog category label.
    pub category: String,

    /// Sorted package/control tags.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Target profiles supported by the control kind.
    #[serde(default)]
    pub target_profiles: Vec<String>,

    /// Required capability labels.
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Required route ids.
    #[serde(default)]
    pub route_ids: Vec<String>,

    /// Fixture ids advertised by the control kind.
    #[serde(default)]
    pub fixture_ids: Vec<String>,

    /// Story ids advertised by the control kind.
    #[serde(default)]
    pub story_ids: Vec<String>,

    /// Diagnostic ids advertised by the control kind.
    #[serde(default)]
    pub diagnostic_ids: Vec<String>,

    /// Whether catalog policy requires a story.
    pub story_required: bool,

    /// Whether the control kind is mount eligible.
    pub mount_eligible: bool,

    /// Whether the control kind exposes diagnostics.
    pub has_diagnostics: bool,

    /// Human-readable mount eligibility explanation.
    pub mount_explanation: String,

    /// Compatibility summary for the control kind.
    pub compatibility: ControlCompatibilitySummary,

    /// Deprecation status for the control kind.
    pub deprecation: ControlCatalogDeprecationStatus,

    /// Reusable interaction state labels projected from package descriptors.
    #[serde(default)]
    pub interaction_states: Vec<String>,

    /// Reusable interaction trigger labels projected from package descriptors.
    #[serde(default)]
    pub interaction_triggers: Vec<String>,

    /// Reusable interaction outcome labels projected from package descriptors.
    #[serde(default)]
    pub interaction_outcomes: Vec<String>,

    /// Whether any reusable interaction requirement needs focus.
    #[serde(default)]
    pub interaction_requires_focus: bool,

    /// Whether text intent may be observed as a probe.
    #[serde(default)]
    pub interaction_text_intent_probe: bool,

    /// Whether reusable runtime interaction is supported.
    #[serde(default)]
    pub runtime_interaction_supported: bool,

    /// Whether the control owns runtime behavior itself.
    #[serde(default)]
    pub control_owned_runtime_behavior: bool,

    /// Whether the control executes host commands.
    #[serde(default)]
    pub executes_host_commands: bool,

    /// Whether the control mutates product state.
    #[serde(default)]
    pub mutates_product_state: bool,
}

impl ControlCatalogEntryDescriptor {
    /// Builds a catalog entry from a package/control descriptor pair.
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

        let mut entry = Self {
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
            interaction_states: Vec::new(),
            interaction_triggers: Vec::new(),
            interaction_outcomes: Vec::new(),
            interaction_requires_focus: false,
            interaction_text_intent_probe: false,
            runtime_interaction_supported: false,
            control_owned_runtime_behavior: false,
            executes_host_commands: false,
            mutates_product_state: false,
        };
        if let Some(descriptor) = package.interaction_descriptor(&kind.control_kind_id) {
            entry = entry.with_interaction_summary(&descriptor.summary());
        }
        entry
    }

    /// Attaches read-only reusable interaction summary data.
    pub fn with_interaction_summary(mut self, summary: &ControlInteractionSupportSummary) -> Self {
        self.interaction_states = summary.states.clone();
        self.interaction_triggers = summary.triggers.clone();
        self.interaction_outcomes = summary.outcomes.clone();
        self.interaction_requires_focus = summary.requires_focus;
        self.interaction_text_intent_probe = summary.text_intent_probe;
        self.runtime_interaction_supported = summary.runtime_interaction_supported;
        self.control_owned_runtime_behavior = summary.control_owned_runtime_behavior;
        self.executes_host_commands = summary.executes_host_commands;
        self.mutates_product_state = summary.mutates_product_state;
        self
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
