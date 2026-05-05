use crate::plugins::render::{RenderFeatureId, RenderFlowId, RenderPassId, RenderPassKind};
use wgpu::TextureFormat;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowPassPipelineKey {
    pub flow_id: RenderFlowId,
    pub pass_id: RenderPassId,
    pub pass_kind: FlowPassKind,
    pub feature_id: Option<RenderFeatureId>,
    pub shader_identity: String,
    pub shader_revision: u64,
    pub bind_group_layout_signature_hash: u64,
    // Core owns the full pipeline key type. Feature domains can contribute a
    // specialization fragment hash that is folded into this key.
    pub material_specialization_fragment_hash: u64,
    pub view_signature_hash: u64,
    pub feature_runtime_version: u64,
    pub color_formats: Vec<TextureFormat>,
    pub depth_format: Option<TextureFormat>,
    pub vertex_layout_signature_hash: u64,
    pub sample_count: u32,
    pub primitive_topology_class: FlowPrimitiveTopologyClass,
}

impl FlowPassPipelineKey {
    pub fn stats_key(&self) -> String {
        format!(
            "flow:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}",
            self.flow_id,
            self.pass_id,
            self.pass_kind,
            self.shader_identity,
            self.shader_revision,
            self.bind_group_layout_signature_hash,
            self.material_specialization_fragment_hash,
            self.view_signature_hash,
            self.feature_runtime_version,
            self.vertex_layout_signature_hash
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowPassBindGroupKey {
    pub pipeline: FlowPassPipelineKey,
    pub resource_generation_signature_hash: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlowPrimitiveTopologyClass {
    None,
    TriangleList,
    TriangleStrip,
    LineList,
    LineStrip,
    PointList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlowPassKind {
    Compute,
    Fullscreen,
    Graphics,
    Copy,
    Present,
    BuiltinUiComposite,
}

impl From<RenderPassKind> for FlowPassKind {
    fn from(value: RenderPassKind) -> Self {
        match value {
            RenderPassKind::Compute => Self::Compute,
            RenderPassKind::Fullscreen => Self::Fullscreen,
            RenderPassKind::Graphics => Self::Graphics,
            RenderPassKind::Copy => Self::Copy,
            RenderPassKind::Present => Self::Present,
            RenderPassKind::BuiltinUiComposite => Self::BuiltinUiComposite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_key() -> FlowPassPipelineKey {
        FlowPassPipelineKey {
            flow_id: RenderFlowId::try_from_raw(1).unwrap(),
            pass_id: RenderPassId::try_from_raw(1).unwrap(),
            pass_kind: FlowPassKind::Fullscreen,
            feature_id: None,
            shader_identity: "shader".to_string(),
            shader_revision: 1,
            bind_group_layout_signature_hash: 2,
            material_specialization_fragment_hash: 3,
            view_signature_hash: 4,
            feature_runtime_version: 5,
            color_formats: vec![TextureFormat::Rgba8Unorm],
            depth_format: None,
            vertex_layout_signature_hash: 0,
            sample_count: 1,
            primitive_topology_class: FlowPrimitiveTopologyClass::TriangleList,
        }
    }

    #[test]
    fn stats_key_reflects_material_and_view_signatures() {
        let key = sample_key();
        let same = sample_key();
        let mut changed_material = key.clone();
        changed_material.material_specialization_fragment_hash = 99;
        let mut changed_view = key.clone();
        changed_view.view_signature_hash = 42;
        let mut changed_feature_runtime = key.clone();
        changed_feature_runtime.feature_runtime_version = 11;

        assert_eq!(key.stats_key(), same.stats_key());
        assert_ne!(key.stats_key(), changed_material.stats_key());
        assert_ne!(key.stats_key(), changed_view.stats_key());
        assert_ne!(key.stats_key(), changed_feature_runtime.stats_key());
    }
}
