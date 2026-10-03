use crate::plugins::render::{GpuParams, GpuUniform};
use bytemuck::{Pod, Zeroable};
use runen_gpu::{
    GpuBindingKey, GpuStorageBufferAccess, GpuStorageTextureAccess, GpuTextureSampleClass,
    GpuWorkResourceId,
};
use std::any::{Any, TypeId, type_name};
use std::marker::PhantomData;
use std::sync::Arc;

/// Explicit shader-visible resource identity for one render pass.
///
/// `GpuBindingKey` is the only shader slot identity. Resource hazard/access
/// lists and uniform projections remain separate execution semantics and must
/// never be used to reconstruct binding numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderShaderBinding {
    key: GpuBindingKey,
    resource: RenderShaderBindingResource,
}

impl RenderShaderBinding {
    pub const fn new(key: GpuBindingKey, resource: RenderShaderBindingResource) -> Self {
        Self { key, resource }
    }

    pub const fn key(&self) -> GpuBindingKey {
        self.key
    }

    pub const fn resource(&self) -> &RenderShaderBindingResource {
        &self.resource
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderShaderBindingResource {
    SampledTexture {
        resource: GpuWorkResourceId,
        sample_class: GpuTextureSampleClass,
    },
    Sampler,
    StorageTexture {
        resource: GpuWorkResourceId,
        access: GpuStorageTextureAccess,
    },
    UniformBuffer(GpuWorkResourceId),
    StorageBuffer {
        resource: GpuWorkResourceId,
        access: GpuStorageBufferAccess,
    },
}

impl RenderShaderBindingResource {
    pub const fn resource_id(&self) -> Option<GpuWorkResourceId> {
        match self {
            Self::SampledTexture { resource, .. }
            | Self::StorageTexture { resource, .. }
            | Self::UniformBuffer(resource)
            | Self::StorageBuffer { resource, .. } => Some(*resource),
            Self::Sampler => None,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct RenderFixedStepIterationUniformRaw {
    pub substep_index: u32,
    pub submitted_substeps: u32,
    pub max_substeps: u32,
    pub saturated_frames_low: u32,
    pub fixed_dt_seconds: f32,
    pub accumulator_seconds: f32,
    pub reserved0: f32,
    pub reserved1: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderFixedStepIterationUniform {
    pub substep_index: u32,
    pub submitted_substeps: u32,
    pub max_substeps: u32,
    pub saturated_frames_low: u32,
    pub fixed_dt_seconds: f32,
    pub accumulator_seconds: f32,
}

impl RenderFixedStepIterationUniform {
    pub const fn new(
        substep_index: u32,
        submitted_substeps: u32,
        max_substeps: u32,
        saturated_frames: u64,
        fixed_dt_seconds: f32,
        accumulator_seconds: f32,
    ) -> Self {
        Self {
            substep_index,
            submitted_substeps,
            max_substeps,
            saturated_frames_low: saturated_frames as u32,
            fixed_dt_seconds,
            accumulator_seconds,
        }
    }

    pub fn with_substep_index(self, substep_index: u32) -> Self {
        Self {
            substep_index,
            ..self
        }
    }

    pub fn to_uniform_bytes(self) -> Vec<u8> {
        let raw = self.to_gpu();
        bytemuck::bytes_of(&raw).to_vec()
    }

    pub fn from_uniform_bytes(bytes: &[u8]) -> Option<Self> {
        bytemuck::try_from_bytes::<RenderFixedStepIterationUniformRaw>(bytes)
            .ok()
            .map(|raw| Self {
                substep_index: raw.substep_index,
                submitted_substeps: raw.submitted_substeps,
                max_substeps: raw.max_substeps,
                saturated_frames_low: raw.saturated_frames_low,
                fixed_dt_seconds: raw.fixed_dt_seconds,
                accumulator_seconds: raw.accumulator_seconds,
            })
    }
}

impl GpuParams for RenderFixedStepIterationUniform {
    type Raw = RenderFixedStepIterationUniformRaw;

    fn to_gpu(&self) -> Self::Raw {
        Self::Raw {
            substep_index: self.substep_index,
            submitted_substeps: self.submitted_substeps,
            max_substeps: self.max_substeps,
            saturated_frames_low: self.saturated_frames_low,
            fixed_dt_seconds: self.fixed_dt_seconds,
            accumulator_seconds: self.accumulator_seconds,
            reserved0: 0.0,
            reserved1: 0.0,
        }
    }
}

impl GpuUniform for RenderFixedStepIterationUniform {}

pub trait ParamProjection: Send + Sync {
    fn state_type_id(&self) -> TypeId;
    fn state_type_name(&self) -> &'static str;
    fn params_type_id(&self) -> TypeId;
    fn params_type_name(&self) -> &'static str;
    fn requires_surface(&self) -> bool;
    fn project_bytes(&self, state: &dyn Any, surface_size: (u32, u32)) -> Option<Vec<u8>>;
}

#[derive(Clone)]
pub struct PassParamBinding {
    uniform_id: GpuWorkResourceId,
    projection: Arc<dyn ParamProjection>,
}

impl std::fmt::Debug for PassParamBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PassParamBinding")
            .field("uniform_id", &self.uniform_id)
            .field("state_type_name", &self.state_type_name())
            .field("params_type_name", &self.params_type_name())
            .field("requires_surface", &self.requires_surface())
            .finish()
    }
}

impl PassParamBinding {
    pub fn uniform_state<S, P, F>(uniform_id: GpuWorkResourceId, build: F) -> Self
    where
        S: runen_ecs::Resource + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> P + Send + Sync + 'static,
    {
        Self {
            uniform_id,
            projection: Arc::new(UniformStateProjection {
                build,
                _marker: PhantomData,
            }),
        }
    }

    pub fn uniform_state_with_surface<S, P, F>(uniform_id: GpuWorkResourceId, build: F) -> Self
    where
        S: runen_ecs::Resource + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
        F: Fn(&S, (u32, u32)) -> P + Send + Sync + 'static,
    {
        Self {
            uniform_id,
            projection: Arc::new(UniformStateWithSurfaceProjection {
                build,
                _marker: PhantomData,
            }),
        }
    }

    pub fn uniform_id(&self) -> &GpuWorkResourceId {
        &self.uniform_id
    }

    pub fn state_type_id(&self) -> TypeId {
        self.projection.state_type_id()
    }

    pub fn state_type_name(&self) -> &'static str {
        self.projection.state_type_name()
    }

    pub fn params_type_id(&self) -> TypeId {
        self.projection.params_type_id()
    }

    pub fn params_type_name(&self) -> &'static str {
        self.projection.params_type_name()
    }

    pub fn requires_surface(&self) -> bool {
        self.projection.requires_surface()
    }

    pub fn project_bytes(&self, state: &dyn Any, surface_size: (u32, u32)) -> Option<Vec<u8>> {
        self.projection.project_bytes(state, surface_size)
    }
}

struct UniformStateProjection<S, P, F>
where
    S: runen_ecs::Resource + 'static,
    P: GpuParams + 'static,
    F: Fn(&S) -> P + Send + Sync + 'static,
{
    build: F,
    _marker: PhantomData<fn(&S) -> P>,
}

impl<S, P, F> ParamProjection for UniformStateProjection<S, P, F>
where
    S: runen_ecs::Resource + Send + Sync + 'static,
    P: GpuParams + Send + Sync + 'static,
    F: Fn(&S) -> P + Send + Sync + 'static,
{
    fn state_type_id(&self) -> TypeId {
        TypeId::of::<S>()
    }

    fn state_type_name(&self) -> &'static str {
        type_name::<S>()
    }

    fn params_type_id(&self) -> TypeId {
        TypeId::of::<P>()
    }

    fn params_type_name(&self) -> &'static str {
        type_name::<P>()
    }

    fn requires_surface(&self) -> bool {
        false
    }

    fn project_bytes(&self, state: &dyn Any, _surface_size: (u32, u32)) -> Option<Vec<u8>> {
        let state = state.downcast_ref::<S>()?;
        let params = (self.build)(state);
        let raw = params.to_gpu();
        Some(crate::plugins::render::bytemuck::bytes_of(&raw).to_vec())
    }
}

struct UniformStateWithSurfaceProjection<S, P, F>
where
    S: runen_ecs::Resource + 'static,
    P: GpuParams + 'static,
    F: Fn(&S, (u32, u32)) -> P + Send + Sync + 'static,
{
    build: F,
    _marker: PhantomData<fn(&S) -> P>,
}

impl<S, P, F> ParamProjection for UniformStateWithSurfaceProjection<S, P, F>
where
    S: runen_ecs::Resource + Send + Sync + 'static,
    P: GpuParams + Send + Sync + 'static,
    F: Fn(&S, (u32, u32)) -> P + Send + Sync + 'static,
{
    fn state_type_id(&self) -> TypeId {
        TypeId::of::<S>()
    }

    fn state_type_name(&self) -> &'static str {
        type_name::<S>()
    }

    fn params_type_id(&self) -> TypeId {
        TypeId::of::<P>()
    }

    fn params_type_name(&self) -> &'static str {
        type_name::<P>()
    }

    fn requires_surface(&self) -> bool {
        true
    }

    fn project_bytes(&self, state: &dyn Any, surface_size: (u32, u32)) -> Option<Vec<u8>> {
        let state = state.downcast_ref::<S>()?;
        let params = (self.build)(state, surface_size);
        let raw = params.to_gpu();
        Some(crate::plugins::render::bytemuck::bytes_of(&raw).to_vec())
    }
}
