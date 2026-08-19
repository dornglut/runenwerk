use super::device_request::{enforce_runengpu_instance_flags, request_with_instance};
use super::{CurrentRenderDeviceQueue, WgpuContextState};
use crate::plugins::gpu::{
    GpuContext, GpuContextDescriptor, GpuContextRequestError, GpuContextRequestErrorCategory,
    GpuExecutionPolicy, GpuRealizationPolicies,
};
use wgpu::rwh::HasDisplayHandle;
use wgpu::{
    Instance, InstanceDescriptor, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceTarget,
};

/// Temporary G7 migration bridge; it owns no surface and exposes no raw fields.
#[derive(Debug)]
pub(crate) struct CurrentHostSurfaceBridge<'a> {
    state: &'a WgpuContextState,
}

impl<'a> CurrentHostSurfaceBridge<'a> {
    pub(crate) fn create_surface<'window>(
        &self,
        target: impl Into<SurfaceTarget<'window>>,
    ) -> Result<Surface<'window>, wgpu::CreateSurfaceError> {
        self.state.instance.create_surface(target)
    }

    pub(crate) fn capabilities(&self, surface: &Surface<'_>) -> SurfaceCapabilities {
        surface.get_capabilities(&self.state.adapter)
    }

    pub(crate) fn configure(&self, surface: &Surface<'_>, config: &SurfaceConfiguration) {
        // Surface configuration still dispatches on the current device. Keep it out of any
        // G4C2 scoped-error interval just like the temporary render operation loan.
        let _error_attribution_gate = self.state.error_attribution_gate.acquire();
        surface.configure(&self.state.device, config);
    }
}

impl GpuContext {
    /// The one crate-private current-host admission terminal. G7 deletes it with surface ownership.
    pub(crate) async fn request_for_current_host<'window, T>(
        descriptor: GpuContextDescriptor,
        target: T,
    ) -> Result<(Self, Surface<'window>), GpuContextRequestError>
    where
        T: Into<SurfaceTarget<'window>>
            + HasDisplayHandle
            + core::fmt::Debug
            + Clone
            + Send
            + Sync
            + 'static,
    {
        let instance_descriptor = enforce_runengpu_instance_flags(
            InstanceDescriptor::new_with_display_handle_from_env(Box::new(target.clone())),
        );
        let instance = Instance::new(instance_descriptor);
        let surface = instance.create_surface(target).map_err(|error| {
            GpuContextRequestError::new(
                GpuContextRequestErrorCategory::TemporaryHostCompatibilityFailure,
                error.to_string(),
            )
        })?;
        let context = request_with_instance(
            instance,
            descriptor,
            Some(&surface),
            GpuRealizationPolicies::default(),
            GpuExecutionPolicy::default(),
        )
        .await?;
        Ok((context, surface))
    }

    /// Temporary G7 bridge: create, inspect capabilities, and configure only.
    pub(crate) fn current_host_surface_bridge(&self) -> CurrentHostSurfaceBridge<'_> {
        CurrentHostSurfaceBridge {
            state: &self.backend,
        }
    }

    /// Temporary G4C bridge for already-admitted renderer encoding only.
    pub(crate) fn current_render_device_queue(&self) -> CurrentRenderDeviceQueue<'_> {
        CurrentRenderDeviceQueue {
            device: &self.backend.device,
            queue: &self.backend.queue,
            _error_attribution_gate: self.backend.error_attribution_gate.acquire(),
        }
    }
}
