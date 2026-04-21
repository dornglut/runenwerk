use crate::plugins::render::RenderFlowId;
use crate::plugins::render::pipelines::{FlowPassBindGroupKey, FlowPassPipelineKey};
use std::collections::HashMap;
use wgpu::{
    BindGroup, BindGroupLayout, ComputePipeline, PipelineLayout, RenderPipeline, Sampler,
    ShaderModule,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct RendererPipelineCacheStats {
    pub hits: u64,
    pub misses: u64,
}

#[derive(Debug, Default)]
pub struct FlowPipelineArtifactCache {
    pub shader_modules: HashMap<FlowPassPipelineKey, ShaderModule>,
    pub bind_group_layouts: HashMap<FlowPassPipelineKey, BindGroupLayout>,
    pub pipeline_layouts: HashMap<FlowPassPipelineKey, PipelineLayout>,
    pub compute_pipelines: HashMap<FlowPassPipelineKey, ComputePipeline>,
    pub render_pipelines: HashMap<FlowPassPipelineKey, RenderPipeline>,
    pub samplers: HashMap<FlowPassPipelineKey, Sampler>,
    pub bind_groups: HashMap<FlowPassBindGroupKey, BindGroup>,
    pub stats: RendererPipelineCacheStats,
}

impl FlowPipelineArtifactCache {
    pub fn stats(&self) -> RendererPipelineCacheStats {
        self.stats
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
            .retain(|key, _| active_flow_ids.iter().any(|id| *id == key.flow_id));
        self.bind_group_layouts
            .retain(|key, _| active_flow_ids.iter().any(|id| *id == key.flow_id));
        self.pipeline_layouts
            .retain(|key, _| active_flow_ids.iter().any(|id| *id == key.flow_id));
        self.compute_pipelines
            .retain(|key, _| active_flow_ids.iter().any(|id| *id == key.flow_id));
        self.render_pipelines
            .retain(|key, _| active_flow_ids.iter().any(|id| *id == key.flow_id));
        self.samplers
            .retain(|key, _| active_flow_ids.iter().any(|id| *id == key.flow_id));
        self.bind_groups
            .retain(|key, _| active_flow_ids.iter().any(|id| *id == key.pipeline.flow_id));
    }
}
