use crate::plugins::gpu::GpuWorkResourceIdAllocationError;
use crate::plugins::render::gpu_primitives::GpuPrimitiveValidationError;
use crate::plugins::render::procedural::ProceduralValidationError;
use thiserror::Error;

/// Structured failures produced while authoring a render flow.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RenderFlowAuthoringError {
    #[error(transparent)]
    ResourceIdAllocation(#[from] GpuWorkResourceIdAllocationError),

    #[error(transparent)]
    ProceduralValidation(#[from] ProceduralValidationError),

    #[error(transparent)]
    GpuPrimitiveValidation(#[from] GpuPrimitiveValidationError),
}
