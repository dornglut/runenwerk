use crate::plugins::gpu::{GpuSurfaceCapabilities, GpuTextureFormat};

pub fn preferred_surface_format(caps: &GpuSurfaceCapabilities) -> Option<GpuTextureFormat> {
    caps.formats()
        .iter()
        .copied()
        .find(|format| format.is_srgb())
        .or_else(|| caps.formats().first().copied())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{GpuSurfaceAlphaMode, GpuSurfacePresentMode, GpuTextureUsage};

    fn capabilities(formats: Vec<GpuTextureFormat>) -> GpuSurfaceCapabilities {
        GpuSurfaceCapabilities::from_normalized_facts(
            formats,
            vec![GpuTextureUsage::ColorAttachment],
            vec![GpuSurfacePresentMode::Fifo],
            vec![GpuSurfaceAlphaMode::Opaque],
        )
    }

    #[test]
    fn renderer_prefers_srgb_surface_format_and_handles_empty_capabilities() {
        let caps = capabilities(vec![
            GpuTextureFormat::Bgra8Unorm,
            GpuTextureFormat::Rgba8UnormSrgb,
        ]);
        assert_eq!(
            preferred_surface_format(&caps),
            Some(GpuTextureFormat::Rgba8UnormSrgb)
        );
        assert_eq!(preferred_surface_format(&capabilities(Vec::new())), None);
    }
}
