use crate::plugins::render::graph::{RenderPassNode, ResourceGraph};
use crate::plugins::render::renderer::frame_bindings::RenderFrameDataRegistry;
use crate::plugins::render::{GpuParams, GpuUniform, RenderPassId, RenderResourceId};
use bytemuck::{Pod, Zeroable};
use std::any::{Any, TypeId, type_name};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ProjectedUniformBuffer {
    pub buffer_id: RenderResourceId,
    pub params_type_name: &'static str,
    pub bytes: Vec<u8>,
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

#[derive(Debug, Clone)]
pub struct PassUniformProjection {
    pub pass_id: RenderPassId,
    pub pass_label: String,
    pub buffers: Vec<ProjectedUniformBuffer>,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectedUniformSet {
    passes: Vec<PassUniformProjection>,
    by_pass: BTreeMap<RenderPassId, usize>,
}

impl ProjectedUniformSet {
    pub fn from_passes(passes: Vec<PassUniformProjection>) -> Self {
        let mut by_pass = BTreeMap::<RenderPassId, usize>::new();
        for (index, pass) in passes.iter().enumerate() {
            by_pass.insert(pass.pass_id, index);
        }
        Self { passes, by_pass }
    }

    pub fn pass(&self, pass_id: RenderPassId) -> Option<&PassUniformProjection> {
        self.by_pass
            .get(&pass_id)
            .and_then(|index| self.passes.get(*index))
    }

    pub fn passes(&self) -> &[PassUniformProjection] {
        &self.passes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParamProjectionError {
    #[error(
        "pass '{pass_label}' is missing with_state::<{state_type_name}>() declaration for uniform projection"
    )]
    MissingStateResourceDeclaration {
        pass_id: RenderPassId,
        pass_label: String,
        state_type_name: &'static str,
        params_type_name: &'static str,
    },

    #[error(
        "pass '{pass_label}' is missing state resource value for '{state_type_name}' during projection"
    )]
    MissingStateResourceValue {
        pass_id: RenderPassId,
        pass_label: String,
        state_type_name: &'static str,
        params_type_name: &'static str,
    },

    #[error("pass '{pass_label}' references missing uniform buffer '{uniform_id:?}'")]
    MissingUniformBuffer {
        pass_id: RenderPassId,
        pass_label: String,
        state_type_name: &'static str,
        params_type_name: &'static str,
        uniform_id: RenderResourceId,
    },

    #[error(
        "pass '{pass_label}' failed to project state '{state_type_name}' into params '{params_type_name}'"
    )]
    ProjectionFailed {
        pass_id: RenderPassId,
        pass_label: String,
        state_type_name: &'static str,
        params_type_name: &'static str,
    },

    #[error("pass '{pass_label}' wrote conflicting bytes for uniform buffer '{uniform_id:?}'")]
    ConflictingUniformProjection {
        pass_id: RenderPassId,
        pass_label: String,
        state_type_name: &'static str,
        params_type_name: &'static str,
        uniform_id: RenderResourceId,
    },
}

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
    uniform_id: RenderResourceId,
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
    pub fn uniform_state<S, P, F>(uniform_id: RenderResourceId, build: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
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

    pub fn uniform_state_with_surface<S, P, F>(uniform_id: RenderResourceId, build: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
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

    pub fn uniform_id(&self) -> &RenderResourceId {
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
    S: ecs::Resource + 'static,
    P: GpuParams + 'static,
    F: Fn(&S) -> P + Send + Sync + 'static,
{
    build: F,
    _marker: PhantomData<fn(&S) -> P>,
}

impl<S, P, F> ParamProjection for UniformStateProjection<S, P, F>
where
    S: ecs::Resource + Send + Sync + 'static,
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
    S: ecs::Resource + 'static,
    P: GpuParams + 'static,
    F: Fn(&S, (u32, u32)) -> P + Send + Sync + 'static,
{
    build: F,
    _marker: PhantomData<fn(&S) -> P>,
}

impl<S, P, F> ParamProjection for UniformStateWithSurfaceProjection<S, P, F>
where
    S: ecs::Resource + Send + Sync + 'static,
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

pub fn project_uniform_bindings_for_pass(
    pass: &RenderPassNode,
    resources: &ResourceGraph,
    frame_data: &RenderFrameDataRegistry<'_>,
    surface_size: (u32, u32),
) -> Result<Vec<ProjectedUniformBuffer>, Vec<ParamProjectionError>> {
    let mut outputs = Vec::<ProjectedUniformBuffer>::new();
    let mut projected_by_buffer = BTreeMap::<RenderResourceId, usize>::new();
    let mut errors = Vec::<ParamProjectionError>::new();

    for binding in &pass.uniform_bindings {
        let state_type_name = binding.state_type_name();
        let params_type_name = binding.params_type_name();
        let pass_id = pass.id;
        let pass_label = pass.label.clone();

        if !resources.has_state_resource(binding.state_type_id()) {
            errors.push(ParamProjectionError::MissingStateResourceDeclaration {
                pass_id,
                pass_label: pass_label.clone(),
                state_type_name,
                params_type_name,
            });
            continue;
        }

        let Some(state) = frame_data.get_by_type_id(binding.state_type_id()) else {
            errors.push(ParamProjectionError::MissingStateResourceValue {
                pass_id,
                pass_label: pass_label.clone(),
                state_type_name,
                params_type_name,
            });
            continue;
        };

        if !resources.has_uniform_buffer(binding.uniform_id()) {
            errors.push(ParamProjectionError::MissingUniformBuffer {
                pass_id,
                pass_label: pass_label.clone(),
                state_type_name,
                params_type_name,
                uniform_id: *binding.uniform_id(),
            });
            continue;
        }

        let Some(bytes) = binding.project_bytes(state, surface_size) else {
            errors.push(ParamProjectionError::ProjectionFailed {
                pass_id,
                pass_label: pass_label.clone(),
                state_type_name,
                params_type_name,
            });
            continue;
        };

        let buffer_id = *binding.uniform_id();
        if let Some(existing_index) = projected_by_buffer.get(&buffer_id) {
            let existing = &outputs[*existing_index];
            if existing.bytes != bytes {
                errors.push(ParamProjectionError::ConflictingUniformProjection {
                    pass_id,
                    pass_label: pass_label.clone(),
                    state_type_name,
                    params_type_name,
                    uniform_id: buffer_id,
                });
            }
            continue;
        }

        projected_by_buffer.insert(buffer_id, outputs.len());
        outputs.push(ProjectedUniformBuffer {
            buffer_id,
            params_type_name,
            bytes,
        });
    }

    if errors.is_empty() {
        Ok(outputs)
    } else {
        Err(errors)
    }
}
