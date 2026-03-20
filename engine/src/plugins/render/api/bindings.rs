use crate::plugins::render::graph::{RenderPassNode, ResourceGraph};
use crate::plugins::render::renderer::frame_bindings::RenderFrameDataRegistry;
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

#[derive(Debug, Clone, Default)]
pub struct ProjectedUniformSet {
    passes: Vec<PassUniformProjection>,
    by_pass: BTreeMap<String, usize>,
}

impl ProjectedUniformSet {
    pub fn from_passes(passes: Vec<PassUniformProjection>) -> Self {
        let mut by_pass = BTreeMap::<String, usize>::new();
        for (index, pass) in passes.iter().enumerate() {
            by_pass.insert(pass.pass_id.clone(), index);
        }
        Self { passes, by_pass }
    }

    pub fn pass(&self, pass_id: &str) -> Option<&PassUniformProjection> {
        self.by_pass
            .get(pass_id)
            .and_then(|index| self.passes.get(*index))
    }

    pub fn passes(&self) -> &[PassUniformProjection] {
        &self.passes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamProjectionErrorKind {
    MissingStateResourceDeclaration,
    MissingStateResourceValue,
    MissingUniformBuffer,
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
    uniform_id: RenderResourceId,
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
    pub fn uniform_state<S, P>(uniform_id: RenderResourceId, build: fn(&S) -> P) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        Self {
            uniform_id,
            projection: Arc::new(UniformStateProjection { build }),
        }
    }

    pub fn uniform_state_with_surface<S, P>(
        uniform_id: RenderResourceId,
        build: fn(&S, (u32, u32)) -> P,
    ) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        Self {
            uniform_id,
            projection: Arc::new(UniformStateWithSurfaceProjection { build }),
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

struct UniformStateProjection<S, P>
where
    S: ecs::Resource + 'static,
    P: GpuParams + 'static,
{
    build: fn(&S) -> P,
}

impl<S, P> ParamProjection for UniformStateProjection<S, P>
where
    S: ecs::Resource + Send + Sync + 'static,
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
    S: ecs::Resource + 'static,
    P: GpuParams + 'static,
{
    build: fn(&S, (u32, u32)) -> P,
}

impl<S, P> ParamProjection for UniformStateWithSurfaceProjection<S, P>
where
    S: ecs::Resource + Send + Sync + 'static,
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

        if !resources.has_state_resource(binding.state_type_id()) {
            errors.push(ParamProjectionError {
                pass_id,
                state_type_name,
                params_type_name,
                kind: ParamProjectionErrorKind::MissingStateResourceDeclaration,
                details: format!(
                    "missing with_state::<{}>() declaration for uniform projection",
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
                kind: ParamProjectionErrorKind::MissingStateResourceValue,
                details: format!(
                    "missing state resource value for '{}' during projection",
                    state_type_name
                ),
            });
            continue;
        };

        if !resources.has_uniform_buffer(binding.uniform_id()) {
            errors.push(ParamProjectionError {
                pass_id,
                state_type_name,
                params_type_name,
                kind: ParamProjectionErrorKind::MissingUniformBuffer,
                details: format!(
                    "pass '{}' references missing uniform buffer '{}'",
                    pass.id.as_str(),
                    binding.uniform_id().as_str()
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

        let buffer_id = binding.uniform_id().clone();
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
