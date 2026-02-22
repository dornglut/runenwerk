use anyhow::Result;
use std::sync::Arc;
use pollster::block_on;
use wgpu::*;
use winit::window::Window;

#[derive(Debug)]
pub struct WgpuCtx<'window> {
    pub surface: Option<Surface<'window>>,
    pub surface_config: SurfaceConfiguration,
    pub adapter: Adapter,
    pub adapter_info: AdapterInfo,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

impl<'window> WgpuCtx<'window> {
    pub fn new_headless() -> Result<Self> {
        let instance = Instance::default();

        let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))?;

        let adapter_info = adapter.get_info();

        let (device, queue) = block_on(adapter.request_device(&DeviceDescriptor {
            label: Some("Headless Device"),
            required_features: Default::default(),
            required_limits: Default::default(),
            experimental_features: Default::default(),
            memory_hints: Default::default(),
            trace: Default::default(),
        }))?;

        let format = TextureFormat::Bgra8Unorm;
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: 1,
            height: 1,
            present_mode: PresentMode::Immediate,
            desired_maximum_frame_latency: 0,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![format],
        };

        Ok(Self{
            surface: None,
            surface_config,
            adapter,
            adapter_info,
            device: Arc::from(device),
            queue: Arc::from(queue),
        })

    }
    pub async fn new_async(window: Arc<Window>) -> Result<Self> {
        let instance = Instance::default();
        let surface = instance.create_surface(Arc::clone(&window))?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions{
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
        }).await.expect("failed to find a suitable GPU adapter");

        let adapter_info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor{
                label: None,
                required_features: Features::VERTEX_WRITABLE_STORAGE,
                required_limits: Default::default(),
                experimental_features: Default::default(),
                memory_hints: Default::default(),
                trace: Trace::Off,
        }).await.expect("failed to create a GPU device");

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 0,
            alpha_mode: Default::default(),
            view_formats: vec![format],
        };

        tracing::info!("Created GPU device: {:?}", adapter.get_info());
        tracing::info!("Surface config: {:?}", surface_config);


        surface.configure(&device, &surface_config);

        Ok(Self {
            surface: Some(surface),
            surface_config,
            adapter,
            adapter_info,
            device: Arc::from(device),
            queue: Arc::from(queue),
        })
    }
    pub fn new(window: Arc<Window>) -> Result<Self> {
        pollster::block_on(Self::new_async(window))
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        if let Some(surface) = &mut self.surface {
            surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn get_surface_texture(&mut self) -> std::result::Result<SurfaceTexture, SurfaceError> {
        if let Some(surface) = &mut self.surface {
            surface.get_current_texture()
        } else {
            Err(SurfaceError::Lost)
        }
    }
}