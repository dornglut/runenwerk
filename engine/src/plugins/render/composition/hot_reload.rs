use crate::plugins::render::composition::{
    FragmentSpecError, RenderFlowContribution, parse_fragment_ron,
};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub enum FragmentReloadOutcome {
    Unchanged,
    Updated {
        source_id: String,
        revision: u64,
        contribution: RenderFlowContribution,
    },
    Failed {
        source_id: String,
        revision: u64,
        error: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentHotReloadEntry {
    pub source_id: String,
    pub content_hash: u64,
    pub revision: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, Default, ecs::Component)]
pub struct RenderFlowFragmentHotReloadState {
    entries: BTreeMap<String, FragmentHotReloadEntry>,
    revision: u64,
}

impl RenderFlowFragmentHotReloadState {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn source_revision(&self, source_id: &str) -> Option<u64> {
        self.entries.get(source_id).map(|entry| entry.revision)
    }

    pub fn last_error(&self, source_id: &str) -> Option<&str> {
        self.entries
            .get(source_id)
            .and_then(|entry| entry.last_error.as_deref())
    }

    pub fn clear_source(&mut self, source_id: &str) -> bool {
        self.entries.remove(source_id).is_some()
    }

    pub fn apply_source(&mut self, source_id: &str, source_text: &str) -> FragmentReloadOutcome {
        let source_id = source_id.trim().to_string();
        if source_id.is_empty() {
            return FragmentReloadOutcome::Failed {
                source_id: "<empty>".to_string(),
                revision: self.revision,
                error: "source_id must not be empty".to_string(),
            };
        }

        let content_hash = stable_hash(source_text);
        if self
            .entries
            .get(source_id.as_str())
            .is_some_and(|entry| entry.content_hash == content_hash)
        {
            return FragmentReloadOutcome::Unchanged;
        }

        let next_revision = self.bump_revision();
        let update = match compile_fragment(source_text) {
            Ok(contribution) => {
                self.entries.insert(
                    source_id.clone(),
                    FragmentHotReloadEntry {
                        source_id: source_id.clone(),
                        content_hash,
                        revision: next_revision,
                        last_error: None,
                    },
                );
                FragmentReloadOutcome::Updated {
                    source_id,
                    revision: next_revision,
                    contribution,
                }
            }
            Err(err) => {
                self.entries.insert(
                    source_id.clone(),
                    FragmentHotReloadEntry {
                        source_id: source_id.clone(),
                        content_hash,
                        revision: next_revision,
                        last_error: Some(err.to_string()),
                    },
                );
                FragmentReloadOutcome::Failed {
                    source_id,
                    revision: next_revision,
                    error: err.to_string(),
                }
            }
        };

        update
    }

    fn bump_revision(&mut self) -> u64 {
        self.revision = self.revision.saturating_add(1);
        self.revision
    }
}

fn compile_fragment(source_text: &str) -> Result<RenderFlowContribution, FragmentSpecError> {
    let spec = parse_fragment_ron(source_text)?;
    spec.to_contribution()
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
