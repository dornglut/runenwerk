use crate::plugins::gpu::{
    GpuBindGroupLayoutDescriptor, GpuContextAffinity, GpuObservedFragmentOutputSignature,
    GpuObservedProgramInterface, GpuObservedVertexInputSignature, GpuPipelineLayoutDescriptor,
    GpuProgramDescriptor, GpuRuntimeBindingValue,
};
use std::sync::Arc;
use wgpu::{BindGroup, BindGroupLayout, PipelineLayout, ShaderModule};

/// One accepted canonical WGSL module plus the normalized evidence required by G4C3.
pub(crate) struct ProgramRealizationRecord {
    pub(super) affinity: GpuContextAffinity,
    pub(super) descriptor: GpuProgramDescriptor,
    pub(super) object: ShaderModule,
    #[allow(
        dead_code,
        reason = "G4C2 retains normalized evidence for the later G4C3 pipeline-compatibility owner"
    )]
    pub(super) observed_interface: GpuObservedProgramInterface,
    #[allow(
        dead_code,
        reason = "G4C2 retains normalized evidence for the later G4C3 pipeline-compatibility owner"
    )]
    pub(super) vertex_inputs: Vec<GpuObservedVertexInputSignature>,
    #[allow(
        dead_code,
        reason = "G4C2 retains normalized evidence for the later G4C3 pipeline-compatibility owner"
    )]
    pub(super) fragment_outputs: Vec<GpuObservedFragmentOutputSignature>,
}

impl ProgramRealizationRecord {
    pub(crate) const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub(crate) fn descriptor(&self) -> &GpuProgramDescriptor {
        &self.descriptor
    }

    #[allow(
        dead_code,
        reason = "G4C3 will consume retained normalized program-interface evidence"
    )]
    pub(crate) fn observed_interface(&self) -> &GpuObservedProgramInterface {
        &self.observed_interface
    }

    #[allow(
        dead_code,
        reason = "G4C3 will consume retained normalized vertex-input evidence"
    )]
    pub(crate) fn vertex_inputs(&self) -> &[GpuObservedVertexInputSignature] {
        &self.vertex_inputs
    }

    #[allow(
        dead_code,
        reason = "G4C3 will consume retained normalized fragment-output evidence"
    )]
    pub(crate) fn fragment_outputs(&self) -> &[GpuObservedFragmentOutputSignature] {
        &self.fragment_outputs
    }
}

/// One accepted typed bind-group layout.
pub(crate) struct BindGroupLayoutRealizationRecord {
    pub(super) affinity: GpuContextAffinity,
    pub(super) descriptor: GpuBindGroupLayoutDescriptor,
    pub(super) object: BindGroupLayout,
}

impl BindGroupLayoutRealizationRecord {
    pub(crate) const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub(crate) fn descriptor(&self) -> &GpuBindGroupLayoutDescriptor {
        &self.descriptor
    }
}

/// One accepted typed pipeline layout and its exact positional WGPU layout records.
pub(crate) struct PipelineLayoutRealizationRecord {
    pub(super) affinity: GpuContextAffinity,
    pub(super) descriptor: GpuPipelineLayoutDescriptor,
    pub(super) object: PipelineLayout,
    #[allow(
        dead_code,
        reason = "retains descriptor layouts and deterministic lower-index empty slots while a pipeline-layout record is live"
    )]
    pub(super) groups: Vec<Arc<BindGroupLayoutRealizationRecord>>,
}

impl PipelineLayoutRealizationRecord {
    pub(crate) const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub(crate) fn descriptor(&self) -> &GpuPipelineLayoutDescriptor {
        &self.descriptor
    }
}

/// G4C1 resource records retained by a bind group so registry-only reclamation cannot invalidate
/// the backend object dependencies it lends through the successor bridge.
#[allow(
    dead_code,
    reason = "records retain backend dependencies; later G4C3/G5 terminals consume the bind group"
)]
pub(super) enum BindGroupResourceDependency {
    Buffer(Arc<super::super::resource_realization::BufferRealizationRecord>),
    TextureView(Arc<super::super::resource_realization::TextureViewRealizationRecord>),
    Sampler(Arc<super::super::resource_realization::SamplerRealizationRecord>),
}

/// One accepted runtime bind-group request and its exact G4C1/G4C2 dependencies.
pub(crate) struct BindGroupRealizationRecord {
    pub(super) affinity: GpuContextAffinity,
    pub(super) layout: Arc<BindGroupLayoutRealizationRecord>,
    pub(super) values: Vec<GpuRuntimeBindingValue>,
    pub(super) object: BindGroup,
    #[allow(
        dead_code,
        reason = "retains exact G4C1 resource dependencies for authoritative record liveness"
    )]
    pub(super) resources: Vec<BindGroupResourceDependency>,
}

impl BindGroupRealizationRecord {
    pub(crate) const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub(crate) fn layout_descriptor(&self) -> &GpuBindGroupLayoutDescriptor {
        self.layout.descriptor()
    }

    pub(crate) fn values(&self) -> impl ExactSizeIterator<Item = &GpuRuntimeBindingValue> {
        self.values.iter()
    }
}
