use super::{
    RenderBackendTimingCapabilities, build_surface_config, configure_surface,
    preferred_surface_format, request_device_and_queue,
};
use anyhow::Result;
use pollster::block_on;
use std::sync::Arc;
use wgpu::*;
use winit::window::Window;

#[derive(Debug)]
pub struct WgpuCtx<'window> {
    pub surface: Surface<'window>,
    pub surface_config: SurfaceConfiguration,
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
            surface,
            surface_config,
            device,
            queue,
            timing_capabilities,
        })
    }

    pub fn new(window: Arc<Window>) -> Result<Self> {
        block_on(Self::new_async(window))
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        configure_surface(&self.surface, &self.device, &self.surface_config);
    }

    pub fn get_current_texture(&self) -> Result<SurfaceTexture, SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn timing_capabilities(&self) -> RenderBackendTimingCapabilities {
        self.timing_capabilities
    }
}
