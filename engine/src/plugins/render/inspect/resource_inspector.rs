use crate::plugins::render::{RenderFlow, ResourceLifetime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInspectionEntry {
    pub id: String,
    pub kind: String,
    pub lifetime: ResourceLifetime,
    pub imported: bool,
}

pub fn inspect_resources(flow: &RenderFlow) -> Vec<ResourceInspectionEntry> {
    flow.graph()
        .resources
        .resources
        .iter()
        .map(|resource| {
            let kind = match resource {
                crate::plugins::render::RenderResourceDescriptor::UniformBuffer(_) => {
                    "uniform_buffer"
                }
                crate::plugins::render::RenderResourceDescriptor::StorageBuffer(_) => {
                    "storage_buffer"
                }
                crate::plugins::render::RenderResourceDescriptor::SampledTexture(_) => {
                    "sampled_texture"
                }
                crate::plugins::render::RenderResourceDescriptor::StorageTexture(_) => {
                    "storage_texture"
                }
                crate::plugins::render::RenderResourceDescriptor::ColorTarget(_) => "color_target",
                crate::plugins::render::RenderResourceDescriptor::DepthTarget(_) => "depth_target",
                crate::plugins::render::RenderResourceDescriptor::HistoryTexture(_) => {
                    "history_texture"
                }
                crate::plugins::render::RenderResourceDescriptor::ImportedTexture(_) => {
                    "imported_texture"
                }
                crate::plugins::render::RenderResourceDescriptor::ImportedBuffer(_) => {
                    "imported_buffer"
                }
            };

            let lifetime = resource.lifetime();
            ResourceInspectionEntry {
                id: resource.id().as_str().to_string(),
                kind: kind.to_string(),
                lifetime,
                imported: lifetime.is_imported(),
            }
        })
        .collect()
}
