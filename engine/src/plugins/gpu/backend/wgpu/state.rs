use super::ResourceRealizationState;
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue};

/// The sole private owner of WGPU context objects.
#[derive(Debug)]
pub(crate) struct WgpuContextState {
    pub(super) instance: Instance,
    pub(super) adapter: Adapter,
    pub(super) device: Arc<Device>,
    pub(super) queue: Arc<Queue>,
    pub(super) resource_realization: ResourceRealizationState,
}

/// Separate temporary backend-operation loan for current renderer execution.
///
/// G4C1 removes generic resource creation, G4C2 removes program/layout/bind-group creation,
/// G4C3 removes pipeline creation, and G5 migrates the remaining execution operations and deletes
/// this loan. It is not the G4C realization-object bridge.
#[derive(Debug)]
pub(crate) struct CurrentRenderDeviceQueue<'a> {
    pub(crate) device: &'a Device,
    pub(crate) queue: &'a Queue,
}
