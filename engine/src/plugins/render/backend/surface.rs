use wgpu::{
    CompositeAlphaMode, Device, PresentMode, Surface, SurfaceConfiguration, TextureFormat,
    TextureUsages,
};

pub fn build_surface_config(
    width: u32,
    height: u32,
    format: TextureFormat,
    alpha_mode: CompositeAlphaMode,
) -> SurfaceConfiguration {
    SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        width: width.max(1),
        height: height.max(1),
        present_mode: PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode,
        view_formats: vec![format],
    }
}

pub fn configure_surface(surface: &Surface<'_>, device: &Device, config: &SurfaceConfiguration) {
    surface.configure(device, config);
}
