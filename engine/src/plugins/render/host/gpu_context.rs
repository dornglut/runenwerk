use super::{
    RenderSurfaceAppliedPresentation, RenderSurfaceLifecycleState,
    RenderSurfacePresentationAttempt, RenderSurfacePresentationDisposition,
    RenderSurfacePresentationError, RenderSurfacePresentationRequest,
    RenderSurfacePresentationSnapshot, RenderSurfaceRecord, build_surface_config,
    preferred_surface_format, select_surface_present_mode,
};
use crate::plugins::render::apply_runenwerk_gpu_context_policy;
use anyhow::Result;
use pollster::block_on;
use runen_gpu::{
    GpuAcquiredSurfaceImage, GpuCapabilityFeature, GpuCapabilityProfile, GpuCapabilityRequirement,
    GpuContext, GpuContextDescriptor, GpuFormatRole, GpuPowerPreference, GpuPreferredFallback,
    GpuSurfaceAcquireErrorCategory, GpuSurfaceCapabilities, GpuSurfaceConfiguration,
    GpuSurfaceHandle, GpuTextureFormat,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use winit::window::Window;

use super::RenderSurfaceId;

#[derive(Debug)]
struct AttachedSurfaceState {
    surface: Option<GpuSurfaceHandle>,
    window: Arc<Window>,
    /// Attachment-time physical capability snapshot, never a live re-probe.
    capabilities: Option<GpuSurfaceCapabilities>,
    config: Option<GpuSurfaceConfiguration>,
    applied_revision: u64,
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

/// Resolve an exact-revision result independently of live physical ownership.
/// Requested surfaces may expose a strict rejection, but never claim configured output.
fn presentation_disposition(
    record: &RenderSurfaceRecord,
    applied: Option<RenderSurfaceAppliedPresentation>,
    capabilities_available: bool,
) -> RenderSurfacePresentationDisposition {
    if record.lifecycle_state == RenderSurfaceLifecycleState::Retired {
        return RenderSurfacePresentationDisposition::Unavailable;
    }
    if let Some(RenderSurfacePresentationAttempt::Rejected { revision, reason }) =
        record.presentation_attempt
        && revision == record.presentation_revision
    {
        return RenderSurfacePresentationDisposition::Rejected(reason);
    }
    if record.lifecycle_state != RenderSurfaceLifecycleState::Attached {
        return RenderSurfacePresentationDisposition::Unavailable;
    }
    if applied.is_some_and(|current| {
        current.revision == record.presentation_revision
            && current.request == record.presentation_request
    }) {
        RenderSurfacePresentationDisposition::Applied
    } else if !capabilities_available {
        RenderSurfacePresentationDisposition::Unavailable
    } else {
        RenderSurfacePresentationDisposition::Pending
    }
}

impl RenderGpuContext {
    async fn new_async(
        window: Arc<Window>,
        request: RenderSurfacePresentationRequest,
        revision: u64,
    ) -> Result<Self> {
        let descriptor = renderer_context_descriptor()?;
        let descriptor = apply_runenwerk_gpu_context_policy(descriptor);
        let (context, surface) =
            GpuContext::request_for_surface(descriptor, Arc::clone(&window)).await?;
        let size = window.inner_size();
        let capabilities = context.surface_capabilities(surface)?;
        let format = preferred_surface_format(&capabilities)
            .ok_or(RenderSurfacePresentationError::NoCompatibleFormat)?;
        // A zero-size attachment must still reject a physically unsupported mode.
        select_surface_present_mode(capabilities.present_modes(), request.present_mode)?;
        // A newly created/minimized window may have no drawable extent yet.
        // Keep the attachment and logical policy, but do not invent a configured swapchain.
        let (surface, config) = if size.width > 0 && size.height > 0 {
            let config =
                build_surface_config(size.width, size.height, format, &capabilities, request)?;
            let configured = context.configure_surface(surface, config.clone())?;
            (configured, Some(config))
        } else {
            (surface, None)
        };
        Ok(Self {
            context,
            surfaces: BTreeMap::from([(
                RenderSurfaceId::primary(),
                AttachedSurfaceState {
                    surface: Some(surface),
                    window,
                    capabilities: Some(capabilities),
                    config,
                    applied_revision: revision,
                },
            )]),
        })
    }

    pub fn new(window: Arc<Window>) -> Result<Self> {
        Self::new_with_presentation(window, RenderSurfacePresentationRequest::default(), 0)
    }

    pub fn new_with_presentation(
        window: Arc<Window>,
        request: RenderSurfacePresentationRequest,
        revision: u64,
    ) -> Result<Self> {
        block_on(Self::new_async(window, request, revision))
    }

    pub fn attach_surface(
        &mut self,
        id: RenderSurfaceId,
        window: Arc<Window>,
        extent: (u32, u32),
    ) -> Result<()> {
        self.attach_surface_with_presentation(
            id,
            window,
            extent,
            RenderSurfacePresentationRequest::default(),
            0,
        )
    }

    pub fn attach_surface_with_presentation(
        &mut self,
        id: RenderSurfaceId,
        window: Arc<Window>,
        extent: (u32, u32),
        request: RenderSurfacePresentationRequest,
        revision: u64,
    ) -> Result<()> {
        if self.surfaces.contains_key(&id) {
            return Err(RenderSurfacePresentationError::RequiresReattachment.into());
        }
        let physical = self
            .context
            .attach_surface(Arc::clone(&window))
            .map_err(|error| RenderSurfacePresentationError::Gpu(error.category()))?;
        let outcome = (|| -> Result<_, RenderSurfacePresentationError> {
            let capabilities = self
                .context
                .surface_capabilities(physical)
                .map_err(|error| RenderSurfacePresentationError::Gpu(error.category()))?;
            let format = preferred_surface_format(&capabilities)
                .ok_or(RenderSurfacePresentationError::NoCompatibleFormat)?;
            select_surface_present_mode(capabilities.present_modes(), request.present_mode)?;
            let config = if extent.0 > 0 && extent.1 > 0 {
                Some(build_surface_config(
                    extent.0,
                    extent.1,
                    format,
                    &capabilities,
                    request,
                )?)
            } else {
                None
            };
            let configured = if let Some(config) = &config {
                self.context
                    .configure_surface(physical, config.clone())
                    .map_err(|error| RenderSurfacePresentationError::Gpu(error.category()))?
            } else {
                physical
            };
            Ok((configured, capabilities, config))
        })();
        let (surface, capabilities, config) = match outcome {
            Ok(outcome) => outcome,
            Err(error) => {
                // Do not lose track of an attachment if rollback itself fails.
                if let Err(detach_error) = self.context.detach_surface(physical) {
                    self.surfaces.insert(
                        id,
                        AttachedSurfaceState {
                            surface: Some(physical),
                            window,
                            capabilities: None,
                            config: None,
                            applied_revision: 0,
                        },
                    );
                    return Err(RenderSurfacePresentationError::Gpu(detach_error.category()).into());
                }
                return Err(error.into());
            }
        };
        self.surfaces.insert(
            id,
            AttachedSurfaceState {
                surface: Some(surface),
                window,
                capabilities: Some(capabilities),
                config,
                applied_revision: revision,
            },
        );
        Ok(())
    }

    pub fn detach_surface(&mut self, id: RenderSurfaceId) -> bool {
        let Some(state) = self.surfaces.get(&id) else {
            return false;
        };
        if let Some(handle) = state.surface
            && self.context.detach_surface(handle).is_err()
        {
            return false;
        }
        self.surfaces.remove(&id);
        true
    }

    pub fn has_surface(&self, id: RenderSurfaceId) -> bool {
        self.surfaces
            .get(&id)
            .is_some_and(|state| state.surface.is_some())
    }

    /// Stable logical attachment record survives failed physical reattachment.
    pub(crate) fn has_surface_record(&self, id: RenderSurfaceId) -> bool {
        self.surfaces.contains_key(&id)
    }

    pub fn surface_config(&self, id: RenderSurfaceId) -> Option<&GpuSurfaceConfiguration> {
        self.surfaces
            .get(&id)
            .and_then(|state| state.config.as_ref())
    }

    /// A normalized logical extent is not proof that a native window can display.
    pub fn is_surface_drawable(&self, id: RenderSurfaceId) -> bool {
        self.surfaces.get(&id).is_some_and(|state| {
            let size = state.window.inner_size();
            size.width > 0 && size.height > 0
        })
    }

    pub fn presentation_snapshot(
        &self,
        record: &RenderSurfaceRecord,
    ) -> RenderSurfacePresentationSnapshot {
        // Only a confirmed logical attachment may publish current physical facts.
        // Requested and retired records must not claim an applied configuration.
        let state = (record.lifecycle_state == RenderSurfaceLifecycleState::Attached)
            .then(|| self.surfaces.get(&record.render_surface_id))
            .flatten();
        let supported = state
            .and_then(|state| state.capabilities.as_ref())
            .map(|caps| caps.present_modes().to_vec());
        let selected = supported.as_ref().and_then(|modes| {
            select_surface_present_mode(modes, record.presentation_request.present_mode).ok()
        });
        let attachment_epoch = state.and_then(|state| {
            state.capabilities.as_ref()?;
            let surface = state.surface?;
            Some((surface.id(), surface.generation()))
        });
        let applied = state.and_then(|state| {
            Some(RenderSurfaceAppliedPresentation {
                request: RenderSurfacePresentationRequest::new(
                    state.config.as_ref()?.present_mode(),
                    state.config.as_ref()?.desired_maximum_frame_latency(),
                )
                .ok()?,
                surface_generation: state.surface?.generation(),
                revision: state.applied_revision,
            })
        });
        let disposition = presentation_disposition(record, applied, supported.is_some());
        RenderSurfacePresentationSnapshot {
            render_surface_id: record.render_surface_id,
            native_window_id: record.native_window_id,
            requested: record.presentation_request,
            request_revision: record.presentation_revision,
            supported_at_attachment: supported,
            attachment_epoch,
            selected,
            applied,
            disposition,
            last_attempt: record.presentation_attempt,
        }
    }

    /// Run only before acquisition in the existing serial RenderSubmit/Gfx owner.
    /// A failed physical reconfiguration invalidates the prior applied claim.
    pub(crate) fn configure_presentation(
        &mut self,
        id: RenderSurfaceId,
        request: RenderSurfacePresentationRequest,
        revision: u64,
        extent: (u32, u32),
    ) -> Result<(), RenderSurfacePresentationError> {
        if extent.0 == 0 || extent.1 == 0 {
            return Err(RenderSurfacePresentationError::ZeroExtent);
        }
        let state = self
            .surfaces
            .get(&id)
            .ok_or(RenderSurfacePresentationError::UnknownSurface)?;
        if state.config.as_ref().is_some_and(|config| {
            config.width() == extent.0
                && config.height() == extent.1
                && config.present_mode() == request.present_mode
                && config.desired_maximum_frame_latency()
                    == request.desired_maximum_frame_latency.get()
        }) {
            // Previously configured physical policy already satisfies this revision.
            // Avoid an unnecessary GPU-idle swapchain reconfiguration.
            if let Some(state) = self.surfaces.get_mut(&id) {
                state.applied_revision = revision;
            }
            return Ok(());
        }
        if state.config.is_none() || state.capabilities.is_none() || state.surface.is_none() {
            self.reattach_surface(id, request, revision, extent)?;
            return Ok(());
        }
        let (surface, capabilities, config) = {
            let state = self
                .surfaces
                .get(&id)
                .ok_or(RenderSurfacePresentationError::UnknownSurface)?;
            let caps = state
                .capabilities
                .as_ref()
                .ok_or(RenderSurfacePresentationError::RequiresReattachment)?;
            let former = state
                .config
                .as_ref()
                .ok_or(RenderSurfacePresentationError::RequiresReattachment)?;
            // Retain the admitted physical format and usages, not a fresh product policy.
            let config = GpuSurfaceConfiguration::new(
                extent.0,
                extent.1,
                former.format(),
                former.color_space(),
                former.usages().iter().copied(),
                request.present_mode,
                former.alpha_mode(),
                request.desired_maximum_frame_latency.get(),
                former.view_formats().iter().copied(),
            )
            .map_err(|error| RenderSurfacePresentationError::Configuration(error.cause()))?;
            select_surface_present_mode(caps.present_modes(), request.present_mode)?;
            (
                state
                    .surface
                    .ok_or(RenderSurfacePresentationError::RequiresReattachment)?,
                caps.clone(),
                config,
            )
        };
        if !capabilities.supports_pair(config.format(), config.color_space())
            || !capabilities.supports_alpha_mode(config.alpha_mode())
            || config
                .usages()
                .iter()
                .any(|usage| !capabilities.supports_usage(*usage))
        {
            return Err(RenderSurfacePresentationError::RequiresReattachment);
        }
        let configured = self.context.configure_surface(surface, config.clone());
        match configured {
            Ok(surface) => {
                let state = self
                    .surfaces
                    .get_mut(&id)
                    .ok_or(RenderSurfacePresentationError::UnknownSurface)?;
                state.surface = Some(surface);
                state.config = Some(config);
                state.applied_revision = revision;
                Ok(())
            }
            Err(error) => {
                // An attempted physical configure can have side effects before returning.
                if let Some(state) = self.surfaces.get_mut(&id) {
                    state.config = None;
                    state.capabilities = None;
                }
                Err(RenderSurfacePresentationError::Gpu(error.category()))
            }
        }
    }

    /// Re-register against this window to refresh attachment-time capabilities.
    /// Never call inside a render or with an outstanding surface image lease.
    pub(crate) fn reattach_surface(
        &mut self,
        id: RenderSurfaceId,
        request: RenderSurfacePresentationRequest,
        revision: u64,
        extent: (u32, u32),
    ) -> Result<(), RenderSurfacePresentationError> {
        if extent.0 == 0 || extent.1 == 0 {
            return Err(RenderSurfacePresentationError::ZeroExtent);
        }
        let state = self
            .surfaces
            .get(&id)
            .ok_or(RenderSurfacePresentationError::UnknownSurface)?;
        let window = Arc::clone(&state.window);
        if let Some(old) = state.surface {
            self.context
                .detach_surface(old)
                .map_err(|error| RenderSurfacePresentationError::Gpu(error.category()))?;
        }
        {
            let state = self
                .surfaces
                .get_mut(&id)
                .ok_or(RenderSurfacePresentationError::UnknownSurface)?;
            state.surface = None;
            state.capabilities = None;
            state.config = None;
        }
        let physical = self
            .context
            .attach_surface(Arc::clone(&window))
            .map_err(|error| RenderSurfacePresentationError::Gpu(error.category()))?;
        let configuration = (|| {
            let capabilities = self
                .context
                .surface_capabilities(physical)
                .map_err(|error| RenderSurfacePresentationError::Gpu(error.category()))?;
            let format = preferred_surface_format(&capabilities)
                .ok_or(RenderSurfacePresentationError::NoCompatibleFormat)?;
            let config = build_surface_config(extent.0, extent.1, format, &capabilities, request)?;
            Ok::<_, RenderSurfacePresentationError>((capabilities, config))
        })();
        let (capabilities, config) = match configuration {
            Ok(config) => config,
            Err(error) => {
                if let Err(detach_error) = self.context.detach_surface(physical) {
                    if let Some(state) = self.surfaces.get_mut(&id) {
                        state.surface = Some(physical);
                    }
                    return Err(RenderSurfacePresentationError::Gpu(detach_error.category()));
                }
                return Err(error);
            }
        };
        let configured = match self.context.configure_surface(physical, config.clone()) {
            Ok(configured) => configured,
            Err(error) => {
                if let Err(detach_error) = self.context.detach_surface(physical) {
                    if let Some(state) = self.surfaces.get_mut(&id) {
                        state.surface = Some(physical);
                    }
                    return Err(RenderSurfacePresentationError::Gpu(detach_error.category()));
                }
                return Err(RenderSurfacePresentationError::Gpu(error.category()));
            }
        };
        let state = self
            .surfaces
            .get_mut(&id)
            .ok_or(RenderSurfacePresentationError::UnknownSurface)?;
        state.surface = Some(configured);
        state.capabilities = Some(capabilities);
        state.config = Some(config);
        state.applied_revision = revision;
        Ok(())
    }

    /// After Outdated/Lost, cached capabilities are not a fresh probe.
    pub(crate) fn invalidate_surface(&mut self, id: RenderSurfaceId) {
        if let Some(state) = self.surfaces.get_mut(&id) {
            state.config = None;
            state.capabilities = None;
        }
    }

    pub fn resize(&mut self, id: RenderSurfaceId, width: u32, height: u32) -> bool {
        let Some(config) = self.surface_config(id) else {
            return false;
        };
        let request = RenderSurfacePresentationRequest::new(
            config.present_mode(),
            config.desired_maximum_frame_latency(),
        )
        .expect("valid configured surface has positive latency");
        let revision = self
            .surfaces
            .get(&id)
            .map(|state| state.applied_revision)
            .unwrap_or(0);
        self.configure_presentation(id, request, revision, (width, height))
            .is_ok()
    }

    pub fn acquire_surface_image(
        &mut self,
        id: RenderSurfaceId,
    ) -> Result<GpuAcquiredSurfaceImage, RenderSurfaceAcquireError> {
        let surface = self
            .surfaces
            .get(&id)
            .and_then(|state| state.surface)
            .ok_or(RenderSurfaceAcquireError::Lost)?;
        let result = self.context.acquire_surface_image(surface);
        if let Err(error) = &result
            && matches!(
                error.category(),
                GpuSurfaceAcquireErrorCategory::Outdated
                    | GpuSurfaceAcquireErrorCategory::Lost
                    | GpuSurfaceAcquireErrorCategory::StaleGeneration
                    | GpuSurfaceAcquireErrorCategory::ContextOrDeviceUnavailableOrLost
            )
        {
            self.invalidate_surface(id);
        }
        result.map_err(|error| match error.category() {
            GpuSurfaceAcquireErrorCategory::Timeout | GpuSurfaceAcquireErrorCategory::Occluded => {
                RenderSurfaceAcquireError::Timeout
            }
            GpuSurfaceAcquireErrorCategory::Outdated => RenderSurfaceAcquireError::Outdated,
            GpuSurfaceAcquireErrorCategory::Lost
            | GpuSurfaceAcquireErrorCategory::UnknownSurface
            | GpuSurfaceAcquireErrorCategory::StaleGeneration
            | GpuSurfaceAcquireErrorCategory::ContextOrDeviceUnavailableOrLost => {
                RenderSurfaceAcquireError::Lost
            }
            _ => RenderSurfaceAcquireError::Validation,
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
    fn strict_pre_attach_rejection_is_visible_and_retirement_hides_physical_state() {
        use super::super::{RenderSurfacePresentationAttempt, RenderSurfaceRegistryResource};
        use crate::runtime::NativeWindowId;
        use runen_gpu::GpuSurfacePresentMode;

        let mut registry = RenderSurfaceRegistryResource::default();
        let id = registry.reserve_surface_for_native_window(NativeWindowId::primary(), (0, 0));
        let requested =
            RenderSurfacePresentationRequest::new(GpuSurfacePresentMode::Mailbox, 1).unwrap();
        assert_eq!(registry.set_presentation_request(id, requested), Ok(1));
        let record = registry.record(id).unwrap();
        assert_eq!(
            presentation_disposition(record, None, false),
            RenderSurfacePresentationDisposition::Unavailable
        );
        registry.record_presentation_attempt(
            id,
            RenderSurfacePresentationAttempt::Rejected {
                revision: 1,
                reason: RenderSurfacePresentationError::UnsupportedPresentMode(
                    GpuSurfacePresentMode::Mailbox,
                ),
            },
        );
        let record = registry.record(id).unwrap();
        assert_eq!(
            presentation_disposition(record, None, false),
            RenderSurfacePresentationDisposition::Rejected(
                RenderSurfacePresentationError::UnsupportedPresentMode(
                    GpuSurfacePresentMode::Mailbox,
                )
            )
        );
        registry.retire_surface_for_native_window(NativeWindowId::primary());
        assert_eq!(
            presentation_disposition(registry.record(id).unwrap(), None, true),
            RenderSurfacePresentationDisposition::Unavailable
        );
    }

    #[test]
    fn alpha_blended_target_formats_are_explicitly_admitted() {
        let descriptor = renderer_context_descriptor().expect("valid renderer requirements");
        for format in [
            GpuTextureFormat::Rgba8Unorm,
            GpuTextureFormat::Rgba8UnormSrgb,
            GpuTextureFormat::Bgra8Unorm,
            GpuTextureFormat::Bgra8UnormSrgb,
        ] {
            for role in [GpuFormatRole::ColorAttachment, GpuFormatRole::Blendable] {
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
