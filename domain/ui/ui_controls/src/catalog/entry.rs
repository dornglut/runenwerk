//! File: domain/ui/ui_controls/src/catalog/entry.rs
//! Crate: ui_controls

use serde::{Deserialize, Serialize};

use crate::interaction::ControlInteractionSupportSummary;
use crate::overlay::ControlOverlaySupportSummary;
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
    #[serde(default)]
    pub interaction_states: Vec<String>,
    #[serde(default)]
    pub interaction_triggers: Vec<String>,
    #[serde(default)]
    pub interaction_outcomes: Vec<String>,
    #[serde(default)]
    pub interaction_requires_focus: bool,
    #[serde(default)]
    pub interaction_text_intent_probe: bool,
    #[serde(default)]
    pub runtime_interaction_supported: bool,
    #[serde(default)]
    pub overlay_kinds: Vec<String>,
    #[serde(default)]
    pub overlay_triggers: Vec<String>,
    #[serde(default)]
    pub overlay_layers: Vec<String>,
    #[serde(default)]
    pub overlay_dismiss_policies: Vec<String>,
    #[serde(default)]
    pub overlay_focus_policies: Vec<String>,
    #[serde(default)]
    pub overlay_supported: bool,
    #[serde(default)]
    pub control_owned_runtime_behavior: bool,
    #[serde(default)]
    pub executes_host_commands: bool,
    #[serde(default)]
    pub mutates_product_state: bool,
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
            overlay_kinds: Vec::new(),
            overlay_triggers: Vec::new(),
            overlay_layers: Vec::new(),
            overlay_dismiss_policies: Vec::new(),
            overlay_focus_policies: Vec::new(),
            overlay_supported: false,
            control_owned_runtime_behavior: false,
            executes_host_commands: false,
            mutates_product_state: false,
        };
        if let Some(descriptor) = package.interaction_descriptor(&kind.control_kind_id) {
            entry = entry.with_interaction_summary(&descriptor.summary());
        }
        if let Some(descriptor) = package.overlay_descriptor(&kind.control_kind_id) {
            entry = entry.with_overlay_summary(&descriptor.summary());
        }
        entry
    }

    pub fn with_interaction_summary(mut self, summary: &ControlInteractionSupportSummary) -> Self {
        self.interaction_states = summary.states.clone();
        self.interaction_triggers = summary.triggers.clone();
        self.interaction_outcomes = summary.outcomes.clone();
        self.interaction_requires_focus = summary.requires_focus;
        self.interaction_text_intent_probe = summary.text_intent_probe;
        self.runtime_interaction_supported = summary.runtime_interaction_supported;
        self.control_owned_runtime_behavior |= summary.control_owned_runtime_behavior;
        self.executes_host_commands |= summary.executes_host_commands;
        self.mutates_product_state |= summary.mutates_product_state;
        self
    }

    pub fn with_overlay_summary(mut self, summary: &ControlOverlaySupportSummary) -> Self {
        self.overlay_kinds = summary.kinds.clone();
        self.overlay_triggers = summary.triggers.clone();
        self.overlay_layers = summary.layers.clone();
        self.overlay_dismiss_policies = summary.dismiss_policies.clone();
        self.overlay_focus_policies = summary.focus_policies.clone();
        self.overlay_supported = summary.overlay_supported;
        self.control_owned_runtime_behavior |= summary.control_owned_runtime_behavior;
        self.executes_host_commands |= summary.executes_host_commands;
        self.mutates_product_state |= summary.mutates_product_state;
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
