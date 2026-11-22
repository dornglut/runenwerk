use crate::runtime::NativeWindowId;
use anyhow::{Result, anyhow};
use id_macros::id;
use runen_gpu::{
    GpuSurfaceCapabilities, GpuSurfaceColorSpace, GpuSurfaceConfiguration,
    GpuSurfaceConfigurationCause, GpuSurfaceErrorCategory, GpuSurfaceGeneration,
    GpuSurfacePresentMode, GpuTextureFormat, GpuTextureUsage,
};
use std::collections::BTreeMap;
use std::num::NonZeroU32;

#[id]
pub struct RenderSurfaceId;

impl RenderSurfaceId {
    pub fn primary() -> Self {
        Self::try_from_raw(1).expect("primary render surface id must be non-zero")
    }
}

/// Declarative policy for one logical native render surface. It does not configure a swapchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSurfacePresentationRequest {
    pub present_mode: GpuSurfacePresentMode,
    pub desired_maximum_frame_latency: NonZeroU32,
}

impl Default for RenderSurfacePresentationRequest {
    fn default() -> Self {
        Self {
            present_mode: GpuSurfacePresentMode::Fifo,
            desired_maximum_frame_latency: NonZeroU32::new(2).expect("nonzero FIFO default"),
        }
    }
}

impl RenderSurfacePresentationRequest {
    pub fn new(
        present_mode: GpuSurfacePresentMode,
        desired_maximum_frame_latency: u32,
    ) -> Result<Self, RenderSurfacePresentationError> {
        let desired_maximum_frame_latency = NonZeroU32::new(desired_maximum_frame_latency)
            .ok_or(RenderSurfacePresentationError::ZeroLatency)?;
        Ok(Self {
            present_mode,
            desired_maximum_frame_latency,
        })
    }
}

/// Typed policy/configuration failure; a failed requested mode is never implicitly replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RenderSurfacePresentationError {
    #[error("unknown render surface")]
    UnknownSurface,
    #[error("render surface has retired")]
    RetiredSurface,
    #[error("presentation request revision exhausted")]
    RevisionExhausted,
    #[error("desired maximum frame latency must be nonzero")]
    ZeroLatency,
    #[error("render surface has no drawable nonzero extent")]
    ZeroExtent,
    #[error("render surface has no compatible sRGB format")]
    NoCompatibleFormat,
    #[error("render surface has no compatible alpha mode or color-attachment usage")]
    NoCompatibleAlphaOrUsage,
    #[error("requested present mode {0:?} is unsupported on this attachment")]
    UnsupportedPresentMode(GpuSurfacePresentMode),
    #[error("current physical surface requires safe reattachment")]
    RequiresReattachment,
    #[error("RunenGPU rejected surface operation: {0:?}")]
    Gpu(GpuSurfaceErrorCategory),
    #[error("RunenGPU rejected surface configuration: {0:?}")]
    Configuration(GpuSurfaceConfigurationCause),
}

/// An attempt is associated with an exact immutable logical request revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSurfacePresentationAttempt {
    Applied {
        revision: u64,
    },
    Rejected {
        revision: u64,
        reason: RenderSurfacePresentationError,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSurfacePresentationDisposition {
    Pending,
    Applied,
    Rejected(RenderSurfacePresentationError),
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSurfaceAppliedPresentation {
    pub request: RenderSurfacePresentationRequest,
    pub surface_generation: GpuSurfaceGeneration,
    /// The request revision that actually returned from RunenGPU configuration.
    pub revision: u64,
}

/// Derived from the registry's logical request and the one Gfx physical owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSurfacePresentationSnapshot {
    pub render_surface_id: RenderSurfaceId,
    pub native_window_id: NativeWindowId,
    pub requested: RenderSurfacePresentationRequest,
    pub request_revision: u64,
    /// Facts last sampled when RunenGPU attached this physical surface, not a live OS probe.
    pub supported_at_attachment: Option<Vec<GpuSurfacePresentMode>>,
    /// Exact RunenGPU attachment identity and configuration generation (not live OS facts).
    pub attachment_epoch: Option<(runen_gpu::GpuSurfaceId, GpuSurfaceGeneration)>,
    pub selected: Option<GpuSurfacePresentMode>,
    pub applied: Option<RenderSurfaceAppliedPresentation>,
    pub disposition: RenderSurfacePresentationDisposition,
    pub last_attempt: Option<RenderSurfacePresentationAttempt>,
}

/// Qualify strictly against facts about the exact physical attachment.
/// Separate slice input permits honest pure tests without constructing private RunenGPU capabilities.
pub fn select_surface_present_mode(
    supported: &[GpuSurfacePresentMode],
    requested: GpuSurfacePresentMode,
) -> Result<GpuSurfacePresentMode, RenderSurfacePresentationError> {
    supported.contains(&requested).then_some(requested).ok_or(
        RenderSurfacePresentationError::UnsupportedPresentMode(requested),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderSurfaceLifecycleState {
    Requested,
    Attached,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSurfaceRecord {
    pub render_surface_id: RenderSurfaceId,
    pub native_window_id: NativeWindowId,
    pub target_size_px: (u32, u32),
    pub lifecycle_state: RenderSurfaceLifecycleState,
    pub presentation_request: RenderSurfacePresentationRequest,
    pub presentation_revision: u64,
    pub presentation_attempt: Option<RenderSurfacePresentationAttempt>,
}

impl RenderSurfaceRecord {
    fn new(
        render_surface_id: RenderSurfaceId,
        native_window_id: NativeWindowId,
        target_size_px: (u32, u32),
        lifecycle_state: RenderSurfaceLifecycleState,
    ) -> Self {
        Self {
            render_surface_id,
            native_window_id,
            target_size_px: normalized_surface_extent(target_size_px),
            lifecycle_state,
            presentation_request: RenderSurfacePresentationRequest::default(),
            presentation_revision: 0,
            presentation_attempt: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSurfaceDiagnostic {
    pub render_surface_id: Option<RenderSurfaceId>,
    pub native_window_id: Option<NativeWindowId>,
    pub message: String,
}

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct RenderSurfaceRegistryResource {
    primary_surface_id: Option<RenderSurfaceId>,
    next_surface_raw: u64,
    records: BTreeMap<RenderSurfaceId, RenderSurfaceRecord>,
    surfaces_by_native_window: BTreeMap<NativeWindowId, RenderSurfaceId>,
    diagnostics: Vec<RenderSurfaceDiagnostic>,
}

impl RenderSurfaceRegistryResource {
    pub fn reserve_surface_for_native_window(
        &mut self,
        native_window_id: NativeWindowId,
        target_size_px: (u32, u32),
    ) -> RenderSurfaceId {
        if let Some(surface_id) = self
            .surfaces_by_native_window
            .get(&native_window_id)
            .copied()
        {
            if let Some(record) = self.records.get_mut(&surface_id) {
                record.target_size_px = normalized_surface_extent(target_size_px);
            }
            return surface_id;
        }
        let render_surface_id = if native_window_id == NativeWindowId::primary() {
            RenderSurfaceId::primary()
        } else {
            self.allocate_secondary_surface_id()
        };
        self.next_surface_raw = self
            .next_surface_raw
            .max(render_surface_id.raw().saturating_add(1));
        self.records.insert(
            render_surface_id,
            RenderSurfaceRecord::new(
                render_surface_id,
                native_window_id,
                target_size_px,
                RenderSurfaceLifecycleState::Requested,
            ),
        );
        self.surfaces_by_native_window
            .insert(native_window_id, render_surface_id);
        if native_window_id == NativeWindowId::primary() {
            self.primary_surface_id = Some(render_surface_id);
        }
        render_surface_id
    }

    pub fn confirm_surface_attachment(
        &mut self,
        render_surface_id: RenderSurfaceId,
        native_window_id: NativeWindowId,
        target_size_px: (u32, u32),
    ) -> Result<()> {
        let render_is_primary = render_surface_id == RenderSurfaceId::primary();
        let native_is_primary = native_window_id == NativeWindowId::primary();
        if render_is_primary != native_is_primary {
            return Err(anyhow!(
                "render surface {} and native window {} disagree on primary identity",
                render_surface_id.raw(),
                native_window_id.raw()
            ));
        }
        if let Some(existing) = self.surfaces_by_native_window.get(&native_window_id)
            && *existing != render_surface_id
        {
            return Err(anyhow!(
                "native window {} is already correlated with render surface {}",
                native_window_id.raw(),
                existing.raw()
            ));
        }
        match self.records.get_mut(&render_surface_id) {
            Some(record) => {
                if record.native_window_id != native_window_id {
                    return Err(anyhow!(
                        "render surface {} is reserved for native window {}, not {}",
                        render_surface_id.raw(),
                        record.native_window_id.raw(),
                        native_window_id.raw()
                    ));
                }
                if record.lifecycle_state == RenderSurfaceLifecycleState::Retired {
                    return Err(anyhow!(
                        "retired render surface {} cannot be reattached",
                        render_surface_id.raw()
                    ));
                }
                record.target_size_px = normalized_surface_extent(target_size_px);
                record.lifecycle_state = RenderSurfaceLifecycleState::Attached;
            }
            None => {
                if !native_is_primary {
                    return Err(anyhow!(
                        "secondary render surface {} must be reserved for native window {} before attachment",
                        render_surface_id.raw(),
                        native_window_id.raw()
                    ));
                }
                self.records.insert(
                    render_surface_id,
                    RenderSurfaceRecord::new(
                        render_surface_id,
                        native_window_id,
                        target_size_px,
                        RenderSurfaceLifecycleState::Attached,
                    ),
                );
            }
        }
        self.next_surface_raw = self
            .next_surface_raw
            .max(render_surface_id.raw().saturating_add(1));
        self.surfaces_by_native_window
            .insert(native_window_id, render_surface_id);
        if native_is_primary {
            self.primary_surface_id = Some(render_surface_id);
        }
        Ok(())
    }

    pub fn update_surface_extent_for_native_window(
        &mut self,
        native_window_id: NativeWindowId,
        target_size_px: (u32, u32),
    ) -> bool {
        let Some(render_surface_id) = self
            .surfaces_by_native_window
            .get(&native_window_id)
            .copied()
        else {
            return false;
        };
        let Some(record) = self.records.get_mut(&render_surface_id) else {
            return false;
        };
        if record.lifecycle_state == RenderSurfaceLifecycleState::Retired {
            return false;
        }
        record.target_size_px = normalized_surface_extent(target_size_px);
        true
    }

    /// Update a logical per-surface request; physical application is a later RenderSubmit event.
    pub fn set_presentation_request(
        &mut self,
        render_surface_id: RenderSurfaceId,
        request: RenderSurfacePresentationRequest,
    ) -> Result<u64, RenderSurfacePresentationError> {
        let record = self
            .records
            .get_mut(&render_surface_id)
            .ok_or(RenderSurfacePresentationError::UnknownSurface)?;
        if record.lifecycle_state == RenderSurfaceLifecycleState::Retired {
            return Err(RenderSurfacePresentationError::RetiredSurface);
        }
        if record.presentation_request != request {
            let revision = record
                .presentation_revision
                .checked_add(1)
                .ok_or(RenderSurfacePresentationError::RevisionExhausted)?;
            record.presentation_request = request;
            record.presentation_revision = revision;
            record.presentation_attempt = None;
        }
        Ok(record.presentation_revision)
    }

    /// Associate an actual configuration result with the same logical request revision.
    /// Superseded attempts must not overwrite newer policy.
    pub(crate) fn record_presentation_attempt(
        &mut self,
        render_surface_id: RenderSurfaceId,
        attempt: RenderSurfacePresentationAttempt,
    ) {
        if let Some(record) = self.records.get_mut(&render_surface_id) {
            let revision = match attempt {
                RenderSurfacePresentationAttempt::Applied { revision }
                | RenderSurfacePresentationAttempt::Rejected { revision, .. } => revision,
            };
            if record.lifecycle_state != RenderSurfaceLifecycleState::Retired
                && record.presentation_revision == revision
            {
                record.presentation_attempt = Some(attempt);
            }
        }
    }

    pub fn primary_surface_id(&self) -> Option<RenderSurfaceId> {
        self.primary_surface_id
    }
    pub fn record(&self, render_surface_id: RenderSurfaceId) -> Option<&RenderSurfaceRecord> {
        self.records.get(&render_surface_id)
    }
    pub fn surface_for_native_window(
        &self,
        native_window_id: NativeWindowId,
    ) -> Option<RenderSurfaceId> {
        self.surfaces_by_native_window
            .get(&native_window_id)
            .copied()
    }
    pub fn records(&self) -> impl Iterator<Item = &RenderSurfaceRecord> {
        self.records.values()
    }

    pub fn retire_surface_for_native_window(
        &mut self,
        native_window_id: NativeWindowId,
    ) -> Option<RenderSurfaceId> {
        let render_surface_id = self.surfaces_by_native_window.remove(&native_window_id)?;
        if let Some(record) = self.records.get_mut(&render_surface_id) {
            record.lifecycle_state = RenderSurfaceLifecycleState::Retired;
        }
        if self.primary_surface_id == Some(render_surface_id) {
            self.primary_surface_id = None;
        }
        Some(render_surface_id)
    }
    pub fn diagnostics(&self) -> &[RenderSurfaceDiagnostic] {
        &self.diagnostics
    }
    pub fn record_diagnostic(&mut self, diagnostic: RenderSurfaceDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    fn allocate_secondary_surface_id(&mut self) -> RenderSurfaceId {
        loop {
            let raw = self
                .next_surface_raw
                .max(RenderSurfaceId::primary().raw().saturating_add(1));
            self.next_surface_raw = raw.saturating_add(1);
            if let Ok(id) = RenderSurfaceId::try_from_raw(raw)
                && !self.records.contains_key(&id)
            {
                return id;
            }
        }
    }
}

fn normalized_surface_extent(target_size_px: (u32, u32)) -> (u32, u32) {
    (target_size_px.0.max(1), target_size_px.1.max(1))
}

pub fn preferred_surface_format(caps: &GpuSurfaceCapabilities) -> Option<GpuTextureFormat> {
    let mut formats = caps
        .format_capabilities()
        .iter()
        .filter(|entry| entry.supports_color_space(GpuSurfaceColorSpace::Srgb))
        .map(|entry| entry.format());
    let first = formats.next()?;
    if first.is_srgb() {
        Some(first)
    } else {
        Some(formats.find(|format| format.is_srgb()).unwrap_or(first))
    }
}

pub fn build_surface_config(
    width: u32,
    height: u32,
    format: GpuTextureFormat,
    capabilities: &GpuSurfaceCapabilities,
    request: RenderSurfacePresentationRequest,
) -> Result<GpuSurfaceConfiguration, RenderSurfacePresentationError> {
    if width == 0 || height == 0 {
        return Err(RenderSurfacePresentationError::ZeroExtent);
    }
    if !capabilities.supports_pair(format, GpuSurfaceColorSpace::Srgb)
        || !capabilities.supports_usage(GpuTextureUsage::ColorAttachment)
    {
        return Err(RenderSurfacePresentationError::NoCompatibleFormat);
    }
    let mode = select_surface_present_mode(capabilities.present_modes(), request.present_mode)?;
    let alpha_mode = capabilities
        .alpha_modes()
        .first()
        .copied()
        .ok_or(RenderSurfacePresentationError::NoCompatibleAlphaOrUsage)?;
    let mut usages = vec![GpuTextureUsage::ColorAttachment];
    for usage in [
        GpuTextureUsage::CopySource,
        GpuTextureUsage::CopyDestination,
    ] {
        if capabilities.supports_usage(usage) {
            usages.push(usage);
        }
    }
    if !capabilities.supports_alpha_mode(alpha_mode) {
        return Err(RenderSurfacePresentationError::NoCompatibleAlphaOrUsage);
    }
    GpuSurfaceConfiguration::new(
        width,
        height,
        format,
        GpuSurfaceColorSpace::Srgb,
        usages,
        mode,
        alpha_mode,
        request.desired_maximum_frame_latency.get(),
        [format],
    )
    .map_err(|error| RenderSurfacePresentationError::Configuration(error.cause()))
}

#[cfg(test)]
mod presentation_policy_tests {
    use super::*;

    #[test]
    fn default_and_explicit_hint_are_stable() {
        let default = RenderSurfacePresentationRequest::default();
        assert_eq!(default.present_mode, GpuSurfacePresentMode::Fifo);
        assert_eq!(default.desired_maximum_frame_latency.get(), 2);
        assert_eq!(
            RenderSurfacePresentationRequest::new(GpuSurfacePresentMode::Mailbox, 0),
            Err(RenderSurfacePresentationError::ZeroLatency)
        );
        assert_eq!(
            RenderSurfacePresentationRequest::new(GpuSurfacePresentMode::Immediate, 5)
                .unwrap()
                .desired_maximum_frame_latency
                .get(),
            5
        );
    }

    #[test]
    fn strict_mode_selection_never_invents_fallback() {
        for supported in [
            vec![GpuSurfacePresentMode::Fifo],
            vec![GpuSurfacePresentMode::Fifo, GpuSurfacePresentMode::Mailbox],
            vec![
                GpuSurfacePresentMode::Fifo,
                GpuSurfacePresentMode::Immediate,
            ],
            vec![
                GpuSurfacePresentMode::Fifo,
                GpuSurfacePresentMode::FifoRelaxed,
            ],
            vec![],
        ] {
            for requested in [
                GpuSurfacePresentMode::Fifo,
                GpuSurfacePresentMode::Mailbox,
                GpuSurfacePresentMode::Immediate,
                GpuSurfacePresentMode::FifoRelaxed,
            ] {
                let result = select_surface_present_mode(&supported, requested);
                if supported.contains(&requested) {
                    assert_eq!(result, Ok(requested));
                } else {
                    assert_eq!(
                        result,
                        Err(RenderSurfacePresentationError::UnsupportedPresentMode(
                            requested
                        ))
                    );
                }
            }
        }
    }

    #[test]
    fn request_updates_are_idempotent_revisioned_and_surface_isolated() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let first =
            registry.reserve_surface_for_native_window(NativeWindowId::primary(), (800, 600));
        let second_native = NativeWindowId::try_from_raw(8).unwrap();
        let second = registry.reserve_surface_for_native_window(second_native, (500, 400));
        let request =
            RenderSurfacePresentationRequest::new(GpuSurfacePresentMode::Mailbox, 1).unwrap();
        assert_eq!(registry.set_presentation_request(first, request), Ok(1));
        assert_eq!(registry.set_presentation_request(first, request), Ok(1));
        assert_eq!(
            registry.record(second).unwrap().presentation_request,
            RenderSurfacePresentationRequest::default()
        );
        registry.record_presentation_attempt(
            first,
            RenderSurfacePresentationAttempt::Applied { revision: 0 },
        );
        assert_eq!(registry.record(first).unwrap().presentation_attempt, None);
        registry.record_presentation_attempt(
            first,
            RenderSurfacePresentationAttempt::Rejected {
                revision: 1,
                reason: RenderSurfacePresentationError::UnsupportedPresentMode(
                    GpuSurfacePresentMode::Mailbox,
                ),
            },
        );
        assert!(matches!(
            registry.record(first).unwrap().presentation_attempt,
            Some(RenderSurfacePresentationAttempt::Rejected { revision: 1, .. })
        ));
        assert_eq!(
            registry.set_presentation_request(first, RenderSurfacePresentationRequest::default()),
            Ok(2)
        );
        assert_eq!(registry.record(first).unwrap().presentation_attempt, None);
        registry.retire_surface_for_native_window(second_native);
        assert_eq!(
            registry.set_presentation_request(second, request),
            Err(RenderSurfacePresentationError::RetiredSurface)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservation_allocates_stable_identity_without_claiming_attachment() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let surface =
            registry.reserve_surface_for_native_window(NativeWindowId::primary(), (1280, 720));
        assert_eq!(surface, RenderSurfaceId::primary());
        assert_eq!(registry.primary_surface_id(), Some(surface));
        assert_eq!(
            registry.record(surface).map(|r| r.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Requested)
        );
    }

    #[test]
    fn attachment_confirmation_promotes_the_exact_reserved_identity() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let surface = registry.reserve_surface_for_native_window(window, (640, 480));
        registry
            .confirm_surface_attachment(surface, window, (800, 600))
            .unwrap();
        assert_eq!(registry.surface_for_native_window(window), Some(surface));
        assert_eq!(
            registry
                .record(surface)
                .map(|r| (r.lifecycle_state, r.target_size_px)),
            Some((RenderSurfaceLifecycleState::Attached, (800, 600)))
        );
    }

    #[test]
    fn reservation_keeps_primary_id_available_when_secondary_is_requested_first() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let secondary = registry.reserve_surface_for_native_window(window, (640, 480));
        let primary =
            registry.reserve_surface_for_native_window(NativeWindowId::primary(), (1280, 720));
        assert_ne!(secondary, RenderSurfaceId::primary());
        assert_eq!(primary, RenderSurfaceId::primary());
    }

    #[test]
    fn secondary_attachment_requires_the_previously_reserved_identity() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let unreserved = RenderSurfaceId::try_from_raw(9).unwrap();

        assert!(
            registry
                .confirm_surface_attachment(unreserved, window, (640, 480))
                .is_err()
        );
        assert_eq!(registry.surface_for_native_window(window), None);
        assert_eq!(registry.record(unreserved), None);
    }

    #[test]
    fn attachment_confirmation_rejects_mismatched_primary_identity() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let secondary = registry.reserve_surface_for_native_window(window, (640, 480));
        assert!(
            registry
                .confirm_surface_attachment(RenderSurfaceId::primary(), window, (640, 480))
                .is_err()
        );
        assert_eq!(
            registry.record(secondary).map(|r| r.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Requested)
        );
    }

    #[test]
    fn extent_updates_never_promote_requested_identity() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let surface = registry.reserve_surface_for_native_window(window, (640, 480));
        assert!(registry.update_surface_extent_for_native_window(window, (900, 600)));
        assert_eq!(
            registry
                .record(surface)
                .map(|r| (r.lifecycle_state, r.target_size_px)),
            Some((RenderSurfaceLifecycleState::Requested, (900, 600)))
        );
    }

    #[test]
    fn retiring_secondary_surface_preserves_attached_primary() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let primary =
            registry.reserve_surface_for_native_window(NativeWindowId::primary(), (1280, 720));
        registry
            .confirm_surface_attachment(primary, NativeWindowId::primary(), (1280, 720))
            .expect("primary surface should attach");

        let secondary_window = NativeWindowId::try_from_raw(2).expect("secondary window id");
        let secondary = registry.reserve_surface_for_native_window(secondary_window, (900, 600));
        registry
            .confirm_surface_attachment(secondary, secondary_window, (900, 600))
            .expect("secondary surface should attach");

        assert_eq!(
            registry.retire_surface_for_native_window(secondary_window),
            Some(secondary)
        );
        assert_eq!(
            registry.surface_for_native_window(NativeWindowId::primary()),
            Some(primary)
        );
        assert_eq!(registry.primary_surface_id(), Some(primary));
        assert_eq!(
            registry
                .record(primary)
                .map(|record| record.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Attached)
        );
        assert_eq!(
            registry
                .record(secondary)
                .map(|record| record.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Retired)
        );
    }

    #[test]
    fn retiring_surface_removes_window_lookup_and_preserves_auditable_record() {
        let mut registry = RenderSurfaceRegistryResource::default();
        let window = NativeWindowId::try_from_raw(2).unwrap();
        let surface = registry.reserve_surface_for_native_window(window, (640, 480));
        assert_eq!(
            registry.retire_surface_for_native_window(window),
            Some(surface)
        );
        assert_eq!(registry.surface_for_native_window(window), None);
        assert_eq!(
            registry.record(surface).map(|r| r.lifecycle_state),
            Some(RenderSurfaceLifecycleState::Retired)
        );
    }
}
