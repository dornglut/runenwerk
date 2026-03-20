use crate::plugins::render::pipelines::PipelineKey;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct BackendPipelineCacheStats {
    pub hits: u64,
    pub misses: u64,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct BackendPipelineCacheResource {
    known: BTreeMap<PipelineKey, u64>,
    stats: BackendPipelineCacheStats,
}

impl BackendPipelineCacheResource {
    pub fn revision_for(&self, key: &PipelineKey) -> Option<u64> {
        self.known.get(key).copied()
    }

    pub fn record_hit(&mut self, key: &PipelineKey) {
        if self.known.contains_key(key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
        }
    }

    pub fn record_miss(&mut self, key: PipelineKey, revision: u64) {
        self.stats.misses = self.stats.misses.saturating_add(1);
        self.known.insert(key, revision);
    }

    pub fn stats(&self) -> BackendPipelineCacheStats {
        self.stats.clone()
    }
}
