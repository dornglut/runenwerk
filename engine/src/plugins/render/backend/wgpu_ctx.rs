use super::{build_surface_config, preferred_surface_format};
use crate::plugins::gpu::{
    GpuCapabilityFeature, GpuCapabilityProfile, GpuCapabilityRequirement, GpuContext,
    GpuContextDescriptor, GpuFormatRole, GpuPowerPreference, GpuPreferredFallback,
    GpuTextureFormat,
};
use anyhow::Result;
use pollster::block_on;
use std::collections::BTreeMap;
use std::sync::Arc;
use wgpu::{Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture};
use winit::window::Window;

use super::RenderSurfaceId;

#[derive(Debug)]
struct WgpuSurfaceState<'window> {
    surface: Surface<'window>,
    config: SurfaceConfiguration,
}

#[derive(Debug)]
pub struct WgpuCtx<'window> {
    context: GpuContext,
    surfaces: BTreeMap<RenderSurfaceId, WgpuSurfaceState<'window>>,
}

impl<'window> WgpuCtx<'window> {
    async fn new_async(window: Arc<Window>) -> Result<Self> {
        let mut requirements = GpuCapabilityProfile::DesktopPresentationBaseline.requirements();
        requirements.insert(GpuCapabilityRequirement::Preferred {
            feature: GpuCapabilityFeature::TimestampQuery,
            fallback: GpuPreferredFallback::DisableInstrumentation,
        })?;
        for feature in [
            GpuCapabilityFeature::Compute,
            GpuCapabilityFeature::IndirectDraw,
            GpuCapabilityFeature::StorageTexture,
            GpuCapabilityFeature::DepthAttachment,
        ] {
            requirements.insert(GpuCapabilityRequirement::Required(feature))?;
        }
        let mut descriptor = GpuContextDescriptor::new(requirements)
            .with_label("Runenwerk current host")
            .with_provenance("temporary G7 host compatibility")
            .with_power_preference(GpuPowerPreference::HighPerformance);
        for (format, roles) in [
            (
                GpuTextureFormat::R8Unorm,
                &[
                    GpuFormatRole::Sampled,
                    GpuFormatRole::Filterable,
                    GpuFormatRole::CopyDestination,
                ][..],
            ),
            (
                GpuTextureFormat::Rgba8Unorm,
                &[
                    GpuFormatRole::Sampled,
                    GpuFormatRole::Filterable,
                    GpuFormatRole::StorageWrite,
                    GpuFormatRole::ColorAttachment,
                    GpuFormatRole::CopySource,
                    GpuFormatRole::CopyDestination,
                ][..],
            ),
            (
                GpuTextureFormat::Rgba8UnormSrgb,
                &[
                    GpuFormatRole::Sampled,
                    GpuFormatRole::Filterable,
                    GpuFormatRole::ColorAttachment,
                    GpuFormatRole::CopySource,
                    GpuFormatRole::CopyDestination,
                ][..],
            ),
            (
                GpuTextureFormat::Bgra8Unorm,
                &[
                    GpuFormatRole::Sampled,
                    GpuFormatRole::Filterable,
                    GpuFormatRole::ColorAttachment,
                    GpuFormatRole::CopySource,
                    GpuFormatRole::CopyDestination,
                ][..],
            ),
            (
                GpuTextureFormat::Bgra8UnormSrgb,
                &[
                    GpuFormatRole::Sampled,
                    GpuFormatRole::Filterable,
                    GpuFormatRole::ColorAttachment,
                    GpuFormatRole::CopySource,
                    GpuFormatRole::CopyDestination,
                ][..],
            ),
            (
                GpuTextureFormat::R32Uint,
                &[
                    GpuFormatRole::Sampled,
                    GpuFormatRole::StorageWrite,
                    GpuFormatRole::ColorAttachment,
                    GpuFormatRole::CopySource,
                    GpuFormatRole::CopyDestination,
                ][..],
            ),
            (
                GpuTextureFormat::Depth32Float,
                &[
                    GpuFormatRole::Sampled,
                    GpuFormatRole::DepthStencil,
                    GpuFormatRole::CopySource,
                    GpuFormatRole::CopyDestination,
                ][..],
            ),
        ] {
            for &role in roles {
                descriptor = descriptor.require_format_role(format, role);
            }
        }
        let (context, surface) =
            GpuContext::request_for_current_host(descriptor, Arc::clone(&window)).await?;

        let surface_config = {
            let bridge = context.current_host_surface_bridge();
            let size = window.inner_size();
            let caps = bridge.capabilities(&surface);
            let format = preferred_surface_format(&caps);
            let surface_config =
                build_surface_config(size.width, size.height, format, caps.alpha_modes[0]);
            bridge.configure(&surface, &surface_config);
            surface_config
        };

        Ok(Self {
            context,
            surfaces: BTreeMap::from([(
                RenderSurfaceId::primary(),
                WgpuSurfaceState {
                    surface,
                    config: surface_config,
                },
            )]),
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
        let (surface, config) = {
            let bridge = self.context.current_host_surface_bridge();
            let surface = bridge.create_surface(window)?;
            let caps = bridge.capabilities(&surface);
            let format = preferred_surface_format(&caps);
            let config = build_surface_config(
                target_size_px.0,
                target_size_px.1,
                format,
                caps.alpha_modes[0],
            );
            bridge.configure(&surface, &config);
            (surface, config)
        };
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
        self.context
            .current_host_surface_bridge()
            .configure(&state.surface, &state.config);
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

    pub(crate) fn context(&self) -> &GpuContext {
        &self.context
    }
}
