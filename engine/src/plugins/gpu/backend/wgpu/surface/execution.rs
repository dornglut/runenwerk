use super::super::WgpuDeviceHealth;
use super::{WgpuSurfaceLease, WgpuSurfaceState, WgpuSurfaceStateInner};
use crate::plugins::gpu::{
    GpuContextAffinity, GpuSurfaceLeaseDisposition, GpuSurfaceLeaseError,
    GpuSurfaceLeaseErrorCategory, GpuSurfaceResourceLease, GpuWorkResourceId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WgpuSurfaceLeaseResource {
    Texture(GpuWorkResourceId),
    TextureView(GpuWorkResourceId),
}

impl WgpuSurfaceLeaseResource {
    const fn identity(self) -> GpuWorkResourceId {
        match self {
            Self::Texture(identity) | Self::TextureView(identity) => identity,
        }
    }
}

impl WgpuSurfaceState {
    /// Validates one surface-acquired logical resource against the sole G7 physical lease owner.
    ///
    /// This is intentionally separate from G4C1 realization: surface-acquired resources are
    /// transient presentation leases and never become ordinary resource-registry records.
    pub(crate) fn validate_execution_lease(
        &self,
        lease: &GpuSurfaceResourceLease,
        resource: WgpuSurfaceLeaseResource,
        health: &WgpuDeviceHealth,
    ) -> Result<(), GpuSurfaceLeaseError> {
        ensure_lease_health(health, lease)?;
        let mut inner = self.shared.inner.lock().map_err(|_| {
            lease_error(
                GpuSurfaceLeaseErrorCategory::ContextOrDeviceUnavailableOrLost,
                lease,
                "surface lease validation authority is unavailable",
            )
        })?;
        if let Some(record) = inner.records.get_mut(&lease.surface().id()) {
            super::release_abandoned_lease(record);
        }
        validate_lease(self.affinity, &inner, lease, resource)?;
        ensure_lease_health(health, lease)
    }
}

fn validate_lease<'a>(
    expected: GpuContextAffinity,
    inner: &'a WgpuSurfaceStateInner,
    lease: &GpuSurfaceResourceLease,
    resource: WgpuSurfaceLeaseResource,
) -> Result<&'a WgpuSurfaceLease, GpuSurfaceLeaseError> {
    validate_affinity(expected, lease)?;
    let record = inner.records.get(&lease.surface().id()).ok_or_else(|| {
        lease_error(
            GpuSurfaceLeaseErrorCategory::UnknownSurface,
            lease,
            "surface identity is absent from this context-local G7 owner",
        )
    })?;
    if record.generation != lease.surface().generation() {
        return Err(lease_error(
            GpuSurfaceLeaseErrorCategory::StaleGeneration,
            lease,
            "surface-acquired resource belongs to a stale surface generation",
        ));
    }
    validate_disposition(lease)?;
    let active = record.active_lease.as_ref().ok_or_else(|| {
        lease_error(
            GpuSurfaceLeaseErrorCategory::InvalidLease,
            lease,
            "the surface has no active acquired-image lease",
        )
    })?;
    if active.lease != *lease {
        return Err(lease_error(
            GpuSurfaceLeaseErrorCategory::InvalidLease,
            lease,
            "surface-acquired resource does not name the current active lease",
        ));
    }
    let identity_matches = match resource {
        WgpuSurfaceLeaseResource::Texture(identity) => active.texture_identity == identity,
        WgpuSurfaceLeaseResource::TextureView(identity) => active.view_identity == identity,
    };
    if !identity_matches {
        return Err(lease_error(
            GpuSurfaceLeaseErrorCategory::InvalidLease,
            lease,
            format!(
                "logical {:?} identity {:?} is not owned by this surface lease",
                resource,
                resource.identity()
            ),
        ));
    }
    Ok(active)
}

fn validate_affinity(
    expected: GpuContextAffinity,
    lease: &GpuSurfaceResourceLease,
) -> Result<(), GpuSurfaceLeaseError> {
    let observed = lease.surface().affinity();
    if observed.context() != expected.context() {
        return Err(lease_error(
            GpuSurfaceLeaseErrorCategory::ForeignContext,
            lease,
            "surface-acquired resource belongs to a different GPU context",
        ));
    }
    if observed.generation() != expected.generation() {
        return Err(lease_error(
            GpuSurfaceLeaseErrorCategory::StaleGeneration,
            lease,
            "surface-acquired resource belongs to a stale device generation",
        ));
    }
    Ok(())
}

fn validate_disposition(lease: &GpuSurfaceResourceLease) -> Result<(), GpuSurfaceLeaseError> {
    match lease.disposition() {
        GpuSurfaceLeaseDisposition::Active => Ok(()),
        GpuSurfaceLeaseDisposition::Abandoned => Err(lease_error(
            GpuSurfaceLeaseErrorCategory::InvalidLease,
            lease,
            "surface acquisition was abandoned without presentation",
        )),
        GpuSurfaceLeaseDisposition::Presented => Err(lease_error(
            GpuSurfaceLeaseErrorCategory::AlreadyConsumed,
            lease,
            "surface acquisition lease was already consumed by Present",
        )),
    }
}

fn ensure_lease_health(
    health: &WgpuDeviceHealth,
    lease: &GpuSurfaceResourceLease,
) -> Result<(), GpuSurfaceLeaseError> {
    if let Some(fault) = health.terminal_fault() {
        Err(lease_error(
            GpuSurfaceLeaseErrorCategory::ContextOrDeviceUnavailableOrLost,
            lease,
            fault.detail,
        ))
    } else {
        Ok(())
    }
}

fn lease_error(
    category: GpuSurfaceLeaseErrorCategory,
    lease: &GpuSurfaceResourceLease,
    detail: impl Into<String>,
) -> GpuSurfaceLeaseError {
    GpuSurfaceLeaseError::new(category, lease.surface().id(), lease.lease_id(), detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::gpu::{
        GpuContextId, GpuDeviceGeneration, GpuSurfaceGeneration, GpuSurfaceHandle,
        allocate_surface_id, allocate_surface_lease_id,
    };
    use std::num::NonZeroU64;

    fn affinity(context: u64, generation: u64) -> GpuContextAffinity {
        GpuContextAffinity::test_value(
            GpuContextId::test_value(NonZeroU64::new(context).unwrap()),
            GpuDeviceGeneration::test_value(NonZeroU64::new(generation).unwrap()),
        )
    }

    fn lease(affinity: GpuContextAffinity) -> GpuSurfaceResourceLease {
        let surface = GpuSurfaceHandle::new(
            allocate_surface_id().unwrap(),
            affinity,
            GpuSurfaceGeneration::first(),
        );
        let lease_id = allocate_surface_lease_id(surface.id()).unwrap();
        GpuSurfaceResourceLease::new(surface, lease_id)
    }

    #[test]
    fn lease_disposition_distinguishes_abandoned_from_present_consumed() {
        let abandoned = lease(affinity(1, 1));
        abandoned.mark_abandoned();
        let error = validate_disposition(&abandoned).unwrap_err();
        assert_eq!(error.category(), GpuSurfaceLeaseErrorCategory::InvalidLease);

        let presented = lease(affinity(1, 1));
        presented.mark_presented().unwrap();
        let error = validate_disposition(&presented).unwrap_err();
        assert_eq!(
            error.category(),
            GpuSurfaceLeaseErrorCategory::AlreadyConsumed
        );
    }

    #[test]
    fn lease_affinity_distinguishes_foreign_context_and_stale_generation() {
        let expected = affinity(1, 1);

        let foreign_lease = lease(affinity(2, 1));
        let foreign = validate_affinity(expected, &foreign_lease).unwrap_err();
        assert_eq!(
            foreign.category(),
            GpuSurfaceLeaseErrorCategory::ForeignContext
        );

        let stale_lease = lease(affinity(1, 2));
        let stale = validate_affinity(expected, &stale_lease).unwrap_err();
        assert_eq!(
            stale.category(),
            GpuSurfaceLeaseErrorCategory::StaleGeneration
        );
    }
}
