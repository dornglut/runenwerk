use crate::plugins::render::inspect::RenderCapturePointIdentity;
use crate::plugins::render::pipelines::{FlowPassKind, FlowPrimitiveTopologyClass};
use wgpu::TextureFormat;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct RenderPassProvenanceState {
    pub frame_index: u64,
    pub records: Vec<RenderPassProvenanceRecord>,
}

impl RenderPassProvenanceState {
    pub fn observe_frame(&mut self, frame_index: u64, records: &[RenderPassProvenanceRecord]) {
        self.frame_index = frame_index;
        self.records.clear();
        self.records.extend_from_slice(records);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPassProvenanceRecord {
    pub frame_index: u64,
    pub flow_id: String,
    pub pass_id: String,
    pub pass_label: String,
    pub pass_kind: FlowPassKind,
    pub order_index: usize,
    pub feature_id: Option<String>,
    pub shader_id: String,
    pub shader_revision: u64,
    pub fallback_used: bool,
    pub pipeline_stats_key: String,
    pub bind_group_layout_signature_hash: u64,
    pub material_specialization_fragment_hash: u64,
    pub view_signature_hash: u64,
    pub feature_runtime_version: u64,
    pub color_formats: Vec<TextureFormat>,
    pub depth_format: Option<TextureFormat>,
    pub sample_count: u32,
    pub primitive_topology_class: FlowPrimitiveTopologyClass,
    pub render_targets: Vec<String>,
    pub sampled_textures: Vec<String>,
    pub storage_textures: Vec<String>,
    pub depth_targets: Vec<String>,
    pub capture_points_available: Vec<RenderCapturePointIdentity>,
}
