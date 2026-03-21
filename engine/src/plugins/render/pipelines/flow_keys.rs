use crate::plugins::render::RenderPassKind;
use wgpu::TextureFormat;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowPassPipelineKey {
    pub flow_id: String,
    pub pass_id: String,
    pub pass_kind: FlowPassKind,
    pub shader_identity: String,
    pub shader_revision: u64,
    pub bind_group_layout_signature_hash: u64,
    pub color_formats: Vec<TextureFormat>,
    pub depth_format: Option<TextureFormat>,
    pub sample_count: u32,
    pub primitive_topology_class: FlowPrimitiveTopologyClass,
}

impl FlowPassPipelineKey {
    pub fn stats_key(&self) -> String {
        format!(
            "flow:{}:{}:{:?}:{}:{}:{}",
            self.flow_id,
            self.pass_id,
            self.pass_kind,
            self.shader_identity,
            self.shader_revision,
            self.bind_group_layout_signature_hash
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
