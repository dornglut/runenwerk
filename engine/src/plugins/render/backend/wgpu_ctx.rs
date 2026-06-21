use super::{
    RenderBackendTimingCapabilities, build_surface_config, configure_surface,
    preferred_surface_format, request_device_and_queue,
};
use anyhow::Result;
use pollster::block_on;
use std::collections::BTreeMap;
use std::sync::Arc;
use wgpu::*;
use winit::window::Window;

use super::RenderSurfaceId;

#[derive(Debug)]
struct WgpuSurfaceState<'window> {
    surface: Surface<'window>,
    config: SurfaceConfiguration,
}

#[derive(Debug)]
pub struct WgpuCtx<'window> {
    instance: Instance,
    adapter: Adapter,
    surfaces: BTreeMap<RenderSurfaceId, WgpuSurfaceState<'window>>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub timing_capabilities: RenderBackendTimingCapabilities,
}

impl<'window> WgpuCtx<'window> {
    async fn new_async(window: Arc<Window>) -> Result<Self> {
        let instance = Instance::default();
        let surface = instance.create_surface(Arc::clone(&window))?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue, timing_capabilities) = request_device_and_queue(&adapter).await?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = preferred_surface_format(&caps);
        let surface_config =
            build_surface_config(size.width, size.height, format, caps.alpha_modes[0]);
        configure_surface(&surface, &device, &surface_config);

        Ok(Self {
            instance,
            adapter,
            surfaces: BTreeMap::from([(
                RenderSurfaceId::primary(),
                WgpuSurfaceState {
                    surface,
                    config: surface_config,
                },
            )]),
            device,
            queue,
            timing_capabilities,
        })
    }

    pub fn new(window: Arc<Window>) -> Result<Self> {
        block_on(Self::new_async(window))
    }

    pub fn attach_surface(
        &mut self,
        render_surface_id: RenderSurfaceId,
        window: Arc<Window>,
        target_size_px: (u32, u32),
    ) -> Result<()> {
        let surface = self.instance.create_surface(window)?;
        let caps = surface.get_capabilities(&self.adapter);
        let format = preferred_surface_format(&caps);
        let config = build_surface_config(
            target_size_px.0,
            target_size_px.1,
            format,
            caps.alpha_modes[0],
        );
        configure_surface(&surface, &self.device, &config);
        self.surfaces
            .insert(render_surface_id, WgpuSurfaceState { surface, config });
        Ok(())
    }

    pub fn detach_surface(&mut self, render_surface_id: RenderSurfaceId) -> bool {
        self.surfaces.remove(&render_surface_id).is_some()
    }

    pub fn has_surface(&self, render_surface_id: RenderSurfaceId) -> bool {
        self.surfaces.contains_key(&render_surface_id)
    }

    pub fn surface_config(
        &self,
        render_surface_id: RenderSurfaceId,
    ) -> Option<&SurfaceConfiguration> {
        self.surfaces
            .get(&render_surface_id)
            .map(|state| &state.config)
    }

    pub fn resize(&mut self, render_surface_id: RenderSurfaceId, width: u32, height: u32) -> bool {
        let Some(state) = self.surfaces.get_mut(&render_surface_id) else {
            return false;
        };
        state.config.width = width.max(1);
        state.config.height = height.max(1);
        configure_surface(&state.surface, &self.device, &state.config);
        true
    }

    pub fn get_current_texture(
        &self,
        render_surface_id: RenderSurfaceId,
    ) -> Result<SurfaceTexture, SurfaceError> {
        self.surfaces
            .get(&render_surface_id)
            .ok_or(SurfaceError::Lost)?
            .surface
            .get_current_texture()
    }

    pub fn timing_capabilities(&self) -> RenderBackendTimingCapabilities {
        self.timing_capabilities
    }
}
