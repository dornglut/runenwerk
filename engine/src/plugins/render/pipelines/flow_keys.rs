use crate::plugins::gpu::{
    GpuBindGroupLayoutDescriptor, GpuProgramSourceIdentity, GpuRenderPipelineStateDescriptor,
    GpuSpecializationValueSet,
};
use crate::plugins::render::{RenderFeatureId, RenderFlowId, RenderPassId, RenderPassKind};
use std::hash::{Hash, Hasher};

/// Renderer-local backend-artifact partition that is independent of program-source identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FlowPassPipelineVariant {
    Default,
    ComputeSpecialization(GpuSpecializationValueSet),
}

impl FlowPassPipelineVariant {
    pub fn specialization(&self) -> Option<&GpuSpecializationValueSet> {
        match self {
            Self::Default => None,
            Self::ComputeSpecialization(values) => Some(values),
        }
    }

    pub fn diagnostic_label(&self) -> String {
        match self {
            Self::Default => "default".to_string(),
            Self::ComputeSpecialization(values) => {
                format!("compute-specialization:{:016x}", diagnostic_hash(values))
            }
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
    // Render passes retain one complete typed G4B state contract. Compute has no render state.
    pub render_pipeline_state: Option<GpuRenderPipelineStateDescriptor>,
    // Core owns the full pipeline key type. Feature domains can contribute a
    // specialization fragment hash that is folded into this key.
    pub material_specialization_fragment_hash: u64,
    pub view_signature_hash: u64,
    pub feature_runtime_version: u64,
}

impl FlowPassPipelineKey {
    pub fn stats_key(&self) -> String {
        format!(
            "flow:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}",
            self.flow_id,
            self.pass_id,
            self.pass_kind,
            self.program_source_identity.diagnostic_label(),
            self.pipeline_variant.diagnostic_label(),
            self.primary_bind_group_layout_diagnostic_hash(),
            self.render_pipeline_state_diagnostic_hash(),
            self.material_specialization_fragment_hash,
            self.view_signature_hash,
            self.feature_runtime_version,
        )
    }

    pub fn primary_bind_group_layout_diagnostic_hash(&self) -> u64 {
        diagnostic_hash(&self.primary_bind_group_layout)
    }

    pub fn render_pipeline_state_diagnostic_hash(&self) -> u64 {
        diagnostic_hash(&self.render_pipeline_state)
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
        GpuBindingDeclaration, GpuBindingKey, GpuBindingKind, GpuBindingProvenance, GpuBlendMode,
        GpuCapabilityRequirements, GpuColorTargetStateDescriptor, GpuColorWriteMask,
        GpuFragmentOutputStateDescriptor, GpuMultisampleStateDescriptor,
        GpuPrimitiveStateDescriptor, GpuProgramSourceKey, GpuProgramSourceOwnerId,
        GpuProgramSourceRevision, GpuSamplerClass, GpuShaderStage, GpuShaderStages,
        GpuSpecializationDeclaration, GpuSpecializationEntry, GpuSpecializationKey,
        GpuSpecializationSchema, GpuSpecializationValue, GpuTextureFormat, GpuVertexAttribute,
        GpuVertexBufferLayoutDescriptor, GpuVertexFormat, GpuVertexInputStateDescriptor,
        GpuVertexStepMode,
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

    fn vertex_input_state() -> GpuVertexInputStateDescriptor {
        GpuVertexInputStateDescriptor::new([GpuVertexBufferLayoutDescriptor::new(
            0,
            12,
            GpuVertexStepMode::Vertex,
            [GpuVertexAttribute::new(0, 0, GpuVertexFormat::Float32x3)],
        )
        .unwrap()])
        .unwrap()
    }

    fn render_pipeline_state(
        vertex_input: GpuVertexInputStateDescriptor,
    ) -> GpuRenderPipelineStateDescriptor {
        let color_target = GpuColorTargetStateDescriptor::new(
            GpuTextureFormat::Rgba8Unorm,
            GpuBlendMode::Alpha,
            GpuColorWriteMask::ALL,
        )
        .unwrap();
        GpuRenderPipelineStateDescriptor::new(
            vertex_input,
            Some(GpuFragmentOutputStateDescriptor::new([color_target])),
            GpuPrimitiveStateDescriptor::default(),
            None,
            GpuMultisampleStateDescriptor::default(),
        )
        .unwrap()
    }

    fn specialization(name: &str, value: GpuSpecializationValue) -> GpuSpecializationValueSet {
        let key = GpuSpecializationKey::new(name).unwrap();
        let schema = GpuSpecializationSchema::new([GpuSpecializationDeclaration::new(
            key.clone(),
            value.value_type(),
            None,
            GpuCapabilityRequirements::new(),
        )
        .unwrap()])
        .unwrap();
        GpuSpecializationValueSet::new(schema, [GpuSpecializationEntry::new(key, value)]).unwrap()
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
            render_pipeline_state: Some(render_pipeline_state(
                GpuVertexInputStateDescriptor::new([]).unwrap(),
            )),
            material_specialization_fragment_hash: 3,
            view_signature_hash: 4,
            feature_runtime_version: 5,
        }
    }

    #[test]
    fn stats_key_reflects_typed_layout_source_variant_render_material_and_view_signatures() {
        let identity = source_identity("shader");
        let key = sample_key(identity.clone());
        let same = sample_key(identity);
        let mut changed_source = key.clone();
        changed_source.program_source_identity = source_identity("other-shader");
        let mut changed_variant = key.clone();
        changed_variant.pipeline_variant = FlowPassPipelineVariant::ComputeSpecialization(
            specialization("COUNT", GpuSpecializationValue::U32(4)),
        );
        let mut changed_variant_type = key.clone();
        changed_variant_type.pipeline_variant = FlowPassPipelineVariant::ComputeSpecialization(
            specialization("COUNT", GpuSpecializationValue::I32(4)),
        );
        let mut changed_layout = key.clone();
        changed_layout.primary_bind_group_layout = primary_layout([sampler_binding()]);
        let mut changed_render_state = key.clone();
        changed_render_state.render_pipeline_state = Some(render_pipeline_state(vertex_input_state()));
        let mut changed_material = key.clone();
        changed_material.material_specialization_fragment_hash = 99;
        let mut changed_view = key.clone();
        changed_view.view_signature_hash = 42;
        let mut changed_feature_runtime = key.clone();
        changed_feature_runtime.feature_runtime_version = 11;

        assert_eq!(key.stats_key(), same.stats_key());
        assert_ne!(key.stats_key(), changed_source.stats_key());
        assert_ne!(key.stats_key(), changed_variant.stats_key());
        assert_ne!(changed_variant, changed_variant_type);
        assert_ne!(
            changed_variant.stats_key(),
            changed_variant_type.stats_key()
        );
        assert_ne!(key.stats_key(), changed_layout.stats_key());
        assert_ne!(key.stats_key(), changed_render_state.stats_key());
        assert_ne!(key.stats_key(), changed_material.stats_key());
        assert_ne!(key.stats_key(), changed_view.stats_key());
        assert_ne!(key.stats_key(), changed_feature_runtime.stats_key());
    }
}
