use super::{build_surface_config, preferred_surface_format};
use crate::plugins::render::apply_runenwerk_gpu_context_policy;
use anyhow::Result;
use pollster::block_on;
use runen_gpu::{
    GpuAcquiredSurfaceImage, GpuCapabilityFeature, GpuCapabilityProfile, GpuCapabilityRequirement,
    GpuContext, GpuContextDescriptor, GpuFormatRole, GpuPowerPreference, GpuPreferredFallback,
    GpuSurfaceAcquireErrorCategory, GpuSurfaceConfiguration, GpuSurfaceHandle, GpuTextureFormat,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use winit::window::Window;

use super::RenderSurfaceId;

#[derive(Debug)]
struct AttachedSurfaceState {
    surface: GpuSurfaceHandle,
    /// None while RunenGPU physically owns a registered but not configured surface.
    /// Its handle must remain reachable for typed rollback and device-loss detach.
    config: Option<GpuSurfaceConfiguration>,
}

#[derive(Debug)]
pub struct RenderGpuContext {
    context: GpuContext,
    surfaces: BTreeMap<RenderSurfaceId, AttachedSurfaceState>,
}

/// Renderer-facing acquisition categories mapped from public RunenGPU surface outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RenderSurfaceAcquireError {
    #[error("render surface was lost")]
    Lost,
    #[error("render surface configuration is outdated")]
    Outdated,
    #[error("render surface acquisition timed out or was occluded")]
    Timeout,
    #[error("render surface acquisition failed validation")]
    Validation,
}

fn renderer_context_descriptor() -> Result<GpuContextDescriptor> {
    let mut requirements = GpuCapabilityProfile::DesktopPresentationBaseline.requirements();
    requirements.insert(GpuCapabilityRequirement::Preferred {
        feature: GpuCapabilityFeature::TimestampQuery,
        fallback: GpuPreferredFallback::DisableInstrumentation,
    })?;
    for feature in [
        GpuCapabilityFeature::Compute,
        GpuCapabilityFeature::IndirectExecution,
        GpuCapabilityFeature::StorageTexture,
        GpuCapabilityFeature::DepthAttachment,
    ] {
        requirements.insert(GpuCapabilityRequirement::Required(feature))?;
    }
    let mut descriptor = GpuContextDescriptor::new(requirements)
        .with_label("Runenwerk renderer")
        .with_provenance("Runenwerk renderer surface execution")
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
                GpuFormatRole::Blendable,
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
                GpuFormatRole::Blendable,
                GpuFormatRole::CopySource,
                GpuFormatRole::CopyDestination,
            ][..],
        ),
        (
            GpuTextureFormat::Rgba16Float,
            &[
                GpuFormatRole::Sampled,
                GpuFormatRole::ColorAttachment,
                GpuFormatRole::Blendable,
            ][..],
        ),
        (
            GpuTextureFormat::Bgra8Unorm,
            &[
                GpuFormatRole::Sampled,
                GpuFormatRole::Filterable,
                GpuFormatRole::ColorAttachment,
                GpuFormatRole::Blendable,
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
                GpuFormatRole::Blendable,
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
            GpuTextureFormat::R32Float,
            &[GpuFormatRole::Sampled, GpuFormatRole::CopyDestination][..],
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
    Ok(descriptor)
}

impl RenderGpuContext {
    async fn new_async(window: Arc<Window>) -> Result<Self> {
        let descriptor = renderer_context_descriptor()?;
        let descriptor = apply_runenwerk_gpu_context_policy(descriptor);
        let (context, surface) =
            GpuContext::request_for_surface(descriptor, Arc::clone(&window)).await?;

        let size = window.inner_size();
        let caps = context.surface_capabilities(surface)?;
        let format = preferred_surface_format(&caps)
            .ok_or_else(|| anyhow::anyhow!("render surface reports no supported format"))?;
        let surface_config = build_surface_config(size.width, size.height, format, &caps)?;
        let surface = context.configure_surface(surface, surface_config.clone())?;

        Ok(Self {
            context,
            surfaces: BTreeMap::from([(
                RenderSurfaceId::primary(),
                AttachedSurfaceState {
                    surface,
                    config: Some(surface_config),
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
        if self.surfaces.contains_key(&render_surface_id) {
            return Err(anyhow::anyhow!(
                "render surface {} already retains physical GPU ownership",
                render_surface_id.raw()
            ));
        }
        let surface = self.context.attach_surface(window)?;
        // RunenGPU physically owns this window as soon as registration succeeds.
        // Publish the handle *before* fallible capabilities/configuration so a
        // failed setup cannot make the Host rollback mistake it for a no-op.
        self.surfaces.insert(
            render_surface_id,
            AttachedSurfaceState {
                surface,
                config: None,
            },
        );
        let caps = self.context.surface_capabilities(surface)?;
        let format = preferred_surface_format(&caps)
            .ok_or_else(|| anyhow::anyhow!("render surface reports no supported format"))?;
        let config = build_surface_config(target_size_px.0, target_size_px.1, format, &caps)?;
        let configured = self.context.configure_surface(surface, config.clone())?;
        let state = self
            .surfaces
            .get_mut(&render_surface_id)
            .expect("registered GPU surface remains tracked across synchronous setup");
        state.surface = configured;
        state.config = Some(config);
        Ok(())
    }

    pub fn detach_surface(&mut self, render_surface_id: RenderSurfaceId) -> bool {
        let Some(state) = self.surfaces.get(&render_surface_id) else {
            return false;
        };
        if self.context.detach_surface(state.surface).is_err() {
            return false;
        }
        self.surfaces.remove(&render_surface_id);
        true
    }

    pub fn has_surface(&self, render_surface_id: RenderSurfaceId) -> bool {
        self.surfaces.contains_key(&render_surface_id)
    }

    pub fn surface_config(
        &self,
        render_surface_id: RenderSurfaceId,
    ) -> Option<&GpuSurfaceConfiguration> {
        self.surfaces
            .get(&render_surface_id)
            .and_then(|state| state.config.as_ref())
    }

    pub fn resize(&mut self, render_surface_id: RenderSurfaceId, width: u32, height: u32) -> bool {
        let Some(state) = self.surfaces.get_mut(&render_surface_id) else {
            return false;
        };
        let Some(current) = state.config.as_ref() else {
            // A failed attach still retains a physical surface for rollback,
            // but no successfully admitted configuration exists to resize.
            return false;
        };
        let Ok(config) = GpuSurfaceConfiguration::new(
            width.max(1),
            height.max(1),
            current.format(),
            current.color_space(),
            current.usages().iter().copied(),
            current.present_mode(),
            current.alpha_mode(),
            current.desired_maximum_frame_latency(),
            current.view_formats().iter().copied(),
        ) else {
            return false;
        };
        let Ok(surface) = self
            .context
            .configure_surface(state.surface, config.clone())
        else {
            return false;
        };
        state.surface = surface;
        state.config = Some(config);
        true
    }

    pub fn acquire_surface_image(
        &self,
        render_surface_id: RenderSurfaceId,
    ) -> Result<GpuAcquiredSurfaceImage, RenderSurfaceAcquireError> {
        let surface = self
            .surfaces
            .get(&render_surface_id)
            .ok_or(RenderSurfaceAcquireError::Lost)?
            .surface;
        self.context
            .acquire_surface_image(surface)
            .map_err(|error| match error.category() {
                GpuSurfaceAcquireErrorCategory::Timeout
                | GpuSurfaceAcquireErrorCategory::Occluded => RenderSurfaceAcquireError::Timeout,
                GpuSurfaceAcquireErrorCategory::Outdated => RenderSurfaceAcquireError::Outdated,
                GpuSurfaceAcquireErrorCategory::Lost
                | GpuSurfaceAcquireErrorCategory::UnknownSurface
                | GpuSurfaceAcquireErrorCategory::StaleGeneration
                | GpuSurfaceAcquireErrorCategory::ContextOrDeviceUnavailableOrLost => {
                    RenderSurfaceAcquireError::Lost
                }
                GpuSurfaceAcquireErrorCategory::NotConfigured
                | GpuSurfaceAcquireErrorCategory::AlreadyAcquired
                | GpuSurfaceAcquireErrorCategory::Validation
                | GpuSurfaceAcquireErrorCategory::ForeignContext
                | GpuSurfaceAcquireErrorCategory::IdentityExhausted
                | GpuSurfaceAcquireErrorCategory::InternalInvariant => {
                    RenderSurfaceAcquireError::Validation
                }
            })
    }

    pub(crate) fn context(&self) -> &GpuContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_blended_target_formats_are_explicitly_admitted() {
        let descriptor = renderer_context_descriptor().expect("valid renderer requirements");
        for format in [
            GpuTextureFormat::Rgba8Unorm,
            GpuTextureFormat::Rgba8UnormSrgb,
            GpuTextureFormat::Rgba16Float,
            GpuTextureFormat::Bgra8Unorm,
            GpuTextureFormat::Bgra8UnormSrgb,
        ] {
            for role in [
                GpuFormatRole::ColorAttachment,
                GpuFormatRole::Blendable,
                GpuFormatRole::Sampled,
            ] {
                assert!(
                    descriptor
                        .semantically_eq(&descriptor.clone().require_format_role(format, role)),
                    "renderer must already require {format:?}::{role:?}"
                );
            }
        }
        for format in [
            GpuTextureFormat::R8Unorm,
            GpuTextureFormat::R32Uint,
            GpuTextureFormat::R32Float,
            GpuTextureFormat::Depth32Float,
        ] {
            assert!(
                !descriptor.semantically_eq(
                    &descriptor
                        .clone()
                        .require_format_role(format, GpuFormatRole::Blendable)
                ),
                "renderer must not demand unused blending for {format:?}"
            );
        }
    }
}
