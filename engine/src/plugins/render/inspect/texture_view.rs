use crate::plugins::render::{RenderFlow, RenderResourceDescriptor, ResourceLifetime};

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct RenderTextureInspectorState {
    pub selected_texture: Option<String>,
    pub hovered_texture: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureResourceView {
    pub id: String,
    pub category: String,
    pub lifetime: ResourceLifetime,
}

pub fn inspect_texture_resources(flow: &RenderFlow) -> Vec<TextureResourceView> {
    flow.graph()
        .resources
        .resources
        .iter()
        .filter_map(|resource| {
            let category = match resource {
                RenderResourceDescriptor::SampledTexture(_) => Some("sampled_texture"),
                RenderResourceDescriptor::StorageTexture(_) => Some("storage_texture"),
                RenderResourceDescriptor::ColorTarget(_) => Some("color_target"),
                RenderResourceDescriptor::DepthTarget(_) => Some("depth_target"),
                RenderResourceDescriptor::HistoryTexture(_) => Some("history_texture"),
                RenderResourceDescriptor::ImportedTexture(_) => Some("imported_texture"),
                _ => None,
            }?;

            Some(TextureResourceView {
                id: resource.id().as_str().to_string(),
                category: category.to_string(),
                lifetime: resource.lifetime(),
            })
        })
        .collect()
}
