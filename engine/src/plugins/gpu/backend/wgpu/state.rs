use super::{
    PipelineRealizationState, ProgramBindingRealizationState, ResourceRealizationState,
    WgpuDeviceHealth, WgpuErrorAttributionGate, WgpuExecutionState, WgpuSurfaceState,
};
use std::sync::{Arc, MutexGuard};
use wgpu::{Adapter, Device, Instance, Queue};

/// The sole private owner of WGPU context objects.
#[derive(Debug)]
pub(crate) struct WgpuContextState {
    pub(super) instance: Instance,
    pub(super) adapter: Adapter,
    pub(super) device: Arc<Device>,
    pub(super) queue: Arc<Queue>,
    /// One shared terminal device-fault truth for every current same-device operation.
    pub(super) health: Arc<WgpuDeviceHealth>,
    /// One non-reentrant error-scope serialization authority for this device generation.
    pub(super) error_attribution_gate: Arc<WgpuErrorAttributionGate>,
    pub(super) resource_realization: ResourceRealizationState,
    pub(super) program_binding_realization: ProgramBindingRealizationState,
    pub(super) pipeline_realization: PipelineRealizationState,
    pub(super) execution: Arc<WgpuExecutionState>,
    pub(super) surfaces: WgpuSurfaceState,
}

/// Separate temporary backend-operation loan for current renderer execution.
///
/// G4C1 removes generic resource creation, G4C2 removes program/layout/bind-group creation,
/// G4C3 removes pipeline creation, and G5C migrates the remaining renderer execution operations and
/// deletes this loan after G7A surface authority is accepted. It is not the G4C realization-object
/// bridge and it owns no reusable surface state.
pub(crate) struct CurrentRenderDeviceQueue<'a> {
    pub(crate) device: &'a Device,
    pub(crate) queue: &'a Queue,
    // The gate guard deliberately makes this operation loan non-reentrant: G4C1/G4C2
    // realization must have completed before the raw G4C3/G5 interval begins.
    pub(super) _error_attribution_gate: MutexGuard<'a, ()>,
}

impl core::fmt::Debug for CurrentRenderDeviceQueue<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CurrentRenderDeviceQueue")
            .finish_non_exhaustive()
    }
}
