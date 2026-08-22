use super::{
    GpuContextAffinity, GpuSurfaceGeneration, GpuSurfaceHandle, GpuSurfaceId, GpuTextureHandle,
    GpuTextureViewHandle,
};
use core::fmt;
use core::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

/// Opaque process-local identity for one acquired surface-image lease.
///
/// The value is correlation identity only. It has no persistence, replay, wire, ABI, or cache
/// representation guarantee and cannot be constructed by callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuSurfaceLeaseId(NonZeroU64);

#[derive(Debug)]
struct GpuSurfaceLeaseIdAllocator {
    next: AtomicU64,
}

impl GpuSurfaceLeaseIdAllocator {
    const fn new(next: u64) -> Self {
        Self {
            next: AtomicU64::new(next),
        }
    }

    fn allocate(
        &self,
        surface: GpuSurfaceId,
    ) -> Result<GpuSurfaceLeaseId, GpuSurfaceAcquireError> {
        let value = self
            .next
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                (current != 0).then_some(if current == u64::MAX { 0 } else { current + 1 })
            })
            .map_err(|_| {
                GpuSurfaceAcquireError::new(
                    GpuSurfaceAcquireErrorCategory::IdentityExhausted,
                    Some(surface),
                    "surface lease identifier allocator exhausted",
                )
            })?;
        Ok(GpuSurfaceLeaseId(
            NonZeroU64::new(value).expect("surface lease identifier allocator never returns zero"),
        ))
    }
}

static PRODUCTION_SURFACE_LEASE_IDS: GpuSurfaceLeaseIdAllocator =
    GpuSurfaceLeaseIdAllocator::new(1);

pub(crate) fn allocate_surface_lease_id(
    surface: GpuSurfaceId,
) -> Result<GpuSurfaceLeaseId, GpuSurfaceAcquireError> {
    PRODUCTION_SURFACE_LEASE_IDS.allocate(surface)
}

/// Non-owning provenance carried by a surface-acquired logical texture lease.
///
/// This value deliberately contains no backend object and no strong owner reference. Logical
/// handle clones can therefore preserve the surface/context/generation/lease identity required for
/// stale and foreign-context validation without extending physical swapchain authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GpuSurfaceResourceLease {
    surface: GpuSurfaceHandle,
    lease_id: GpuSurfaceLeaseId,
}

impl GpuSurfaceResourceLease {
    pub(crate) const fn new(surface: GpuSurfaceHandle, lease_id: GpuSurfaceLeaseId) -> Self {
        Self { surface, lease_id }
    }

    pub(crate) const fn surface(self) -> GpuSurfaceHandle {
        self.surface
    }

    pub(crate) const fn lease_id(self) -> GpuSurfaceLeaseId {
        self.lease_id
    }
}

/// Successful backend-neutral surface acquisition quality.
///
/// `Suboptimal` preserves backend evidence that the acquired image is usable but the current
/// surface configuration is no longer ideal. RunenGPU does not turn that evidence into automatic
/// reconfiguration policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuSurfaceAcquisitionStatus {
    Optimal,
    Suboptimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuSurfaceAcquireErrorCategory {
    NotConfigured,
    AlreadyAcquired,
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
    UnknownSurface,
    ForeignContext,
    StaleGeneration,
    ContextOrDeviceUnavailableOrLost,
    IdentityExhausted,
    InternalInvariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuSurfaceAcquireError {
    category: GpuSurfaceAcquireErrorCategory,
    surface: Option<GpuSurfaceId>,
    detail: Option<String>,
}

impl GpuSurfaceAcquireError {
    pub(crate) fn new(
        category: GpuSurfaceAcquireErrorCategory,
        surface: Option<GpuSurfaceId>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            category,
            surface,
            detail: super::context::sanitized_diagnostic(detail.into()),
        }
    }

    pub const fn category(&self) -> GpuSurfaceAcquireErrorCategory {
        self.category
    }

    pub const fn surface(&self) -> Option<GpuSurfaceId> {
        self.surface
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for GpuSurfaceAcquireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "GPU surface acquisition failed ({:?})",
            self.category
        )?;
        if let Some(detail) = &self.detail {
            write!(formatter, ": {detail}")?;
        }
        Ok(())
    }
}

impl std::error::Error for GpuSurfaceAcquireError {}

/// Structured rejection for using a logical surface-acquired resource after its physical lease is
/// no longer valid for the current context/surface generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuSurfaceLeaseErrorCategory {
    UnknownSurface,
    InvalidLease,
    ForeignContext,
    StaleGeneration,
    AlreadyConsumed,
    ContextOrDeviceUnavailableOrLost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuSurfaceLeaseError {
    category: GpuSurfaceLeaseErrorCategory,
    surface: GpuSurfaceId,
    lease_id: GpuSurfaceLeaseId,
    detail: Option<String>,
}

impl GpuSurfaceLeaseError {
    pub(crate) fn new(
        category: GpuSurfaceLeaseErrorCategory,
        surface: GpuSurfaceId,
        lease_id: GpuSurfaceLeaseId,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            category,
            surface,
            lease_id,
            detail: super::context::sanitized_diagnostic(detail.into()),
        }
    }

    pub const fn category(&self) -> GpuSurfaceLeaseErrorCategory {
        self.category
    }

    pub const fn surface(&self) -> GpuSurfaceId {
        self.surface
    }

    pub const fn lease_id(&self) -> GpuSurfaceLeaseId {
        self.lease_id
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for GpuSurfaceLeaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "GPU surface lease use failed ({:?})",
            self.category
        )?;
        if let Some(detail) = &self.detail {
            write!(formatter, ": {detail}")?;
        }
        Ok(())
    }
}

impl std::error::Error for GpuSurfaceLeaseError {}

/// Private ownership marker for the one active physical surface-image lease.
///
/// Only `GpuAcquiredSurfaceImage` owns a strong reference. Logical texture/view handles never do,
/// so cloning those handles cannot keep physical swapchain authority alive.
#[derive(Debug)]
pub(crate) struct GpuSurfaceLeaseOwner;

impl GpuSurfaceLeaseOwner {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

/// One acquired surface image and its logical RunenGPU resource identities.
///
/// This owner is intentionally not `Clone`. Dropping it (or consuming it through `abandon`) ends
/// caller ownership of the acquisition lease. The logical texture/view handles may be cloned for
/// work authoring, but those clones do not retain the private physical surface image.
///
/// ```compile_fail
/// use engine::plugins::gpu::GpuAcquiredSurfaceImage;
/// fn require_clone<T: Clone>() {}
/// require_clone::<GpuAcquiredSurfaceImage>();
/// ```
pub struct GpuAcquiredSurfaceImage {
    surface: GpuSurfaceHandle,
    lease_id: GpuSurfaceLeaseId,
    status: GpuSurfaceAcquisitionStatus,
    texture: GpuTextureHandle,
    default_view: GpuTextureViewHandle,
    owner: Arc<GpuSurfaceLeaseOwner>,
}

impl GpuAcquiredSurfaceImage {
    pub(crate) fn new(
        surface: GpuSurfaceHandle,
        lease_id: GpuSurfaceLeaseId,
        status: GpuSurfaceAcquisitionStatus,
        texture: GpuTextureHandle,
        default_view: GpuTextureViewHandle,
        owner: Arc<GpuSurfaceLeaseOwner>,
    ) -> Self {
        Self {
            surface,
            lease_id,
            status,
            texture,
            default_view,
            owner,
        }
    }

    pub const fn surface_id(&self) -> GpuSurfaceId {
        self.surface.id()
    }

    pub const fn surface_generation(&self) -> GpuSurfaceGeneration {
        self.surface.generation()
    }

    pub const fn affinity(&self) -> GpuContextAffinity {
        self.surface.affinity()
    }

    pub const fn lease_id(&self) -> GpuSurfaceLeaseId {
        self.lease_id
    }

    pub const fn status(&self) -> GpuSurfaceAcquisitionStatus {
        self.status
    }

    pub fn texture(&self) -> &GpuTextureHandle {
        &self.texture
    }

    /// Returns the explicit single-mip/single-layer view for ordinary G5 render attachments.
    ///
    /// G5 keeps its explicit `GpuTextureViewHandle` attachment contract; G7A does not weaken that
    /// boundary merely because the parent texture came from a presentation surface.
    pub fn default_view(&self) -> &GpuTextureViewHandle {
        &self.default_view
    }

    /// Explicitly abandons this acquisition without claiming presentation.
    ///
    /// This is semantically equivalent to dropping the owner. Any logical texture/view clones
    /// remain ordinary stale logical references and do not retain the physical lease.
    pub fn abandon(self) {}

    pub(crate) fn owner_weak(&self) -> Weak<GpuSurfaceLeaseOwner> {
        Arc::downgrade(&self.owner)
    }
}

impl fmt::Debug for GpuAcquiredSurfaceImage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GpuAcquiredSurfaceImage")
            .field("surface", &self.surface)
            .field("lease_id", &self.lease_id)
            .field("status", &self.status)
            .field("texture", &self.texture.diagnostic_identity())
            .field("default_view", &self.default_view.diagnostic_identity())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquisition_categories_keep_timeout_and_occlusion_distinct() {
        assert_ne!(
            GpuSurfaceAcquireErrorCategory::Timeout,
            GpuSurfaceAcquireErrorCategory::Occluded
        );
    }
}
