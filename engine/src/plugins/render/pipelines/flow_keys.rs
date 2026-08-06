use crate::plugins::gpu::{GpuBindGroupLayoutDescriptor, GpuProgramSourceIdentity};
use crate::plugins::render::{RenderFeatureId, RenderFlowId, RenderPassId, RenderPassKind};
use std::hash::{Hash, Hasher};
use wgpu::TextureFormat;

/// Renderer-local backend-artifact partition that is independent of program-source identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FlowPassPipelineVariant {
    Default,
    ComputeSpecialization(String),
}

impl FlowPassPipelineVariant {
    pub fn diagnostic_label(&self) -> &str {
        match self {
            Self::Default => "default",
            Self::ComputeSpecialization(signature) => signature.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowPassPipelineKey {
    pub flow_id: RenderFlowId,
    pub pass_id: RenderPassId,
    pub pass_kind: FlowPassKind,
    pub feature_id: Option<RenderFeatureId>,
    pub program_source_identity: GpuProgramSourceIdentity,
    pub pipeline_variant: FlowPassPipelineVariant,
    // Current renderer group 0. Material group 1 remains a separately classified migration.
    pub primary_bind_group_layout: GpuBindGroupLayoutDescriptor,
    // Core owns the full pipeline key type. Feature domains can contribute a
    // specialization fragment hash that is folded into this key.
    pub material_specialization_fragment_hash: u64,
    pub view_signature_hash: u64,
    pub feature_runtime_version: u64,
    pub color_formats: Vec<TextureFormat>,
    pub depth_format: Option<TextureFormat>,
    pub vertex_layout_signature_hash: u64,
    pub raster_state_signature_hash: u64,
    pub sample_count: u32,
    pub primitive_topology_class: FlowPrimitiveTopologyClass,
}

impl FlowPassPipelineKey {
    pub fn stats_key(&self) -> String {
        format!(
            "flow:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.flow_id,
            self.pass_id,
            self.pass_kind,
            self.program_source_identity.diagnostic_label(),
            self.pipeline_variant.diagnostic_label(),
            self.primary_bind_group_layout_diagnostic_hash(),
            self.material_specialization_fragment_hash,
            self.view_signature_hash,
            self.feature_runtime_version,
            self.vertex_layout_signature_hash,
            self.raster_state_signature_hash
        )
    }

    pub fn primary_bind_group_layout_diagnostic_hash(&self) -> u64 {
        diagnostic_hash(&self.primary_bind_group_layout)
    }
}

fn diagnostic_hash(value: &impl Hash) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
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
    use crate::plugins::gpu::{
        GpuBindingDeclaration, GpuBindingKey, GpuBindingKind, GpuBindingProvenance,
        GpuProgramSourceKey, GpuProgramSourceOwnerId, GpuProgramSourceRevision, GpuSamplerClass,
        GpuShaderStage, GpuShaderStages,
    };

    fn source_identity(key: &str) -> GpuProgramSourceIdentity {
        GpuProgramSourceIdentity::new(
            GpuProgramSourceOwnerId::allocate().unwrap(),
            GpuProgramSourceKey::new(key).unwrap(),
            GpuProgramSourceRevision::try_from_raw(1).unwrap(),
        )
    }

    fn primary_layout(
        bindings: impl IntoIterator<Item = GpuBindingDeclaration>,
    ) -> GpuBindGroupLayoutDescriptor {
        GpuBindGroupLayoutDescriptor::new(0, bindings).unwrap()
    }

    fn sampler_binding() -> GpuBindingDeclaration {
        GpuBindingDeclaration::new(
            GpuBindingKey::try_new(0, 0).unwrap(),
            GpuShaderStages::one(GpuShaderStage::Fragment),
            GpuBindingKind::sampler(GpuSamplerClass::Filtering),
            None,
            "sample-sampler",
            GpuBindingProvenance::new("flow-key-test", None).unwrap(),
        )
        .unwrap()
    }

    fn sample_key(program_source_identity: GpuProgramSourceIdentity) -> FlowPassPipelineKey {
        FlowPassPipelineKey {
            flow_id: RenderFlowId::try_from_raw(1).unwrap(),
            pass_id: RenderPassId::try_from_raw(1).unwrap(),
            pass_kind: FlowPassKind::Fullscreen,
            feature_id: None,
            program_source_identity,
            pipeline_variant: FlowPassPipelineVariant::Default,
            primary_bind_group_layout: primary_layout([]),
            material_specialization_fragment_hash: 3,
            view_signature_hash: 4,
            feature_runtime_version: 5,
            color_formats: vec![TextureFormat::Rgba8Unorm],
            depth_format: None,
            vertex_layout_signature_hash: 0,
            raster_state_signature_hash: 0,
            sample_count: 1,
            primitive_topology_class: FlowPrimitiveTopologyClass::TriangleList,
        }
    }

    #[test]
    fn stats_key_reflects_typed_layout_source_variant_material_and_view_signatures() {
        let identity = source_identity("shader");
        let key = sample_key(identity.clone());
        let same = sample_key(identity);
        let mut changed_source = key.clone();
        changed_source.program_source_identity = source_identity("other-shader");
        let mut changed_variant = key.clone();
        changed_variant.pipeline_variant =
            FlowPassPipelineVariant::ComputeSpecialization("COUNT=4".to_string());
        let mut changed_layout = key.clone();
        changed_layout.primary_bind_group_layout = primary_layout([sampler_binding()]);
        let mut changed_material = key.clone();
        changed_material.material_specialization_fragment_hash = 99;
        let mut changed_view = key.clone();
        changed_view.view_signature_hash = 42;
        let mut changed_feature_runtime = key.clone();
        changed_feature_runtime.feature_runtime_version = 11;

        assert_eq!(key.stats_key(), same.stats_key());
        assert_ne!(key.stats_key(), changed_source.stats_key());
        assert_ne!(key.stats_key(), changed_variant.stats_key());
        assert_ne!(key.stats_key(), changed_layout.stats_key());
        assert_ne!(key.stats_key(), changed_material.stats_key());
        assert_ne!(key.stats_key(), changed_view.stats_key());
        assert_ne!(key.stats_key(), changed_feature_runtime.stats_key());
    }
}
