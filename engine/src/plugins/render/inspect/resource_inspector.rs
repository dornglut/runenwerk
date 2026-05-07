use crate::plugins::render::resource::{ImportedBufferSemantic, ImportedTextureSemantic};
use crate::plugins::render::{
    RenderFlow, RenderResourceDescriptor, RenderTargetAliasKind, ResourceLifetime,
};

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct RenderDebugOverlayState {
    pub enabled: bool,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct RenderRuntimeResourceInspectorState {
    pub entries: Vec<RuntimeResourceInspectionEntry>,
    pub pipeline_cache_hits: u64,
    pub pipeline_cache_misses: u64,
    pub world_resident_chunks: usize,
    pub world_stale_chunks: usize,
    pub world_page_miss_count: u64,
    pub world_interactive_queue_depth: usize,
    pub world_background_queue_depth: usize,
}

impl RenderRuntimeResourceInspectorState {
    pub fn observe_runtime_resources(&mut self, entries: &[RuntimeResourceInspectionEntry]) {
        self.entries.clear();
        self.entries.extend_from_slice(entries);
    }

    pub fn observe_pipeline_cache_stats(&mut self, hits: u64, misses: u64) {
        self.pipeline_cache_hits = hits;
        self.pipeline_cache_misses = misses;
    }

    pub fn observe_world_runtime(
        &mut self,
        resident_chunks: usize,
        stale_chunks: usize,
        page_miss_count: u64,
        interactive_queue_depth: usize,
        background_queue_depth: usize,
    ) {
        self.world_resident_chunks = resident_chunks;
        self.world_stale_chunks = stale_chunks;
        self.world_page_miss_count = page_miss_count;
        self.world_interactive_queue_depth = interactive_queue_depth;
        self.world_background_queue_depth = background_queue_depth;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInspectionEntry {
    pub id: String,
    pub kind: String,
    pub lifetime: ResourceLifetime,
    pub imported: bool,
    pub target_alias_label: Option<String>,
    pub target_alias_kind: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeResourceReuse {
    Created,
    Reused,
    NotRealized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeResourceInspectionEntry {
    pub flow_id: String,
    pub id: String,
    pub kind: String,
    pub lifetime: ResourceLifetime,
    pub imported: bool,
    pub realized: bool,
    pub reuse: RuntimeResourceReuse,
    pub size_bytes: Option<u64>,
    pub texture_size: Option<(u32, u32)>,
    pub element_count: Option<u64>,
    pub generation: Option<u64>,
}

pub fn resource_kind_name(resource: &RenderResourceDescriptor) -> &'static str {
    match resource {
        RenderResourceDescriptor::UniformBuffer(_) => "uniform_buffer",
        RenderResourceDescriptor::StorageBuffer(_) => "storage_buffer",
        RenderResourceDescriptor::SampledTexture(_) => "sampled_texture",
        RenderResourceDescriptor::StorageTexture(_) => "storage_texture",
        RenderResourceDescriptor::ColorTarget(_) => "color_target",
        RenderResourceDescriptor::DepthTarget(_) => "depth_target",
        RenderResourceDescriptor::HistoryTexture(_) => "history_texture",
        RenderResourceDescriptor::TargetAlias(value) => target_alias_kind_resource_name(value.kind),
        RenderResourceDescriptor::ImportedTexture(value) => match value.semantic {
            ImportedTextureSemantic::SurfaceColor => "imported_texture(surface_color)",
            ImportedTextureSemantic::SurfaceDepth => "imported_texture(surface_depth)",
            ImportedTextureSemantic::HistoryTexture => "imported_texture(history_texture)",
            ImportedTextureSemantic::External => "imported_texture(external)",
        },
        RenderResourceDescriptor::ImportedBuffer(value) => match value.semantic {
            ImportedBufferSemantic::HistoryBuffer => "imported_buffer(history_buffer)",
            ImportedBufferSemantic::External => "imported_buffer(external)",
        },
    }
}

pub fn target_alias_kind_name(kind: RenderTargetAliasKind) -> &'static str {
    match kind {
        RenderTargetAliasKind::Color => "color",
        RenderTargetAliasKind::Depth => "depth",
        RenderTargetAliasKind::Texture => "texture",
    }
}

fn target_alias_kind_resource_name(kind: RenderTargetAliasKind) -> &'static str {
    match kind {
        RenderTargetAliasKind::Color => "target_alias(color)",
        RenderTargetAliasKind::Depth => "target_alias(depth)",
        RenderTargetAliasKind::Texture => "target_alias(texture)",
    }
}

pub fn inspect_resources(flow: &RenderFlow) -> Vec<ResourceInspectionEntry> {
    flow.graph()
        .resources
        .resources
        .iter()
        .map(|resource| {
            let lifetime = resource.lifetime();
            ResourceInspectionEntry {
                id: resource.id().to_string(),
                kind: resource_kind_name(resource).to_string(),
                lifetime,
                imported: lifetime.is_imported(),
                target_alias_label: target_alias_label(resource),
                target_alias_kind: target_alias_kind(resource),
            }
        })
        .collect()
}

fn target_alias_label(resource: &RenderResourceDescriptor) -> Option<String> {
    match resource {
        RenderResourceDescriptor::TargetAlias(value) => Some(value.label.clone()),
        _ => None,
    }
}

fn target_alias_kind(resource: &RenderResourceDescriptor) -> Option<String> {
    match resource {
        RenderResourceDescriptor::TargetAlias(value) => {
            Some(target_alias_kind_name(value.kind).to_string())
        }
        _ => None,
    }
}
