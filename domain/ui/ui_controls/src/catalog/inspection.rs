//! File: domain/ui/ui_controls/src/catalog/inspection.rs
//! Crate: ui_controls

use serde::{Deserialize, Serialize};

use crate::input::ControlInputCapabilitySummary;
use crate::package::descriptor::{ControlKindDescriptor, ControlPackageDescriptor};
use crate::package::story_proof::ControlStoryProofSummary;

use super::{
    diagnostic_badges, ControlCatalogEntryDescriptor, ControlDiagnosticBadge, ControlStoryProofBadge,
};

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

    pub fn with_input_summary(mut self, summary: &ControlInputCapabilitySummary) -> Self {
        for fact in summary.inspection_facts() {
            push_fact(
                &mut self.facts,
                ControlInspectionSection::Input,
                &fact.key,
                &fact.value,
            );
        }
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
    Input,
    Diagnostics,
    MountEligibility,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlInspectionFact {
    pub section: ControlInspectionSection,
    pub key: String,
    pub value: String,
}

fn push_fact(
    facts: &mut Vec<ControlInspectionFact>,
    section: ControlInspectionSection,
    key: &str,
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
