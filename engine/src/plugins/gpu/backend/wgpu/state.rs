use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue};

/// The sole private owner of WGPU context objects.
#[derive(Debug)]
pub(crate) struct WgpuContextState {
    pub(super) instance: Instance,
    pub(super) adapter: Adapter,
    pub(super) device: Arc<Device>,
    pub(super) queue: Arc<Queue>,
}

/// The sole temporary G4C loan to current renderer encoding. G4C deletes it.
#[derive(Debug)]
pub(crate) struct CurrentRenderDeviceQueue<'a> {
    pub(crate) device: &'a Device,
    pub(crate) queue: &'a Queue,
}
