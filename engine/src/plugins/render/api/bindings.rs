use crate::plugins::render::graph::{RenderPassNode, ResourceGraph};
use crate::plugins::render::resources::RenderFrameDataRegistry;
use crate::plugins::render::{GpuParams, RenderResourceId};
use std::any::{Any, TypeId, type_name};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ProjectedUniformBuffer {
    pub buffer_id: RenderResourceId,
    pub params_type_name: &'static str,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PassUniformProjection {
    pub pass_id: String,
    pub buffers: Vec<ProjectedUniformBuffer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamProjectionErrorKind {
    MissingEcsResourceDeclaration,
    MissingEcsResourceValue,
    MissingUniformBuffer,
    AmbiguousUniformBuffer,
    ProjectionFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamProjectionError {
    pub pass_id: String,
    pub state_type_name: &'static str,
    pub params_type_name: &'static str,
    pub kind: ParamProjectionErrorKind,
    pub details: String,
}

impl std::fmt::Display for ParamProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pass '{}': {}", self.pass_id, self.details,)
    }
}

impl std::error::Error for ParamProjectionError {}

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
    projection: Arc<dyn ParamProjection>,
}

impl std::fmt::Debug for PassParamBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PassParamBinding")
            .field("state_type_name", &self.state_type_name())
            .field("params_type_name", &self.params_type_name())
            .field("requires_surface", &self.requires_surface())
            .finish()
    }
}

impl PassParamBinding {
    pub fn uniform_state<S, P>(build: fn(&S) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        Self {
            projection: Arc::new(UniformStateProjection { build }),
        }
    }

    pub fn uniform_state_with_surface<S, P>(build: fn(&S, (u32, u32)) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        Self {
            projection: Arc::new(UniformStateWithSurfaceProjection { build }),
        }
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

struct UniformStateProjection<S, P>
where
    S: ecs::Component + 'static,
    P: GpuParams + 'static,
{
    build: fn(&S) -> P,
}

impl<S, P> ParamProjection for UniformStateProjection<S, P>
where
    S: ecs::Component + Send + Sync + 'static,
    P: GpuParams + Send + Sync + 'static,
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

struct UniformStateWithSurfaceProjection<S, P>
where
    S: ecs::Component + 'static,
    P: GpuParams + 'static,
{
    build: fn(&S, (u32, u32)) -> P,
}

impl<S, P> ParamProjection for UniformStateWithSurfaceProjection<S, P>
where
    S: ecs::Component + Send + Sync + 'static,
    P: GpuParams + Send + Sync + 'static,
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
    let mut projected_by_buffer = BTreeMap::<String, usize>::new();
    let mut errors = Vec::<ParamProjectionError>::new();

    for binding in &pass.uniform_bindings {
        let state_type_name = binding.state_type_name();
        let params_type_name = binding.params_type_name();
        let pass_id = pass.id.as_str().to_string();

        if !resources.has_ecs_resource(binding.state_type_id()) {
            errors.push(ParamProjectionError {
                pass_id,
                state_type_name,
                params_type_name,
                kind: ParamProjectionErrorKind::MissingEcsResourceDeclaration,
                details: format!(
                    "missing ecs_resource::<{}>() declaration for uniform_state projection",
                    state_type_name
                ),
            });
            continue;
        }

        let Some(state) = frame_data.get_by_type_id(binding.state_type_id()) else {
            errors.push(ParamProjectionError {
                pass_id,
                state_type_name,
                params_type_name,
                kind: ParamProjectionErrorKind::MissingEcsResourceValue,
                details: format!(
                    "missing ECS resource value for '{}' during projection",
                    state_type_name
                ),
            });
            continue;
        };

        let matching_buffers =
            resources.uniform_buffer_ids_by_params_type(binding.params_type_id());
        if matching_buffers.is_empty() {
            errors.push(ParamProjectionError {
                pass_id,
                state_type_name,
                params_type_name,
                kind: ParamProjectionErrorKind::MissingUniformBuffer,
                details: format!(
                    "missing uniform_buffer::<{}>(...) for pass '{}'",
                    params_type_name,
                    pass.id.as_str()
                ),
            });
            continue;
        }
        if matching_buffers.len() > 1 {
            errors.push(ParamProjectionError {
                pass_id,
                state_type_name,
                params_type_name,
                kind: ParamProjectionErrorKind::AmbiguousUniformBuffer,
                details: format!(
                    "multiple uniform buffers match params type '{}' for pass '{}'",
                    params_type_name,
                    pass.id.as_str()
                ),
            });
            continue;
        }

        let Some(bytes) = binding.project_bytes(state, surface_size) else {
            errors.push(ParamProjectionError {
                pass_id,
                state_type_name,
                params_type_name,
                kind: ParamProjectionErrorKind::ProjectionFailed,
                details: format!(
                    "failed to project state '{}' into params '{}'",
                    state_type_name, params_type_name
                ),
            });
            continue;
        };

        let buffer_id = matching_buffers[0].clone();
        let key = buffer_id.as_str().to_string();
        if let Some(existing_index) = projected_by_buffer.get(&key) {
            let existing = &outputs[*existing_index];
            if existing.bytes != bytes {
                errors.push(ParamProjectionError {
                    pass_id,
                    state_type_name,
                    params_type_name,
                    kind: ParamProjectionErrorKind::ProjectionFailed,
                    details: format!(
                        "conflicting uniform_state projections wrote different bytes for uniform buffer '{}'",
                        buffer_id.as_str()
                    ),
                });
            }
            continue;
        }

        projected_by_buffer.insert(key, outputs.len());
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
