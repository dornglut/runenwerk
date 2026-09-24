//! File: domain/ui/ui_controls/src/catalog/query.rs
//! Crate: ui_controls

use serde::{Deserialize, Serialize};

use super::ControlCatalogEntryDescriptor;

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
            .is_none_or(|value| entry.package_id == value)
            && self
                .control_kind_id
                .as_deref()
                .is_none_or(|value| entry.control_kind_id == value)
            && self
                .category
                .as_deref()
                .is_none_or(|value| entry.category == value)
            && self
                .tag
                .as_deref()
                .is_none_or(|value| entry.tags.iter().any(|tag| tag == value))
            && self
                .target_profile
                .as_deref()
                .is_none_or(|value| entry.target_profiles.iter().any(|target| target == value))
            && self.capability.as_deref().is_none_or(|value| {
                entry
                    .capabilities
                    .iter()
                    .any(|capability| capability == value)
            })
            && self
                .story_required
                .is_none_or(|value| entry.story_required == value)
            && self
                .mount_eligible
                .is_none_or(|value| entry.mount_eligible == value)
            && self
                .has_diagnostics
                .is_none_or(|value| entry.has_diagnostics == value)
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
