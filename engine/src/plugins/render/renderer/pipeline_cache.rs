use super::render_flow::RendererProgramSourceAuthority;
use crate::plugins::render::RenderFlowId;
use crate::plugins::render::pipelines::{FlowPassBindGroupKey, FlowPassPipelineKey};
use std::collections::HashMap;
use wgpu::{
    BindGroup, BindGroupLayout, ComputePipeline, PipelineLayout, RenderPipeline, Sampler,
    ShaderModule,
};

const RENDERER_PROGRAM_SOURCE_MAX_RECORDS: usize = 1024;
const RENDERER_PROGRAM_SOURCE_MAX_RETAINED_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Default)]
pub struct RendererPipelineCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub failures: u64,
    pub program_source_owner: u64,
    pub program_source_records: usize,
    pub program_source_bytes: usize,
    pub program_source_max_records: usize,
    pub program_source_max_bytes: usize,
}

#[derive(Debug)]
pub struct FlowPipelineArtifactCache {
    pub shader_modules: HashMap<FlowPassPipelineKey, ShaderModule>,
    pub bind_group_layouts: HashMap<FlowPassPipelineKey, BindGroupLayout>,
    pub pipeline_layouts: HashMap<FlowPassPipelineKey, PipelineLayout>,
    pub compute_pipelines: HashMap<FlowPassPipelineKey, ComputePipeline>,
    pub render_pipelines: HashMap<FlowPassPipelineKey, RenderPipeline>,
    pub samplers: HashMap<FlowPassPipelineKey, Sampler>,
    pub bind_groups: HashMap<FlowPassBindGroupKey, BindGroup>,
    pub stats: RendererPipelineCacheStats,
    program_sources: RendererProgramSourceAuthority,
}

impl Default for FlowPipelineArtifactCache {
    fn default() -> Self {
        Self {
            shader_modules: HashMap::new(),
            bind_group_layouts: HashMap::new(),
            pipeline_layouts: HashMap::new(),
            compute_pipelines: HashMap::new(),
            render_pipelines: HashMap::new(),
            samplers: HashMap::new(),
            bind_groups: HashMap::new(),
            stats: RendererPipelineCacheStats::default(),
            program_sources: RendererProgramSourceAuthority::new(
                RENDERER_PROGRAM_SOURCE_MAX_RECORDS,
                RENDERER_PROGRAM_SOURCE_MAX_RETAINED_BYTES,
            )
            .expect("renderer program-source authority policy is nonzero and process-local"),
        }
    }
}

impl FlowPipelineArtifactCache {
    pub fn stats(&self) -> RendererPipelineCacheStats {
        let source_stats = self.program_sources.stats();
        RendererPipelineCacheStats {
            program_source_owner: self.program_sources.owner().diagnostic_raw(),
            program_source_records: source_stats.retained_records(),
            program_source_bytes: source_stats.retained_source_bytes(),
            program_source_max_records: source_stats.max_records(),
            program_source_max_bytes: source_stats.max_retained_source_bytes(),
            ..self.stats
        }
    }

    pub fn get_or_create_shader_module<F>(
        &mut self,
        key: FlowPassPipelineKey,
        create: F,
    ) -> ShaderModule
    where
        F: FnOnce() -> ShaderModule,
    {
        if let Some(value) = self.shader_modules.get(&key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            return value.clone();
        }
        self.stats.misses = self.stats.misses.saturating_add(1);
        let value = create();
        self.shader_modules.insert(key, value.clone());
        value
    }

    pub fn get_or_create_bind_group_layout<F>(
        &mut self,
        key: FlowPassPipelineKey,
        create: F,
    ) -> BindGroupLayout
    where
        F: FnOnce() -> BindGroupLayout,
    {
        if let Some(value) = self.bind_group_layouts.get(&key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            return value.clone();
        }
        self.stats.misses = self.stats.misses.saturating_add(1);
        let value = create();
        self.bind_group_layouts.insert(key, value.clone());
        value
    }

    pub fn get_or_create_pipeline_layout<F>(
        &mut self,
        key: FlowPassPipelineKey,
        create: F,
    ) -> PipelineLayout
    where
        F: FnOnce() -> PipelineLayout,
    {
        if let Some(value) = self.pipeline_layouts.get(&key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            return value.clone();
        }
        self.stats.misses = self.stats.misses.saturating_add(1);
        let value = create();
        self.pipeline_layouts.insert(key, value.clone());
        value
    }

    pub fn get_or_create_compute_pipeline<F>(
        &mut self,
        key: FlowPassPipelineKey,
        create: F,
    ) -> ComputePipeline
    where
        F: FnOnce() -> ComputePipeline,
    {
        if let Some(value) = self.compute_pipelines.get(&key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            return value.clone();
        }
        self.stats.misses = self.stats.misses.saturating_add(1);
        let value = create();
        self.compute_pipelines.insert(key, value.clone());
        value
    }

    pub fn get_or_create_render_pipeline<F>(
        &mut self,
        key: FlowPassPipelineKey,
        create: F,
    ) -> RenderPipeline
    where
        F: FnOnce() -> RenderPipeline,
    {
        if let Some(value) = self.render_pipelines.get(&key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            return value.clone();
        }
        self.stats.misses = self.stats.misses.saturating_add(1);
        let value = create();
        self.render_pipelines.insert(key, value.clone());
        value
    }

    pub fn get_or_create_sampler<F>(&mut self, key: FlowPassPipelineKey, create: F) -> Sampler
    where
        F: FnOnce() -> Sampler,
    {
        if let Some(value) = self.samplers.get(&key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            return value.clone();
        }
        self.stats.misses = self.stats.misses.saturating_add(1);
        let value = create();
        self.samplers.insert(key, value.clone());
        value
    }

    pub fn get_or_create_bind_group<F>(&mut self, key: FlowPassBindGroupKey, create: F) -> BindGroup
    where
        F: FnOnce() -> BindGroup,
    {
        if let Some(value) = self.bind_groups.get(&key) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            return value.clone();
        }
        self.stats.misses = self.stats.misses.saturating_add(1);
        let value = create();
        self.bind_groups.insert(key, value.clone());
        value
    }

    pub fn retain_flows(&mut self, active_flow_ids: &[RenderFlowId]) {
        self.shader_modules
            .retain(|key, _| active_flow_ids.contains(&key.flow_id));
        self.bind_group_layouts
            .retain(|key, _| active_flow_ids.contains(&key.flow_id));
        self.pipeline_layouts
            .retain(|key, _| active_flow_ids.contains(&key.flow_id));
        self.compute_pipelines
            .retain(|key, _| active_flow_ids.contains(&key.flow_id));
        self.render_pipelines
            .retain(|key, _| active_flow_ids.contains(&key.flow_id));
        self.samplers
            .retain(|key, _| active_flow_ids.contains(&key.flow_id));
        self.bind_groups
            .retain(|key, _| active_flow_ids.contains(&key.pipeline.flow_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_cache_owns_one_bounded_program_source_registry() {
        let cache = FlowPipelineArtifactCache::default();
        let stats = cache.stats();

        assert_ne!(stats.program_source_owner, 0);
        assert_eq!(stats.program_source_records, 0);
        assert_eq!(stats.program_source_bytes, 0);
        assert_eq!(
            stats.program_source_max_records,
            RENDERER_PROGRAM_SOURCE_MAX_RECORDS
        );
        assert_eq!(
            stats.program_source_max_bytes,
            RENDERER_PROGRAM_SOURCE_MAX_RETAINED_BYTES
        );
    }

    #[test]
    fn independent_renderer_caches_do_not_share_source_owner_identity() {
        let first = FlowPipelineArtifactCache::default().stats();
        let second = FlowPipelineArtifactCache::default().stats();

        assert_ne!(first.program_source_owner, second.program_source_owner);
    }
}
