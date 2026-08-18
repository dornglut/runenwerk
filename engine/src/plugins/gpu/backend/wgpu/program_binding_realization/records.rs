use crate::plugins::gpu::{
    GpuBindGroupLayoutDescriptor, GpuBindingKey, GpuBufferHandle, GpuContextAffinity,
    GpuObservedFragmentOutputSignature, GpuObservedProgramInterface,
    GpuObservedVertexInputSignature, GpuPipelineLayoutDescriptor, GpuProgramDescriptor,
    GpuRuntimeBindingResource, GpuRuntimeBindingValue, GpuRuntimeTextureViewBinding,
    GpuSamplerHandle,
};
use core::num::NonZeroU64;
use std::sync::Arc;
use wgpu::{BindGroup, BindGroupLayout, PipelineLayout, ShaderModule};

/// Private physical bind-group resource identity. Dynamic offsets are deliberately absent because
/// WGPU applies them at bind time rather than storing them in the bind-group object.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum StaticBindGroupResource {
    Buffer {
        handle: GpuBufferHandle,
        offset: u64,
        size: NonZeroU64,
    },
    TextureView(GpuRuntimeTextureViewBinding),
    Sampler(GpuSamplerHandle),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct StaticBindGroupValue {
    key: GpuBindingKey,
    resources: Vec<StaticBindGroupResource>,
}

impl StaticBindGroupValue {
    pub(super) fn from_runtime(value: &GpuRuntimeBindingValue) -> Self {
        let resources = value
            .resources()
            .map(|resource| match resource {
                GpuRuntimeBindingResource::Buffer(buffer) => StaticBindGroupResource::Buffer {
                    handle: buffer.handle().clone(),
                    offset: buffer.offset(),
                    size: buffer.size(),
                },
                GpuRuntimeBindingResource::TextureView(view) => {
                    StaticBindGroupResource::TextureView(view.clone())
                }
                GpuRuntimeBindingResource::Sampler(sampler) => {
                    StaticBindGroupResource::Sampler(sampler.clone())
                }
            })
            .collect();
        Self {
            key: value.key(),
            resources,
        }
    }
}

pub(super) fn static_bind_group_values(
    values: impl IntoIterator<Item = GpuRuntimeBindingValue>,
) -> Vec<StaticBindGroupValue> {
    values
        .into_iter()
        .map(|value| StaticBindGroupValue::from_runtime(&value))
        .collect()
}

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

    pub(crate) fn wgpu_object(&self) -> &ShaderModule {
        &self.object
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

    pub(crate) fn wgpu_object(&self) -> &PipelineLayout {
        &self.object
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

/// One physical bind group and its exact G4C1/G4C2 dependencies. Per-use dynamic offsets are not
/// record state; they remain logical execution-use state above this physical object.
pub(crate) struct BindGroupRealizationRecord {
    pub(super) affinity: GpuContextAffinity,
    pub(super) layout: Arc<BindGroupLayoutRealizationRecord>,
    pub(super) static_values: Vec<StaticBindGroupValue>,
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

    pub(super) fn static_values(&self) -> &[StaticBindGroupValue] {
        &self.static_values
    }
}
