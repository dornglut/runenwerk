use crate::plugins::gpu::{
    GpuComputePipelineDescriptor, GpuPipelineLayoutDescriptor, GpuProgramDescriptor,
    GpuRenderPipelineDescriptor, GpuRenderPipelineStateDescriptor, GpuSpecializationValueSet,
};
use crate::plugins::render::{RenderFeatureId, RenderFlowId, RenderPassId, RenderPassKind};
use std::hash::{Hash, Hasher};

/// Renderer-local discrimination over the complete generic G4B pipeline contracts.
///
/// This adds no renderer semantics to RunenGPU. It only lets renderer-local
/// cache partitioning retain one complete generic pipeline descriptor rather
/// than mirroring its source/layout/specialization/state fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FlowPassPipelineDescriptor {
    Compute(GpuComputePipelineDescriptor),
    Render(GpuRenderPipelineDescriptor),
}

impl FlowPassPipelineDescriptor {
    pub fn program(&self) -> &GpuProgramDescriptor {
        match self {
            Self::Compute(descriptor) => descriptor.program(),
            Self::Render(descriptor) => descriptor.program(),
        }
    }

    pub fn layout(&self) -> &GpuPipelineLayoutDescriptor {
        match self {
            Self::Compute(descriptor) => descriptor.layout(),
            Self::Render(descriptor) => descriptor.layout(),
        }
    }

    pub fn specialization(&self) -> &GpuSpecializationValueSet {
        match self {
            Self::Compute(descriptor) => descriptor.specialization(),
            Self::Render(descriptor) => descriptor.specialization(),
        }
    }

    pub fn render_state(&self) -> Option<&GpuRenderPipelineStateDescriptor> {
        match self {
            Self::Compute(_) => None,
            Self::Render(descriptor) => Some(descriptor.state()),
        }
    }

    pub fn diagnostic_label(&self) -> String {
        match self {
            Self::Compute(descriptor) => {
                format!("compute:{:016x}", diagnostic_hash(descriptor))
            }
            Self::Render(descriptor) => format!("render:{:016x}", diagnostic_hash(descriptor)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowPassPipelineKey {
    pub flow_id: RenderFlowId,
    pub pass_id: RenderPassId,
    pub pass_kind: FlowPassKind,
    pub feature_id: Option<RenderFeatureId>,
    pub pipeline_descriptor: FlowPassPipelineDescriptor,
}

impl FlowPassPipelineKey {
    pub fn stats_key(&self) -> String {
        format!(
            "flow:{}:{}:{:?}:{}",
            self.flow_id,
            self.pass_id,
            self.pass_kind,
            self.pipeline_descriptor.diagnostic_label(),
        )
    }

    pub fn pipeline_descriptor_diagnostic_hash(&self) -> u64 {
        diagnostic_hash(&self.pipeline_descriptor)
    }

    pub fn pipeline_layout_diagnostic_hash(&self) -> u64 {
        diagnostic_hash(self.pipeline_descriptor.layout())
    }

    pub fn primary_bind_group_layout_diagnostic_hash(&self) -> u64 {
        diagnostic_hash(&self.pipeline_descriptor.layout().group(0))
    }

    pub fn render_pipeline_state_diagnostic_hash(&self) -> u64 {
        diagnostic_hash(&self.pipeline_descriptor.render_state())
    }

    pub fn render_pipeline_state(&self) -> Option<&GpuRenderPipelineStateDescriptor> {
        self.pipeline_descriptor.render_state()
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
        GpuAdmittedProgramSource, GpuBindGroupLayoutDescriptor, GpuBindingDeclaration,
        GpuBindingKey, GpuBindingKind, GpuBindingProvenance, GpuBlendMode,
        GpuCapabilityRequirements, GpuColorTargetStateDescriptor, GpuColorWriteMask,
        GpuEntryPointDescriptor, GpuEntryPointName, GpuFragmentOutputStateDescriptor,
        GpuMultisampleStateDescriptor, GpuPrimitiveStateDescriptor, GpuProgramInterfaceDescriptor,
        GpuProgramSourceIdentity, GpuProgramSourceKey, GpuProgramSourceOwnerId,
        GpuProgramSourceProvenance, GpuProgramSourceRegistry, GpuProgramSourceRevision,
        GpuRenderEntryPoints, GpuSamplerClass, GpuShaderStage, GpuShaderStages,
        GpuSpecializationDeclaration, GpuSpecializationEntry, GpuSpecializationKey,
        GpuSpecializationSchema, GpuSpecializationValue, GpuTextureFormat, GpuVertexAttribute,
        GpuVertexBufferLayoutDescriptor, GpuVertexFormat, GpuVertexInputStateDescriptor,
        GpuVertexStepMode,
    };

    fn admitted_source(key: &str) -> GpuAdmittedProgramSource {
        let identity = GpuProgramSourceIdentity::new(
            GpuProgramSourceOwnerId::allocate().unwrap(),
            GpuProgramSourceKey::new(key).unwrap(),
            GpuProgramSourceRevision::try_from_raw(1).unwrap(),
        );
        let mut registry = GpuProgramSourceRegistry::new(4, 4096).unwrap();
        registry
            .admit_wgsl(
                identity,
                "fn shader_contract_test() {}",
                GpuProgramSourceProvenance::new("flow-key-test", None).unwrap(),
            )
            .unwrap()
    }

    fn bind_group_layout(
        group: u32,
        bindings: impl IntoIterator<Item = GpuBindingDeclaration>,
    ) -> GpuBindGroupLayoutDescriptor {
        GpuBindGroupLayoutDescriptor::new(group, bindings).unwrap()
    }

    fn pipeline_layout(
        groups: impl IntoIterator<Item = GpuBindGroupLayoutDescriptor>,
    ) -> GpuPipelineLayoutDescriptor {
        GpuPipelineLayoutDescriptor::new(groups).unwrap()
    }

    fn interface(layout: &GpuPipelineLayoutDescriptor) -> GpuProgramInterfaceDescriptor {
        GpuProgramInterfaceDescriptor::new(
            layout.groups().flat_map(|group| group.bindings().cloned()),
        )
        .unwrap()
    }

    fn empty_specialization() -> GpuSpecializationValueSet {
        GpuSpecializationValueSet::new(GpuSpecializationSchema::new([]).unwrap(), []).unwrap()
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

    fn sampler_binding(group: u64, binding: u64) -> GpuBindingDeclaration {
        GpuBindingDeclaration::new(
            GpuBindingKey::try_new(group, binding).unwrap(),
            GpuShaderStages::one(GpuShaderStage::Fragment),
            GpuBindingKind::sampler(GpuSamplerClass::Filtering),
            None,
            "sample-sampler",
            GpuBindingProvenance::new("flow-key-test", None).unwrap(),
        )
        .unwrap()
    }

    fn render_key(
        source: GpuAdmittedProgramSource,
        layout: GpuPipelineLayoutDescriptor,
        state: GpuRenderPipelineStateDescriptor,
    ) -> FlowPassPipelineKey {
        let interface = interface(&layout);
        let vertex = GpuEntryPointName::new("vs_main").unwrap();
        let fragment = GpuEntryPointName::new("fs_main").unwrap();
        let program = GpuProgramDescriptor::new(
            source,
            interface.clone(),
            [
                GpuEntryPointDescriptor::new(
                    vertex.clone(),
                    GpuShaderStage::Vertex,
                    interface.clone(),
                ),
                GpuEntryPointDescriptor::new(fragment.clone(), GpuShaderStage::Fragment, interface),
            ],
        )
        .unwrap();
        let descriptor = GpuRenderPipelineDescriptor::new(
            program,
            GpuRenderEntryPoints::new(vertex, Some(fragment)),
            state,
            layout,
            empty_specialization(),
            GpuCapabilityRequirements::new(),
        )
        .unwrap();
        FlowPassPipelineKey {
            flow_id: RenderFlowId::try_from_raw(1).unwrap(),
            pass_id: RenderPassId::try_from_raw(1).unwrap(),
            pass_kind: FlowPassKind::Fullscreen,
            feature_id: None,
            pipeline_descriptor: FlowPassPipelineDescriptor::Render(descriptor),
        }
    }

    fn compute_key(
        source: GpuAdmittedProgramSource,
        specialization: GpuSpecializationValueSet,
    ) -> FlowPassPipelineKey {
        let layout = pipeline_layout([]);
        let interface = interface(&layout);
        let entry_point = GpuEntryPointName::new("cs_main").unwrap();
        let program = GpuProgramDescriptor::new(
            source,
            interface.clone(),
            [GpuEntryPointDescriptor::new(
                entry_point.clone(),
                GpuShaderStage::Compute,
                interface,
            )],
        )
        .unwrap();
        let descriptor = GpuComputePipelineDescriptor::new(
            program,
            entry_point,
            layout,
            specialization,
            GpuCapabilityRequirements::new(),
        )
        .unwrap();
        FlowPassPipelineKey {
            flow_id: RenderFlowId::try_from_raw(2).unwrap(),
            pass_id: RenderPassId::try_from_raw(2).unwrap(),
            pass_kind: FlowPassKind::Compute,
            feature_id: None,
            pipeline_descriptor: FlowPassPipelineDescriptor::Compute(descriptor),
        }
    }

    #[test]
    fn stats_key_reflects_complete_render_descriptor_semantics() {
        let source = admitted_source("shader");
        let base_layout = pipeline_layout([]);
        let base_state = render_pipeline_state(GpuVertexInputStateDescriptor::new([]).unwrap());
        let key = render_key(source.clone(), base_layout.clone(), base_state.clone());
        let same = key.clone();
        let changed_source = render_key(
            admitted_source("other-shader"),
            base_layout.clone(),
            base_state.clone(),
        );
        let changed_group0_layout = render_key(
            source.clone(),
            pipeline_layout([bind_group_layout(0, [sampler_binding(0, 0)])]),
            base_state.clone(),
        );
        let changed_group1_layout = render_key(
            source.clone(),
            pipeline_layout([bind_group_layout(1, [sampler_binding(1, 0)])]),
            base_state,
        );
        let changed_render_state = render_key(
            source,
            base_layout,
            render_pipeline_state(vertex_input_state()),
        );

        assert_eq!(key.stats_key(), same.stats_key());
        assert_ne!(key.stats_key(), changed_source.stats_key());
        assert_ne!(key.stats_key(), changed_group0_layout.stats_key());
        assert_ne!(key.stats_key(), changed_group1_layout.stats_key());
        assert_ne!(key.stats_key(), changed_render_state.stats_key());
    }

    #[test]
    fn compute_descriptor_preserves_typed_specialization_identity() {
        let source = admitted_source("compute-shader");
        let default = compute_key(source.clone(), empty_specialization());
        let typed = compute_key(
            source.clone(),
            specialization("COUNT", GpuSpecializationValue::U32(4)),
        );
        let signed = compute_key(
            source,
            specialization("COUNT", GpuSpecializationValue::I32(4)),
        );

        assert_ne!(default, typed);
        assert_ne!(typed, signed);
        assert_ne!(default.stats_key(), typed.stats_key());
        assert_ne!(typed.stats_key(), signed.stats_key());
    }
}
