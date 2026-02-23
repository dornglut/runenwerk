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

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("engine_device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
                experimental_features: ExperimentalFeatures::disabled(),
                memory_hints: MemoryHints::Performance,
                trace: Trace::Off,
            })
            .await?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(TextureFormat::is_srgb)
            .unwrap_or(caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![format],
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            surface,
            surface_config,
            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }

    pub fn new(window: Arc<Window>) -> Result<Self> {
        block_on(Self::new_async(window))
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn get_current_texture(&self) -> Result<SurfaceTexture, SurfaceError> {
        self.surface.get_current_texture()
    }
}
