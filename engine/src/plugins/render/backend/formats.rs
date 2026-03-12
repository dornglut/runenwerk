use wgpu::{SurfaceCapabilities, TextureFormat};

pub fn preferred_surface_format(caps: &SurfaceCapabilities) -> TextureFormat {
    caps.formats
        .iter()
        .copied()
        .find(TextureFormat::is_srgb)
        .unwrap_or(caps.formats[0])
}
