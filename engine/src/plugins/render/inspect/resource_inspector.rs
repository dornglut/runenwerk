use crate::plugins::render::{RenderFlow, ResourceLifetime};

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct RenderDebugOverlayState {
    pub enabled: bool,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct RenderRuntimeResourceInspectorState {
    pub entries: Vec<RuntimeResourceInspectionEntry>,
}

impl RenderRuntimeResourceInspectorState {
    pub fn observe_runtime_resources(&mut self, entries: &[RuntimeResourceInspectionEntry]) {
        self.entries.clear();
        self.entries.extend_from_slice(entries);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInspectionEntry {
    pub id: String,
    pub kind: String,
    pub lifetime: ResourceLifetime,
    pub imported: bool,
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

pub fn resource_kind_name(
    resource: &crate::plugins::render::RenderResourceDescriptor,
) -> &'static str {
    match resource {
        crate::plugins::render::RenderResourceDescriptor::UniformBuffer(_) => "uniform_buffer",
        crate::plugins::render::RenderResourceDescriptor::StorageBuffer(_) => "storage_buffer",
        crate::plugins::render::RenderResourceDescriptor::SampledTexture(_) => "sampled_texture",
        crate::plugins::render::RenderResourceDescriptor::StorageTexture(_) => "storage_texture",
        crate::plugins::render::RenderResourceDescriptor::ColorTarget(_) => "color_target",
        crate::plugins::render::RenderResourceDescriptor::DepthTarget(_) => "depth_target",
        crate::plugins::render::RenderResourceDescriptor::HistoryTexture(_) => "history_texture",
        crate::plugins::render::RenderResourceDescriptor::ImportedTexture(_) => "imported_texture",
        crate::plugins::render::RenderResourceDescriptor::ImportedBuffer(_) => "imported_buffer",
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
                id: resource.id().as_str().to_string(),
                kind: resource_kind_name(resource).to_string(),
                lifetime,
                imported: lifetime.is_imported(),
            }
        })
        .collect()
}
