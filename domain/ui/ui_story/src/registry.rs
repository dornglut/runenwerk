//! Deterministic registry for story manifests.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    manifest::{UiStoryId, UiStoryManifest},
    runner::UiStoryRunRequest,
};

pub const DIAGNOSTIC_REGISTRY_DUPLICATE_ID: &str = "ui.story.registry.duplicate_id";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiStoryRegistry {
    stories: BTreeMap<UiStoryId, UiStoryManifest>,
    diagnostics: Vec<UiStoryRegistryDiagnostic>,
}

impl UiStoryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_manifests(manifests: impl IntoIterator<Item = UiStoryManifest>) -> Self {
        let mut registry = Self::new();
        for manifest in manifests {
            registry.insert(manifest);
        }
        registry
    }

    pub fn insert(&mut self, manifest: UiStoryManifest) {
        let story_id = manifest.story_id.clone();
        if self.stories.contains_key(&story_id) {
            self.diagnostics.push(UiStoryRegistryDiagnostic::new(
                DIAGNOSTIC_REGISTRY_DUPLICATE_ID,
                format!("duplicate story id {}", story_id.as_str()),
            ));
            return;
        }
        self.stories.insert(story_id, manifest);
    }

    pub fn get(&self, story_id: &UiStoryId) -> Option<&UiStoryManifest> {
        self.stories.get(story_id)
    }

    pub fn contains(&self, story_id: &UiStoryId) -> bool {
        self.stories.contains_key(story_id)
    }

    pub fn stories(&self) -> impl Iterator<Item = &UiStoryManifest> {
        self.stories.values()
    }

    pub fn diagnostics(&self) -> &[UiStoryRegistryDiagnostic] {
        &self.diagnostics
    }

    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn run_request(&self, story_id: impl Into<String>) -> UiStoryRunRequest {
        UiStoryRunRequest::new(UiStoryId::new(story_id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryRegistryDiagnostic {
    pub code: String,
    pub message: String,
}

impl UiStoryRegistryDiagnostic {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
